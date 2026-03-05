//! AMM 自动做市商核心合约
//! 
//! 恒定乘积公式: x * y = k
//! 
//! 特性:
//! - 无需订单簿，自动定价
//! - 流动性提供者赚取手续费
//! - 支持任意 ERC-20 代币对
//! - 闪电贷支持
//! - 价格预言机

use std::collections::HashMap;

/// 流动性池
#[derive(Debug, Clone)]
pub struct Pool {
    /// 池子地址
    pub address: String,
    /// Token A
    pub token_a: String,
    /// Token B  
    pub token_b: String,
    /// Token A 储备
    pub reserve_a: u128,
    /// Token B 储备
    pub reserve_b: u128,
    /// 总流动性
    pub total_supply: u128,
    /// 流动性提供者份额
    pub liquidity: HashMap<String, u128>,
    /// 累积价格 (用于预言机)
    pub price_cumulative_a: u128,
    pub price_cumulative_b: u128,
    /// 最后更新时间
    pub last_update: u64,
    /// 手续费率 (0.3% = 3/1000)
    pub fee_rate: u16,
    /// 协议手续费 (0.05% = 5/10000)
    pub protocol_fee: u16,
    /// 手续费接收地址
    pub fee_to: String,
}

/// 交易对信息
#[derive(Debug, Clone)]
pub struct PairInfo {
    pub address: String,
    pub token_a: String,
    pub token_b: String,
    pub reserve_a: u128,
    pub reserve_b: u128,
    pub total_supply: u128,
    pub price: f64,
    pub volume_24h: u128,
    pub tvl: u128,  // Total Value Locked
}

/// 交易请求
#[derive(Debug, Clone)]
pub struct SwapRequest {
    /// 交易对
    pub pair: String,
    /// 输入代币
    pub token_in: String,
    /// 输出代币
    pub token_out: String,
    /// 输入金额
    pub amount_in: u128,
    /// 最小输出金额 (滑点保护)
    pub min_amount_out: u128,
    /// 接收地址
    pub recipient: String,
    /// 截止时间
    pub deadline: u64,
}

/// 流动性操作
#[derive(Debug, Clone)]
pub struct LiquidityRequest {
    /// 交易对
    pub pair: String,
    /// Token A 金额
    pub amount_a: u128,
    /// Token B 金额
    pub amount_b: u128,
    /// 最小流动性 (滑点保护)
    pub min_liquidity: u128,
    /// 截止时间
    pub deadline: u64,
}

/// AMM 核心引擎
pub struct AMMEngine {
    /// 所有流动性池
    pub pools: HashMap<String, Pool>,
    /// 交易对映射: (token_a, token_b) -> pool_address
    pub pairs: HashMap<(String, String), String>,
    /// 工厂地址
    pub factory: String,
    /// 协议手续费接收地址
    pub fee_to: String,
    /// 交易历史
    pub trades: Vec<Trade>,
}

/// 交易记录
#[derive(Debug, Clone)]
pub struct Trade {
    pub timestamp: u64,
    pub pair: String,
    pub trader: String,
    pub token_in: String,
    pub token_out: String,
    pub amount_in: u128,
    pub amount_out: u128,
    pub price: f64,
}

impl AMMEngine {
    /// 创建新 AMM
    pub fn new(factory: String, fee_to: String) -> Self {
        AMMEngine {
            pools: HashMap::new(),
            pairs: HashMap::new(),
            factory,
            fee_to,
            trades: Vec::new(),
        }
    }

