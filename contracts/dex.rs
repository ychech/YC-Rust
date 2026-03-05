//! DEX (去中心化交易所)
//! 
//! 功能：
//! - 创建交易对
//! - 添加/移除流动性
//! - 代币交换（AMM 自动做市商）
//! - 恒定乘积公式: x * y = k

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// 流动性池
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidityPool {
    /// 交易对名称 (如 "YRU/ETH")
    pub pair: String,
    /// Token A 地址
    pub token_a: String,
    /// Token B 地址
    pub token_b: String,
    /// Token A 储备量
    pub reserve_a: u64,
    /// Token B 储备量
    pub reserve_b: u64,
    /// 流动性提供者份额
    pub total_supply: u64,
    /// LP Token 余额
    pub lp_balances: HashMap<String, u64>,
    /// 手续费率 (0.3% = 30/10000)
    pub fee_rate: u64,
}

/// DEX 合约
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DEXContract {
    /// 流动性池列表: pair_hash -> Pool
    pub pools: HashMap<String, LiquidityPool>,
    /// 工厂地址
    pub factory: String,
    /// 手续费接收地址
    pub fee_to: String,
}

/// 添加流动性事件
#[derive(Debug, Clone)]
pub struct AddLiquidityEvent {
    pub provider: String,
    pub pair: String,
    pub amount_a: u64,
    pub amount_b: u64,
    pub lp_tokens: u64,
}

/// 移除流动性事件
#[derive(Debug, Clone)]
pub struct RemoveLiquidityEvent {
    pub provider: String,
    pub pair: String,
    pub amount_a: u64,
    pub amount_b: u64,
    pub lp_tokens: u64,
}

/// 交换事件
#[derive(Debug, Clone)]
pub struct SwapEvent {
    pub sender: String,
    pub pair: String,
    pub amount_in: u64,
    pub amount_out: u64,
    pub direction: SwapDirection,
}

/// 交换方向
#[derive(Debug, Clone)]
pub enum SwapDirection {
    AtoB,
    BtoA,
}

impl DEXContract {
    /// 创建新 DEX
    pub fn new(factory: String, fee_to: String) -> Self {
        DEXContract {
            pools: HashMap::new(),
            factory,
            fee_to,
        }
    }
    
    /// 计算交易对哈希
    fn get_pair_hash(token_a: &str, token_b: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        
        // 确保顺序一致
        if token_a < token_b {
            hasher.update(token_a.as_bytes());
            hasher.update(token_b.as_bytes());
        } else {
            hasher.update(token_b.as_bytes());
            hasher.update(token_a.as_bytes());
        }
        
        hex::encode(hasher.finalize())[..40].to_string()
    }
    
    /// 创建流动性池
    pub fn create_pool(
        &mut self,
        token_a: String,
        token_b: String,
        caller: &str,
    ) -> Result<String, DEXError> {
        if caller != self.factory {
            return Err(DEXError::NotFactory);
        }
        
        if token_a == token_b {
            return Err(DEXError::IdenticalTokens);
        }
        
        let pair_hash = Self::get_pair_hash(&token_a, &token_b);
        
        if self.pools.contains_key(&pair_hash) {
            return Err(DEXError::PoolExists);
        }
        
        let pair_name = format!("{}/{}", 
            token_a.split(':').last().unwrap_or(&token_a),
            token_b.split(':').last().unwrap_or(&token_b)
        );
        
        let pool = LiquidityPool {
            pair: pair_name,
            token_a,
            token_b,
            reserve_a: 0,
            reserve_b: 0,
            total_supply: 0,
            lp_balances: HashMap::new(),
            fee_rate: 30, // 0.3%
        };
        
        self.pools.insert(pair_hash.clone(), pool);
        
        Ok(pair_hash)
    }
    
    /// 添加流动性
    pub fn add_liquidity(
        &mut self,
        pair_hash: &str,
        amount_a: u64,
        amount_b: u64,
        provider: &str,
    ) -> Result<u64, DEXError> {
        let pool = self.pools.get_mut(pair_hash)
            .ok_or(DEXError::PoolNotFound)?;
        
        // 计算 LP Token 数量
        let lp_tokens = if pool.total_supply == 0 {
            // 首次添加流动性
            // LP = sqrt(amount_a * amount_b)
            ((amount_a as u128 * amount_b as u128) as f64).sqrt() as u64
        } else {
            // 按比例计算
            let share_a = (amount_a as u128 * pool.total_supply as u128 / pool.reserve_a as u128) as u64;
            let share_b = (amount_b as u128 * pool.total_supply as u128 / pool.reserve_b as u128) as u64;
            share_a.min(share_b)
        };
        
        if lp_tokens == 0 {
            return Err(DEXError::InsufficientLiquidity);
        }
        
        // 更新储备
        pool.reserve_a += amount_a;
        pool.reserve_b += amount_b;
        pool.total_supply += lp_tokens;
        
        // 发放 LP Token
        let balance = pool.lp_balances.get(provider).copied().unwrap_or(0);
        pool.lp_balances.insert(provider.to_string(), balance + lp_tokens);
        
        // 触发事件
        let _event = AddLiquidityEvent {
            provider: provider.to_string(),
            pair: pool.pair.clone(),
            amount_a,
            amount_b,
            lp_tokens,
        };
        
        Ok(lp_tokens)
    }
    
