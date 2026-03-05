//! RocksDB 持久化存储
//! 
//! 高性能键值存储，支持：
//! - 区块存储
//! - UTXO 索引
//! - 账户状态
//! - 交易索引

use rocksdb::{DB, ColumnFamilyDescriptor, Options, WriteBatch};
use serde::{Serialize, Deserialize};
use std::path::Path;
use std::sync::Arc;

use crate::core::{Block, Transaction};
use crate::core::models::{UtxoSet, WorldState, Account};

/// 数据库列族
pub const CF_BLOCKS: &str = "blocks";
pub const CF_BLOCK_HEIGHT: &str = "block_height";
pub const CF_TXS: &str = "transactions";
pub const CF_UTXO: &str = "utxo";
pub const CF_ACCOUNTS: &str = "accounts";
pub const CF_META: &str = "meta";

/// RocksDB 存储引擎
pub struct RocksStorage {
    db: Arc<DB>,
}

impl RocksStorage {
    /// 打开或创建数据库
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, StorageError> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.create_missing_column_families(true);
        
        // 配置性能参数
        opts.set_write_buffer_size(64 * 1024 * 1024); // 64MB
        opts.set_max_write_buffer_number(3);
        opts.set_target_file_size_base(64 * 1024 * 1024);
        opts.set_level_zero_file_num_compaction_trigger(4);
        
        // 创建列族
        let cfs = vec![
            ColumnFamilyDescriptor::new(CF_BLOCKS, Options::default()),
            ColumnFamilyDescriptor::new(CF_BLOCK_HEIGHT, Options::default()),
            ColumnFamilyDescriptor::new(CF_TXS, Options::default()),
            ColumnFamilyDescriptor::new(CF_UTXO, Options::default()),
            ColumnFamilyDescriptor::new(CF_ACCOUNTS, Options::default()),
            ColumnFamilyDescriptor::new(CF_META, Options::default()),
        ];
        
        let db = DB::open_cf_descriptors(&opts, path, cfs)
            .map_err(|e| StorageError::RocksDbError(e.to_string()))?;
        
