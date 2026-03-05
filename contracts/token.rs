//! YRUST Token (YRU)
//! 
//! ERC-20 风格的代币合约
//! 总供应量: 21,000,000 YRU (模仿比特币)
//! 精度: 8 位小数

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// 代币合约状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenContract {
    /// 代币名称
    pub name: String,
    /// 代币符号
    pub symbol: String,
    /// 精度 (小数位数)
    pub decimals: u8,
    /// 总供应量
    pub total_supply: u64,
    /// 余额表: address -> balance
    pub balances: HashMap<String, u64>,
    /// 授权表: (owner, spender) -> amount
    pub allowances: HashMap<(String, String), u64>,
    /// 合约所有者
    pub owner: String,
}

/// 转账事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferEvent {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

/// 授权事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalEvent {
    pub owner: String,
    pub spender: String,
    pub amount: u64,
}

impl TokenContract {
    /// 创建新代币
    pub fn new(name: String, symbol: String, decimals: u8, owner: String) -> Self {
        // 总供应量: 21,000,000 * 10^8 = 2,100,000,000,000,000
        let total_supply = 21_000_000u64 * 10u64.pow(decimals as u32);
        
        let mut contract = TokenContract {
            name,
            symbol,
            decimals,
            total_supply,
            balances: HashMap::new(),
            allowances: HashMap::new(),
            owner: owner.clone(),
        };
        
        // 将所有代币铸造给创建者
        contract.balances.insert(owner, total_supply);
        
        contract
    }
    
    /// 获取代币名称
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// 获取代币符号
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
    
    /// 获取精度
    pub fn decimals(&self) -> u8 {
        self.decimals
    }
    
    /// 获取总供应量
    pub fn total_supply(&self) -> u64 {
        self.total_supply
    }
    
    /// 查询余额
    pub fn balance_of(&self, account: &str) -> u64 {
        self.balances.get(account).copied().unwrap_or(0)
    }
    
    /// 转账
    pub fn transfer(&mut self, from: &str, to: &str, amount: u64) -> Result<(), TokenError> {
        let from_balance = self.balance_of(from);
        
        if from_balance < amount {
            return Err(TokenError::InsufficientBalance);
        }
        
        // 扣减发送方余额
        self.balances.insert(from.to_string(), from_balance - amount);
        
        // 增加接收方余额
        let to_balance = self.balance_of(to);
        self.balances.insert(to.to_string(), to_balance + amount);
        
        // 触发事件
        let _event = TransferEvent {
            from: from.to_string(),
            to: to.to_string(),
            amount,
        };
        
        Ok(())
    }
    
    /// 授权
    pub fn approve(&mut self, owner: &str, spender: &str, amount: u64) -> Result<(), TokenError> {
        self.allowances.insert((owner.to_string(), spender.to_string()), amount);
        
        // 触发事件
        let _event = ApprovalEvent {
            owner: owner.to_string(),
            spender: spender.to_string(),
            amount,
        };
        
        Ok(())
    }
    
    /// 查询授权额度
    pub fn allowance(&self, owner: &str, spender: &str) -> u64 {
        self.allowances.get(&(owner.to_string(), spender.to_string())).copied().unwrap_or(0)
    }
    
    /// 从授权额度转账
    pub fn transfer_from(&mut self, spender: &str, from: &str, to: &str, amount: u64) -> Result<(), TokenError> {
        let allowed = self.allowance(from, spender);
        
        if allowed < amount {
            return Err(TokenError::InsufficientAllowance);
        }
        
        // 执行转账
        self.transfer(from, to, amount)?;
        
        // 减少授权额度
        self.allowances.insert((from.to_string(), spender.to_string()), allowed - amount);
        
        Ok(())
    }
    
    /// 铸造代币 (仅所有者)
    pub fn mint(&mut self, to: &str, amount: u64, caller: &str) -> Result<(), TokenError> {
        if caller != self.owner {
            return Err(TokenError::NotOwner);
        }
        
        // 增加总供应量
        self.total_supply += amount;
        
        // 增加接收方余额
        let to_balance = self.balance_of(to);
        self.balances.insert(to.to_string(), to_balance + amount);
        
        Ok(())
    }
    
