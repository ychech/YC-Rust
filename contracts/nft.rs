//! NFT 合约 (Non-Fungible Token)
//! 
//! ERC-721 风格的 NFT 合约
//! 支持：铸造、转账、授权、元数据

use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// NFT 合约
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTContract {
    /// 合约名称
    pub name: String,
    /// 合约符号
    pub symbol: String,
    /// Token ID -> 所有者
    pub owners: HashMap<u64, String>,
    /// 所有者 -> Token 数量
    pub balances: HashMap<String, u64>,
    /// Token ID -> 授权地址
    pub token_approvals: HashMap<u64, String>,
    /// 所有者 -> 操作员授权
    pub operator_approvals: HashMap<(String, String), bool>,
    /// Token ID -> 元数据 URI
    pub token_uris: HashMap<u64, String>,
    /// 当前最大 Token ID
    pub current_token_id: u64,
    /// 合约所有者
    pub owner: String,
}

/// NFT 元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTMetadata {
    pub name: String,
    pub description: String,
    pub image: String,
    pub attributes: Vec<NFTAttribute>,
}

/// NFT 属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTAttribute {
    pub trait_type: String,
    pub value: String,
}

/// 转账事件
#[derive(Debug, Clone)]
pub struct TransferEvent {
    pub from: String,
    pub to: String,
    pub token_id: u64,
}

/// 授权事件
#[derive(Debug, Clone)]
pub struct ApprovalEvent {
    pub owner: String,
    pub approved: String,
    pub token_id: u64,
}

impl NFTContract {
    /// 创建新 NFT 合约
    pub fn new(name: String, symbol: String, owner: String) -> Self {
        NFTContract {
            name,
            symbol,
            owners: HashMap::new(),
            balances: HashMap::new(),
            token_approvals: HashMap::new(),
            operator_approvals: HashMap::new(),
            token_uris: HashMap::new(),
            current_token_id: 0,
            owner,
        }
    }
    
    /// 获取合约名称
    pub fn name(&self) -> &str {
        &self.name
    }
    
    /// 获取合约符号
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
    
    /// 查询所有者
    pub fn owner_of(&self, token_id: u64) -> Option<String> {
        self.owners.get(&token_id).cloned()
    }
    
    /// 查询余额（拥有的 NFT 数量）
    pub fn balance_of(&self, owner: &str) -> u64 {
        self.balances.get(owner).copied().unwrap_or(0)
    }
    
    /// 铸造 NFT
    pub fn mint(&mut self, to: &str, uri: String, caller: &str) -> Result<u64, NFTError> {
        if caller != self.owner {
            return Err(NFTError::NotOwner);
        }
        
        // 生成新的 Token ID
        self.current_token_id += 1;
        let token_id = self.current_token_id;
        
        // 设置所有者
        self.owners.insert(token_id, to.to_string());
        
        // 更新余额
        let balance = self.balance_of(to);
        self.balances.insert(to.to_string(), balance + 1);
        
        // 设置元数据 URI
        self.token_uris.insert(token_id, uri);
        
        // 触发事件
        let _event = TransferEvent {
            from: "0".to_string(),
            to: to.to_string(),
            token_id,
        };
        
        Ok(token_id)
    }
    
    /// 批量铸造
    pub fn mint_batch(&mut self, to: &str, uris: Vec<String>, caller: &str) -> Result<Vec<u64>, NFTError> {
        let mut token_ids = Vec::new();
        
        for uri in uris {
            let token_id = self.mint(to, uri, caller)?;
            token_ids.push(token_id);
        }
        
        Ok(token_ids)
    }
    
    /// 转账
    pub fn transfer(&mut self, from: &str, to: &str, token_id: u64, caller: &str) -> Result<(), NFTError> {
        // 检查所有权
        let owner = self.owner_of(token_id)
            .ok_or(NFTError::TokenNotFound)?;
        
        if owner != from {
            return Err(NFTError::NotOwner);
        }
        
        // 检查调用者权限
        if !self.is_approved_or_owner(caller, token_id) {
            return Err(NFTError::NotAuthorized);
        }
        
        // 执行转账
        self.owners.insert(token_id, to.to_string());
        
        // 更新余额
        let from_balance = self.balance_of(from);
        self.balances.insert(from.to_string(), from_balance - 1);
        
        let to_balance = self.balance_of(to);
        self.balances.insert(to.to_string(), to_balance + 1);
        
        // 清除授权
        self.token_approvals.remove(&token_id);
        
        // 触发事件
        let _event = TransferEvent {
            from: from.to_string(),
            to: to.to_string(),
            token_id,
        };
        
        Ok(())
    }
    
    /// 授权单个 NFT
    pub fn approve(&mut self, approved: &str, token_id: u64, caller: &str) -> Result<(), NFTError> {
        let owner = self.owner_of(token_id)
            .ok_or(NFTError::TokenNotFound)?;
        
        if caller != owner && !self.is_approved_for_all(&owner, caller) {
            return Err(NFTError::NotAuthorized);
        }
        
        self.token_approvals.insert(token_id, approved.to_string());
        
        // 触发事件
        let _event = ApprovalEvent {
            owner: owner.to_string(),
            approved: approved.to_string(),
            token_id,
        };
        
        Ok(())
    }
    
    /// 获取授权地址
    pub fn get_approved(&self, token_id: u64) -> Option<String> {
        self.token_approvals.get(&token_id).cloned()
    }
    
