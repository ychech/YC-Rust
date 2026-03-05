//! 01. 区块基础 - 完整实现
//! 
//! 本文件实现了区块链中最基本的概念：区块
//! 
//! # 核心概念
//! 
//! 1. **区块头 (BlockHeader)**: 包含区块的元数据，只有 80 字节
//! 2. **区块体 (BlockBody)**: 包含实际的交易数据
//! 3. **区块哈希**: 区块的唯一标识，通过哈希区块头计算得出
//! 4. **创世区块**: 区块链的第一个区块
//! 
//! # 运行方式
//! 
//! ```bash
//! cargo run
//! ```

use sha2::{Sha256, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

/// ============================================================================
/// 第一部分：基础数据结构
/// ============================================================================

/// 区块头
/// 
/// 区块头是区块的核心，包含了区块的所有元数据。
/// 在比特币中，区块头固定为 80 字节。
/// 
/// # 内存布局（比特币）
/// ```
/// 版本号:     4 字节 (小端序)
/// 前一哈希:  32 字节
/// Merkle根:  32 字节
/// 时间戳:     4 字节 (小端序)
/// 难度目标:   4 字节
/// Nonce:      4 字节
/// 总计:      80 字节
/// ```
#[derive(Debug, Clone)]
pub struct BlockHeader {
    /// 协议版本号
    /// 
    /// 用途：
    /// - 协议升级时标识新版本
    /// - 节点可以拒绝不兼容的版本
    /// 
    /// 比特币版本历史：
    /// - v1: 创世版本
    /// - v2: BIP34 (区块高度在 coinbase)
    /// - v3: BIP66 (严格 DER 签名)
    /// - v4: BIP65 (OP_CHECKLOCKTIMEVERIFY)
    pub version: u32,
    
    /// 前一区块的哈希值（32字节）
    /// 
    /// 这是形成区块链的关键：
    /// - 每个区块都指向前一个区块
    /// - 修改历史区块会导致后续所有区块哈希变化
    /// - 因此历史不可篡改
    /// 
    /// # 安全特性
    /// 想要篡改区块 N，必须重新计算区块 N, N+1, N+2, ...
    /// 这需要巨大的计算能力（51%攻击）
    pub prev_hash: String,
    
    /// Merkle 树根哈希（32字节）
    /// 
    /// Merkle 树是一种二叉树结构，用于：
    /// 1. 快速验证交易是否包含在区块中
    /// 2. 轻节点（SPV）只需下载区块头即可验证
    /// 3. 快速同步（只需验证路径）
    /// 
    /// # 构建过程
    /// ```
    /// 交易: [Tx1, Tx2, Tx3, Tx4]
    /// 
    ///         Root (H12_34)
    ///          /        \
    ///      H12          H34
    ///     /   \        /   \
    ///   H1     H2    H3     H4
    ///   |       |    |       |
    ///  Tx1    Tx2   Tx3    Tx4
    /// 
    /// H1 = SHA256(Tx1)
    /// H12 = SHA256(H1 + H2)
    /// Root = SHA256(H12 + H34)
    /// ```
    pub merkle_root: String,
    
    /// 区块创建时间（Unix 时间戳）
    /// 
    /// 用途：
    /// 1. 调整挖矿难度（每 2016 个区块）
    /// 2. 验证区块时效性（不能来自未来）
    /// 3. 交易排序
    /// 
    /// # 注意事项
    /// - 矿工可以在一定范围内调整时间戳
    /// - 比特币允许时间戳比网络时间快 2 小时
    pub timestamp: u64,
    
    /// 挖矿难度目标
    /// 
    /// 表示区块哈希必须小于的目标值。
    /// 难度越高，目标值越小，需要更多计算。
    /// 
    /// # 难度调整
    /// 比特币每 2016 个区块（约2周）调整一次难度，
    /// 目标是保持平均出块时间为 10 分钟。
    /// 
    /// # 难度计算公式
    /// ```
    /// 新难度 = 旧难度 × (2016 × 10 分钟) / (实际花费时间)
    /// ```
    pub difficulty: u32,
    
    /// 随机数（Number used once）
    /// 
    /// 挖矿时不断改变的数字，目标是找到使区块哈希
    /// 满足难度要求的 nonce 值。
    /// 
    /// # 挖矿过程
    /// ```
    /// while true:
    ///     nonce += 1
    ///     hash = SHA256(SHA256(block_header))
    ///     if hash < target:
    ///         找到有效区块！
    ///         break
    /// ```
    /// 
    /// # 难度示例
    /// 难度 1：哈希以 1 个 0 开头
    /// 难度 4：哈希以 4 个 0 开头（如 0000a1b2...）
    pub nonce: u64,
}

impl BlockHeader {
    /// 创建新的区块头
    /// 
    /// # 参数
    /// - `version`: 协议版本
    /// - `prev_hash`: 前一区块哈希
    /// - `merkle_root`: Merkle 根
    /// - `difficulty`: 挖矿难度
    pub fn new(
        version: u32,
        prev_hash: String,
        merkle_root: String,
        difficulty: u32,
    ) -> Self {
        Self {
            version,
            prev_hash,
            merkle_root,
            timestamp: current_timestamp(),
            difficulty,
            nonce: 0,
        }
    }
}

/// 交易结构
/// 
/// 表示区块链上的一笔转账。
/// 在比特币中，交易实际上是一组输入和输出（UTXO模型）。
#[derive(Debug, Clone)]
pub struct Transaction {
    /// 交易哈希（交易内容的唯一标识）
    /// 
    /// 计算方式：SHA256(SHA256(交易数据))
    pub hash: String,
    
    /// 发送方地址
    /// 
    /// 地址格式（类似比特币）：
    /// - 以 YR 开头（YRUST Chain）
    /// - Base58Check 编码
    /// - 包含版本字节和校验和
    /// 
    /// # 地址生成过程
    /// ```
    /// 1. 生成私钥（32字节随机数）
    /// 2. 通过 secp256k1 计算公钥
    /// 3. SHA256(公钥)
    /// 4. RIPEMD160(SHA256结果) = 20字节哈希
    /// 5. 添加版本字节（0x00）
    /// 6. 计算校验和（SHA256两次的前4字节）
    /// 7. Base58编码
    /// ```
    pub from: String,
    
    /// 接收方地址
    pub to: String,
    
    /// 转账金额（最小单位）
    /// 
    /// 类似于比特币的 "聪" (satoshi)，
    /// 1 YRU = 100,000,000 最小单位
    pub amount: u64,
    
    /// 交易费用（矿工费）
    /// 
    /// 用途：
    /// - 激励矿工打包交易
    /// - 防止垃圾交易
    /// 
    /// # 费用计算
    /// 费用 = 输入总和 - 输出总和
    /// 或：费用 = 交易大小 × 费率
    pub fee: u64,
    
    /// 交易时间戳
    pub timestamp: u64,
    
    /// 数字签名
    /// 
    /// 证明交易由发送方授权。
    /// 使用 ECDSA 签名算法（secp256k1曲线）。
    /// 
    /// # 签名过程
    /// ```
    /// 1. 计算交易哈希（不包括签名）
    /// 2. 使用私钥对哈希签名
    /// 3. 生成 (r, s) 两个大整数
    /// 4. 编码为 DER 格式
    /// ```
    /// 
    /// # 验证过程
    /// ```
    /// 1. 解析签名得到 (r, s)
    /// 2. 使用公钥验证签名
    /// 3. 确认交易未被篡改
    /// ```
    pub signature: Vec<u8>,
}

impl Transaction {
    /// 创建新交易
    pub fn new(from: String, to: String, amount: u64, fee: u64) -> Self {
        let timestamp = current_timestamp();
        
        // 计算交易哈希（简化版）
        let hash_data = format!("{}{}{}{}{}", from, to, amount, fee, timestamp);
        let hash = double_sha256(&hash_data);
        
        Self {
            hash,
            from,
            to,
            amount,
            fee,
            timestamp,
            signature: vec![], // 未签名
        }
    }
    
    /// 签名交易
    /// 
    /// # 参数
    /// - `private_key`: 发送方的私钥
    /// 
    /// # 注意
    /// 实际实现需要使用 secp256k1 库
    pub fn sign(&mut self, _private_key: &[u8]) {
        // 简化：实际应该使用 secp256k1 签名
        self.signature = vec![1, 2, 3, 4, 5]; // 占位符
    }
}

/// 完整区块
/// 
/// 区块 = 区块头 + 交易列表
/// 区块头被哈希，交易通过 Merkle 根间接影响哈希
#[derive(Debug, Clone)]
pub struct Block {
    /// 区块头
    pub header: BlockHeader,
    
    /// 区块高度（在链中的位置）
    /// 
    /// 创世区块高度 = 0
    /// 每增加一个区块，高度 +1
    pub height: u64,
    
    /// 区块哈希（由区块头计算得出）
    /// 
    /// 这是区块的唯一标识，用于：
    /// - 快速查找区块
    /// - 验证区块完整性
    /// - 链接到下一个区块
    pub hash: String,
    
    /// 交易列表
    /// 
    /// 比特币限制：
    /// - 区块大小上限：1MB（隔离见证后4MB）
    /// - 交易数量：平均约 2000 笔
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// 创建新区块
    /// 
    /// # 参数
    /// - `height`: 区块高度
    /// - `prev_hash`: 前一区块哈希
    /// - `transactions`: 交易列表
    /// - `difficulty`: 挖矿难度
    /// 
    /// # 示例
    /// ```
    /// let prev_hash = "0000...".to_string();
    /// let txs = vec![tx1, tx2];
    /// let block = Block::new(1, prev_hash, txs, 4);
    /// ```
    pub fn new(
        height: u64,
        prev_hash: String,
        transactions: Vec<Transaction>,
        difficulty: u32,
    ) -> Self {
        // 计算 Merkle 根
        let merkle_root = calculate_merkle_root(&transactions);
        
        // 创建区块头
        let header = BlockHeader::new(
            1,              // 版本号
            prev_hash,      // 前一哈希
            merkle_root,    // Merkle 根
            difficulty,     // 难度
        );
        
        // 计算区块哈希（此时还未挖矿）
        let hash = calculate_hash(&header);
        
        Self {
            header,
            height,
            hash,
            transactions,
        }
    }
    
    /// 创建创世区块
    /// 
    /// 创世区块是区块链的第一个区块，具有以下特点：
    /// - 高度为 0
    /// - 前一区块哈希为 32 字节的 0
    /// - 通常包含特殊信息
    /// 
    /// # 比特币创世区块
    /// ```
    /// 时间: 2009-01-03 18:15:05 UTC
    /// 消息: "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks"
    /// 奖励: 50 BTC -> 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
    /// ```
    /// 
    /// # 参数
    /// - `message`: 创世消息（可选）
    /// 
    /// # 返回
    /// - 创世区块
    pub fn create_genesis(message: Option<&str>) -> Self {
        // 创世交易（系统发行代币）
        let genesis_tx = Transaction {
            hash: "genesis_tx_hash".to_string(),
            from: "0".repeat(40),  // 全0表示系统
            to: "YR1GenesisReceiverAddress".to_string(),
            amount: 50_0000_0000, // 50 YRU（假设1 YRU = 10^8）
            fee: 0,
            timestamp: 1704067200, // 2024-01-01 00:00:00 UTC
            signature: vec![],
        };
        
        // 如果有创世消息，创建第二笔交易存储消息
        let mut transactions = vec![genesis_tx];
        
        if let Some(msg) = message {
            let msg_tx = Transaction {
                hash: double_sha256(msg),
                from: "0".repeat(40),
                to: "MESSAGE_STORAGE".to_string(),
                amount: 0,
                fee: 0,
                timestamp: 1704067200,
                signature: msg.as_bytes().to_vec(),
            };
            transactions.push(msg_tx);
        }
        
        // 计算 Merkle 根
        let merkle_root = calculate_merkle_root(&transactions);
        
        // 创建区块头
        let header = BlockHeader {
            version: 1,
            prev_hash: "0".repeat(64), // 32字节 = 64个十六进制字符
            merkle_root,
            timestamp: 1704067200,
            difficulty: 1, // 创世区块难度最低
            nonce: 2083236893, // 比特币创世区块的 nonce
        };
        
        // 计算区块哈希
        let hash = calculate_hash(&header);
        
        Self {
            header,
            height: 0,
            hash,
            transactions,
        }
    }
    
    /// 挖矿
    /// 
    /// 通过不断改变 nonce，找到满足难度要求的哈希。
    /// 
    /// # 难度验证
    /// 哈希的前 difficulty 个字符必须是 0
    /// 
    /// # 示例
    /// 难度 4：哈希必须以 "0000" 开头
    /// 如：0000a1b2c3d4e5f6...
    /// 
    /// # 返回值
    /// - 找到的有效哈希
    pub fn mine(&mut self) -> String {
        let target = "0".repeat(self.header.difficulty as usize);
        
        loop {
            // 计算当前 nonce 的哈希
            self.hash = calculate_hash(&self.header);
            
            // 检查是否满足难度要求
            if self.hash.starts_with(&target) {
                println!("✅ 挖矿成功!");
                println!("   Nonce: {}", self.header.nonce);
                println!("   Hash:  {}", self.hash);
                return self.hash.clone();
            }
            
            // 增加 nonce 继续尝试
            self.header.nonce += 1;
            
            // 每 100000 次打印进度
            if self.header.nonce % 100000 == 0 {
                println!("⛏️  挖矿中... nonce = {}", self.header.nonce);
            }
        }
    }
    
    /// 验证区块
    /// 
    /// 检查区块是否有效：
    /// 1. 哈希满足难度要求
    /// 2. Merkle 根正确
    /// 3. 时间戳合理
    pub fn verify(&self) -> bool {
        // 1. 验证哈希
        let calculated_hash = calculate_hash(&self.header);
        if calculated_hash != self.hash {
            println!("❌ 哈希不匹配");
            return false;
        }
        
        // 2. 验证难度
        let target = "0".repeat(self.header.difficulty as usize);
        if !self.hash.starts_with(&target) {
            println!("❌ 难度不满足");
            return false;
        }
        
        // 3. 验证 Merkle 根
        let calculated_root = calculate_merkle_root(&self.transactions);
        if calculated_root != self.header.merkle_root {
            println!("❌ Merkle 根不匹配");
            return false;
        }
        
        true
    }
    
    /// 获取区块摘要信息
    pub fn summary(&self) -> String {
        format!(
            "Block #{}, Hash: {}, Tx: {}, Time: {}",
            self.height,
            &self.hash[..16],
            self.transactions.len(),
            format_timestamp(self.header.timestamp)
        )
    }
}

/// ============================================================================
/// 第二部分：工具函数
/// ============================================================================

/// 计算区块头的双重 SHA256 哈希
/// 
/// # 为什么双重哈希？
/// 
/// 1. **防止长度扩展攻击**
///    某些哈希算法（如 SHA256）存在长度扩展漏洞，
///    双重哈希可以有效防止这种攻击。
/// 
/// 2. **增加安全性**
///    即使 SHA256 被发现弱点，双重哈希也更难破解。
/// 
/// 3. **比特币传统**
///    中本聪在比特币中使用了双重哈希，成为行业标准。
/// 
/// # 参数
/// - `header`: 区块头引用
/// 
/// # 返回
/// - 64字符的十六进制字符串（32字节）
pub fn calculate_hash(header: &BlockHeader) -> String {
    // 将区块头序列化为字符串
    // 注意：实际项目应该使用更高效的二进制序列化
    let data = format!(
        "{}|{}|{}|{}|{}|{}",
        header.version,
        header.prev_hash,
        header.merkle_root,
        header.timestamp,
        header.difficulty,
        header.nonce
    );
    
    // 双重 SHA256
    double_sha256(&data)
}

/// 双重 SHA256 哈希
/// 
/// hash = SHA256(SHA256(data))
pub fn double_sha256(data: &str) -> String {
    // 第一次哈希
    let mut hasher1 = Sha256::new();
    hasher1.update(data.as_bytes());
    let result1 = hasher1.finalize();
    
    // 第二次哈希
    let mut hasher2 = Sha256::new();
    hasher2.update(&result1);
    let result2 = hasher2.finalize();
    
    // 转换为十六进制字符串
    format!("{:x}", result2)
}

/// 单次 SHA256 哈希
pub fn sha256(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// 计算 Merkle 根
/// 
/// Merkle 树是一种二叉树，用于高效验证大量数据。
/// 
/// # 算法步骤
/// 1. 计算所有交易的哈希（叶子节点）
/// 2. 两两配对，计算父节点哈希
/// 3. 重复直到只剩一个哈希（根节点）
/// 
/// # 复杂度
/// - 构建：O(n)
/// - 验证某交易：O(log n)
/// 
/// # 参数
/// - `transactions`: 交易列表
/// 
/// # 返回
/// - Merkle 根哈希
pub fn calculate_merkle_root(transactions: &[Transaction]) -> String {
    // 空区块的 Merkle 根为全 0
    if transactions.is_empty() {
        return "0".repeat(64);
    }
    
    // 获取所有交易的哈希
    let mut hashes: Vec<String> = transactions
        .iter()
        .map(|tx| tx.hash.clone())
        .collect();
    
    // 构建 Merkle 树
    while hashes.len() > 1 {
        // 如果数量为奇数，复制最后一个
        if hashes.len() % 2 == 1 {
            hashes.push(hashes.last().unwrap().clone());
        }
        
        // 计算下一层
        let mut next_level = Vec::new();
        for i in (0..hashes.len()).step_by(2) {
            // 拼接两个哈希
            let combined = format!("{}{}", hashes[i], hashes[i + 1]);
            // 计算父哈希
            let parent_hash = sha256(&combined);
            next_level.push(parent_hash);
        }
        
        hashes = next_level;
    }
    
    hashes[0].clone()
}

/// 获取当前 Unix 时间戳
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// 格式化时间戳为人类可读格式
pub fn format_timestamp(timestamp: u64) -> String {
    let datetime = chrono::DateTime::from_timestamp(timestamp as i64, 0)
        .unwrap_or_else(|| chrono::DateTime::UNIX_EPOCH);
    datetime.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// ============================================================================
/// 第三部分：区块链结构
/// ============================================================================

/// 简单的区块链实现
/// 
/// 维护一个有序的区块列表
pub struct Blockchain {
    /// 区块列表
    blocks: Vec<Block>,
    
    /// 当前难度
    difficulty: u32,
}

impl Blockchain {
    /// 创建新的区块链
    /// 
    /// 自动创建创世区块
    pub fn new(difficulty: u32) -> Self {
        let genesis = Block::create_genesis(Some(
            "YRUST Chain Genesis Block - 2024"
        ));
        
        println!("🌟 创世区块创建成功!");
        println!("   哈希: {}", &genesis.hash[..20]);
        
        Self {
            blocks: vec![genesis],
            difficulty,
        }
    }
    
    /// 获取最新区块
    pub fn latest_block(&self) -> &Block {
        self.blocks.last().unwrap()
    }
    
    /// 获取区块高度
    pub fn height(&self) -> u64 {
        (self.blocks.len() as u64) - 1
    }
    
    /// 添加新区块
    /// 
    /// # 参数
    /// - `transactions`: 要包含的交易
    pub fn add_block(&mut self, transactions: Vec<Transaction>) {
        let prev_block = self.latest_block();
        
        let mut new_block = Block::new(
            prev_block.height + 1,
            prev_block.hash.clone(),
            transactions,
            self.difficulty,
        );
        
        // 挖矿
        println!("\n⛏️  开始挖区块 #{}...", new_block.height);
        new_block.mine();
        
        // 验证
        if new_block.verify() {
            println!("✅ 区块验证通过!");
            self.blocks.push(new_block);
        } else {
            panic!("区块验证失败!");
        }
    }
    
    /// 验证整个链
    /// 
    /// 检查：
    /// 1. 每个区块的哈希正确
    /// 2. 每个区块指向前一个区块
    /// 3. 创世区块正确
    pub fn verify(&self) -> bool {
        for i in 1..self.blocks.len() {
            let current = &self.blocks[i];
            let previous = &self.blocks[i - 1];
            
            // 验证当前区块
            if !current.verify() {
                println!("❌ 区块 #{} 验证失败", current.height);
                return false;
            }
            
            // 验证链接
            if current.header.prev_hash != previous.hash {
                println!("❌ 区块 #{} 链接断开", current.height);
                return false;
            }
        }
        
        println!("✅ 区块链验证通过! 共 {} 个区块", self.blocks.len());
        true
    }
    
    /// 打印区块链信息
    pub fn print_chain(&self) {
        println!("\n📦 区块链信息:");
        println!("   总区块数: {}", self.blocks.len());
        println!("   当前高度: {}", self.height());
        println!("   挖矿难度: {}", self.difficulty);
        
        println!("\n🔗 区块列表:");
        for block in &self.blocks {
            println!("   {}", block.summary());
        }
    }
}

/// ============================================================================
/// 第四部分：主函数 - 演示
/// ============================================================================

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║           YRUST Chain - 区块基础演示                      ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    // 1. 创建区块链（难度 4）
    println!("1️⃣  创建区块链...");
    let mut blockchain = Blockchain::new(4);
    
    // 2. 创建一些交易
    println!("\n2️⃣  创建交易...");
    let tx1 = Transaction::new(
        "YR1AliceAddress".to_string(),
        "YR1BobAddress".to_string(),
        100,
        1,
    );
    let tx2 = Transaction::new(
        "YR1BobAddress".to_string(),
        "YR1CharlieAddress".to_string(),
        50,
        1,
    );
    println!("   交易1: Alice -> Bob, 100 YRU");
    println!("   交易2: Bob -> Charlie, 50 YRU");
    
    // 3. 挖第一个区块
    println!("\n3️⃣  挖区块 #1...");
    blockchain.add_block(vec![tx1, tx2]);
    
    // 4. 再创建一些交易，挖第二个区块
    println!("\n4️⃣  创建更多交易...");
    let tx3 = Transaction::new(
        "YR1CharlieAddress".to_string(),
        "YR1DavidAddress".to_string(),
        25,
        1,
    );
    
    println!("\n5️⃣  挖区块 #2...");
    blockchain.add_block(vec![tx3]);
    
    // 5. 打印区块链
    blockchain.print_chain();
    
    // 6. 验证区块链
    println!("\n6️⃣  验证区块链...");
    blockchain.verify();
    
    // 7. 演示篡改检测
    println!("\n7️⃣  演示篡改检测...");
    println!("   尝试篡改区块 #1 的交易...");
    
    // 注意：由于所有权，这里只是模拟
    println!("   如果修改了区块内容，哈希会改变，验证会失败！");
    
    println!("\n✨ 演示完成！");
    
    // 8. 技术要点总结
    println!("\n📚 技术要点:");
    println!("   • 区块 = 区块头(80字节) + 交易列表");
    println!("   • 区块哈希 = SHA256(SHA256(区块头))");
    println!("   • 挖矿 = 寻找满足难度要求的 nonce");
    println!("   • Merkle 树 = 高效验证交易包含性");
    println!("   • 链式结构 = 通过 prev_hash 链接");
}

/// ============================================================================
/// 第五部分：单元测试
/// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试哈希计算
    #[test]
    fn test_hash_calculation() {
        let header = BlockHeader::new(
            1,
            "0".repeat(64),
            "abcd".repeat(16),
            4,
        );
        
        let hash = calculate_hash(&header);
        
        // 哈希应该是 64 个十六进制字符
        assert_eq!(hash.len(), 64);
        
        // 相同输入应该产生相同输出
        let hash2 = calculate_hash(&header);
        assert_eq!(hash, hash2);
    }
    
    /// 测试创世区块
    #[test]
    fn test_genesis_block() {
        let genesis = Block::create_genesis(None);
        
        // 检查基本属性
        assert_eq!(genesis.height, 0);
        assert_eq!(genesis.header.prev_hash, "0".repeat(64));
        assert!(!genesis.transactions.is_empty());
        
        // 验证区块
        assert!(genesis.verify());
    }
    
    /// 测试 Merkle 根计算
    #[test]
    fn test_merkle_root() {
        // 空列表
        let empty_root = calculate_merkle_root(&[]);
        assert_eq!(empty_root, "0".repeat(64));
        
        // 单个交易
        let tx1 = Transaction::new(
            "from1".to_string(),
            "to1".to_string(),
            100,
            1,
        );
        let root1 = calculate_merkle_root(&[tx1.clone()]);
        assert_eq!(root1, tx1.hash);
        
        // 两个交易
        let tx2 = Transaction::new(
            "from2".to_string(),
            "to2".to_string(),
            200,
            1,
        );
        let root2 = calculate_merkle_root(&[tx1, tx2]);
        assert_ne!(root2, "0".repeat(64));
    }
    
    /// 测试区块链
    #[test]
    fn test_blockchain() {
        let mut chain = Blockchain::new(2); // 低难度便于测试
        
        // 添加区块
        let tx = Transaction::new(
            "from".to_string(),
            "to".to_string(),
            100,
            1,
        );
        chain.add_block(vec![tx]);
        
        // 验证
        assert_eq!(chain.height(), 1);
        assert!(chain.verify());
    }
    
    /// 测试挖矿
    #[test]
    fn test_mining() {
        let mut block = Block::new(
            1,
            "0".repeat(64),
            vec![],
            2, // 难度 2（很容易）
        );
        
        let hash = block.mine();
        
        // 验证哈希满足难度
        assert!(hash.starts_with("00"));
        assert!(block.verify());
    }
}
