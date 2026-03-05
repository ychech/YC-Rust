//! 03. 区块链核心 - 完整实现
//! 
//! 本文件实现区块链的链式结构和POW挖矿

use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// 交易输出
#[derive(Debug, Clone)]
pub struct TxOutput {
    pub address: String,
    pub amount: u64,
}

/// 交易输入
#[derive(Debug, Clone)]
pub struct TxInput {
    pub tx_id: String,
    pub output_index: u32,
    pub signature: Vec<u8>,
}

/// 交易
#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub timestamp: u64,
}

impl Transaction {
    /// 创建Coinbase交易（挖矿奖励）
    pub fn new_coinbase(miner: String, reward: u64) -> Self {
        let timestamp = current_timestamp();
        let id = format!("coinbase_{}_{}", miner, timestamp);
        
        Self {
            id,
            inputs: vec![], // Coinbase没有输入
            outputs: vec![TxOutput {
                address: miner,
                amount: reward,
            }],
            timestamp,
        }
    }
    
    /// 计算交易哈希
    pub fn hash(&self) -> String {
        let data = format!("{:?}{:?}{}", self.inputs, self.outputs, self.timestamp);
        sha256(&data)
    }
}

/// 区块头
#[derive(Debug, Clone)]
pub struct BlockHeader {
    pub version: u32,
    pub prev_hash: String,
    pub merkle_root: String,
    pub timestamp: u64,
    pub difficulty: u32,
    pub nonce: u64,
}

/// 区块
#[derive(Debug, Clone)]
pub struct Block {
    pub height: u64,
    pub header: BlockHeader,
    pub hash: String,
    pub transactions: Vec<Transaction>,
    pub miner: String,
}

impl Block {
    /// 创建新区块
    pub fn new(
        height: u64,
        prev_hash: String,
        transactions: Vec<Transaction>,
        difficulty: u32,
        miner: String,
    ) -> Self {
        let merkle_root = calculate_merkle_root(&transactions);
        
        let header = BlockHeader {
            version: 1,
            prev_hash,
            merkle_root,
            timestamp: current_timestamp(),
            difficulty,
            nonce: 0,
        };
        
        let hash = calculate_block_hash(&header);
        
        Self {
            height,
            header,
            hash,
            transactions,
            miner,
        }
    }
    
    /// 挖矿 - 寻找满足难度要求的nonce
    pub fn mine(&mut self) -> String {
        let target = "0".repeat(self.header.difficulty as usize);
        
        println!("⛏️  开始挖区块 #{}，难度: {}", self.height, self.header.difficulty);
        
        loop {
            self.hash = calculate_block_hash(&self.header);
            
            if self.hash.starts_with(&target) {
                println!("✅ 挖矿成功! Nonce: {}", self.header.nonce);
                println!("   Hash: {}...", &self.hash[..20]);
                return self.hash.clone();
            }
            
            self.header.nonce += 1;
            
            // 每10万次打印进度
            if self.header.nonce % 100000 == 0 {
                println!("   已尝试 {} 次...", self.header.nonce);
            }
        }
    }
}

/// 区块链
pub struct Blockchain {
    pub blocks: Vec<Block>,
    pub utxo_set: HashMap<String, TxOutput>,
    pub mempool: Vec<Transaction>,
    pub difficulty: u32,
    pub mining_reward: u64,
}

impl Blockchain {
    /// 创建新区块链（包含创世区块）
    pub fn new() -> Self {
        let genesis = Self::create_genesis_block();
        
        let mut utxo_set = HashMap::new();
        // 将创世区块的coinbase输出加入UTXO集
        for (i, output) in genesis.transactions[0].outputs.iter().enumerate() {
            let key = format!("{}:{}", genesis.transactions[0].id, i);
            utxo_set.insert(key, output.clone());
        }
        
        Self {
            blocks: vec![genesis],
            utxo_set,
            mempool: Vec::new(),
            difficulty: 2, // 初始难度较低，便于测试
            mining_reward: 50_0000_0000, // 50 YRU
        }
    }
    