    /// 授权/取消授权操作员
    pub fn set_approval_for_all(&mut self, operator: &str, approved: bool, caller: &str) -> Result<(), NFTError> {
        self.operator_approvals.insert((caller.to_string(), operator.to_string()), approved);
        Ok(())
    }
    
    /// 查询是否授权操作员
    pub fn is_approved_for_all(&self, owner: &str, operator: &str) -> bool {
        self.operator_approvals.get(&(owner.to_string(), operator.to_string())).copied().unwrap_or(false)
    }
    
    /// 检查是否是所有者或已授权
    fn is_approved_or_owner(&self, spender: &str, token_id: u64) -> bool {
        let owner = match self.owner_of(token_id) {
            Some(o) => o,
            None => return false,
        };
        
        if spender == owner {
            return true;
        }
        
        if let Some(approved) = self.get_approved(token_id) {
            if spender == approved {
                return true;
            }
        }
        
        self.is_approved_for_all(&owner, spender)
    }
    
    /// 获取 Token URI
    pub fn token_uri(&self, token_id: u64) -> Option<String> {
        self.token_uris.get(&token_id).cloned()
    }
    
    /// 销毁 NFT
    pub fn burn(&mut self, token_id: u64, caller: &str) -> Result<(), NFTError> {
        let owner = self.owner_of(token_id)
            .ok_or(NFTError::TokenNotFound)?;
        
        if !self.is_approved_or_owner(caller, token_id) {
            return Err(NFTError::NotAuthorized);
        }
        
        // 清除数据
        self.owners.remove(&token_id);
        self.token_approvals.remove(&token_id);
        self.token_uris.remove(&token_id);
        
        // 更新余额
        let balance = self.balance_of(&owner);
        self.balances.insert(owner, balance - 1);
        
        Ok(())
    }
    
    /// 获取某地址拥有的所有 Token ID
    pub fn tokens_of_owner(&self, owner: &str) -> Vec<u64> {
        self.owners.iter()
            .filter(|(_, o)| o == owner)
            .map(|(id, _)| *id)
            .collect()
    }
    
    /// 获取总供应量
    pub fn total_supply(&self) -> u64 {
        self.owners.len() as u64
    }
}

/// NFT 错误
#[derive(Debug, Clone)]
pub enum NFTError {
    NotOwner,
    NotAuthorized,
    TokenNotFound,
    AlreadyExists,
}

impl std::fmt::Display for NFTError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NFTError::NotOwner => write!(f, "不是 Token 所有者"),
            NFTError::NotAuthorized => write!(f, "未授权"),
            NFTError::TokenNotFound => write!(f, "Token 不存在"),
            NFTError::AlreadyExists => write!(f, "Token 已存在"),
        }
    }
}

impl std::error::Error for NFTError {}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_nft_creation() {
        let nft = NFTContract::new(
            "CryptoArt".to_string(),
            "ART".to_string(),
            "alice".to_string(),
        );
        
        assert_eq!(nft.name(), "CryptoArt");
        assert_eq!(nft.symbol(), "ART");
        assert_eq!(nft.total_supply(), 0);
    }
    
    #[test]
    fn test_mint() {
        let mut nft = NFTContract::new(
            "CryptoArt".to_string(),
            "ART".to_string(),
            "alice".to_string(),
        );
        
        let token_id = nft.mint("bob", "https://example.com/1.json", "alice").unwrap();
        
        assert_eq!(token_id, 1);
        assert_eq!(nft.owner_of(token_id), Some("bob".to_string()));
        assert_eq!(nft.balance_of("bob"), 1);
        assert_eq!(nft.total_supply(), 1);
    }
    
    #[test]
    fn test_transfer() {
        let mut nft = NFTContract::new(
            "CryptoArt".to_string(),
            "ART".to_string(),
            "alice".to_string(),
        );
        
        let token_id = nft.mint("bob", "https://example.com/1.json", "alice").unwrap();
        
        nft.transfer("bob", "charlie", token_id, "bob").unwrap();
        
        assert_eq!(nft.owner_of(token_id), Some("charlie".to_string()));
        assert_eq!(nft.balance_of("bob"), 0);
        assert_eq!(nft.balance_of("charlie"), 1);
    }
    
    #[test]
    fn test_approve() {
        let mut nft = NFTContract::new(
            "CryptoArt".to_string(),
            "ART".to_string(),
            "alice".to_string(),
        );
        
        let token_id = nft.mint("bob", "https://example.com/1.json", "alice").unwrap();
        
        nft.approve("charlie", token_id, "bob").unwrap();
        
        assert_eq!(nft.get_approved(token_id), Some("charlie".to_string()));
        
        // Charlie 可以转账
        nft.transfer("bob", "dave", token_id, "charlie").unwrap();
        assert_eq!(nft.owner_of(token_id), Some("dave".to_string()));
    }
    
    #[test]
    fn test_burn() {
        let mut nft = NFTContract::new(
            "CryptoArt".to_string(),
            "ART".to_string(),
            "alice".to_string(),
        );
        
        let token_id = nft.mint("bob", "https://example.com/1.json", "alice").unwrap();
        
        nft.burn(token_id, "bob").unwrap();
        
        assert_eq!(nft.owner_of(token_id), None);
        assert_eq!(nft.balance_of("bob"), 0);
        assert_eq!(nft.total_supply(), 0);
    }
}
