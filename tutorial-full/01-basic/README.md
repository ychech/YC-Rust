# 01. 区块基础 - 区块链的基石

## 本章目标

理解区块的基本结构，学会计算区块哈希，创建创世区块。

## 什么是区块？

区块是区块链的基本单位，就像账本中的一页。

```
┌────────────────────────────────────────────────────────────┐
│                        区块结构                             │
├────────────────────────────────────────────────────────────┤
│  区块头 (Block Header)                                     │
│  ├── 版本号 (Version)      - 协议版本                      │
│  ├── 前一区块哈希          - 链接前一个区块                 │
│  ├── Merkle根              - 所有交易的哈希摘要            │
│  ├── 时间戳                - 区块创建时间                   │
│  ├── 难度目标              - 挖矿难度                       │
│  └── 随机数 (Nonce)        - 挖矿找到的随机数              │
├────────────────────────────────────────────────────────────┤
│  区块体 (Block Body)                                       │
│  └── 交易列表              - 包含的所有交易                │
└────────────────────────────────────────────────────────────┘
```

## 核心概念详解

### 1. 区块哈希

区块哈希是整个区块的"指纹"，具有以下特性：
- **唯一性**：不同内容必定产生不同哈希
- **确定性**：相同内容必定产生相同哈希
- **不可逆**：无法从哈希反推原始内容
- **敏感性**：微小改动导致哈希完全不同

### 2. 链式结构

```
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐
│ 区块 0  │───→│ 区块 1  │───→│ 区块 2  │───→│ 区块 3  │
│ (创世)  │    │         │    │         │    │  (最新) │
│ Hash: 0a│    │ Prev: 0a│    │ Prev: 1b│    │ Prev: 2c│
│         │    │ Hash: 1b│    │ Hash: 2c│    │ Hash: 3d│
└─────────┘    └─────────┘    └─────────┘    └─────────┘
```

每个区块都包含前一个区块的哈希，形成不可篡改的链条。

### 3. 创世区块

创世区块是区块链的第一个区块：
- 高度为 0
- 前一区块哈希为全 0
- 通常包含特殊信息（如比特币的《泰晤士报》头版）

## 代码实现

### 步骤 1：定义区块结构

```rust
/// 区块头
/// 
/// 区块头是区块的元数据，包含了区块的核心信息。
/// 在比特币中，区块头只有 80 字节，但包含了所有关键信息。
#[derive(Debug, Clone)]
pub struct BlockHeader {
    /// 版本号 - 用于协议升级
    /// 例如：比特币从 v1 升级到 v2 用于 BIP34
    pub version: u32,
    
    /// 前一区块的哈希值
    /// 这是形成区块链的关键，将区块链接在一起
    /// 创世区块的前一哈希为 32 字节的 0
    pub prev_hash: String,
    
    /// Merkle 树根哈希
    /// 这是所有交易哈希组成的二叉树的根
    /// 用于快速验证交易是否包含在区块中
    pub merkle_root: String,
    
    /// 时间戳 - 区块创建时间（Unix时间戳）
    /// 用于调整难度和验证区块时效性
    pub timestamp: u64,
    
    /// 难度目标
    /// 决定挖矿难度，表示哈希值必须小于的目标值
    /// 每 2016 个区块调整一次（比特币）
    pub difficulty: u32,
    
    /// 随机数 - 挖矿时不断改变的数字
    /// 矿工通过改变这个值来寻找满足难度要求的哈希
    pub nonce: u64,
}

/// 完整区块
/// 
/// 区块 = 区块头 + 交易列表
/// 区块头被哈希，交易列表通过 Merkle 根间接影响哈希
#[derive(Debug, Clone)]
pub struct Block {
    /// 区块头 - 包含元数据
    pub header: BlockHeader,
    
    /// 区块高度 - 在链中的位置
    /// 创世区块高度为 0
    pub height: u64,
    
    /// 区块哈希 - 区块的唯一标识
    /// 由区块头计算得出
    pub hash: String,
    
    /// 交易列表 - 区块包含的所有交易
    /// 在比特币中平均每个区块约 2000 笔交易
    pub transactions: Vec<Transaction>,
}

/// 交易结构
/// 
/// 表示一笔转账或合约调用
#[derive(Debug, Clone)]
pub struct Transaction {
    /// 交易哈希
    pub hash: String,
    
    /// 发送方地址
    pub from: String,
    
    /// 接收方地址
    pub to: String,
    
    /// 转账金额
    pub amount: u64,
    
    /// 交易费用（矿工费）
    pub fee: u64,
    
    /// 时间戳
    pub timestamp: u64,
    
    /// 数字签名 - 证明交易由发送方授权
    pub signature: Vec<u8>,
}
```

### 步骤 2：计算区块哈希