    /// 创建创世区块
    fn create_genesis_block() -> Block {
        let coinbase = Transaction::new_coinbase("YRGenesis".to_string(), 50_0000_0000);
        
        Block {
            height: 0,
            header: BlockHeader {
                version: 1,
                prev_hash: "0".repeat(64),
                merkle_root: coinbase.id.clone(),
                timestamp: 1704067200,
                difficulty: 1,
                nonce: 0,
            },
            hash: "genesis".to_string(),
            transactions: vec![coinbase],
            miner: "YRGenesis".to_string(),
        }
    }
    
    /// 获取最新区块
    pub fn latest_block(&self) -> &Block {
        self.blocks.last().unwrap()
    }
    
    /// 获取区块链高度
    pub fn height(&self) -> u64 {
        (self.blocks.len() as u64) - 1
    }
    
    /// 挖矿并添加新区块
    pub fn mine_block(&mut self, miner: String) -> Result<Block, String> {
        let prev_block = self.latest_block();
        
        // 创建coinbase交易
        let coinbase = Transaction::new_coinbase(miner.clone(), self.mining_reward);
        
        // 收集内存池交易
        let mut transactions = vec![coinbase];
        transactions.extend(self.mempool.clone());
        
        // 创建并挖矿
        let mut new_block = Block::new(
            prev_block.height + 1,
            prev_block.hash.clone(),
            transactions,
            self.difficulty,
            miner,
        );
        
        new_block.mine();
        
        // 验证并添加
        if self.validate_block(&new_block) {
            self.add_block(new_block.clone());
            Ok(new_block)
        } else {
            Err("区块验证失败".to_string())
        }
    }
    
    /// 验证区块
    fn validate_block(&self, block: &Block) -> bool {
        // 1. 验证前一区块哈希
        if block.height > 0 {
            let prev = self.latest_block();
            if block.header.prev_hash != prev.hash {
                println!("❌ 前一区块哈希不匹配");
                return false;
            }
        }
        
        // 2. 验证难度
        let target = "0".repeat(block.header.difficulty as usize);
        if !block.hash.starts_with(&target) {
            println!("❌ 难度不满足");
            return false;
        }
        
        // 3. 验证Merkle根
        let merkle = calculate_merkle_root(&block.transactions);
        if merkle != block.header.merkle_root {
            println!("❌ Merkle根不匹配");
            return false;
        }
        
        true
    }
    
    /// 添加区块到链
    fn add_block(&mut self, block: Block) {
        // 更新UTXO集
        for tx in &block.transactions {
            // 移除已花费的输入
            for input in &tx.inputs {
                let key = format!("{}:{}", input.tx_id, input.output_index);
                self.utxo_set.remove(&key);
            }
            // 添加新输出
            for (i, output) in tx.outputs.iter().enumerate() {
                let key = format!("{}:{}", tx.id, i);
                self.utxo_set.insert(key, output.clone());
            }
        }
        
        self.blocks.push(block);
        self.mempool.clear(); // 清空已确认的交易
    }
    
    /// 获取地址余额
    pub fn get_balance(&self, address: &str) -> u64 {
        self.utxo_set
            .values()
            .filter(|output| output.address == address)
            .map(|output| output.amount)
            .sum()
    }
    
    /// 验证整个链
    pub fn validate_chain(&self) -> bool {
        for i in 1..self.blocks.len() {
            let current = &self.blocks[i];
            let previous = &self.blocks[i - 1];
            
            if current.header.prev_hash != previous.hash {
                println!("❌ 区块 #{} 链接断开", current.height);
                return false;
            }
            
            let target = "0".repeat(current.header.difficulty as usize);
            if !current.hash.starts_with(&target) {
                println!("❌ 区块 #{} 难度不满足", current.height);
                return false;
            }
        }
        
        println!("✅ 区块链验证通过！共 {} 个区块", self.blocks.len());
        true
    }
    
