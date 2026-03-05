# YRUST Chain 完全指南

## 📚 文档索引

### 快速入门
- [快速开始](../tutorial-full/QUICK_START.md) - 5分钟上手
- [主README](../README.md) - 项目概览

### 完整教程（10章）
1. [区块基础](../tutorial-full/01-basic/) - 区块结构、哈希、创世区块
2. [密码学基础](../tutorial-full/02-crypto/) - SHA256、Merkle树、地址
3. [区块链核心](../tutorial-full/03-blockchain/) - 链式结构、POW挖矿
4. [P2P网络](../tutorial-full/04-network/) - libp2p、节点发现
5. [共识算法](../tutorial-full/05-consensus/) - Raft选举、领导者
6. [数据存储](../tutorial-full/06-storage/) - RocksDB、UTXO索引
7. [虚拟机](../tutorial-full/07-vm/) - WASM、Gas计费
8. [智能合约](../tutorial-full/08-contracts/) - Token、NFT、DEX
9. [安全加固](../tutorial-full/09-security/) - 重放保护、权限
10. [API与部署](../tutorial-full/10-api/) - REST API、Docker

### API文档
- [API参考](API.md) - 完整的REST API文档

### 部署文档
- [部署脚本](../scripts/deploy.sh) - 一键部署
- [Docker配置](../docker-compose.yml) - 容器编排

---

## 🗂️ 代码结构

```
src/
├── main.rs              # 程序入口（已注释）
├── lib.rs               # 库导出
├── core/                # 区块链核心
│   ├── mod.rs           # 模块组织
│   ├── models.rs        # 数据模型（Block, Transaction）
│   ├── blockchain.rs    # 区块链逻辑
│   └── types.rs         # 类型定义
├── network/             # 网络模块
│   ├── mod.rs           # 网络接口
│   ├── p2p.rs           # P2P实现（libp2p）
│   └── consensus.rs     # 共识算法（Raft）
├── storage/             # 存储模块
│   ├── mod.rs           # 存储接口
│   └── rocksdb.rs       # RocksDB实现
├── vm/                  # 虚拟机
│   ├── mod.rs           # VM接口
│   └── engine.rs        # WASM执行引擎
├── security/            # 安全模块
│   └── mod.rs           # 重放保护、权限控制
├── monitoring/          # 监控模块
│   └── mod.rs           # 指标、告警
├── api/                 # API模块
│   └── mod.rs           # REST API实现
└── wallet/              # 钱包模块
    └── mod.rs           # 密钥管理
```

---

## 🔑 核心概念速查

### 区块哈希计算
```rust
// 双重SHA256
hash = SHA256(SHA256(block_header))

// 验证难度
assert!(hash.starts_with("0".repeat(difficulty)))
```

### 交易验证流程
```rust
1. 检查交易格式
2. 验证签名
3. 检查输入在UTXO集中
4. 验证输入金额 >= 输出金额 + 手续费
5. 检查双花
6. 加入内存池
```

### 挖矿流程
```rust
1. 从内存池选择交易
2. 创建coinbase交易（奖励）
3. 计算Merkle根
4. 尝试nonce直到hash < target
5. 广播新区块
6. 更新UTXO集
```

---

## 📝 关键代码片段

### 1. 创建交易
```rust
// 创建转账交易
let tx = Transaction::new(
    from_address,
    to_address,
    amount,
    fee,
);

// 签名
tx.sign(&private_key);

// 广播
network.broadcast(tx);
```

### 2. 验证区块
```rust
fn verify_block(&self, block: &Block) -> bool {
    // 检查前一区块
    if block.header.prev_hash != self.latest_hash() {
        return false;
    }
    
    // 验证难度
    if !block.hash.starts_with("0".repeat(difficulty)) {
        return false;
    }
    
    // 验证Merkle根
    if block.header.merkle_root != calculate_merkle(&block.transactions) {
        return false;
    }
    
    // 验证所有交易
    for tx in &block.transactions {
        if !self.verify_transaction(tx) {
            return false;
        }
    }
    
    true
}
```

### 3. P2P消息处理
```rust
match message {
    NetworkMessage::NewBlock(block) => {
        if self.verify_block(&block) {
            self.chain.add_block(block);
            self.broadcast(message);
        }
    }
    NetworkMessage::NewTransaction(tx) => {
        if self.verify_transaction(&tx) {
            self.mempool.add(tx);
        }
    }
    NetworkMessage::GetBlocks(height) => {
        let blocks = self.chain.get_blocks_from(height);
        peer.send(NetworkMessage::Blocks(blocks));
    }
}
```

---

## 🧪 测试命令

```bash
# 运行所有测试
cargo test

# 运行特定测试
cargo test test_mining

# 运行带输出
cargo test -- --nocapture

# 运行示例
cargo run -- demo
cargo run -- node
cargo run -- miner
cargo run -- wallet
```

---

## 🚀 部署命令

```bash
# 本地启动
cargo run -- node

# Docker启动
docker-compose up -d

# 服务器部署
sudo bash scripts/deploy.sh
```

---

## 📖 学习建议

### 初学者路径
1. 阅读 [01-basic](../tutorial-full/01-basic/) - 理解区块结构
2. 阅读 [02-crypto](../tutorial-full/02-crypto/) - 理解哈希和地址
3. 运行 `cargo run -- demo` - 看完整演示
4. 阅读 [API文档](API.md) - 了解接口

### 进阶路径
1. 完成所有10章教程
2. 阅读核心源码（带注释）
3. 实现练习题
4. 部署测试网

### 专家路径
1. 深入共识算法源码
2. 优化VM性能
3. 实现新功能
4. 贡献代码

---

## 🔗 外部资源

- [比特币白皮书](https://bitcoin.org/bitcoin.pdf)
- [以太坊黄皮书](https://ethereum.github.io/yellowpaper/paper.pdf)
- [Rust Book](https://doc.rust-lang.org/book/)
- [libp2p Docs](https://docs.libp2p.io/)

---

**开始你的区块链之旅！** 🚀
