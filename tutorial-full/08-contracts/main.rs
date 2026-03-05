//! 08. 智能合约 - Token、NFT、DEX实现

use std::collections::HashMap;

/// Token合约
pub struct TokenContract {
    name: String,
    symbol: String,
    total_supply: u64,
    balances: HashMap<String, u64>,
    allowances: HashMap<(String, String), u64>, // (owner, spender) => amount
}

impl TokenContract {
    pub fn new(name: String, symbol: String, initial_supply: u64, creator: String) -> Self {
        let mut balances = HashMap::new();
        balances.insert(creator.clone(), initial_supply);
        
        Self {
            name,
            symbol,
            total_supply: initial_supply,
            balances,
            allowances: HashMap::new(),
        }
    }
    
    pub fn balance_of(&self, account: &str) -> u64 {
        *self.balances.get(account).unwrap_or(&0)
    }
    
    pub fn transfer(&mut self, from: &str, to: &str, amount: u64) -> Result<(), String> {
        let from_balance = self.balance_of(from);
        if from_balance < amount {
            return Err("余额不足".to_string());
        }
        
        *self.balances.entry(from.to_string()).or_insert(0) -= amount;
        *self.balances.entry(to.to_string()).or_insert(0) += amount;
        
        Ok(())
    }
}

/// NFT合约
pub struct NFTContract {
    name: String,
    tokens: HashMap<u64, String>, // token_id => owner
    token_uris: HashMap<u64, String>,
    next_token_id: u64,
}

impl NFTContract {
    pub fn new(name: String) -> Self {
        Self {
            name,
            tokens: HashMap::new(),
            token_uris: HashMap::new(),
            next_token_id: 1,
        }
    }
    
    pub fn mint(&mut self, to: String, uri: String) -> u64 {
        let token_id = self.next_token_id;
        self.tokens.insert(token_id, to);
        self.token_uris.insert(token_id, uri);
        self.next_token_id += 1;
        token_id
    }
    
    pub fn owner_of(&self, token_id: u64) -> Option<String> {
        self.tokens.get(&token_id).cloned()
    }
    
    pub fn transfer(&mut self, from: &str, to: &str, token_id: u64) -> Result<(), String> {
        match self.tokens.get(&token_id) {
            Some(owner) if owner == from => {
                self.tokens.insert(token_id, to.to_string());
                Ok(())
            }
            _ => Err("无权转移".to_string()),
        }
    }
}

/// AMM DEX合约
pub struct AMMContract {
    token_a_reserve: u64,
    token_b_reserve: u64,
    total_liquidity: u64,
    liquidity_providers: HashMap<String, u64>,
}

impl AMMContract {
    pub fn new() -> Self {
        Self {
            token_a_reserve: 0,
            token_b_reserve: 0,
            total_liquidity: 0,
            liquidity_providers: HashMap::new(),
        }
    }
    
    /// 添加流动性
    pub fn add_liquidity(&mut self, provider: String, amount_a: u64, amount_b: u64) -> u64 {
        let liquidity = if self.total_liquidity == 0 {
            (amount_a * amount_b).sqrt() as u64
        } else {
            let liquidity_a = (amount_a * self.total_liquidity) / self.token_a_reserve;
            let liquidity_b = (amount_b * self.total_liquidity) / self.token_b_reserve;
            liquidity_a.min(liquidity_b)
        };
        
        self.token_a_reserve += amount_a;
        self.token_b_reserve += amount_b;
        self.total_liquidity += liquidity;
        *self.liquidity_providers.entry(provider).or_insert(0) += liquidity;
        
        liquidity
    }
    
    /// 交换代币（恒定乘积公式: x * y = k）
    pub fn swap(&mut self, amount_in: u64, is_a_to_b: bool) -> u64 {
        // 收取0.3%手续费
        let amount_in_with_fee = amount_in * 997 / 1000;
        
        if is_a_to_b {
            let amount_out = (amount_in_with_fee * self.token_b_reserve) 
                / (self.token_a_reserve + amount_in_with_fee);
            self.token_a_reserve += amount_in;
            self.token_b_reserve -= amount_out;
            amount_out
        } else {
            let amount_out = (amount_in_with_fee * self.token_a_reserve) 
                / (self.token_b_reserve + amount_in_with_fee);
            self.token_b_reserve += amount_in;
            self.token_a_reserve -= amount_out;
            amount_out
        }
    }
}

fn main() {
    println!("智能合约演示\n");
    
    // Token合约演示
    println!("=== Token合约 ===");
    let mut token = TokenContract::new(
        "YRUST Token".to_string(),
        "YRU".to_string(),
        1_000_000,
        "Alice".to_string(),
    );
    println!("创建代币: {} ({})，总量: {}", token.name, token.symbol, token.total_supply);
    
    token.transfer("Alice", "Bob", 1000).unwrap();
    println!("Alice余额: {}", token.balance_of("Alice"));
    println!("Bob余额: {}\n", token.balance_of("Bob"));
    
    // NFT合约演示
    println!("=== NFT合约 ===");
    let mut nft = NFTContract::new("YRUST NFT".to_string());
    let token_id = nft.mint("Alice".to_string(), "https://yrust.chain/nft/1".to_string());
    println!("铸造NFT #{} 给 Alice", token_id);
    println!("NFT #{} 所有者: {:?}", token_id, nft.owner_of(token_id));
    
    nft.transfer("Alice", "Bob", token_id).unwrap();
    println!("转移后所有者: {:?}\n", nft.owner_of(token_id));
    
    // DEX合约演示
    println!("=== DEX合约 (AMM) ===");
    let mut dex = AMMContract::new();
    dex.add_liquidity("LP1".to_string(), 10000, 10000);
    println!("添加流动性: 10000 A + 10000 B");
    println!("储备: A={}, B={}", dex.token_a_reserve, dex.token_b_reserve);
    
    let out = dex.swap(100, true);
    println!("用100 A交换得到 {} B", out);
    println!("新储备: A={}, B={}", dex.token_a_reserve, dex.token_b_reserve);
}