    /// 创建交易对
    pub fn create_pair(
        &mut self,
        token_a: String,
        token_b: String,
        caller: &str,
    ) -> Result<String, AMMError> {
        // 检查权限
        if caller != self.factory {
            return Err(AMMError::NotFactory);
        }

        // 确保顺序一致
        let (t0, t1) = if token_a < token_b {
            (token_a, token_b)
        } else {
            (token_b, token_a)
        };

        // 检查是否已存在
        if self.pairs.contains_key(&(t0.clone(), t1.clone())) {
            return Err(AMMError::PairExists);
        }

        // 生成池子地址
        let address = self.generate_pair_address(&t0, &t1);

        // 创建池子
        let pool = Pool {
            address: address.clone(),
            token_a: t0.clone(),
            token_b: t1.clone(),
            reserve_a: 0,
            reserve_b: 0,
            total_supply: 0,
            liquidity: HashMap::new(),
            price_cumulative_a: 0,
            price_cumulative_b: 0,
            last_update: 0,
            fee_rate: 30,      // 0.3%
            protocol_fee: 5,   // 0.05%
            fee_to: self.fee_to.clone(),
        };

        self.pools.insert(address.clone(), pool);
        self.pairs.insert((t0, t1), address.clone());

        Ok(address)
    }

    /// 添加流动性
    pub fn add_liquidity(
        &mut self,
        pair: &str,
        amount_a: u128,
        amount_b: u128,
        provider: &str,
    ) -> Result<u128, AMMError> {
        let pool = self.pools.get_mut(pair)
            .ok_or(AMMError::PairNotFound)?;

        // 计算流动性
        let liquidity = if pool.total_supply == 0 {
            // 首次添加: L = sqrt(x * y)
            self.sqrt(amount_a * amount_b)
        } else {
            // 后续添加: 按比例
            let liq_a = amount_a * pool.total_supply / pool.reserve_a;
            let liq_b = amount_b * pool.total_supply / pool.reserve_b;
            liq_a.min(liq_b)
        };

        if liquidity == 0 {
            return Err(AMMError::InsufficientLiquidity);
        }

        // 更新储备
        pool.reserve_a += amount_a;
        pool.reserve_b += amount_b;
        pool.total_supply += liquidity;

        // 更新提供者份额
        let current = pool.liquidity.get(provider).copied().unwrap_or(0);
        pool.liquidity.insert(provider.to_string(), current + liquidity);

        // 更新价格累积器
        self.update(pool);

        Ok(liquidity)
    }

    /// 移除流动性
    pub fn remove_liquidity(
        &mut self,
        pair: &str,
        liquidity: u128,
        provider: &str,
    ) -> Result<(u128, u128), AMMError> {
        let pool = self.pools.get_mut(pair)
            .ok_or(AMMError::PairNotFound)?;

        // 检查余额
        let current = pool.liquidity.get(provider).copied().unwrap_or(0);
        if current < liquidity {
            return Err(AMMError::InsufficientLiquidity);
        }

        // 计算可提取金额
        let amount_a = liquidity * pool.reserve_a / pool.total_supply;
        let amount_b = liquidity * pool.reserve_b / pool.total_supply;

        // 更新
        pool.reserve_a -= amount_a;
        pool.reserve_b -= amount_b;
        pool.total_supply -= liquidity;
        pool.liquidity.insert(provider.to_string(), current - liquidity);

        // 更新价格累积器
        self.update(pool);

        Ok((amount_a, amount_b))
    }

    /// 交换代币
    pub fn swap(&mut self, req: SwapRequest) -> Result<u128, AMMError> {
        let pool = self.pools.get_mut(&req.pair)
            .ok_or(AMMError::PairNotFound)?;

        // 检查截止时间
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        if now > req.deadline {
            return Err(AMMError::Expired);
        }

        // 确定输入输出方向
        let (reserve_in, reserve_out) = if req.token_in == pool.token_a {
            (pool.reserve_a, pool.reserve_b)
        } else {
            (pool.reserve_b, pool.reserve_a)
        };

        // 计算输出金额
        let amount_out = self.get_amount_out(req.amount_in, reserve_in, reserve_out, pool.fee_rate);

        // 检查滑点
        if amount_out < req.min_amount_out {
            return Err(AMMError::InsufficientOutput);
        }

        // 更新储备
        if req.token_in == pool.token_a {
            pool.reserve_a += req.amount_in;
            pool.reserve_b -= amount_out;
        } else {
            pool.reserve_b += req.amount_in;
            pool.reserve_a -= amount_out;
        }

        // 记录交易
        let price = reserve_in as f64 / reserve_out as f64;
        self.trades.push(Trade {
            timestamp: now,
            pair: req.pair,
            trader: req.recipient,
            token_in: req.token_in,
            token_out: req.token_out,
            amount_in: req.amount_in,
            amount_out,
            price,
        });

        // 更新价格累积器
        self.update(pool);

        Ok(amount_out)
    }

