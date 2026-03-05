//! Uniswap V2 风格 AMM Swap 合约
//! 
//! 核心公式: x * y = k
//! 手续费: 0.3%

use std::collections::HashMap;

/// ERC20 接口
pub trait IERC20 {
    fn balance_of(&self, account: &str) -> u64;
    fn transfer(&mut self, from: &str, to: &str, amount: u64) -> Result<(), String>;
    fn approve(&mut self, owner: &str, spender: &str, amount: u64);
    fn transfer_from(&mut self, spender: &str, from: &str, to: &str, amount: u64) -> Result<(), String>;
}

/// 简单 ERC20 实现
pub struct ERC20 {
    name: String,
    symbol: String,
    decimals: u8,
    total_supply: u64,
    balances: HashMap<String, u64>,
    allowances: HashMap<(String, String), u64>,
}

impl ERC20 {
    pub fn new(name: &str, symbol: &str, decimals: u8, initial_supply: u64, creator: &str) -> Self {
        let mut balances = HashMap::new();
        balances.insert(creator.to_string(), initial_supply);
        
        Self {
            name: name.to_string(),
            symbol: symbol.to_string(),
            decimals,
            total_supply: initial_supply,
            balances,
            allowances: HashMap::new(),
        }
    }
}

impl IERC20 for ERC20 {
    fn balance_of(&self, account: &str) -> u64 {
        *self.balances.get(account).unwrap_or(&0)
    }
    
    fn transfer(&mut self, from: &str, to: &str, amount: u64) -> Result<(), String> {
        let from_balance = self.balance_of(from);
        if from_balance < amount {
            return Err("余额不足".to_string());
        }
        
        *self.balances.entry(from.to_string()).or_insert(0) -= amount;
        *self.balances.entry(to.to_string()).or_insert(0) += amount;
        
        Ok(())
    }
    
    fn approve(&mut self, owner: &str, spender: &str, amount: u64) {
        self.allowances.insert((owner.to_string(), spender.to_string()), amount);
    }
    
    fn transfer_from(&mut self, spender: &str, from: &str, to: &str, amount: u64) -> Result<(), String> {
        let allowance = *self.allowances.get(&(from.to_string(), spender.to_string())).unwrap_or(&0);
        if allowance < amount {
            return Err("授权额度不足".to_string());
        }
        
        self.transfer(from, to, amount)?;
        
        *self.allowances.get_mut(&(from.to_string(), spender.to_string())).unwrap() -= amount;
        
        Ok(())
    }
}

/// LP Token (流动性提供者代币)
pub struct LPToken {
    token: ERC20,
}

impl LPToken {
    pub fn new(pair_name: &str) -> Self {
        Self {
            token: ERC20::new(
                &format!("{} LP", pair_name),
                &format!("{}-LP", pair_name),
                18,
                0,
                "contract"
            ),
        }
    }
    
    pub fn mint(&mut self, to: &str, amount: u64) {
        *self.token.balances.entry(to.to_string()).or_insert(0) += amount;
        self.token.total_supply += amount;
    }
    
    pub fn burn(&mut self, from: &str, amount: u64) -> Result<(), String> {
        let balance = self.token.balance_of(from);
        if balance < amount {
            return Err("LP余额不足".to_string());
        }
        *self.token.balances.get_mut(from).unwrap() -= amount;
        self.token.total_supply -= amount;
        Ok(())
    }
    
    pub fn balance_of(&self, account: &str) -> u64 {
        self.token.balance_of(account)
    }
    
    pub fn total_supply(&self) -> u64 {
        self.token.total_supply
    }
}

/// Swap Pair (交易对)
pub struct SwapPair {
    pub token_a: String,           // 代币A地址
    pub token_b: String,           // 代币B地址
    pub reserve_a: u64,            // A储备量
    pub reserve_b: u64,            // B储备量
    pub lp_token: LPToken,         // LP代币
    pub fee_rate: u64,             // 手续费率 (0.3% = 3/1000)
}