    /// 销毁代币
    pub fn burn(&mut self, from: &str, amount: u64) -> Result<(), TokenError> {
        let from_balance = self.balance_of(from);
        
        if from_balance < amount {
            return Err(TokenError::InsufficientBalance);
        }
        
        // 减少余额
        self.balances.insert(from.to_string(), from_balance - amount);
        
        // 减少总供应量
        self.total_supply -= amount;
        
        Ok(())
    }
    
    /// 格式化显示金额 (考虑精度)
    pub fn format_amount(&self, amount: u64) -> String {
        let divisor = 10u64.pow(self.decimals as u32);
        let integer_part = amount / divisor;
        let decimal_part = amount % divisor;
        
        format!("{}.{:0>8} {}", integer_part, decimal_part, self.symbol)
    }
}

/// 代币错误
#[derive(Debug, Clone)]
pub enum TokenError {
    InsufficientBalance,
    InsufficientAllowance,
    NotOwner,
}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenError::InsufficientBalance => write!(f, "余额不足"),
            TokenError::InsufficientAllowance => write!(f, "授权额度不足"),
            TokenError::NotOwner => write!(f, "不是合约所有者"),
        }
    }
}

impl std::error::Error for TokenError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_token_creation() {
        let token = TokenContract::new(
            "YRUST Token".to_string(),
            "YRU".to_string(),
            8,
            "alice".to_string(),
        );
        
        assert_eq!(token.name(), "YRUST Token");
        assert_eq!(token.symbol(), "YRU");
        assert_eq!(token.decimals(), 8);
        assert_eq!(token.total_supply(), 21_000_000 * 10u64.pow(8));
        assert_eq!(token.balance_of("alice"), token.total_supply());
    }
    
    #[test]
    fn test_transfer() {
        let mut token = TokenContract::new(
            "YRUST Token".to_string(),
            "YRU".to_string(),
            8,
            "alice".to_string(),
        );
        
        let amount = 1000 * 10u64.pow(8); // 1000 YRU
        token.transfer("alice", "bob", amount).unwrap();
        
        assert_eq!(token.balance_of("bob"), amount);
        assert_eq!(token.balance_of("alice"), token.total_supply() - amount);
    }
    
    #[test]
    fn test_approve_and_transfer_from() {
        let mut token = TokenContract::new(
            "YRUST Token".to_string(),
            "YRU".to_string(),
            8,
            "alice".to_string(),
        );
        
        // Alice 授权 Bob 可以使用 1000 YRU
        let amount = 1000 * 10u64.pow(8);
        token.approve("alice", "bob", amount).unwrap();
        
        assert_eq!(token.allowance("alice", "bob"), amount);
        
        // Bob 从 Alice 账户转账给 Charlie
        token.transfer_from("bob", "alice", "charlie", amount / 2).unwrap();
        
        assert_eq!(token.balance_of("charlie"), amount / 2);
        assert_eq!(token.allowance("alice", "bob"), amount / 2);
    }
    
    #[test]
    fn test_mint() {
        let mut token = TokenContract::new(
            "YRUST Token".to_string(),
            "YRU".to_string(),
            8,
            "alice".to_string(),
        );
        
        let initial_supply = token.total_supply();
        let mint_amount = 1000 * 10u64.pow(8);
        
        token.mint("bob", mint_amount, "alice").unwrap();
        
        assert_eq!(token.total_supply(), initial_supply + mint_amount);
        assert_eq!(token.balance_of("bob"), mint_amount);
    }
    
    #[test]
    fn test_burn() {
        let mut token = TokenContract::new(
            "YRUST Token".to_string(),
            "YRU".to_string(),
            8,
            "alice".to_string(),
        );
        
        let initial_supply = token.total_supply();
        let burn_amount = 1000 * 10u64.pow(8);
        
        token.burn("alice", burn_amount).unwrap();
        
        assert_eq!(token.total_supply(), initial_supply - burn_amount);
        assert_eq!(token.balance_of("alice"), initial_supply - burn_amount);
    }
}
