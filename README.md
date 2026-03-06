# YRust Chain - AI 驱动的智能公链

<p align="center">
  <img src="https://img.shields.io/badge/Rust-1.75+-orange.svg" alt="Rust Version">
  <img src="https://img.shields.io/badge/License-MIT-blue.svg" alt="License">
  <img src="https://img.shields.io/badge/AI-Powered-purple.svg" alt="AI Powered">
</p>

YRust Chain 是一个 **AI 驱动的智能公链**，将人工智能能力深度集成到区块链核心，提供智能挖矿、风控、查询、运维、合约生成和钱包助手六大 AI 功能。

## ✨ 核心特性

### 🔥 六大 AI 模块

| 模块 | 功能 | 状态 |
|------|------|------|
| **AI 智能挖矿调度** | 学习出块规律，预测最佳挖矿时机，动态调整难度 | ✅ 已完成 |
| **AI 风控系统** | 实时检测双花、洗钱、粉尘攻击等恶意行为 | ✅ 已完成 |
| **AI 区块浏览器** | 自然语言查询区块链数据，支持语音输入 | ✅ 已完成 |
| **AI 智能运维** | 节点健康监控、自动故障恢复、性能优化 | ✅ 已完成 |
| **AI 合约生成** | 自然语言生成智能合约，内置安全审计 | ✅ 已完成 |
| **AI 钱包助手** | 语音转账、智能提醒、交易分析 | ✅ 已完成 |

### 🛠️ 区块链特性

- **POW 共识** - 工作量证明挖矿
- **UTXO 模型** - 比特币式交易模型
- **WASM 合约** - WebAssembly 智能合约引擎
- **P2P 网络** - 去中心化节点通信
- **双模型支持** - UTXO + Account 混合模型

## 🚀 快速开始

### 环境要求

- Rust 1.75+
- 4GB+ RAM
- (可选) Node.js 18+ 用于前端开发

### 安装

```bash
# 克隆仓库
git clone https://github.com/ychech/YC-Rust.git
cd YC-Rust

# 编译
cargo build --release

# 运行测试
cargo test
```

### 启动节点

```bash
# 启动完整节点 (包含 API 服务)
cargo run -- node

# 启动矿工节点
cargo run -- miner

# 启动 P2P 节点
cargo run -- p2p --port 8000
```

### 访问 AI 区块浏览器

```bash
# 打开前端页面
open frontend/ai-explorer.html

# 或使用 Python 启动简单服务器
cd frontend && python3 -m http.server 8080
# 然后访问 http://localhost:8080/ai-explorer.html
```

## 📖 使用示例

### AI 自然语言查询

在区块浏览器中输入：

```
"查询地址 Y1abc... 的余额"
"最新区块的高度是多少"
"显示区块链统计信息"
"生成一个 ERC20 代币合约"
```

### AI 挖矿调度

```rust
use yrust_chain::ai::mining::SmartMining;

let mut mining = SmartMining::new();
mining.record_block_time(580);

let advice = mining.get_mining_advice();
println!("建议难度: {}", advice.suggested_difficulty);
println!("收益评分: {}/100", advice.profit_score);
```

### AI 风控检测

```rust
use yrust_chain::ai::risk::RiskEngine;

let mut risk = RiskEngine::new();
let result = risk.detect_transaction_risk(&transaction);

if result.score > 50 {
    println!("高风险交易! 建议操作: {:?}", result.recommended_action);
}
```

### AI 合约生成

```rust
use yrust_chain::ai::contract::{ContractGenerator, ContractGenerationRequest};

let generator = ContractGenerator::new(None);
let request = ContractGenerationRequest {
    description: "创建一个代币".to_string(),
    template: Some(ContractTemplate::ERC20),
    name: "MyToken".to_string(),
    symbol: Some("MTK".to_string()),
    ..Default::default()
};

let result = generator.generate_contract(request)?;
println!("生成的合约:\n{}", result.code);
```

## 📚 API 文档

### 基础 API

| 端点 | 方法 | 描述 |
|------|------|------|
| `/api/stats` | GET | 区块链统计 |
| `/api/blocks` | GET | 区块列表 |
| `/api/tx/:id` | GET | 查询交易 |
| `/api/mine` | POST | 挖矿 |

### AI API

| 端点 | 方法 | 描述 |
|------|------|------|
| `/api/ai/query` | POST | 自然语言查询 |
| `/api/ai/mining/advice` | GET | 挖矿建议 |
| `/api/ai/risk/check` | POST | 风险检测 |
| `/api/ai/contract/generate` | POST | 生成合约 |
| `/api/ai/wallet/voice` | POST | 语音指令 |
| `/api/ai/ops/status` | GET | 运维状态 |

## 📁 项目结构

```
YRust/
├── src/
│   ├── ai/              # AI 模块
│   │   ├── mod.rs       # AI 模块入口
│   │   ├── mining.rs    # 智能挖矿调度
│   │   ├── risk.rs      # 风控系统
│   │   ├── nlp.rs       # 自然语言查询
│   │   ├── ops.rs       # 智能运维
│   │   ├── contract.rs  # 合约生成
│   │   └── wallet.rs    # 钱包助手
│   ├── core/            # 区块链核心
│   ├── api/             # API 服务
│   ├── network/         # P2P 网络
│   ├── vm/              # WASM 虚拟机
│   ├── wallet/          # 钱包
│   ├── storage/         # 存储
│   └── main.rs          # 主程序
├── frontend/            # AI 区块浏览器前端
├── tutorials/           # 教程文档
├── Cargo.toml
└── README.md
```

## 📖 教程

- [AI 集成概述](tutorials/00_AI_Integration_Overview.md)
- [AI 智能挖矿调度](tutorials/01_AI_Mining.md)
- [AI 风控系统](tutorials/02_AI_Risk.md)
- [AI 自然语言查询](tutorials/03_AI_NLP.md) *(待完善)*
- [AI 智能运维](tutorials/04_AI_Ops.md) *(待完善)*
- [AI 合约生成](tutorials/05_AI_Contract.md) *(待完善)*
- [AI 钱包助手](tutorials/06_AI_Wallet.md) *(待完善)*

## 🤝 贡献

欢迎贡献代码和文档！请遵循以下步骤：

1. Fork 仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

## 🔗 链接

- GitHub: https://github.com/ychech/YC-Rust
- 文档: https://github.com/ychech/YC-Rust/tree/main/tutorials

## 🙏 致谢

感谢豆包 AI 的技术支持，让 YRust Chain 成为真正的 AI 驱动智能公链。

---

<p align="center">
  <strong>YRust Chain - 让区块链更智能</strong>
</p>