impl SwapPair {
    pub fn new(token_a: &str, token_b: &str) -> Self {
        let pair_name = format!("{}-{}", token_a, token_b);
        
        Self {
            token_a: token_a.to_string(),
            token_b: token_b.to_string(),
            reserve_a: 0,
            reserve_b: 0,
            lp_token: LPToken::new(&pair_name),
            fee_rate: 3, // 0.3%
        }
    }
    
    /// 添加流动性
    /// 
    /// # 公式
    /// - 首次添加: liquidity = sqrt(amount_a * amount_b)
    /// - 后续添加: liquidity = min(amount_a * total_lp / reserve_a, amount_b * total_lp / reserve_b)
    pub fn add_liquidity(
        &mut self,
        provider: &str,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<u64, String> {
        // 计算LP代币数量
        let liquidity = if self.lp_token.total_supply() == 0 {
            // 首次添加，使用几何平均数
            ((amount_a as u128 * amount_b as u128).sqrt()) as u64
        } else {
            // 按现有比例添加
            let liquidity_a = (amount_a as u128 * self.lp_token.total_supply() as u128 / self.reserve_a as u128) as u64;
            let liquidity_b = (amount_b as u128 * self.lp_token.total_supply() as u128 / self.reserve_b as u128) as u64;
            liquidity_a.min(liquidity_b)
        };
        
        if liquidity == 0 {
            return Err("流动性不足".to_string());
        }
        
        // 更新储备
        self.reserve_a += amount_a;
        self.reserve_b += amount_b;
        
        // 铸造LP代币
        self.lp_token.mint(provider, liquidity);
        
        Ok(liquidity)
    }
    
    /// 移除流动性
    pub fn remove_liquidity(
        &mut self,
        provider: &str,
        liquidity: u64,
    ) -> Result<(u64, u64), String> {
        // 检查LP余额
        let lp_balance = self.lp_token.balance_of(provider);
        if lp_balance < liquidity {
            return Err("LP余额不足".to_string());
        }
        
        // 计算能取回的代币数量
        let total_supply = self.lp_token.total_supply();
        let amount_a = (liquidity as u128 * self.reserve_a as u128 / total_supply as u128) as u64;
        let amount_b = (liquidity as u128 * self.reserve_b as u128 / total_supply as u128) as u64;
        
        // 销毁LP代币
        self.lp_token.burn(provider, liquidity)?;
        
        // 更新储备
        self.reserve_a -= amount_a;
        self.reserve_b -= amount_b;
        
        Ok((amount_a, amount_b))
    }
    
    /// 交换代币 (A -> B)
    /// 
    /// # 公式 (恒定乘积)
    /// (reserve_a + amount_in) * (reserve_b - amount_out) = reserve_a * reserve_b
    /// 
    /// # 手续费
    /// 收取 0.3% 手续费，加入储备池给LP提供者
    pub fn swap_a_to_b(&mut self, amount_in: u64) -> u64 {
        // 计算含手续费的输入
        let amount_in_with_fee = amount_in * (1000 - self.fee_rate) / 1000;
        
        // 计算输出 (恒定乘积公式)
        // amount_out = reserve_b * amount_in_with_fee / (reserve_a + amount_in_with_fee)
        let numerator = self.reserve_b as u128 * amount_in_with_fee as u128;
        let denominator = self.reserve_a as u128 + amount_in_with_fee as u128;
        let amount_out = (numerator / denominator) as u64;
        
        // 更新储备
        self.reserve_a += amount_in;
        self.reserve_b -= amount_out;
        
        amount_out
    }
    
    /// 交换代币 (B -> A)
    pub fn swap_b_to_a(&mut self, amount_in: u64) -> u64 {
        let amount_in_with_fee = amount_in * (1000 - self.fee_rate) / 1000;
        
        let numerator = self.reserve_a as u128 * amount_in_with_fee as u128;
        let denominator = self.reserve_b as u128 + amount_in_with_fee as u128;
        let amount_out = (numerator / denominator) as u64;
        
        self.reserve_b += amount_in;
        self.reserve_a -= amount_out;
        
        amount_out
    }
    
    /// 获取当前价格 (A相对于B)
    pub fn get_price_a(&self) -> f64 {
        if self.reserve_b == 0 {
            return 0.0;
        }
        self.reserve_a as f64 / self.reserve_b as f64
    }
    
    /// 获取当前价格 (B相对于A)
    pub fn get_price_b(&self) -> f64 {
        if self.reserve_a == 0 {
            return 0.0;
        }
        self.reserve_b as f64 / self.reserve_a as f64
    }
}

/// Swap Router (路由合约)
pub struct SwapRouter {
    pairs: HashMap<(String, String), SwapPair>,
}

impl SwapRouter {
    pub fn new() -> Self {
        Self {
            pairs: HashMap::new(),
        }
    }
    