    /// 打印链信息
    pub fn print_chain(&self) {
        println!("\n📦 区块链信息:");
        println!("   总区块数: {}", self.blocks.len());
        println!("   当前高度: {}", self.height());
        println!("   挖矿难度: {}", self.difficulty);
        println!("   UTXO数量: {}", self.utxo_set.len());
        
        println!("\n🔗 区块列表:");
        for block in &self.blocks {
            println!("   #{} [{}...] 矿工: {} 交易: {}",
                block.height,
                &block.hash[..8],
                &block.miner[..10],
                block.transactions.len()
            );
        }
    }
}

/// 工具函数

fn sha256(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn calculate_block_hash(header: &BlockHeader) -> String {
    let data = format!(
        "{}|{}|{}|{}|{}|{}",
        header.version,
        header.prev_hash,
        header.merkle_root,
        header.timestamp,
        header.difficulty,
        header.nonce
    );
    sha256(&sha256(&data))
}

fn calculate_merkle_root(transactions: &[Transaction]) -> String {
    if transactions.is_empty() {
        return "0".repeat(64);
    }
    
    let mut hashes: Vec<String> = transactions.iter().map(|tx| tx.id.clone()).collect();
    
    while hashes.len() > 1 {
        if hashes.len() % 2 == 1 {
            hashes.push(hashes.last().unwrap().clone());
        }
        
        let mut next_level = Vec::new();
        for i in (0..hashes.len()).step_by(2) {
            let combined = format!("{}{}", hashes[i], hashes[i + 1]);
            next_level.push(sha256(&combined));
        }
        hashes = next_level;
    }
    
    hashes[0].clone()
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// 主函数
fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║           YRUST Chain - 区块链核心演示                    ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    // 1. 创建区块链
    println!("1️⃣  创建区块链...");
    let mut blockchain = Blockchain::new();
    println!("   ✅ 创世区块已创建");
    
    // 2. 模拟挖矿
    println!("\n2️⃣  模拟挖矿...");
    let miners = vec!["Alice", "Bob", "Charlie"];
    
    for i in 0..5 {
        let miner = miners[i % miners.len()];
        println!("\n   ⛏️  {} 正在挖区块 #{}...", miner, i + 1);
        
        match blockchain.mine_block(miner.to_string()) {
            Ok(block) => {
                println!("   ✅ 区块 #{} 已挖出!", block.height);
            }
            Err(e) => {
                println!("   ❌ 挖矿失败: {}", e);
            }
        }
    }
    
    // 3. 显示区块链
    blockchain.print_chain();
    
    // 4. 显示余额
    println!("\n💰 矿工余额:");
    for miner in &miners {
        let balance = blockchain.get_balance(miner);
        println!("   {}: {} 聪 ({} YRU)", 
            miner, 
            balance,
            balance as f64 / 100_000_000.0
        );
    }
    
    // 5. 验证链
    println!("\n5️⃣  验证区块链...");
    blockchain.validate_chain();
    
    println!("\n✨ 演示完成！");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_genesis_block() {
        let blockchain = Blockchain::new();
        assert_eq!(blockchain.height(), 0);
        assert_eq!(blockchain.blocks.len(), 1);
    }
    
    #[test]
    fn test_mining() {
        let mut blockchain = Blockchain::new();
        let block = blockchain.mine_block("test".to_string()).unwrap();
        assert_eq!(block.height, 1);
        assert!(block.hash.starts_with("00"));
    }
    
    #[test]
    fn test_chain_validation() {
        let mut blockchain = Blockchain::new();
        blockchain.mine_block("test".to_string()).unwrap();
        blockchain.mine_block("test".to_string()).unwrap();
        assert!(blockchain.validate_chain());
    }
}