    /// 计算输出金额
    /// 公式: Δy = (y * Δx * (1 - f)) / (x + Δx * (1 - f))
    fn get_amount_out(&self, amount_in: u128, reserve_in: u128, reserve_out: u128, fee_rate: u16) -> u128 {
        let amount_in_with_fee = amount_in * (10000 - fee_rate as u128) / 10000;
        let numerator = amount_in_with_fee * reserve_out;
        let denominator = reserve_in + amount_in_with_fee;
        numerator / denominator
    }

    /// 计算输入金额
    /// 公式: Δx = (x * Δy) / ((y - Δy) * (1 - f))
    fn get_amount_in(&self, amount_out: u128, reserve_in: u128, reserve_out: u128, fee_rate: u16) -> u128 {
        let numerator = reserve_in * amount_out * 10000;
        let denominator = (reserve_out - amount_out) * (10000 - fee_rate as u128);
        (numerator / denominator) + 1
    }

    /// 更新价格累积器
    fn update(&self, pool: &mut Pool) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let time_elapsed = now - pool.last_update;
        
        if time_elapsed > 0 && pool.reserve_a > 0 && pool.reserve_b > 0 {
            // 价格累积
            pool.price_cumulative_a += (pool.reserve_b * time_elapsed as u128 / pool.reserve_a);
            pool.price_cumulative_b += (pool.reserve_a * time_elapsed as u128 / pool.reserve_b);
            pool.last_update = now;
        }
    }

    /// 获取价格
    pub fn get_price(&self, pair: &str) -> Result<f64, AMMError> {
        let pool = self.pools.get(pair)
            .ok_or(AMMError::PairNotFound)?;
        
        if pool.reserve_b == 0 {
            return Ok(0.0);
        }
        
        Ok(pool.reserve_a as f64 / pool.reserve_b as f64)
    }

    /// 获取交易对信息
    pub fn get_pair_info(&self, pair: &str) -> Result<PairInfo, AMMError> {
        let pool = self.pools.get(pair)
            .ok_or(AMMError::PairNotFound)?;
        
        let price = self.get_price(pair)?;
        let tvl = pool.reserve_a + pool.reserve_b; // 简化计算
        
        // 计算 24h 交易量
        let volume_24h = self.trades.iter()
            .filter(|t| t.pair == pair)
            .map(|t| t.amount_in)
            .sum();
        
        Ok(PairInfo {
            address: pool.address.clone(),
            token_a: pool.token_a.clone(),
            token_b: pool.token_b.clone(),
            reserve_a: pool.reserve_a,
            reserve_b: pool.reserve_b,
            total_supply: pool.total_supply,
            price,
            volume_24h,
            tvl,
        })
    }

    /// 获取流动性提供者份额
    pub fn get_liquidity(&self, pair: &str, provider: &str) -> Result<u128, AMMError> {
        let pool = self.pools.get(pair)
            .ok_or(AMMError::PairNotFound)?;
        
        Ok(pool.liquidity.get(provider).copied().unwrap_or(0))
    }

    /// 获取所有交易对
    pub fn get_all_pairs(&self) -> Vec<PairInfo> {
        self.pools.keys()
            .filter_map(|addr| self.get_pair_info(addr).ok())
            .collect()
    }

    /// 闪电贷
    pub fn flash_loan<F>(&mut self, pair: &str, amount_a: u128, amount_b: u128, callback: F) -> Result<(), AMMError>
    where
        F: FnOnce(&mut Self) -> Result<(), AMMError>,
    {
        let pool = self.pools.get_mut(pair)
            .ok_or(AMMError::PairNotFound)?;

        // 检查流动性
        if pool.reserve_a < amount_a || pool.reserve_b < amount_b {
            return Err(AMMError::InsufficientLiquidity);
        }

        // 记录原始储备
        let reserve_a_before = pool.reserve_a;
        let reserve_b_before = pool.reserve_b;

        // 借出
        pool.reserve_a -= amount_a;
        pool.reserve_b -= amount_b;

        // 执行回调
        callback(self)?;

        // 检查归还（需要支付 0.3% 手续费）
        let pool = self.pools.get(pair).unwrap();
        let fee_a = amount_a * 3 / 1000;
        let fee_b = amount_b * 3 / 1000;

        if pool.reserve_a < reserve_a_before + fee_a || pool.reserve_b < reserve_b_before + fee_b {
            return Err(AMMError::FlashLoanNotRepaid);
        }

        Ok(())
    }

    /// 生成交易对地址
    fn generate_pair_address(&self, token_a: &str, token_b: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(token_a.as_bytes());
        hasher.update(token_b.as_bytes());
        hasher.update(self.factory.as_bytes());
        let hash = hex::encode(hasher.finalize());
        format!("0x{}", &hash[..40])
    }

    /// 平方根计算
    fn sqrt(&self, y: u128) -> u128 {
        if y == 0 {
            return 0;
        }
        let mut z = y;
        let mut x = y / 2 + 1;
        while x < z {
            z = x;
            x = (y / x + x) / 2;
        }
        z
    }
}