```rust
use sha2::{Sha256, Digest};

/// 计算区块哈希
/// 
/// # 算法说明
/// 
/// 使用 SHA256 进行双重哈希：
/// ```
/// hash = SHA256(SHA256(block_header))
/// ```
/// 
/// # 为什么双重哈希？
/// 
/// 1. **防止长度扩展攻击**：双重哈希可以抵抗某些密码学攻击
/// 2. **增加安全性**：即使 SHA256 有弱点，双重哈希也更安全
/// 3. **比特币传统**：中本聪的设计选择
/// 
/// # 参数
/// - `header`: 区块头引用
/// 
/// # 返回
/// - `String`: 32字节哈希的十六进制表示（64个字符）
/// 
/// # 示例
/// ```
/// let header = BlockHeader::new(...);
/// let hash = calculate_hash(&header);
/// assert_eq!(hash.len(), 64); // 32字节 = 64个十六进制字符
/// ```
pub fn calculate_hash(header: &BlockHeader) -> String {
    // 第一步：将区块头序列化为字节
    // 实际项目中应该使用更高效的序列化方式（如 protobuf）
    let data = format!(
        "{}{}{}{}{}{}",
        header.version,
        header.prev_hash,
        header.merkle_root,
        header.timestamp,
        header.difficulty,
        header.nonce
    );
    
    // 第二步：第一次 SHA256 哈希
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let first_hash = hasher.finalize();
    
    // 第三步：第二次 SHA256 哈希
    let mut hasher2 = Sha256::new();
    hasher2.update(&first_hash);
    let second_hash = hasher2.finalize();
    
    // 第四步：转换为十六进制字符串
    // format!("{:x}") 将字节数组格式化为十六进制字符串
    format!("{:x}", second_hash)
}
```

### 步骤 3：创建创世区块

```rust
impl Block {
    /// 创建创世区块
    /// 
    /// 创世区块是区块链的第一个区块，具有以下特点：
    /// - 高度为 0
    /// - 前一区块哈希为全 0
    /// - 包含一条特殊消息（可选）
    /// 
    /// # 参数
    /// - `message`: 创世消息（如比特币的《泰晤士报》头版）
    /// 
    /// # 返回
    /// - `Block`: 创世区块
    /// 
    /// # 比特币创世区块
    /// ```
    /// 消息: "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks"
    /// 时间: 2009-01-03 18:15:05 UTC
    /// 奖励: 50 BTC
    /// ```
    pub fn create_genesis(message: &str) -> Self {
        // 创建创世交易（给中本聪的奖励）
        let genesis_tx = Transaction {
            hash: "0".repeat(64), // 简化处理
            from: "0".repeat(40),  // 全0表示系统发行
            to: "YR1GenesisAddress".to_string(),
            amount: 50_0000_0000, // 50 YRU（最小单位）
            fee: 0,
            timestamp: 1704067200, // 2024-01-01 00:00:00 UTC
            signature: vec![],
        };
        
        // 计算 Merkle 根（创世区块只有一笔交易）
        let merkle_root = calculate_merkle_root(&[genesis_tx.clone()]);
        
        // 创建区块头
        let header = BlockHeader {
            version: 1,
            prev_hash: "0".repeat(64), // 32字节的0
            merkle_root,
            timestamp: 1704067200,
            difficulty: 1, // 创世区块难度最低
            nonce: 0,      // 创世区块不需要挖矿
        };
        
        // 计算区块哈希
        let hash = calculate_hash(&header);
        
        Block {
            header,
            height: 0,
            hash,
            transactions: vec![genesis_tx],
        }
    }
}

/// 计算 Merkle 根
/// 
/// Merkle 树是一种二叉树，用于高效验证大量数据。
/// 
/// # 构建过程
/// ```
/// 交易: [Tx1, Tx2, Tx3, Tx4]
/// 
///         Root (Hash12_34)
///          /            \
///    Hash12            Hash34
///    /    \            /    \
/// Hash1  Hash2    Hash3  Hash4
///  |       |        |       |
/// Tx1    Tx2      Tx3     Tx4
/// ```
/// 
/// # 优势
/// - 快速验证：只需 O(log n) 的哈希计算
/// - 轻节点友好：SPV 节点只需下载区块头
fn calculate_merkle_root(transactions: &[Transaction]) -> String {
    if transactions.is_empty() {
        return "0".repeat(64);
    }
    
    // 获取所有交易的哈希
    let mut hashes: Vec<String> = transactions
        .iter()
        .map(|tx| tx.hash.clone())
        .collect();
    
    // 如果交易数量为奇数，复制最后一个
    while hashes.len() > 1 {
        if hashes.len() % 2 == 1 {
            hashes.push(hashes.last().unwrap().clone());
        }
        
        let mut next_level = Vec::new();
        for i in (0..hashes.len()).step_by(2) {
            let combined = format!("{}{}", hashes[i], hashes[i + 1]);
            let hash = sha256(&combined);
            next_level.push(hash);
        }
        hashes = next_level;
    }
    
    hashes[0].clone()
}

/// 单次 SHA256 哈希
fn sha256(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}
```

## 完整代码

见 [main.rs](main.rs)

## 运行测试

```bash
cd tutorial-full/01-basic
cargo run
```

预期输出：
```
创世区块创建成功!
高度: 0
哈希: 0000a1b2c3d4...
前一哈希: 000000000000...
交易数: 1
时间戳: 2024-01-01 00:00:00 UTC
```

## 练习题

### 练习 1：验证哈希特性
修改区块中的任意一个字段，观察哈希如何变化。

### 练习 2：创建多个区块
创建创世区块后，再创建区块 1 和区块 2，形成链式结构。

### 练习 3：实现 Merkle 树
实现完整的 Merkle 树构建和验证功能。

## 挑战任务

🔥 **实现区块浏览器**
- 显示区块列表
- 点击查看区块详情
- 显示交易列表

## 下一章

[02. 密码学基础](../02-crypto/) - 学习 SHA256、Merkle Tree、地址生成
