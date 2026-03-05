//! 第一章：区块结构
//! 
//! 运行: cargo run

use std::fmt;

/// 区块结构
#[derive(Debug, Clone)]
struct Block {
    /// 区块高度
    index: u64,
    /// 时间戳
    timestamp: i64,
    /// 数据
    data: String,
    /// 前一个区块哈希
    prev_hash: String,
    /// 本区块哈希
    hash: String,
}

impl Block {
    /// 创建新区块
    fn new(index: u64, data: String, prev_hash: String) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        
        Block {
            index,
            timestamp,
            data,
            prev_hash,
            hash: String::new(), // 暂时为空，下一章计算
        }
    }
    
    /// 美化打印
    fn display(&self) {
        println!("╔════════════════════════════════╗");
        println!("║ Block #{:<24} ║", self.index);
        println!("╠════════════════════════════════╣");
        println!("║ Time:  {:<23} ║", self.timestamp);
        println!("║ Data:  {:<23} ║", self.data);
        println!("║ Prev:  {:<23} ║", &self.prev_hash[..self.prev_hash.len().min(20)]);
        println!("╚════════════════════════════════╝");
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Block #{} [{}] {}", self.index, self.timestamp, self.data)
    }
}

fn main() {
    println!("=== 区块结构演示 ===\n");
    
    // 创建创世区块（第一个区块）
    let genesis = Block::new(
        0,
        "Genesis Block - 创世块".to_string(),
        "0".to_string(),
    );
    
    println!("【Debug 输出】");
    println!("{:?}\n", genesis);
    
    println!("【Display 输出】");
    println!("{}\n", genesis);
    
    println!("【美化输出】");
    genesis.display();
    
    // 创建第二个区块
    println!("\n");
    let block1 = Block::new(
        1,
        "转账: Alice -> Bob 10 BTC".to_string(),
        "0000abc123...".to_string(),
    );
    block1.display();
    
    println!("\n✅ 区块结构演示完成！");
    println!("下一章: 计算区块哈希");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_block_creation() {
        let block = Block::new(0, "test".to_string(), "0".to_string());
        assert_eq!(block.index, 0);
        assert_eq!(block.data, "test");
        assert_eq!(block.prev_hash, "0");
    }
    
    #[test]
    fn test_block_display() {
        let block = Block::new(1, "data".to_string(), "prev".to_string());
        let output = format!("{}", block);
        assert!(output.contains("Block #1"));
    }
}
