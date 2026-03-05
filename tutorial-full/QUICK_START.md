# YRUST Chain 快速开始指南

## 5分钟上手

### 1. 环境准备

```bash
# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# 验证安装
rustc --version  # rustc 1.75.0
```

### 2. 运行第一章示例

```bash
cd tutorial-full/01-basic
cargo run
```

**预期输出：**
```
🌟 创世区块创建成功!
   哈希: 0000a1b2c3d4...

⛏️  开始挖区块 #1...
✅ 挖矿成功!
   Nonce: 12345
   Hash:  0000e5f6a7b8...

📦 区块链信息:
   总区块数: 3
   当前高度: 2
```

### 3. 运行第二章示例

```bash
cd tutorial-full/02-crypto
cargo run
```

**预期输出：**
```
SHA256("hello") = 2cf24dba5fb0a30e26e83b2ac5b9e29e...

Merkle 根: a3f2b8c1d4e5...

地址: YR1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
验证: ✅ 有效
```

## 核心概念速查

### 区块结构
```
┌────────────────────────────────────────────────────────────┐
│  区块头 (80 bytes)                                          │
│  ├── 版本号 (4 bytes)                                       │
│  ├── 前一哈希 (32 bytes)                                    │
│  ├── Merkle根 (32 bytes)                                    │
│  ├── 时间戳 (4 bytes)                                       │
│  ├── 难度 (4 bytes)                                         │
│  └── Nonce (4 bytes)                                        │
├────────────────────────────────────────────────────────────┤
│  交易列表 (可变)                                            │
└────────────────────────────────────────────────────────────┘
```

### 关键公式

```rust
// 区块哈希
block_hash = SHA256(SHA256(block_header))

// Merkle 根
merkle_root = build_tree(tx_hashes).root

// 地址生成
address = Base58Check(version + RIPEMD160(SHA256(pubkey)) + checksum)

// 挖矿验证
check(block_hash.starts_with("0".repeat(difficulty)))
```

## 代码片段

### 创建区块
```rust
let block = Block::new(
    1,                          // 高度
    prev_hash,                  // 前一哈希
    transactions,               // 交易列表
    4,                          // 难度
);
block.mine();                   // 挖矿
```

### 计算哈希
```rust
let hash = sha256("data");      // 单次哈希
let hash = double_sha256("data"); // 双重哈希（比特币标准）
```

### 验证交易
```rust
let valid = verify_signature(
    &public_key,
    &message,
    &signature
);
```

## 学习路径

```
Week 1: 基础
├── 01-basic/     - 区块结构、挖矿
├── 02-crypto/    - 哈希、签名、地址
└── 03-blockchain/ - 链式结构、POW

Week 2: 进阶
├── 04-network/   - P2P网络
├── 05-consensus/ - 共识算法
└── 06-storage/   - 数据存储

Week 3: 高级
├── 07-vm/        - WASM虚拟机
├── 08-contracts/ - 智能合约
└── 09-security/  - 安全加固

Week 4: 部署
└── 10-api/       - API开发、部署
```

## 常见问题

**Q: 挖矿难度如何调整？**
```rust
// 每 2016 个区块调整一次
new_difficulty = old_difficulty * (2016 * 10_minutes) / actual_time
```

**Q: 为什么需要 Merkle 树？**
```
轻节点只需下载：
- 区块头（80字节）
- Merkle路径（约320字节）

而不需要下载：
- 完整区块（约1MB）
```

**Q: 如何防止双花？**
```
1. 交易打包进区块
2. 等待6个确认
3. UTXO模型确保每笔输入只能花费一次
```

## 下一步

- 📖 阅读完整教程：[01-basic/README.md](01-basic/)
- 💻 查看完整代码：[01-basic/main.rs](01-basic/main.rs)
- 🧪 运行测试：`cargo test`
- 🎯 完成练习：每章末尾的练习题

## 资源链接

- [比特币白皮书](https://bitcoin.org/bitcoin.pdf)
- [以太坊黄皮书](https://ethereum.github.io/yellowpaper/paper.pdf)
- [Rust 官方文档](https://doc.rust-lang.org/)
- [libp2p 文档](https://docs.libp2p.io/)
