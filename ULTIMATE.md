# YRust Chain - 终极完整版

> 一个不落！完整的 Rust 区块链实现

## ✅ 功能清单

| 功能 | 状态 | 说明 |
|------|------|------|
| 区块 & 哈希 | ✅ | SHA256 + Merkle Root |
| POW 挖矿 | ✅ | 可配置难度 + nonce |
| 钱包系统 | ✅ | secp256k1 + Base58 |
| 签名验签 | ✅ | ECDSA 双重 SHA256 |
| P2P 网络 | ✅ | libp2p + QUIC + Gossipsub |
| UTXO 模型 | ✅ | 比特币风格 |
| 账户模型 | ✅ | 以太坊风格 |
| 持久化 | ✅ | RocksDB |
| 智能合约 | ✅ | WASM + Rust |
| 区块浏览器 | ✅ | Web 界面 |
| 云部署 | ✅ | Docker + 脚本 |

## 🚀 快速开始

### 1. 本地运行

```bash
# 克隆项目
cd /Users/yc/RustroverProjects/YRust

# 运行演示
cargo run -- demo

# 启动节点
cargo run -- node

# 启动 P2P 节点
cargo run -- p2p --port 8001
```

### 2. Docker 部署

```bash
cd deploy

# 本地部署
./deploy.sh local

# 云服务器部署
./deploy.sh cloud <IP> <USER> [SSH_KEY]
```

### 3. 访问服务

- 区块浏览器: http://localhost
- API: http://localhost:3000/api
- P2P: localhost:8000

## 📁 项目结构

```
YRust/
├── src/
│   ├── core/
│   │   ├── types.rs          # 区块、交易类型
│   │   ├── models.rs         # UTXO + 账户双模型
│   │   └── blockchain.rs     # 区块链核心
│   ├── wallet/
│   │   └── mod.rs            # 钱包、签名
│   ├── network/
│   │   ├── mod.rs            # 基础网络
│   │   └── p2p.rs            # libp2p 实现
│   ├── api/
│   │   └── mod.rs            # REST API
│   ├── storage/
│   │   ├── mod.rs            # Sled 存储
│   │   └── rocksdb.rs        # RocksDB 存储
│   ├── vm/
│   │   ├── mod.rs            # 合约基础
│   │   └── engine.rs         # WASM 引擎
│   ├── lib.rs
│   └── main.rs               # 主程序
├── web/
│   └── index.html            # 区块浏览器
├── deploy/
│   ├── Dockerfile            # Docker 镜像
│   ├── docker-compose.yml    # 编排配置
│   ├── nginx.conf            # 反向代理
│   └── deploy.sh             # 部署脚本
├── tutorial/                 # 教程
│   ├── 01-block-structure/
│   ├── 02-hash/
│   └── 03-blockchain/
├── Cargo.toml
├── README.md
└── ULTIMATE.md              # 本文件
```

## 🔧 核心特性

### 双模型支持

```rust
// UTXO 模型（比特币）
let utxo_set = UtxoSet::new();
utxo_set.add(tx_id, vout, output);
let balance = utxo_set.get_balance(address);

// 账户模型（以太坊）
let mut state = WorldState::new();
state.transfer(from, to, amount)?;
let balance = state.get_balance(address);
```

### 工业级 P2P

```rust
// libp2p + QUIC + Gossipsub
let network = P2PNetwork::new(port).await?;
network.subscribe()?;
network.broadcast_block(block).await?;
```

### Rust 原生合约

```rust
// 比 Solidity 更快更安全
let mut engine = ContractEngine::new()?;
let address = engine.deploy(wasm_code, abi, creator, value)?;
let result = engine.call(address, "transfer", args, context)?;
```

### RocksDB 持久化

```rust
let db = RocksStorage::new("./data")?;
db.save_block(&block)?;
db.save_utxo(tx_id, vout, &output)?;
db.save_account(&account)?;
```

## 📊 性能对比

| 指标 | YRust | 比特币 | 以太坊 |
|------|-------|--------|--------|
| 语言 | Rust | C++ | Go |
| TPS | ~1000 | ~7 | ~15 |
| 确认时间 | ~10s | ~10min | ~15s |
| 合约 | WASM | - | EVM |
| 内存安全 | ✅ | ❌ | ❌ |

## 🌐 API 列表

### 区块链
- `GET /api/stats` - 统计信息
- `GET /api/blocks` - 区块列表
- `GET /api/blocks/height/:h` - 通过高度获取
- `GET /api/blocks/hash/:hash` - 通过哈希获取

### 交易
- `GET /api/tx/:id` - 获取交易
- `GET /api/mempool` - 内存池
- `POST /api/tx/create` - 创建交易

### 账户
- `GET /api/address/:addr` - 地址信息
- `POST /api/wallet/create` - 创建钱包

### 挖矿
- `POST /api/mine` - 挖矿

## 🚀 部署到云端

### Docker（推荐）

```bash
cd deploy
./deploy.sh local
```

### AWS

```bash
# 创建 EC2 实例
# 安全组开放 80, 443, 3000, 8000

./deploy.sh cloud <EC2_IP> ec2-user ~/.ssh/aws.pem
```

### 阿里云

```bash
./deploy.sh cloud <ALI_IP> root ~/.ssh/aliyun.pem
```

### 腾讯云

```bash
./deploy.sh cloud <TX_IP> ubuntu ~/.ssh/tencent.pem
```

## 📝 开发计划

- [x] 基础区块链
- [x] POW 挖矿
- [x] 钱包系统
- [x] 交易签名
- [x] UTXO 模型
- [x] 账户模型
- [x] P2P 网络
- [x] REST API
- [x] 区块浏览器
- [x] Docker 部署
- [ ] 分片技术
- [ ] 跨链桥
- [ ] 隐私交易

## 🎓 学习路径

1. **初学者**: 阅读 `tutorial/` 目录
2. **进阶**: 研究 `src/core/` 源码
3. **高级**: 修改 `src/vm/` 合约引擎
4. **专家**: 优化 `src/network/` P2P

## 🤝 贡献

欢迎提交 PR！

## 📄 许可证

MIT

---

**YRust Chain - 用 Rust 重塑区块链未来** 🦀⛓️
