//! 第三章：区块链
//! 
//! 运行: cargo run

use sha2::{Sha256, Digest};

/// 区块
#[derive(Debug, Clone)]
struct Block {
    index: u64,
    timestamp: i64,
    data: String,
    prev_hash: String,
    hash: String,
}

impl Block {
    fn new(index: u64, data: String, prev_hash: String) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        let mut block = Block {
            index,
            timestamp,
            data,
            prev_hash,
            hash: String::new(),
        };
        block.hash = block.calculate_hash();
        block
    }
    
    fn calculate_hash(&self) -> String {
        let data = format!("{}{}{}{}", self.index, self.timestamp, self.data, self.prev_hash);
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        hex::encode(hasher.finalize())
    }
}

/// 区块链
#[derive(Debug)]
struct Blockchain {
    chain: Vec<Block>,
}

impl Blockchain {
    /// 创建新区块链（包含创世区块）
    fn new() -> Self {
        let genesis = Block::new(0, "Genesis Block".to_string(), "0".to_string());
        Blockchain {
            chain: vec![genesis],
        }
    }
    
    /// 获取最新区块
    fn get_last_block(&self) -> &Block {
        self.chain.last().expect("链不能为空")
    }
    
    /// 添加新区块
    fn add_block(&mut self, data: String) {
        let last_block = self.get_last_block();
        let new_block = Block::new(
            last_block.index + 1,
            data,
            last_block.hash.clone(),
        );
        self.chain.push(new_block);
    }
    
    /// 验证区块链完整性
    fn is_valid(&self) -> bool {
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];
            
            // 1. 验证当前区块哈希
            if current.hash != current.calculate_hash() {
                println!("❌ 区块 #{} 哈希被篡改!", current.index);
                return false;
            }
            
            // 2. 验证链接
            if current.prev_hash != previous.hash {
                println!("❌ 区块 #{} 链接断裂!", current.index);
                return false;
            }
        }
        true
    }
    
    /// 打印区块链
    fn display(&self) {
        println!("╔══════════════════════════════════════════╗");
        println!("║           区块链状态                      ║");
        println!("╠══════════════════════════════════════════╣");
        println!("║ 区块数量: {:<32} ║", self.chain.len());
        println!("╚══════════════════════════════════════════╝");
        
        for block in &self.chain {
            println!("\n  Block #{}: {}", block.index, &block.hash[..16]);
            println!("  Data: {}", block.data);
            println!("  Prev: {}", &block.prev_hash[..block.prev_hash.len().min(16)]);
        }
    }
}

fn main() {
    println!("=== 区块链演示 ===\n");
    
    // 1. 创建区块链
    println!("【1. 创建区块链】");
    let mut blockchain = Blockchain::new();
    println!("✅ 创世区块已创建\n");
    
    // 2. 添加区块
    println!("【2. 添加区块】");
    blockchain.add_block("Alice -> Bob 10 BTC".to_string());
    println!("✅ 区块 #1 添加成功");
    
    blockchain.add_block("Bob -> Charlie 5 BTC".to_string());
    println!("✅ 区块 #2 添加成功\n");
    
    // 3. 显示区块链
    println!("【3. 区块链状态】");
    blockchain.display();
    
    // 4. 验证
    println!("\n【4. 验证区块链】");
    if blockchain.is_valid() {
        println!("✅ 区块链有效！");
    } else {
        println!("❌ 区块链无效！");
    }
    
    // 5. 篡改测试
    println!("\n【5. 篡改测试】");
    println!("尝试篡改区块 #1 的数据...");
    
    // 注意：我们需要可变引用，所以这里用 clone 演示
    let mut tampered_chain = blockchain.clone();
    if let Some(block) = tampered_chain.chain.get_mut(1) {
        block.data = "Alice -> Bob 1000 BTC".to_string();
        // 注意：我们没有重新计算哈希
    }
    
    if tampered_chain.is_valid() {
        println!("❌ 篡改未被发现！");
    } else {
        println!("✅ 篡改被检测到了！区块链保护数据安全。");
    }
    
    println!("\n✅ 区块链演示完成！");
    println!("下一章: 工作量证明");
}

impl Clone for Blockchain {
    fn clone(&self) -> Self {
        Blockchain {
            chain: self.chain.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_genesis_block() {
        let bc = Blockchain::new();
        assert_eq!(bc.chain.len(), 1);
        assert_eq!(bc.chain[0].index, 0);
    }
    
    #[test]
    fn test_add_block() {
        let mut bc = Blockchain::new();
        bc.add_block("test".to_string());
        assert_eq!(bc.chain.len(), 2);
        assert_eq!(bc.chain[1].index, 1);
    }
    
    #[test]
    fn test_valid_chain() {
        let mut bc = Blockchain::new();
        bc.add_block("test".to_string());
        assert!(bc.is_valid());
    }
}