    /// 移除流动性
    pub fn remove_liquidity(
        &mut self,
        pair_hash: &str,
        lp_tokens: u64,
        provider: &str,
    ) -> Result<(u64, u64), DEXError> {
        let pool = self.pools.get_mut(pair_hash)
            .ok_or(DEXError::PoolNotFound)?;
        
        // 检查 LP Token 余额
        let balance = pool.lp_balances.get(provider).copied().unwrap_or(0);
        if balance < lp_tokens {
            return Err(DEXError::InsufficientLPBalance);
        }
        
        // 计算可提取的代币数量
        let amount_a = (lp_tokens as u128 * pool.reserve_a as u128 / pool.total_supply as u128) as u64;
        let amount_b = (lp_tokens as u128 * pool.reserve_b as u128 / pool.total_supply as u128) as u64;
        
        // 更新储备
        pool.reserve_a -= amount_a;
        pool.reserve_b -= amount_b;
        pool.total_supply -= lp_tokens;
        
        // 销毁 LP Token
        pool.lp_balances.insert(provider.to_string(), balance - lp_tokens);
        
        // 触发事件
        let _event = RemoveLiquidityEvent {
            provider: provider.to_string(),
            pair: pool.pair.clone(),
            amount_a,
            amount_b,
            lp_tokens,
        };
        
        Ok((amount_a, amount_b))
    }
    
    /// 计算交换输出金额（恒定乘积公式）
    /// x * y = k
    /// (x + dx) * (y - dy) = k
    /// dy = y * dx / (x + dx)
    pub fn calculate_swap_output(
        &self,
        pair_hash: &str,
        amount_in: u64,
        direction: SwapDirection,
    ) -> Result<u64, DEXError> {
        let pool = self.pools.get(pair_hash)
            .ok_or(DEXError::PoolNotFound)?;
        
        if pool.reserve_a == 0 || pool.reserve_b == 0 {
            return Err(DEXError::InsufficientLiquidity);
        }
        
        let (reserve_in, reserve_out) = match direction {
            SwapDirection::AtoB => (pool.reserve_a, pool.reserve_b),
            SwapDirection::BtoA => (pool.reserve_b, pool.reserve_a),
        };
        
        // 扣除手续费
        let amount_in_with_fee = amount_in as u128 * (10000 - pool.fee_rate) as u128 / 10000;
        
        // 计算输出
        let numerator = amount_in_with_fee * reserve_out as u128;
        let denominator = reserve_in as u128 + amount_in_with_fee;
        let amount_out = (numerator / denominator) as u64;
        
        Ok(amount_out)
    }
    
    /// 执行交换
    pub fn swap(
        &mut self,
        pair_hash: &str,
        amount_in: u64,
        min_amount_out: u64,
        direction: SwapDirection,
        sender: &str,
    ) -> Result<u64, DEXError> {
        let pool = self.pools.get_mut(pair_hash)
            .ok_or(DEXError::PoolNotFound)?;
        
        // 计算输出
        let amount_out = self.calculate_swap_output(pair_hash, amount_in, direction.clone())?;
        
        // 检查滑点
        if amount_out < min_amount_out {
            return Err(DEXError::InsufficientOutputAmount);
        }
        
        // 更新储备
        match direction {
            SwapDirection::AtoB => {
                pool.reserve_a += amount_in;
                pool.reserve_b -= amount_out;
            }
            SwapDirection::BtoA => {
                pool.reserve_b += amount_in;
                pool.reserve_a -= amount_out;
            }
        }
        
        // 触发事件
        let _event = SwapEvent {
            sender: sender.to_string(),
            pair: pool.pair.clone(),
            amount_in,
            amount_out,
            direction,
        };
        
        Ok(amount_out)
    }
    
    /// 获取池子信息
    pub fn get_pool_info(&self, pair_hash: &str) -> Option<(u64, u64, u64)> {
        self.pools.get(pair_hash).map(|p| (p.reserve_a, p.reserve_b, p.total_supply))
    }
    
