# 🔗 YRUST Chain - 下一代高性能区块链

<p align="center">
  <img src="https://img.shields.io/badge/Rust-2021-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License">
  <img src="https://img.shields.io/badge/TPS-1000+-green" alt="TPS">
  <img src="https://img.shields.io/badge/Consensus-POW-yellow" alt="Consensus">
</p>

**YRUST Chain** 是一个基于 Rust 构建的高性能区块链，支持智能合约、DeFi、NFT 等去中心化应用。

## ✨ 特性

- ⚡ **极速交易** - 1000+ TPS，3 秒确认
- 🔒 **安全可靠** - Rust 内存安全，形式化验证合约
- 💰 **极低费用** - 平均手续费低于 $0.001
- 🌐 **真正去中心化** - libp2p P2P 网络，任何人可运行节点
- 🛠️ **开发者友好** - Rust 智能合约，WASM 执行环境
- 🔄 **跨链互操作** - 支持与其他区块链桥接

## 🚀 快速开始

### 一键部署

```bash
# 克隆仓库
git clone https://github.com/yourusername/y-rust.git
cd y-rust

# 运行部署脚本
sudo bash scripts/deploy.sh
```

### Docker 部署

```bash
# 构建并启动
docker-compose up -d

# 查看日志
docker-compose logs -f
```

### 手动构建

```bash
# 安装依赖
sudo apt-get install -y build-essential pkg-config libssl-dev clang

# 构建
cargo build --release

# 运行节点
./target/release/y-rust
```

## 📁 项目结构

```
y-rust/
├── src/                    # 核心源代码
│   ├── core/              # 区块链核心
│   │   ├── models.rs      # 区块、交易模型
│   │   ├── blockchain.rs  # 区块链逻辑
│   │   └── types.rs       # 类型定义
│   ├── network/           # 网络模块
│   │   ├── p2p.rs         # P2P 网络
│   │   └── consensus.rs   # 共识算法
│   ├── storage/           # 存储模块
│   │   └── rocksdb.rs     # RocksDB 实现
│   ├── vm/                # 虚拟机
│   │   └── engine.rs      # WASM 执行引擎
│   ├── security/          # 安全模块
│   │   └── mod.rs         # 重放保护、权限控制
│   ├── monitoring/        # 监控模块
│   │   └── mod.rs         # 指标、告警
│   ├── api/               # API 接口
│   ├── wallet/            # 钱包模块
│   └── main.rs            # 入口
├── contracts/             # 智能合约
│   ├── token.rs           # 代币合约
│   ├── nft.rs             # NFT 合约
│   └── dex.rs             # DEX 合约
├── website/               # 官网
├── admin/                 # 管理后台
├── web/                   # 区块浏览器
├── wallet/frontend/       # 网页钱包
├── dex/frontend/          # DEX 前端
├── scripts/               # 部署脚本
├── docs/                  # 文档
└── tutorial/              # 教程
```

## 🌐 生态系统

| 组件 | 描述 | 访问地址 |
|------|------|----------|
| 🌐 官网 | 项目介绍 | http://localhost:80 |
| 🔍 区块浏览器 | 查看区块、交易 | http://localhost:3000 |
| 👛 网页钱包 | 管理资产 | http://localhost:3001 |
| 💱 DEX | 去中心化交易 | http://localhost:3002 |
| 📊 管理后台 | 节点管理 | http://localhost:3003 |
| 📈 监控面板 | Grafana 监控 | http://localhost:3004 |

## 🔧 API 接口

### REST API

```bash
# 获取最新区块
curl http://localhost:8080/api/v1/blocks/latest

# 获取账户余额
curl http://localhost:8080/api/v1/accounts/YR111.../balance

# 发送交易
curl -X POST http://localhost:8080/api/v1/transactions \
  -H "Content-Type: application/json" \
  -d '{"from":"YR111...","to":"YR222...","amount":100}'
```

完整 API 文档: [docs/API.md](docs/API.md)

### WebSocket

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
  ws.send(JSON.stringify({
    action: 'subscribe',
    channel: 'blocks'
  }));
};

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('新区块:', data);
};
```

## 📝 智能合约

### 代币合约示例

```rust
#[contract]
mod token {
    use yrust::prelude::*;

    #[storage]
    struct Token {
        name: String,
        symbol: String,
        total_supply: u64,
        balances: Map<Address, u64>,
    }

    #[init]
    fn init(name: String, symbol: String, total_supply: u64) {
        let caller = msg::caller();
        storage.balances.insert(caller, total_supply);
    }

    #[call]
    fn transfer(to: Address, amount: u64) -> bool {
        let from = msg::caller();
        let from_balance = storage.balances.get(&from).unwrap_or(0);
        
        require(from_balance >= amount, "余额不足");
        
        storage.balances.insert(from, from_balance - amount);
        let to_balance = storage.balances.get(&to).unwrap_or(0);
        storage.balances.insert(to, to_balance + amount);
        
        true
    }
}
```

## 🔒 安全特性

- ✅ **重放攻击防护** - 时间戳 + Nonce 机制
- ✅ **签名防篡改** - S 值规范化
- ✅ **速率限制** - 防止 DDoS 攻击
- ✅ **权限控制** - 管理员、部署者白名单
- ✅ **输入验证** - 严格的参数校验

## 📊 监控告警

### 指标收集

- 区块高度、交易数量
- 活跃节点数、内存使用
- API 请求延迟、错误率

### 告警规则

```yaml
rules:
  - name: 节点离线
    condition: peer_count < 3
    level: critical
  
  - name: 区块同步延迟
    condition: sync_delay > 60
    level: warning
```

## 🧪 测试

```bash
# 运行单元测试
cargo test

# 运行集成测试
cargo test --test integration

# 代码覆盖率
cargo tarpaulin --out Html
```

## 📚 教程

- [01. 区块结构](tutorial/01-block-structure/)
- [02. 哈希计算](tutorial/02-hash/)
- [03. 区块链](tutorial/03-blockchain/)

## 🤝 贡献

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送分支 (`git push origin feature/amazing`)
5. 创建 Pull Request

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE)

## 🙏 致谢

- [Rust](https://www.rust-lang.org/) - 系统编程语言
- [libp2p](https://libp2p.io/) - P2P 网络协议
- [RocksDB](https://rocksdb.org/) - 高性能存储
- [WASM](https://webassembly.org/) - 智能合约执行环境

---

<p align="center">
  Made with ❤️ by Rust enthusiasts
</p>
