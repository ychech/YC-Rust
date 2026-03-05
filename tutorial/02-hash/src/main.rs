//! 第二章：哈希计算
//! 
//! 运行: cargo run

use sha2::{Sha256, Digest};
use std::fmt;

/// 区块结构（带哈希计算）
#[derive(Debug, Clone)]
struct Block {
    index: u64,
    timestamp: i64,
    data: String,
    prev_hash: String,
    hash: String,
}

impl Block {
    /// 创建新区块（自动计算哈希）
    fn new(index: u64, data: String, prev_hash: String) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        
        let mut block = Block {
            index,
            timestamp,
            data,
            prev_hash,
            hash: String::new(),
        };
        
        // 计算哈希
        block.hash = block.calculate_hash();
        
        block
    }
    
    /// 计算区块哈希
    fn calculate_hash(&self) -> String {
        // 1. 把区块数据拼接成字符串
        let data = format!(
            "{}{}{}{}",
            self.index, self.timestamp, self.data, self.prev_hash
        );
        
        // 2. 创建 SHA256 哈希器
        let mut hasher = Sha256::new();
        
        // 3. 写入数据
        hasher.update(data.as_bytes());
        
        // 4. 获取结果（字节数组）
        let result = hasher.finalize();
        
        // 5. 转成十六进制字符串
        hex::encode(result)
    }
    
    /// 验证哈希是否正确
    fn verify_hash(&self) -> bool {
        self.hash == self.calculate_hash()
    }
    
    /// 显示区块信息
    fn display(&self) {
        println!("╔══════════════════════════════════════════╗");
        println!("║ Block #{:<34} ║", self.index);
        println!("╠══════════════════════════════════════════╣");
        println!("║ Hash:    {:<33} ║", &self.hash[..32]);
        println!("║ Prev:    {:<33} ║", &self.prev_hash[..self.prev_hash.len().min(32)]);
        println!("║ Time:    {:<34} ║", self.timestamp);
        println!("║ Data:    {:<34} ║", self.data);
        println!("╚══════════════════════════════════════════╝");
    }
}

fn main() {
    println!("=== 哈希计算演示 ===\n");
    
    // 1. 演示 SHA256
    println!("【1. SHA256 哈希演示】");
    let mut hasher = Sha256::new();
    hasher.update(b"Hello, World!");
    let result = hasher.finalize();
    println!("输入: \"Hello, World!\"");
    println!("输出: {}", hex::encode(result));
    println!();
    
    // 2. 雪崩效应
    println!("【2. 雪崩效应演示】");
    let mut hasher1 = Sha256::new();
    hasher1.update(b"abc");
    let hash1 = hex::encode(hasher1.finalize());
    
    let mut hasher2 = Sha256::new();
    hasher2.update(b"abd"); // 只改了一个字母
    let hash2 = hex::encode(hasher2.finalize());
    
    println!("abc -> {}", &hash1[..16]);
    println!("abd -> {}", &hash2[..16]);
    println!("完全不相同！✅\n");
    
    // 3. 创建带哈希的区块
    println!("【3. 区块哈希】");
    let genesis = Block::new(
        0,
        "Genesis Block".to_string(),
        "0".to_string(),
    );
    genesis.display();
    
    // 4. 验证哈希
    println!("\n【4. 哈希验证】");
    println!("哈希有效: {}", if genesis.verify_hash() { "✅" } else { "❌" });
    
    // 5. 创建链式区块
    println!("\n【5. 区块链式结构】");
    let block1 = Block::new(1, "Tx1".to_string(), genesis.hash.clone());
    let block2 = Block::new(2, "Tx2".to_string(), block1.hash.clone());
    
    block1.display();
    println!();
    block2.display();
    
    println!("\n✅ 哈希计算演示完成！");
    println!("下一章: 构建区块链");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hash_consistency() {
        let block = Block::new(0, "test".to_string(), "0".to_string());
        let hash1 = block.hash.clone();
        let hash2 = block.calculate_hash();
        assert_eq!(hash1, hash2);
    }
    
    #[test]
    fn test_hash_verification() {
        let block = Block::new(0, "test".to_string(), "0".to_string());
        assert!(block.verify_hash());
    }
    
    #[test]
    fn test_different_blocks_different_hashes() {
        let block1 = Block::new(0, "a".to_string(), "0".to_string());
        let block2 = Block::new(0, "b".to_string(), "0".to_string());
        assert_ne!(block1.hash, block2.hash);
    }
}
