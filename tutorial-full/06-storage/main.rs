//! 06. 数据存储 - RocksDB实现

use std::collections::HashMap;

/// 存储接口
pub trait Storage {
    fn put(&mut self, key: &str, value: &str) -> Result<(), String>;
    fn get(&self, key: &str) -> Result<Option<String>, String>;
    fn delete(&mut self, key: &str) -> Result<(), String>;
}

/// 内存存储（用于测试）
pub struct MemoryStorage {
    data: HashMap<String, String>,
}

impl MemoryStorage {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }
}

impl Storage for MemoryStorage {
    fn put(&mut self, key: &str, value: &str) -> Result<(), String> {
        self.data.insert(key.to_string(), value.to_string());
        Ok(())
    }
    
    fn get(&self, key: &str) -> Result<Option<String>, String> {
        Ok(self.data.get(key).cloned())
    }
    
    fn delete(&mut self, key: &str) -> Result<(), String> {
        self.data.remove(key);
        Ok(())
    }
}

/// 区块链存储
pub struct BlockchainStorage<S: Storage> {
    storage: S,
}

impl<S: Storage> BlockchainStorage<S> {
    pub fn new(storage: S) -> Self {
        Self { storage }
    }
    
    /// 保存区块
    pub fn save_block(&mut self, height: u64, hash: &str, data: &str) -> Result<(), String> {
        let key = format!("block:{}", height);
        self.storage.put(&key, data)?;
        
        let hash_key = format!("hash:{}", hash);
        self.storage.put(&hash_key, &height.to_string())?;
        
        // 更新最新高度
        self.storage.put("meta:latest_height", &height.to_string())?;
        
        Ok(())
    }
    
    /// 获取区块
    pub fn get_block(&self, height: u64) -> Result<Option<String>, String> {
        let key = format!("block:{}", height);
        self.storage.get(&key)
    }
    
    /// 获取最新高度
    pub fn get_latest_height(&self) -> Result<u64, String> {
        match self.storage.get("meta:latest_height")? {
            Some(h) => h.parse().map_err(|e| format!("Parse error: {}", e)),
            None => Ok(0),
        }
    }
}

fn main() {
    println!("存储演示");
    
    let storage = MemoryStorage::new();
    let mut blockchain_storage = BlockchainStorage::new(storage);
    
    // 保存区块
    blockchain_storage.save_block(1, "abc123", "block1_data").unwrap();
    blockchain_storage.save_block(2, "def456", "block2_data").unwrap();
    
    // 读取
    let block1 = blockchain_storage.get_block(1).unwrap();
    println!("区块1: {:?}", block1);
    
    let height = blockchain_storage.get_latest_height().unwrap();
    println!("最新高度: {}", height);
}
