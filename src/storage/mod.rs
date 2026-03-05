//! 数据持久化模块
//! 使用 Sled 数据库实现区块和 UTXO 的持久化存储

use crate::core::{Block, Transaction, TxOutput};
use std::collections::HashMap;
use std::path::Path;

/// 存储引擎
pub struct Storage {
    db: sled::Db,
}

/// 存储键前缀
const PREFIX_BLOCK: &str = "blk:";
const PREFIX_TX: &str = "tx:";
const PREFIX_UTXO: &str = "utxo:";
const PREFIX_META: &str = "meta:";
const KEY_HEIGHT: &str = "meta:height";
const KEY_BEST_HASH: &str = "meta:best_hash";

impl Storage {
    /// 打开或创建数据库
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let db = sled::open(path)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        Ok(Storage { db })
    }

    /// 保存区块
    pub fn save_block(&self, block: &Block) -> Result<(), StorageError> {
        let key = format!("{}{}", PREFIX_BLOCK, block.hash);
        let value = serde_json::to_vec(block)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        self.db.insert(key, value)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        // 更新元数据
        self.db.insert(KEY_HEIGHT, block.height.to_string().as_bytes())
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        self.db.insert(KEY_BEST_HASH, block.hash.as_bytes())
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    /// 获取区块
    pub fn get_block(&self, hash: &str) -> Result<Option<Block>, StorageError> {
        let key = format!("{}{}", PREFIX_BLOCK, hash);
        
        match self.db.get(key) {
            Ok(Some(value)) => {
                let block: Block = serde_json::from_slice(&value)
                    .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
                Ok(Some(block))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(StorageError::DatabaseError(e.to_string())),
        }
    }

    /// 获取区块通过高度
    pub fn get_block_by_height(&self, height: u64) -> Result<Option<Block>, StorageError> {
        // 遍历查找
        for item in self.db.scan_prefix(PREFIX_BLOCK) {
            let (_, value) = item.map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            let block: Block = serde_json::from_slice(&value)
                .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
            
            if block.height == height {
                return Ok(Some(block));
            }
        }
        
        Ok(None)
    }

    /// 获取所有区块
    pub fn get_all_blocks(&self) -> Result<Vec<Block>, StorageError> {
        let mut blocks = Vec::new();
        
        for item in self.db.scan_prefix(PREFIX_BLOCK) {
            let (_, value) = item.map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            let block: Block = serde_json::from_slice(&value)
                .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
            blocks.push(block);
        }
        
        // 按高度排序
        blocks.sort_by_key(|b| b.height);
        
        Ok(blocks)
    }

    /// 保存交易
    pub fn save_transaction(&self, tx: &Transaction, block_hash: &str) -> Result<(), StorageError> {
        let key = format!("{}{}", PREFIX_TX, tx.id);
        let value = serde_json::to_vec(&(tx, block_hash))
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        self.db.insert(key, value)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    /// 获取交易
    pub fn get_transaction(&self, tx_id: &str) -> Result<Option<(Transaction, String)>, StorageError> {
        let key = format!("{}{}", PREFIX_TX, tx_id);
        
        match self.db.get(key) {
            Ok(Some(value)) => {
                let result: (Transaction, String) = serde_json::from_slice(&value)
                    .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
                Ok(Some(result))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(StorageError::DatabaseError(e.to_string())),
        }
    }

    /// 保存 UTXO
    pub fn save_utxo(
        &self,
        tx_id: &str,
        vout: u32,
        output: &TxOutput,
    ) -> Result<(), StorageError> {
        let key = format!("{}{}:{}", PREFIX_UTXO, tx_id, vout);
        let value = serde_json::to_vec(output)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        self.db.insert(key, value)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    /// 删除 UTXO
    pub fn delete_utxo(&self, tx_id: &str, vout: u32) -> Result<(), StorageError> {
        let key = format!("{}{}:{}", PREFIX_UTXO, tx_id, vout);
        
        self.db.remove(key)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    /// 获取所有 UTXO
    pub fn get_all_utxos(&self) -> Result<HashMap<(String, u32), TxOutput>, StorageError> {
        let mut utxos = HashMap::new();
        
        for item in self.db.scan_prefix(PREFIX_UTXO) {
            let (key, value) = item.map_err(|e| StorageError::DatabaseError(e.to_string()))?;
            
            let key_str = String::from_utf8_lossy(&key);
            let parts: Vec<&str> = key_str.trim_start_matches(PREFIX_UTXO).split(':').collect();
            
            if parts.len() == 2 {
                let tx_id = parts[0].to_string();
                let vout: u32 = parts[1].parse().unwrap_or(0);
                
                let output: TxOutput = serde_json::from_slice(&value)
                    .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
                
                utxos.insert((tx_id, vout), output);
            }
        }
        
        Ok(utxos)
    }

    /// 获取当前高度
    pub fn get_height(&self) -> Result<u64, StorageError> {
        match self.db.get(KEY_HEIGHT) {
            Ok(Some(value)) => {
                let height_str = String::from_utf8_lossy(&value);
                height_str.parse().map_err(|_| StorageError::InvalidData)
            }
            Ok(None) => Ok(0),
            Err(e) => Err(StorageError::DatabaseError(e.to_string())),
        }
    }

    /// 获取最新区块哈希
    pub fn get_best_block_hash(&self) -> Result<Option<String>, StorageError> {
        match self.db.get(KEY_BEST_HASH) {
            Ok(Some(value)) => {
                Ok(Some(String::from_utf8_lossy(&value).to_string()))
            }
            Ok(None) => Ok(None),
            Err(e) => Err(StorageError::DatabaseError(e.to_string())),
        }
    }

    /// 批量保存区块
    pub fn save_blocks_batch(&self, blocks: &[Block]) -> Result<(), StorageError> {
        let mut batch = sled::Batch::default();
        
        for block in blocks {
            let key = format!("{}{}", PREFIX_BLOCK, block.hash);
            let value = serde_json::to_vec(block)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            batch.insert(key.as_bytes(), value);
        }
        
        self.db.apply_batch(batch)
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        
        Ok(())
    }

    /// 刷新数据到磁盘
    pub fn flush(&self) -> Result<(), StorageError> {
        self.db.flush()
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(())
    }

    /// 关闭数据库
    pub fn close(self) -> Result<(), StorageError> {
        self.db.flush()
            .map_err(|e| StorageError::DatabaseError(e.to_string()))?;
        Ok(())
    }
}

/// 持久化的区块链
pub struct PersistentBlockchain {
    pub chain: Vec<Block>,
    pub storage: Storage,
}

impl PersistentBlockchain {
    /// 加载或创建区块链
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self, StorageError> {
        let storage = Storage::new(db_path)?;
        
        // 尝试加载已有区块
        let blocks = storage.get_all_blocks()?;
        
        let chain = if blocks.is_empty() {
            // 创建创世区块
            use crate::core::{DIFFICULTY, MINING_REWARD};
            
            use sha2::Digest;
            
            let coinbase = Transaction::new_coinbase(
                "YRustGenesis".to_string(),
                MINING_REWARD,
                0,
            );
            
            let mut genesis = Block::new(
                0,
                "0".to_string(),
                vec![coinbase],
                0x1d00ffff,
            );
            genesis.mine(DIFFICULTY);
            
            storage.save_block(&genesis)?;
            
            vec![genesis]
        } else {
            blocks
        };
        
        Ok(PersistentBlockchain { chain, storage })
    }

    /// 保存新区块
    pub fn save_block(&mut self, block: Block) -> Result<(), StorageError> {
        self.storage.save_block(&block)?;
        self.chain.push(block);
        Ok(())
    }

    /// 获取最新区块
    pub fn get_last_block(&self) -> Option<&Block> {
        self.chain.last()
    }

    /// 获取高度
    pub fn get_height(&self) -> u64 {
        self.chain.len() as u64 - 1
    }
}

/// 存储错误类型
#[derive(Debug)]
pub enum StorageError {
    DatabaseError(String),
    SerializationError(String),
    DeserializationError(String),
    InvalidData,
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::DatabaseError(e) => write!(f, "数据库错误: {}", e),
            StorageError::SerializationError(e) => write!(f, "序列化错误: {}", e),
            StorageError::DeserializationError(e) => write!(f, "反序列化错误: {}", e),
            StorageError::InvalidData => write!(f, "无效的数据"),
        }
    }
}

impl std::error::Error for StorageError {}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Storage::new(temp_dir.path()).unwrap();
        
        // 测试保存和获取区块
        let coinbase = Transaction::new_coinbase(
            "test".to_string(),
            100_000_000,
            0,
        );
        let block = Block::new(0, "0".to_string(), vec![coinbase], 0x1d00ffff);
        
        storage.save_block(&block).unwrap();
        
        let retrieved = storage.get_block(&block.hash).unwrap().unwrap();
        assert_eq!(retrieved.height, block.height);
    }
}