    /// 创建交易对
    pub fn create_pair(&mut self, token_a: &str, token_b: &str) -> Result<(), String> {
        let key = Self::get_pair_key(token_a, token_b);
        
        if self.pairs.contains_key(&key) {
            return Err("交易对已存在".to_string());
        }
        
        let pair = SwapPair::new(token_a, token_b);
        self.pairs.insert(key, pair);
        
        Ok(())
    }
    
    /// 获取交易对
    pub fn get_pair(&self, token_a: &str, token_b: &str) -> Option<&SwapPair> {
        let key = Self::get_pair_key(token_a, token_b);
        self.pairs.get(&key)
    }
    
    /// 获取交易对 (可变)
    pub fn get_pair_mut(&mut self, token_a: &str, token_b: &str) -> Option<&mut SwapPair> {
        let key = Self::get_pair_key(token_a, token_b);
        self.pairs.get_mut(&key)
    }
    
    /// 计算pair key (排序确保一致性)
    fn get_pair_key(token_a: &str, token_b: &str) -> (String, String) {
        if token_a < token_b {
            (token_a.to_string(), token_b.to_string())
        } else {
            (token_b.to_string(), token_a.to_string())
        }
    }
    
    /// 添加流动性
    pub fn add_liquidity(
        &mut self,
        token_a: &str,
        token_b: &str,
        provider: &str,
        amount_a: u64,
        amount_b: u64,
    ) -> Result<u64, String> {
        let pair = self.get_pair_mut(token_a, token_b)
            .ok_or("交易对不存在")?;
        
        pair.add_liquidity(provider, amount_a, amount_b)
    }
    
    /// 交换 (精确输入)
    pub fn swap_exact_tokens_for_tokens(
        &mut self,
        token_in: &str,
        token_out: &str,
        amount_in: u64,
        min_amount_out: u64,
    ) -> Result<u64, String> {
        let pair = self.get_pair_mut(token_in, token_out)
            .ok_or("交易对不存在")?;
        
        let amount_out = if pair.token_a == token_in {
            pair.swap_a_to_b(amount_in)
        } else {
            pair.swap_b_to_a(amount_in)
        };
        
        if amount_out < min_amount_out {
            return Err("滑点过高".to_string());
        }
        
        Ok(amount_out)
    }
}

/// 测试
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_swap_math() {
        let mut pair = SwapPair::new("TOKEN_A", "TOKEN_B");
        
        // 添加初始流动性 10000:10000 = 1:1
        pair.add_liquidity("alice", 10000, 10000).unwrap();
        
        assert_eq!(pair.reserve_a, 10000);
        assert_eq!(pair.reserve_b, 10000);
        
        // 用100 A换B，应该得到约99.7 B (扣除0.3%手续费)
        let amount_out = pair.swap_a_to_b(100);
        println!("100 A -> {} B", amount_out);
        
        // 验证k值增加 (手续费加入储备)
        let k_before = 10000u128 * 10000u128;
        let k_after = pair.reserve_a as u128 * pair.reserve_b as u128;
        assert!(k_after > k_before, "k值应该增加");
    }
    
