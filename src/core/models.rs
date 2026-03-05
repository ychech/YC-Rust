//! 双模式账户系统：UTXO + 账户模型
//! 
//! UTXO 模型（比特币风格）：
//! - 交易输入引用之前的输出
//! - 余额 = 所有未花费输出的总和
//! 
//! 账户模型（以太坊风格）：
//! - 每个地址有全局状态
//! - 余额直接存储在账户中

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 金额类型
pub type Amount = u64;

/// 地址类型
pub type Address = String;

/// 交易 ID
pub type TxId = String;

// ==================== UTXO 模型 ====================

/// UTXO 交易输入
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UtxoInput {
    /// 引用的交易 ID
    pub tx_id: TxId,
    /// 引用的输出索引
    pub vout: u32,
    /// 解锁脚本（签名）
    pub script_sig: ScriptSig,
}

/// UTXO 交易输出
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UtxoOutput {
    /// 金额
    pub value: Amount,
    /// 锁定脚本（接收地址）
    pub script_pubkey: Address,
}

/// 解锁脚本
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptSig {
    /// 签名
    pub signature: Vec<u8>,
    /// 公钥
    pub pubkey: Vec<u8>,
}

/// UTXO 交易
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UtxoTransaction {
    pub id: TxId,
    pub timestamp: i64,
    pub inputs: Vec<UtxoInput>,
    pub outputs: Vec<UtxoOutput>,
    pub version: u32,
}

impl UtxoTransaction {
    /// 创建 Coinbase 交易（挖矿奖励）
    pub fn new_coinbase(to: Address, reward: Amount, nonce: u64) -> Self {
        let input = UtxoInput {
            tx_id: "0".to_string(),
            vout: 0,
            script_sig: ScriptSig {
                signature: nonce.to_le_bytes().to_vec(),
                pubkey: vec![],
            },
        };

        let output = UtxoOutput {
            value: reward,
            script_pubkey: to,
        };

        let mut tx = UtxoTransaction {
            id: String::new(),
            timestamp: chrono::Utc::now().timestamp(),
            inputs: vec![input],
            outputs: vec![output],
            version: 1,
        };
        tx.id = tx.calculate_hash();
        tx
    }

    /// 计算交易哈希
    pub fn calculate_hash(&self) -> String {
        use sha2::{Sha256, Digest};
        let data = format!(
            "{}{:?}{:?}{}",
            self.timestamp, self.inputs, self.outputs, self.version
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// 是否为 Coinbase 交易
    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1 && self.inputs[0].tx_id == "0"
    }

    /// 获取序列化数据（用于签名）
    pub fn serialize_for_sign(&self) -> Vec<u8> {
        let inputs: Vec<String> = self.inputs.iter()
            .map(|i| format!("{}:{}", i.tx_id, i.vout))
            .collect();
        let outputs: Vec<String> = self.outputs.iter()
            .map(|o| format!("{}:{}", o.value, o.script_pubkey))
            .collect();
        
        format!("{:?}{:?}{}", inputs, outputs, self.version)
            .into_bytes()
    }
}

/// UTXO 集合
#[derive(Debug, Clone, Default)]
pub struct UtxoSet {
    /// (tx_id, vout) -> UtxoOutput
    pub utxos: HashMap<(TxId, u32), UtxoOutput>,
    /// 已花费的输出（防止双花）
    pub spent: HashMap<(TxId, u32), bool>,
}

impl UtxoSet {
    pub fn new() -> Self {
        UtxoSet {
            utxos: HashMap::new(),
            spent: HashMap::new(),
        }
    }

    /// 查找地址的可用 UTXO
    pub fn find_by_address(&self, address: &str) -> Vec<((TxId, u32), UtxoOutput)> {
        self.utxos.iter()
            .filter(|(k, v)| {
                v.script_pubkey == address && !self.spent.contains_key(k)
            })
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// 获取地址余额
    pub fn get_balance(&self, address: &str) -> Amount {
        self.find_by_address(address)
            .iter()
            .map(|(_, output)| output.value)
            .sum()
    }

    /// 花费 UTXO
    pub fn spend(&mut self, tx_id: &str, vout: u32) -> Result<(), String> {
        let key = (tx_id.to_string(), vout);
        if !self.utxos.contains_key(&key) {
            return Err("UTXO not found".to_string());
        }
        if self.spent.contains_key(&key) {
            return Err("UTXO already spent".to_string());
        }
        self.spent.insert(key, true);
        Ok(())
    }

    /// 添加新的 UTXO
    pub fn add(&mut self, tx_id: &str, vout: u32, output: UtxoOutput) {
        self.utxos.insert((tx_id.to_string(), vout), output);
    }

    /// 从交易中更新 UTXO 集合
    pub fn update_from_tx(&mut self, tx: &UtxoTransaction) {
        // 1. 花费输入
        if !tx.is_coinbase() {
            for input in &tx.inputs {
                self.spend(&input.tx_id, input.vout).ok();
            }
        }

        // 2. 添加新的输出
        for (vout, output) in tx.outputs.iter().enumerate() {
            self.add(&tx.id, vout as u32, output.clone());
        }
    }
}

// ==================== 账户模型 ====================

/// 账户状态
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Account {
    /// 地址
    pub address: Address,
    /// 余额
    pub balance: Amount,
    /// 交易计数器（用于防止重放攻击）
    pub nonce: u64,
    /// 代码（如果是合约账户）
    pub code: Option<Vec<u8>>,
    /// 存储（合约账户使用）
    pub storage: HashMap<String, Vec<u8>>,
}

impl Account {
    pub fn new(address: Address) -> Self {
        Account {
            address,
            balance: 0,
            nonce: 0,
            code: None,
            storage: HashMap::new(),
        }
    }