    /// 获取价格 (token_a / token_b)
    pub fn get_price(&self, pair_hash: &str) -> Option<f64> {
        self.pools.get(pair_hash).map(|p| {
            if p.reserve_b == 0 {
                0.0
            } else {
                p.reserve_a as f64 / p.reserve_b as f64
            }
        })
    }
    
    /// 获取 LP Token 余额
    pub fn get_lp_balance(&self, pair_hash: &str, provider: &str) -> u64 {
        self.pools.get(pair_hash)
            .and_then(|p| p.lp_balances.get(provider).copied())
            .unwrap_or(0)
    }
}

/// DEX 错误
#[derive(Debug, Clone)]
pub enum DEXError {
    NotFactory,
    IdenticalTokens,
    PoolExists,
    PoolNotFound,
    InsufficientLiquidity,
    InsufficientLPBalance,
    InsufficientOutputAmount,
    InvalidAmount,
}

impl std::fmt::Display for DEXError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DEXError::NotFactory => write!(f, "不是工厂地址"),
            DEXError::IdenticalTokens => write!(f, "代币地址相同"),
            DEXError::PoolExists => write!(f, "流动性池已存在"),
            DEXError::PoolNotFound => write!(f, "流动性池不存在"),
            DEXError::InsufficientLiquidity => write!(f, "流动性不足"),
            DEXError::InsufficientLPBalance => write!(f, "LP Token 余额不足"),
            DEXError::InsufficientOutputAmount => write!(f, "输出金额不足（滑点过大）"),
            DEXError::InvalidAmount => write!(f, "无效金额"),
        }
    }
}

impl std::error::Error for DEXError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_pool() {
        let mut dex = DEXContract::new("factory".to_string(), "fee_to".to_string());
        
        let pair = dex.create_pool(
            "token_a".to_string(),
            "token_b".to_string(),
            "factory",
        ).unwrap();
        
        assert!(!pair.is_empty());
        assert!(dex.pools.contains_key(&pair));
    }
    
    #[test]
    fn test_add_liquidity() {
        let mut dex = DEXContract::new("factory".to_string(), "fee_to".to_string());
        
        let pair = dex.create_pool("token_a".to_string(), "token_b".to_string(), "factory").unwrap();
        
        let lp = dex.add_liquidity(&pair, 10000, 20000, "alice").unwrap();
        
        assert!(lp > 0);
        assert_eq!(dex.get_lp_balance(&pair, "alice"), lp);
        
        let (reserve_a, reserve_b, _) = dex.get_pool_info(&pair).unwrap();
        assert_eq!(reserve_a, 10000);
        assert_eq!(reserve_b, 20000);
    }
    
    #[test]
    fn test_swap() {
        let mut dex = DEXContract::new("factory".to_string(), "fee_to".to_string());
        
        let pair = dex.create_pool("token_a".to_string(), "token_b".to_string(), "factory").unwrap();
        
        // 添加流动性
        dex.add_liquidity(&pair, 10000, 10000, "alice").unwrap();
        
        // 交换
        let amount_out = dex.swap(&pair, 1000, 0, SwapDirection::AtoB, "bob").unwrap();
        
        // 由于手续费，输出应该略小于输入
        assert!(amount_out < 1000);
        assert!(amount_out > 900); // 大约 997
        
        // 检查储备
        let (reserve_a, reserve_b, _) = dex.get_pool_info(&pair).unwrap();
        assert_eq!(reserve_a, 11000);
        assert_eq!(reserve_b, 10000 - amount_out);
    }
    
    #[test]
    fn test_remove_liquidity() {
        let mut dex = DEXContract::new("factory".to_string(), "fee_to".to_string());
        
        let pair = dex.create_pool("token_a".to_string(), "token_b".to_string(), "factory").unwrap();
        
        let lp = dex.add_liquidity(&pair, 10000, 20000, "alice").unwrap();
        
        // 移除一半流动性
        let (amount_a, amount_b) = dex.remove_liquidity(&pair, lp / 2, "alice").unwrap();
        
        assert_eq!(amount_a, 5000);
        assert_eq!(amount_b, 10000);
        assert_eq!(dex.get_lp_balance(&pair, "alice"), lp / 2);
    }
    
    #[test]
    fn test_price() {
        let mut dex = DEXContract::new("factory".to_string(), "fee_to".to_string());
        
        let pair = dex.create_pool("token_a".to_string(), "token_b".to_string(), "factory").unwrap();
        
        dex.add_liquidity(&pair, 10000, 5000, "alice").unwrap();
        
        let price = dex.get_price(&pair).unwrap();
        assert!((price - 2.0).abs() < 0.01); // 10000 / 5000 = 2.0
    }
}