/// AMM 错误
#[derive(Debug, Clone)]
pub enum AMMError {
    NotFactory,
    PairExists,
    PairNotFound,
    InsufficientLiquidity,
    InsufficientOutput,
    Expired,
    FlashLoanNotRepaid,
}

impl std::fmt::Display for AMMError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AMMError::NotFactory => write!(f, "不是工厂地址"),
            AMMError::PairExists => write!(f, "交易对已存在"),
            AMMError::PairNotFound => write!(f, "交易对不存在"),
            AMMError::InsufficientLiquidity => write!(f, "流动性不足"),
            AMMError::InsufficientOutput => write!(f, "输出金额不足（滑点过大）"),
            AMMError::Expired => write!(f, "交易已过期"),
            AMMError::FlashLoanNotRepaid => write!(f, "闪电贷未归还"),
        }
    }
}

impl std::error::Error for AMMError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_pair() {
        let mut amm = AMMEngine::new("factory".to_string(), "fee_to".to_string());
        let pair = amm.create_pair("YRU".to_string(), "ETH".to_string(), "factory").unwrap();
        assert!(!pair.is_empty());
    }

    #[test]
    fn test_add_liquidity() {
        let mut amm = AMMEngine::new("factory".to_string(), "fee_to".to_string());
        let pair = amm.create_pair("YRU".to_string(), "ETH".to_string(), "factory").unwrap();
        
        let lp = amm.add_liquidity(&pair, 10000, 20000, "alice").unwrap();
        assert!(lp > 0);
    }

    #[test]
    fn test_swap() {
        let mut amm = AMMEngine::new("factory".to_string(), "fee_to".to_string());
        let pair = amm.create_pair("YRU".to_string(), "ETH".to_string(), "factory").unwrap();
        
        // 添加流动性
        amm.add_liquidity(&pair, 10000, 10000, "alice").unwrap();
        
        // 交换
        let req = SwapRequest {
            pair: pair.clone(),
            token_in: "YRU".to_string(),
            token_out: "ETH".to_string(),
            amount_in: 1000,
            min_amount_out: 0,
            recipient: "bob".to_string(),
            deadline: u64::MAX,
        };
        
        let amount_out = amm.swap(req).unwrap();
        assert!(amount_out < 1000); // 有手续费
        assert!(amount_out > 900);  // 大约 997
    }

    #[test]
    fn test_price() {
        let mut amm = AMMEngine::new("factory".to_string(), "fee_to".to_string());
        let pair = amm.create_pair("YRU".to_string(), "ETH".to_string(), "factory").unwrap();
        
        amm.add_liquidity(&pair, 10000, 5000, "alice").unwrap();
        
        let price = amm.get_price(&pair).unwrap();
        assert!((price - 2.0).abs() < 0.01);
    }
}
