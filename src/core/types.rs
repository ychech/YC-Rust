//! 核心类型定义
//! 包含区块、交易、UTXO 等核心数据结构

use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;

/// 区块哈希类型
pub type Hash = String;
/// 地址类型
pub type Address = String;
/// 金额类型 (使用 u64 表示最小单位，避免浮点精度问题)
pub type Amount = u64;
/// 交易 ID 类型
pub type TxId = String;

/// 区块难度（前导零个数）
pub const DIFFICULTY: usize = 4;
/// 挖矿奖励
pub const MINING_REWARD: Amount = 100_000_000; // 1.0 币 = 100_000_000 最小单位
/// 创世区块数据
pub const GENESIS_DATA: &str = "YRust Chain Genesis Block - 2024";

/// 交易输入 (UTXO 模型)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxInput {
    /// 引用的交易 ID
    pub tx_id: TxId,
    /// 引用的输出索引
    pub vout: u32,
    /// 解锁脚本 (签名 + 公钥)
    pub script_sig: ScriptSig,
}

/// 交易输出
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxOutput {
    /// 金额
    pub value: Amount,
    /// 锁定脚本 (接收地址)
    pub script_pubkey: String,
}

/// 解锁脚本 (包含签名和公钥)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScriptSig {
    /// 签名 (DER 格式)
    pub signature: Vec<u8>,
    /// 公钥 (压缩格式)
    pub pubkey: Vec<u8>,
}

/// 交易结构
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Transaction {
    /// 交易 ID (交易的哈希)
    pub id: TxId,
    /// 交易时间戳
    pub timestamp: i64,
    /// 交易输入列表
    pub inputs: Vec<TxInput>,
    /// 交易输出列表
    pub outputs: Vec<TxOutput>,
    /// 交易版本
    pub version: u32,
}

impl Transaction {
    /// 创建新交易
    pub fn new(inputs: Vec<TxInput>, outputs: Vec<TxOutput>) -> Self {
        let mut tx = Transaction {
            id: String::new(),
            timestamp: Utc::now().timestamp(),
            inputs,
            outputs,
            version: 1,
        };
        tx.id = tx.calculate_hash();
        tx
    }

    /// 创建 Coinbase 交易 (挖矿奖励)
    pub fn new_coinbase(to: Address, reward: Amount, nonce: u64) -> Self {
        let input = TxInput {
            tx_id: String::from("0"),
            vout: 0,
            script_sig: ScriptSig {
                signature: nonce.to_le_bytes().to_vec(),
                pubkey: vec![],
            },
        };

        let output = TxOutput {
            value: reward,
            script_pubkey: to,
        };

        let mut tx = Transaction {
            id: String::new(),
            timestamp: Utc::now().timestamp(),
            inputs: vec![input],
            outputs: vec![output],
            version: 1,
        };
        tx.id = tx.calculate_hash();
        tx
    }