    pub fn is_contract(&self) -> bool {
        self.code.is_some()
    }
}

/// 账户模型交易
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountTransaction {
    pub id: TxId,
    pub from: Address,
    pub to: Address,
    pub value: Amount,
    pub nonce: u64,
    pub gas_price: Amount,
    pub gas_limit: u64,
    pub data: Vec<u8>,
    pub timestamp: i64,
    pub signature: Vec<u8>,
}

/// 世界状态（所有账户的集合）
#[derive(Debug, Clone, Default)]
pub struct WorldState {
    pub accounts: HashMap<Address, Account>,
}

impl WorldState {
    pub fn new() -> Self {
        WorldState {
            accounts: HashMap::new(),
        }
    }

    /// 获取或创建账户
    pub fn get_or_create(&mut self, address: &str) -> &mut Account {
        self.accounts.entry(address.to_string())
            .or_insert_with(|| Account::new(address.to_string()))
    }

    /// 获取账户余额
    pub fn get_balance(&self, address: &str) -> Amount {
        self.accounts.get(address)
            .map(|acc| acc.balance)
            .unwrap_or(0)
    }

    /// 转账
    pub fn transfer(&mut self, from: &str, to: &str, amount: Amount) -> Result<(), String> {
        let from_balance = self.get_balance(from);
        if from_balance < amount {
            return Err("Insufficient balance".to_string());
        }

        // 扣减发送方
        let from_acc = self.get_or_create(from);
        from_acc.balance -= amount;
        from_acc.nonce += 1;

        // 增加接收方
        let to_acc = self.get_or_create(to);
        to_acc.balance += amount;

        Ok(())
    }

    /// 执行账户模型交易
    pub fn execute_tx(&mut self, tx: &AccountTransaction) -> Result<(), String> {
        // 验证 nonce
        let from_acc = self.get_or_create(&tx.from);
        if from_acc.nonce != tx.nonce {
            return Err(format!(
                "Invalid nonce: expected {}, got {}",
                from_acc.nonce, tx.nonce
            ));
        }

        // 转账
        self.transfer(&tx.from, &tx.to, tx.value)?;

        // 如果是合约调用，执行合约代码
        if !tx.data.is_empty() {
            // TODO: 执行合约
        }

        Ok(())
    }
}

// ==================== 统一交易类型 ====================

/// 通用交易枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Transaction {
    Utxo(UtxoTransaction),
    Account(AccountTransaction),
}

/// 统一账户接口
pub trait AccountModel {
    fn get_balance(&self, address: &str) -> Amount;
    fn transfer(&mut self, from: &str, to: &str, amount: Amount) -> Result<(), String>;
}

impl AccountModel for UtxoSet {
    fn get_balance(&self, address: &str) -> Amount {
        self.get_balance(address)
    }

    fn transfer(&mut self, _from: &str, _to: &str, _amount: Amount) -> Result<(), String> {
        // UTXO 模型不直接支持 transfer，需要通过交易
        Err("Use create_transaction for UTXO model".to_string())
    }
}

impl AccountModel for WorldState {
    fn get_balance(&self, address: &str) -> Amount {
        self.get_balance(address)
    }

    fn transfer(&mut self, from: &str, to: &str, amount: Amount) -> Result<(), String> {
        self.transfer(from, to, amount)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utxo() {
        let mut utxo_set = UtxoSet::new();
        
        // 添加 UTXO
        let tx = UtxoTransaction::new_coinbase("addr1".to_string(), 100, 0);
        utxo_set.update_from_tx(&tx);
        
        // 检查余额
        assert_eq!(utxo_set.get_balance("addr1"), 100);
    }

    #[test]
    fn test_account_model() {
        let mut state = WorldState::new();
        
        // 创建账户并充值
        let acc = state.get_or_create("addr1");
        acc.balance = 1000;
        
        // 转账
        state.transfer("addr1", "addr2", 300).unwrap();
        
        // 验证余额
        assert_eq!(state.get_balance("addr1"), 700);
        assert_eq!(state.get_balance("addr2"), 300);
    }

    #[test]
    fn test_insufficient_balance() {
        let mut state = WorldState::new();
        
        let acc = state.get_or_create("addr1");
        acc.balance = 100;
        
        let result = state.transfer("addr1", "addr2", 200);
        assert!(result.is_err());
    }
}
