# Rust 区块链开发完全教程

> 从零开始用 Rust 构建一个完整的区块链系统

## 📚 课程目录

### 第一阶段：基础篇

| 章节 | 内容 | 代码路径 |
|------|------|----------|
| [第1章](#第1章区块结构) | 区块结构 | `tutorial/01-block-structure/` |
| [第2章](#第2章哈希计算) | 哈希计算 | `tutorial/02-hash/` |
| [第3章](#第3章区块链) | 区块链 | `tutorial/03-blockchain/` |
| [第4章](#第4章工作量证明) | 工作量证明 | `tutorial/04-pow/` |

### 第二阶段：密码学与账户

| 章节 | 内容 | 代码路径 |
|------|------|----------|
| [第5章](#第5章钱包系统) | 钱包系统 | `src/wallet/` |
| [第6章](#第6章交易签名) | 交易签名 | `src/core/` |

### 第三阶段：交易模型

| 章节 | 内容 | 代码路径 |
|------|------|----------|
| [第7章](#第7章utxo模型) | UTXO 模型 | `src/core/` |
| [第8章](#第8章p2p网络) | P2P 网络 | `src/network/` |

### 第四阶段：应用层

| 章节 | 内容 | 代码路径 |
|------|------|----------|
| [第9章](#第9章api接口) | API 接口 | `src/api/` |
| [第10章](#第10章智能合约) | 智能合约 | `src/vm/` |

---

## 快速开始

```bash
# 克隆项目
cd /Users/yc/RustroverProjects/YRust

# 运行演示
cargo run -- demo

# 运行测试
cargo test
```

---

## 第1章：区块结构

### 1.1 什么是区块？

区块是区块链的基本单位：

```rust
struct Block {
    index: u64,        // 区块高度
    timestamp: i64,    // 创建时间
    data: String,      // 数据
    prev_hash: String, // 前一个区块哈希
    hash: String,      // 本区块哈希
}
```

### 1.2 创建区块

```rust
impl Block {
    fn new(index: u64, data: String, prev_hash: String) -> Self {
        Block {
            index,
            timestamp: chrono::Utc::now().timestamp(),
            data,
            prev_hash,
            hash: String::new(),
        }
    }
}
```

---

## 第2章：哈希计算

### 2.1 SHA256 哈希

```rust
use sha2::{Sha256, Digest};

fn calculate_hash(&self) -> String {
    let data = format!("{}{}{}{}", self.index, self.timestamp, self.data, self.prev_hash);
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}
```

### 2.2 哈希特性

- **确定性**：相同输入 → 相同输出
- **不可逆**：无法反推原文
- **抗碰撞**：难以找到碰撞
- **雪崩效应**：微小变化导致巨大差异

---

## 第3章：区块链

### 3.1 链式结构

```rust
struct Blockchain {
    chain: Vec<Block>,
}

impl Blockchain {
    fn new() -> Self {
        let genesis = Block::new(0, "Genesis".to_string(), "0".to_string());
        Blockchain { chain: vec![genesis] }
    }
}
```

### 3.2 验证链

```rust
fn is_valid(&self) -> bool {
    for i in 1..self.chain.len() {
        let current = &self.chain[i];
        let previous = &self.chain[i-1];
        
        if current.hash != current.calculate_hash() {
            return false;  // 数据被篡改
        }
        if current.prev_hash != previous.hash {
            return false;  // 链接断裂
        }
    }
    true
}
```

---

## 第4章：工作量证明

### 4.1 什么是 POW？

通过计算难题来竞争记账权：

```rust
fn mine(&mut self, difficulty: usize) {
    let target = "0".repeat(difficulty);
    
    loop {
        self.hash = self.calculate_hash();
        if self.hash.starts_with(&target) {
            break;  // 找到符合条件的哈希
        }
        self.nonce += 1;  // 尝试下一个
    }
}
```

### 4.2 难度调整

- 难度 = 前导零个数
- 难度 4：约 1-2 秒
- 难度 6：约 1-2 分钟

---

## 第5章：钱包系统

### 5.1 密钥对生成

```rust
use secp256k1::{Secp256k1, SecretKey, PublicKey};

fn generate_wallet() -> Wallet {
    let secp = Secp256k1::new();
    let (secret_key, public_key) = secp.generate_keypair(&mut rand::thread_rng());
    
    Wallet {
        private_key: hex::encode(secret_key.secret_bytes()),
        public_key: hex::encode(public_key.serialize()),
        address: generate_address(&public_key),
    }
}
```

### 5.2 地址生成

1. SHA256(公钥)
2. RIPEMD160(SHA256(公钥))
3. 添加版本字节
4. 计算校验和
5. Base58 编码

---

## 第6章：交易签名

### 6.1 交易结构

```rust
struct Transaction {
    id: String,
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
    timestamp: i64,
}
```

### 6.2 签名流程

```rust
fn sign_transaction(&self, private_key: &[u8]) -> Vec<u8> {
    let message = Message::from_slice(&hash_transaction(self));
    let sig = secp.sign_ecdsa(&message, &secret_key);
    sig.serialize_der().to_vec()
}
```

---

## 第7章：UTXO模型

### 7.1 什么是 UTXO？

- **U**nspent **T**ransaction **O**utput
- 未花费的交易输出
- 比特币使用的模型

### 7.2 UTXO 集合

```rust
struct UtxoSet {
    utxos: HashMap<(TxId, Vout), TxOutput>,
}

impl UtxoSet {
    fn find_spendable(&self, address: &str, amount: u64) -> Vec<Utxo> {
        // 查找足够金额的 UTXO
    }
}
```

---

## 第8章：P2P网络

### 8.1 网络消息

```rust
enum NetworkMessage {
    NewBlock(Block),
    NewTransaction(Transaction),
    GetBlocks { start: u64, end: u64 },
    BlocksResponse(Vec<Block>),
}
```

### 8.2 节点通信

```rust
async fn handle_message(msg: NetworkMessage) {
    match msg {
        NewBlock(block) => validate_and_add(block),
        NewTransaction(tx) => add_to_mempool(tx),
        ...
    }
}
```

---

## 第9章：API接口

### 9.1 REST API

```rust
// 获取区块链统计
GET /api/stats

// 获取区块列表
GET /api/blocks

// 创建钱包
POST /api/wallet/create

// 挖矿
POST /api/mine

// 创建交易
POST /api/tx/create
```

### 9.2 启动 API

```bash
cargo run -- node
# API 在 http://localhost:3000
```

---

## 第10章：智能合约

### 10.1 WASM 合约

```rust
struct ContractEngine {
    wasm_runtime: wasmtime::Engine,
}

impl ContractEngine {
    fn deploy(&mut self, code: Vec<u8>) -> ContractAddress {
        // 部署合约
    }
    
    fn call(&self, contract: &Address, func: &str, args: Vec<u8>) -> Result {
        // 执行合约
    }
}
```

---

## 实战练习

### 练习1：创建自己的区块链

```bash
cd tutorial/03-blockchain
cargo run
```

### 练习2：挖矿

```bash
cargo run -- miner
```

### 练习3：P2P 网络

```bash
# 终端1
cargo run -- p2p --port 8001

# 终端2
cargo run -- p2p --port 8002 --peer 127.0.0.1:8001
```

---

## 进阶学习

1. **共识算法**：POW、POS、DPOS
2. **分片技术**：提高吞吐量
3. **跨链技术**：链间通信
4. **隐私保护**：零知识证明

---

## 参考资料

- [比特币白皮书](https://bitcoin.org/bitcoin.pdf)
- [Rust 官方文档](https://doc.rust-lang.org/)
- [精通比特币](https://github.com/inoutcode/bitcoin_book_2nd)

---

祝你学习愉快！🚀