    /// 计算交易哈希
    pub fn calculate_hash(&self) -> String {
        let data = format!(
            "{}{:?}{:?}{}",
            self.timestamp, self.inputs, self.outputs, self.version
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// 获取交易序列化数据 (用于签名)
    /// 注意：不包含签名和公钥，因为这些是在签名后添加的
    pub fn serialize_for_sign(&self) -> Vec<u8> {
        // 序列化时不包含签名数据，只包含交易的基本信息
        // 输入只包含 tx_id 和 vout，不包含 script_sig
        let inputs_data: Vec<String> = self.inputs.iter().map(|i| {
            format!("{}:{}", i.tx_id, i.vout)
        }).collect();
        
        let outputs_data: Vec<String> = self.outputs.iter().map(|o| {
            format!("{}:{}", o.value, o.script_pubkey)
        }).collect();
        
        let data = format!(
            "{:?}{:?}{}",
            inputs_data, outputs_data, self.version
        );
        data.into_bytes()
    }
    
    /// 获取交易序列化数据 (用于验证签名)
    /// 验证时使用相同的序列化方式
    pub fn serialize_for_verify(&self) -> Vec<u8> {
        self.serialize_for_sign()
    }

    /// 验证交易
    pub fn is_valid(&self) -> bool {
        // 1. 验证交易 ID
        if self.id != self.calculate_hash() {
            return false;
        }

        // 2. Coinbase 交易特殊处理
        if self.is_coinbase() {
            return self.inputs.len() == 1 
                && self.inputs[0].tx_id == "0" 
                && self.outputs.len() == 1;
        }

        // 3. 普通交易验证
        if self.inputs.is_empty() || self.outputs.is_empty() {
            return false;
        }

        true
    }

    /// 检查是否为 Coinbase 交易
    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1 && self.inputs[0].tx_id == "0"
    }

    /// 计算交易手续费
    pub fn calculate_fee(&self, utxo_set: &std::collections::HashMap<(TxId, u32), TxOutput>) -> Amount {
        if self.is_coinbase() {
            return 0;
        }

        let input_sum: Amount = self.inputs.iter()
            .filter_map(|input| {
                utxo_set.get(&(input.tx_id.clone(), input.vout))
                    .map(|output| output.value)
            })
            .sum();

        let output_sum: Amount = self.outputs.iter().map(|o| o.value).sum();

        input_sum.saturating_sub(output_sum)
    }
}

/// 区块头
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BlockHeader {
    /// 区块版本
    pub version: u32,
    /// 上一个区块的哈希
    pub prev_hash: Hash,
    /// Merkle Root (所有交易的哈希树根)
    pub merkle_root: Hash,
    /// 时间戳
    pub timestamp: i64,
    /// 难度目标
    pub bits: u32,
    /// 随机数
    pub nonce: u64,
}

/// 完整区块
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Block {
    /// 区块头
    pub header: BlockHeader,
    /// 区块高度
    pub height: u64,
    /// 交易列表
    pub transactions: Vec<Transaction>,
    /// 区块哈希 (缓存)
    pub hash: Hash,
}

impl Block {
    /// 创建新区块
    pub fn new(
        height: u64,
        prev_hash: Hash,
        transactions: Vec<Transaction>,
        bits: u32,
    ) -> Self {
        let merkle_root = Self::calculate_merkle_root(&transactions);
        
        let mut block = Block {
            header: BlockHeader {
                version: 1,
                prev_hash,
                merkle_root,
                timestamp: Utc::now().timestamp(),
                bits,
                nonce: 0,
            },
            height,
            transactions,
            hash: String::new(),
        };

        block.hash = block.calculate_hash();
        block
    }

    /// 计算区块哈希
    pub fn calculate_hash(&self) -> String {
        let data = format!(
            "{}{}{}{}{}{}",
            self.header.version,
            self.header.prev_hash,
            self.header.merkle_root,
            self.header.timestamp,
            self.header.bits,
            self.header.nonce
        );
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }

    /// 计算 Merkle Root
    fn calculate_merkle_root(transactions: &[Transaction]) -> Hash {
        if transactions.is_empty() {
            return "0".to_string();
        }

        let mut hashes: Vec<String> = transactions
            .iter()
            .map(|tx| tx.id.clone())
            .collect();

        while hashes.len() > 1 {
            if hashes.len() % 2 != 0 {
                hashes.push(hashes.last().unwrap().clone());
            }

            let mut new_level = Vec::new();
            for chunk in hashes.chunks(2) {
                let combined = format!("{}{}", chunk[0], chunk[1]);
                let mut hasher = Sha256::new();
                hasher.update(combined.as_bytes());
                new_level.push(hex::encode(hasher.finalize()));
            }
            hashes = new_level;
        }

        hashes[0].clone()
    }

    /// 检查哈希是否满足难度要求 (POW)
    pub fn check_hash(&self, difficulty: usize) -> bool {
        let target = "0".repeat(difficulty);
        self.hash.starts_with(&target)
    }