        Ok(RocksStorage {
            db: Arc::new(db),
        })
    }

    // ==================== 区块操作 ====================

    /// 保存区块
    pub fn save_block(&self, block: &Block) -> Result<(), StorageError> {
        let cf = self.db.cf_handle(CF_BLOCKS)
            .ok_or(StorageError::CfNotFound)?;
        
        let key = format!("blk:{}", block.hash);
        let value = serde_json::to_vec(block)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        self.db.put_cf(&cf, &key, &value)
            .map_err(|e| StorageError::RocksDbError(e.to_string()))?;
        
        // 同时保存高度索引
        let cf_height = self.db.cf_handle(CF_BLOCK_HEIGHT)
            .ok_or(StorageError::CfNotFound)?;
        let height_key = format!("height:{:010}", block.height);
        
        self.db.put_cf(&cf_height, &height_key, block.hash.as_bytes())
            .map_err(|e| StorageError::RocksDbError(e.to_string()))?;
        
        // 更新最新区块
        let cf_meta = self.db.cf_handle(CF_META)
            .ok_or(StorageError::CfNotFound)?;
        self.db.put_cf(&cf_meta, b"best_block_hash", block.hash.as_bytes())
            .map_err(|e| StorageError::RocksDbError(e.to_string()))?;
        self.db.put_cf(&cf_meta, b"best_block_height", block.height.to_string().as_bytes())
            .map_err(|e| StorageError::RocksDbError(e.to_string()))?;
        
        Ok(())
    }

    /// 获取区块
    pub fn get_block(&self, hash: &str) -> Result<Option<Block>, StorageError> {
        let cf = self.db.cf_handle(CF_BLOCKS)
            .ok_or(StorageError::CfNotFound)?;
        
        let key = format!("blk:{}", hash);
        match self.db.get_cf(&cf, &key)
            .map_err(|e| StorageError::RocksDbError(e.to_string()))? {
            Some(value) => {
                let block: Block = serde_json::from_slice(&value)
                    .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
                Ok(Some(block))
            }
            None => Ok(None),
        }
    }

    /// 通过高度获取区块
    pub fn get_block_by_height(&self, height: u64) -> Result<Option<Block>, StorageError> {
        let cf_height = self.db.cf_handle(CF_BLOCK_HEIGHT)
            .ok_or(StorageError::CfNotFound)?;
        
        let height_key = format!("height:{:010}", height);
        match self.db.get_cf(&cf_height, &height_key)
            .map_err(|e| StorageError::RocksDbError(e.to_string()))? {
            Some(hash_bytes) => {
                let hash = String::from_utf8_lossy(&hash_bytes);
                self.get_block(&hash)
            }
            None => Ok(None),
        }
    }

    /// 批量保存区块
    pub fn save_blocks_batch(&self, blocks: &[Block]) -> Result<(), StorageError> {
        let cf = self.db.cf_handle(CF_BLOCKS)
            .ok_or(StorageError::CfNotFound)?;
        let cf_height = self.db.cf_handle(CF_BLOCK_HEIGHT)
            .ok_or(StorageError::CfNotFound)?;
        
        let mut batch = WriteBatch::default();
        
        for block in blocks {
            let key = format!("blk:{}", block.hash);
            let value = serde_json::to_vec(block)
                .map_err(|e| StorageError::SerializationError(e.to_string()))?;
            batch.put_cf(&cf, &key, &value);
            
            let height_key = format!("height:{:010}", block.height);
            batch.put_cf(&cf_height, &height_key, block.hash.as_bytes());
        }
        
        self.db.write(batch)
            .map_err(|e| StorageError::RocksDbError(e.to_string()))?;
        
        Ok(())
    }

    // ==================== 交易操作 ====================

    /// 保存交易
    pub fn save_transaction(&self, tx: &Transaction, block_hash: &str) -> Result<(), StorageError> {
        let cf = self.db.cf_handle(CF_TXS)
            .ok_or(StorageError::CfNotFound)?;
        
        let key = format!("tx:{}", tx.id);
        let value = serde_json::to_vec(&(tx, block_hash))
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        self.db.put_cf(&cf, &key, &value)
            .map_err(|e| StorageError::RocksDbError(e.to_string()))?;
        
        Ok(())
    }

    /// 获取交易
    pub fn get_transaction(&self, tx_id: &str) -> Result<Option<(Transaction, String)>, StorageError> {
        let cf = self.db.cf_handle(CF_TXS)
            .ok_or(StorageError::CfNotFound)?;
        
        let key = format!("tx:{}", tx_id);
        match self.db.get_cf(&cf, &key)
            .map_err(|e| StorageError::RocksDbError(e.to_string()))? {
            Some(value) => {
                let result: (Transaction, String) = serde_json::from_slice(&value)
                    .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
                Ok(Some(result))
            }
            None => Ok(None),
        }
    }

    // ==================== UTXO 操作 ====================

    /// 保存 UTXO
    pub fn save_utxo(&self, tx_id: &str, vout: u32, output: &UtxoOutput) -> Result<(), StorageError> {
        let cf = self.db.cf_handle(CF_UTXO)
            .ok_or(StorageError::CfNotFound)?;
        
        let key = format!("utxo:{}:{}", tx_id, vout);
        let value = serde_json::to_vec(output)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        self.db.put_cf(&cf, &key, &value)
            .map_err(|e| StorageError::RocksDbError(e.to_string()))?;
        
        Ok(())
    }

    /// 删除 UTXO（花费）
    pub fn delete_utxo(&self, tx_id: &str, vout: u32) -> Result<(), StorageError> {
        let cf = self.db.cf_handle(CF_UTXO)
            .ok_or(StorageError::CfNotFound)?;
        
        let key = format!("utxo:{}:{}", tx_id, vout);
        self.db.delete_cf(&cf, &key)
            .map_err(|e| StorageError::RocksDbError(e.to_string()))?;
        
        Ok(())
    }

    /// 获取所有 UTXO
    pub fn get_all_utxos(&self) -> Result<UtxoSet, StorageError> {
        let cf = self.db.cf_handle(CF_UTXO)
            .ok_or(StorageError::CfNotFound)?;
        
        let mut utxo_set = UtxoSet::new();
        
        let iter = self.db.iterator_cf(&cf, rocksdb::IteratorMode::Start);
        for item in iter {
            let (key, value) = item.map_err(|e| StorageError::RocksDbError(e.to_string()))?;
            
            let key_str = String::from_utf8_lossy(&key);
            let parts: Vec<&str> = key_str.split(':').collect();
            if parts.len() == 3 {
                let tx_id = parts[1].to_string();
                let vout: u32 = parts[2].parse().unwrap_or(0);
                
                let output: UtxoOutput = serde_json::from_slice(&value)
                    .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
                
                utxo_set.add(&tx_id, vout, output);
            }
        }
        
        Ok(utxo_set)
    }

    // ==================== 账户操作 ====================

    /// 保存账户
    pub fn save_account(&self, account: &Account) -> Result<(), StorageError> {
        let cf = self.db.cf_handle(CF_ACCOUNTS)
            .ok_or(StorageError::CfNotFound)?;
        
        let key = format!("acc:{}", account.address);
        let value = serde_json::to_vec(account)
            .map_err(|e| StorageError::SerializationError(e.to_string()))?;
        
        self.db.put_cf(&cf, &key, &value)
            .map_err(|e| StorageError::RocksDbError(e.to_string()))?;
        
        Ok(())
    }

    /// 获取账户
    pub fn get_account(&self, address: &str) -> Result<Option<Account>, StorageError> {
        let cf = self.db.cf_handle(CF_ACCOUNTS)
            .ok_or(StorageError::CfNotFound)?;
        
        let key = format!("acc:{}", address);
        match self.db.get_cf(&cf, &key)
            .map_err(|e| StorageError::RocksDbError(e.to_string()))? {
            Some(value) => {
                let account: Account = serde_json::from_slice(&value)
                    .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
                Ok(Some(account))
            }
            None => Ok(None),
        }
    }

    /// 获取所有账户
    pub fn get_all_accounts(&self) -> Result<WorldState, StorageError> {
        let cf = self.db.cf_handle(CF_ACCOUNTS)
            .ok_or(StorageError::CfNotFound)?;
        
        let mut state = WorldState::new();
        
        let iter = self.db.iterator_cf(&cf, rocksdb::IteratorMode::Start);
        for item in iter {
            let (_, value) = item.map_err(|e| StorageError::RocksDbError(e.to_string()))?;
            
            let account: Account = serde_json::from_slice(&value)
                .map_err(|e| StorageError::DeserializationError(e.to_string()))?;
            
            state.accounts.insert(account.address.clone(), account);
        }
        
        Ok(state)
    }

    // ==================== 元数据 ====================

    /// 获取最新区块高度
    pub fn get_best_height(&self) -> Result<u64, StorageError> {
        let cf = self.db.cf_handle(CF_META)
            .ok_or(StorageError::CfNotFound)?;
        
        match self.db.get_cf(&cf, b"best_block_height")
            .map_err(|e| StorageError::RocksDbError(e.to_string()))? {
            Some(value) => {
                let height_str = String::from_utf8_lossy(&value);
                height_str.parse().map_err(|_| StorageError::InvalidData)
            }
            None => Ok(0),
        }
    }

    /// 获取最新区块哈希
    pub fn get_best_hash(&self) -> Result<Option<String>, StorageError> {
        let cf = self.db.cf_handle(CF_META)
            .ok_or(StorageError::CfNotFound)?;
        
        match self.db.get_cf(&cf, b"best_block_hash")
            .map_err(|e| StorageError::RocksDbError(e.to_string()))? {
            Some(value) => Ok(Some(String::from_utf8_lossy(&value).to_string())),
            None => Ok(None),
        }
    }

    /// 刷新数据到磁盘
    pub fn flush(&self) -> Result<(), StorageError> {
        self.db.flush()
            .map_err(|e| StorageError::RocksDbError(e.to_string()))?;
        Ok(())
    }
}

/// 存储错误
#[derive(Debug)]
pub enum StorageError {
    RocksDbError(String),
    SerializationError(String),
    DeserializationError(String),
    CfNotFound,
    InvalidData,
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StorageError::RocksDbError(e) => write!(f, "RocksDB error: {}", e),
            StorageError::SerializationError(e) => write!(f, "Serialization error: {}", e),
            StorageError::DeserializationError(e) => write!(f, "Deserialization error: {}", e),
            StorageError::CfNotFound => write!(f, "Column family not found"),
            StorageError::InvalidData => write!(f, "Invalid data"),
        }
    }
}

impl std::error::Error for StorageError {}

// 导入需要的类型
use crate::core::models::UtxoOutput;