    #[test]
    fn test_lp_token() {
        let mut pair = SwapPair::new("A", "B");
        
        // 首次添加流动性
        let lp1 = pair.add_liquidity("alice", 10000, 40000).unwrap();
        // liquidity = sqrt(10000 * 40000) = sqrt(400000000) = 20000
        assert_eq!(lp1, 20000);
        
        // 第二次按相同比例添加
        let lp2 = pair.add_liquidity("bob", 5000, 20000).unwrap();
        // 应该得到10000 LP
        assert_eq!(lp2, 10000);
        
        assert_eq!(pair.lp_token.total_supply(), 30000);
    }
}

/// 主函数 - 演示
fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║           ChengChain Swap DEX 演示                        ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    // 1. 创建代币
    println!("1️⃣  创建代币...");
    let mut token_a = ERC20::new("Cheng USD", "cUSD", 18, 1_000_000, "deployer");
    let mut token_b = ERC20::new("Cheng ETH", "cETH", 18, 500_000, "deployer");
    println!("   cUSD 总量: {}", token_a.total_supply);
    println!("   cETH 总量: {}", token_b.total_supply);
    
    // 2. 创建Router和交易对
    println!("\n2️⃣  创建交易对...");
    let mut router = SwapRouter::new();
    router.create_pair("cUSD", "cETH").unwrap();
    println!("   交易对: cUSD/cETH");
    
    // 3. 添加流动性
    println!("\n3️⃣  添加流动性...");
    let lp = router.add_liquidity("cUSD", "cETH", "alice", 10000, 20).unwrap();
    println!("   Alice 添加: 10000 cUSD + 20 cETH");
    println!("   获得 LP 代币: {}", lp);
    
    let pair = router.get_pair("cUSD", "cETH").unwrap();
    println!("   当前储备: {} cUSD / {} cETH", pair.reserve_a, pair.reserve_b);
    println!("   当前价格: 1 cETH = {} cUSD", pair.get_price_b());
    
    // 4. 执行交换
    println!("\n4️⃣  执行交换...");
    let amount_out = router.swap_exact_tokens_for_tokens("cUSD", "cETH", 100, 0).unwrap();
    println!("   用 100 cUSD 交换得到 {} cETH", amount_out);
    
    let pair = router.get_pair("cUSD", "cETH").unwrap();
    println!("   新储备: {} cUSD / {} cETH", pair.reserve_a, pair.reserve_b);
    println!("   新价格: 1 cETH = {} cUSD", pair.get_price_b());
    
    // 5. 再执行一次（展示价格影响）
    println!("\n5️⃣  大额交换（展示滑点）...");
    let amount_out2 = router.swap_exact_tokens_for_tokens("cUSD", "cETH", 1000, 0).unwrap();
    println!("   用 1000 cUSD 交换得到 {} cETH", amount_out2);
    println!("   实际汇率: 1 cETH = {:.2} cUSD", 1000.0 / amount_out2 as f64);
    
    let pair = router.get_pair("cUSD", "cETH").unwrap();
    println!("   最终储备: {} cUSD / {} cETH", pair.reserve_a, pair.reserve_b);
    
    // 6. 移除流动性
    println!("\n6️⃣  移除流动性...");
    let pair = router.get_pair_mut("cUSD", "cETH").unwrap();
    let (amount_a, amount_b) = pair.remove_liquidity("alice", lp / 2).unwrap();
    println!("   Alice 移除一半流动性");
    println!("   取回: {} cUSD + {} cETH", amount_a, amount_b);
    println!("   剩余 LP: {}", pair.lp_token.balance_of("alice"));
    
    println!("\n✨ DEX演示完成！");
}