    /// 挖矿 (寻找满足难度的 nonce)
    pub fn mine(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);
        
        loop {
            self.hash = self.calculate_hash();
            if self.hash.starts_with(&target) {
                break;
            }
            self.header.nonce += 1;
        }
    }

    /// 验证区块
    pub fn is_valid(&self, difficulty: usize) -> bool {
        // 1. 验证区块哈希
        if self.hash != self.calculate_hash() {
            return false;
        }

        // 2. 验证 POW
        if !self.check_hash(difficulty) {
            return false;
        }

        // 3. 验证所有交易
        for tx in &self.transactions {
            if !tx.is_valid() {
                return false;
            }
        }

        // 4. 验证 Merkle Root
        if self.header.merkle_root != Self::calculate_merkle_root(&self.transactions) {
            return false;
        }

        true
    }

    /// 获取 Coinbase 交易
    pub fn get_coinbase(&self) -> Option<&Transaction> {
        self.transactions.first()
    }
}

/// UTXO 条目
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UtxoEntry {
    pub tx_output: TxOutput,
    pub height: u64,
    pub is_coinbase: bool,
}

/// 区块链状态
#[derive(Debug, Clone)]
pub struct BlockchainState {
    /// 当前区块高度
    pub height: u64,
    /// 最新区块哈希
    pub best_block_hash: Hash,
    /// 总难度
    pub total_difficulty: u128,
    /// 待处理交易池
    pub mempool: Vec<Transaction>,
}

impl Default for BlockchainState {
    fn default() -> Self {
        Self {
            height: 0,
            best_block_hash: String::new(),
            total_difficulty: 0,
            mempool: Vec::new(),
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hash_short = &self.hash[..self.hash.len().min(32)];
        let prev_short = &self.header.prev_hash[..self.header.prev_hash.len().min(32)];
        let merkle_short = &self.header.merkle_root[..self.header.merkle_root.len().min(32)];
        
        writeln!(f, "╔══════════════════════════════════════════╗")?;
        writeln!(f, "║  Block #{:<34} ║", self.height)?;
        writeln!(f, "╠══════════════════════════════════════════╣")?;
        writeln!(f, "║ Hash:     {:<32} ║", hash_short)?;
        writeln!(f, "║ Prev:     {:<32} ║", prev_short)?;
        writeln!(f, "║ Merkle:   {:<32} ║", merkle_short)?;
        writeln!(f, "║ Time:     {:<34} ║", self.header.timestamp)?;
        writeln!(f, "║ Nonce:    {:<34} ║", self.header.nonce)?;
        writeln!(f, "║ Txs:      {:<34} ║", self.transactions.len())?;
        writeln!(f, "╚══════════════════════════════════════════╝")
    }
}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  Transaction: {}", &self.id[..16])?;
        writeln!(f, "    Inputs:  {}", self.inputs.len())?;
        writeln!(f, "    Outputs: {}", self.outputs.len())?;
        for (i, output) in self.outputs.iter().enumerate() {
            writeln!(f, "      [{}] {} -> {} sat", i, 
                &output.script_pubkey[..16.min(output.script_pubkey.len())],
                output.value)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_hash() {
        let tx = Transaction::new(vec![], vec![]);
        assert_eq!(tx.id, tx.calculate_hash());
    }

    #[test]
    fn test_block_hash() {
        let block = Block::new(0, "0".to_string(), vec![], 0x1d00ffff);
        assert_eq!(block.hash, block.calculate_hash());
    }

    #[test]
    fn test_merkle_root() {
        let tx1 = Transaction::new(vec![], vec![]);
        let tx2 = Transaction::new(vec![], vec![]);
        let block = Block::new(0, "0".to_string(), vec![tx1, tx2], 0x1d00ffff);
        assert_ne!(block.header.merkle_root, "0");
    }
}
