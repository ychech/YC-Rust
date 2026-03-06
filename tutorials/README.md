# YRust Chain 教程文档

## AI 集成教程系列

本教程系列介绍如何将 AI 能力集成到 YRust Chain 中，打造 AI 驱动的智能公链。

### 教程目录

1. **[AI 集成概述](00_AI_Integration_Overview.md)**
   - 六大 AI 模块介绍
   - 架构设计
   - 快速开始指南

2. **[AI 智能挖矿调度](01_AI_Mining.md)**
   - 出块时间预测算法
   - 挖矿策略选择
   - 收益优化

3. **[AI 风控系统](02_AI_Risk.md)**
   - 风险类型识别
   - 双花检测
   - 异常交易分析

4. **[AI 自然语言查询](03_AI_NLP.md)** *(待完善)*
   - 意图识别
   - 语音查询
   - 智能回答生成

5. **[AI 智能运维](04_AI_Ops.md)** *(待完善)*
   - 节点监控
   - 自动告警
   - 性能优化

6. **[AI 合约生成](05_AI_Contract.md)** *(待完善)*
   - 模板库
   - 安全审计
   - 代码生成

7. **[AI 钱包助手](06_AI_Wallet.md)** *(待完善)*
   - 语音转账
   - 智能提醒
   - 交易分析

## 快速开始

### 环境要求

- Rust 1.75+
- Node.js 18+ (前端开发)
- 4GB+ RAM

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
# 启动完整节点
cargo run -- node

# 启动矿工
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
```

## API 文档

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

## 示例代码

### 使用 AI 查询

```rust
use yrust_chain::ai::nlp::NLPQueryEngine;

let mut nlp = NLPQueryEngine::new();
let query = NLPQuery {
    text: "查询最新区块".to_string(),
    session_id: None,
    user_address: None,
};

let response = nlp.process_query(query);
println!("{}", response.natural_answer);
```

### 检测交易风险

```rust
use yrust_chain::ai::risk::RiskEngine;

let mut risk = RiskEngine::new();
let result = risk.detect_transaction_risk(&transaction);

if result.score > 50 {
    println!("高风险交易!");
}
```

### 生成智能合约

```rust
use yrust_chain::ai::contract::{ContractGenerator, ContractGenerationRequest};

let generator = ContractGenerator::new(None);
let request = ContractGenerationRequest {
    description: "创建一个代币".to_string(),
    template: Some(ContractTemplate::ERC20),
    name: "MyToken".to_string(),
    symbol: Some("MTK".to_string()),
    parameters: HashMap::new(),
    language: ContractLanguage::Rust,
    security_level: SecurityLevel::Standard,
};

let result = generator.generate_contract(request)?;
println!("{}", result.code);
```

## 贡献指南

欢迎贡献代码和文档！请遵循以下步骤：

1. Fork 仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 创建 Pull Request

## 许可证

MIT License - 详见 [LICENSE](../LICENSE) 文件

## 联系方式

- GitHub: https://github.com/ychech/YC-Rust
- Email: your-email@example.com

---

**YRust Chain - AI 驱动的智能公链** 🚀
