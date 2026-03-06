# YRust Chain AI 集成指南

## 概述

YRust Chain 现已升级为 **AI 驱动的智能公链**，集成了六大 AI 核心模块：

1. **AI 智能挖矿调度** - 学习出块规律，智能调节算力
2. **AI 风控系统** - 识别恶意交易、双花、攻击
3. **AI 区块浏览器** - 自然语言查询区块链数据
4. **AI 智能运维** - 节点监控、自动故障恢复
5. **AI 合约生成** - 语音/文字生成并部署合约
6. **AI 钱包助手** - 语音转账、智能提醒

## 架构图

```
┌─────────────────────────────────────────────────────────────┐
│                     YRust Chain AI Layer                     │
├─────────────┬─────────────┬─────────────┬───────────────────┤
│  AI Mining  │  AI Risk    │   AI NLP    │    AI Ops         │
│  Scheduler  │  Detection  │   Query     │   Monitor         │
├─────────────┴─────────────┴─────────────┴───────────────────┤
│                    AI Contract Generator                     │
│                    AI Wallet Assistant                       │
├─────────────────────────────────────────────────────────────┤
│              Core Blockchain (POW + UTXO + WASM)            │
├─────────────────────────────────────────────────────────────┤
│              P2P Network + API + Storage                     │
└─────────────────────────────────────────────────────────────┘
```

## 快速开始

### 1. 启动 AI 增强节点

```bash
# 编译
cargo build --release

# 启动节点
cargo run -- node

# 启动矿工 (带 AI 调度)
cargo run -- miner --ai-enabled
```

### 2. 访问 AI 区块浏览器

打开 `frontend/ai-explorer.html` 或在浏览器中访问：

```
http://localhost:3000
```

### 3. 使用 AI 助手

在区块浏览器中，你可以用自然语言查询：

- "查询地址 Y1abc... 的余额"
- "最新区块的高度是多少"
- "显示区块链统计信息"
- "生成一个 ERC20 代币合约"

## AI 模块详解

### AI 智能挖矿调度 (`src/ai/mining.rs`)

**功能：**
- 分析历史出块时间，预测最佳挖矿时机
- 动态调整挖矿难度目标
- 智能算力分配策略
- 挖矿收益优化建议

**API：**
```rust
// 获取挖矿建议
GET /api/ai/mining/advice

// 响应示例
{
  "suggested_difficulty": 4,
  "predicted_block_time": 580,
  "profit_score": 85,
  "recommended_strategy": "Adaptive"
}
```

### AI 风控系统 (`src/ai/risk.rs`)

**功能：**
- 实时交易风险评分
- 双花攻击检测
- 异常账户行为识别
- 自动拦截可疑交易
- 黑名单/灰名单管理

**风险类型：**
- DoubleSpending - 双花攻击
- LargeTransaction - 异常大额交易
- RapidTransactions - 快速连续交易
- SuspiciousAddress - 可疑地址
- MoneyLaundering - 洗钱模式
- DustAttack - 粉尘攻击

**API：**
```rust
// 检测交易风险
POST /api/ai/risk/check
{
  "tx_id": "transaction_hash"
}
```

### AI 自然语言查询 (`src/ai/nlp.rs`)

**功能：**
- 自然语言理解区块链查询
- 智能意图识别
- 多轮对话支持
- 语音查询接口

**支持的查询意图：**
- QueryBalance - 查询余额
- QueryTransaction - 查询交易
- QueryBlock - 查询区块
- QueryStats - 查询统计
- QueryMiningInfo - 查询挖矿信息

**API：**
```rust
// 自然语言查询
POST /api/ai/query
{
  "text": "查询最新区块",
  "session_id": "optional_session_id"
}
```

### AI 智能运维 (`src/ai/ops.rs`)

**功能：**
- 节点健康监控
- 性能指标分析
- 自动故障检测
- 智能告警系统
- 资源优化建议

**监控指标：**
- CPU/Memory/Disk 使用率
- 区块同步延迟
- 对等节点数量
- 交易处理速率 (TPS)
- 数据库查询时间

**API：**
```rust
// 获取系统状态
GET /api/ai/ops/status
```

### AI 合约生成 (`src/ai/contract.rs`)

**功能：**
- 自然语言生成合约代码
- 内置合约模板库
- 自动安全审计
- 部署参数生成

**支持的模板：**
- ERC20 代币
- ERC721 NFT
- 多签钱包
- 时间锁
- 众筹合约
- 投票合约
- 质押挖矿
- 流动性池

**API：**
```rust
// 生成合约
POST /api/ai/contract/generate
{
  "description": "创建一个代币",
  "template": "ERC20",
  "name": "MyToken",
  "symbol": "MTK",
  "security_level": "Standard"
}
```

### AI 钱包助手 (`src/ai/wallet.rs`)

**功能：**
- 语音转账指令解析
- 智能交易提醒
- 地址簿管理
- 交易历史分析
- 个性化推荐

**语音指令示例：**
- "转账 10 个 YRU 给张三"
- "查询我的余额"
- "添加联系人 李四 地址 Y2def..."

**API：**
```rust
// 语音转账
POST /api/ai/wallet/voice
{
  "transcript": "转账 10 YRU 给张三",
  "from_address": "Y1sender..."
}
```

## 配置

### AI 配置 (`AIConfig`)

```rust
pub struct AIConfig {
    pub enable_smart_mining: bool,      // 启用 AI 挖矿
    pub enable_risk_detection: bool,    // 启用风控
    pub enable_nlp_query: bool,         // 启用自然语言查询
    pub enable_smart_ops: bool,         // 启用智能运维
    pub enable_ai_contract: bool,       // 启用合约生成
    pub enable_ai_wallet: bool,         // 启用钱包助手
    pub ai_api_endpoint: Option<String>, // AI API 端点
    pub ai_api_key: Option<String>,     // API 密钥
}
```

### 环境变量

```bash
# AI 功能开关
export YRUST_AI_MINING=true
export YRUST_AI_RISK=true
export YRUST_AI_NLP=true
export YRUST_AI_OPS=true
export YRUST_AI_CONTRACT=true
export YRUST_AI_WALLET=true

# 外部 AI 服务 (可选)
export YRUST_AI_ENDPOINT="https://api.doubao.com/v1"
export YRUST_AI_KEY="your_api_key"
```

## 开发指南

### 添加新的 AI 功能

1. 在 `src/ai/` 目录创建新模块
2. 实现核心逻辑
3. 在 `mod.rs` 中导出
4. 添加 API 端点
5. 更新前端界面

### 扩展示例

```rust
// src/ai/new_feature.rs
pub struct NewFeature {
    // 实现
}

impl NewFeature {
    pub fn new() -> Self { ... }
    pub fn process(&self, input: Input) -> Output { ... }
}
```

## 性能优化

### AI 模块性能考虑

1. **异步处理** - 所有 AI 功能都是非阻塞的
2. **缓存机制** - 频繁查询结果缓存
3. **批量处理** - 支持批量查询优化
4. **资源限制** - 可配置的内存和 CPU 限制

### 调优建议

```rust
// 调整历史记录大小
mining.set_history_size(200);

// 调整告警阈值
ops.set_alert_threshold(MetricType::CpuUsage, 85.0);

// 调整风控敏感度
risk.set_sensitivity(SecurityLevel::High);
```

## 安全注意事项

1. **API 密钥保护** - 不要将密钥硬编码
2. **输入验证** - 所有 AI 输入都经过验证
3. **速率限制** - API 端点有访问频率限制
4. **审计日志** - 所有 AI 决策都有日志记录

## 故障排除

### 常见问题

**Q: AI 功能不响应**
- 检查 AI 功能是否启用
- 查看日志中的错误信息
- 确认 API 端点可访问

**Q: 自然语言查询不准确**
- 使用更具体的描述
- 检查地址格式是否正确
- 查看支持的查询类型

**Q: 合约生成失败**
- 检查描述是否清晰
- 确认选择了正确的模板
- 查看安全审计报告

## 下一步

- [第1章：AI 挖矿调度详解](01_AI_Mining.md)
- [第2章：AI 风控系统详解](02_AI_Risk.md)
- [第3章：自然语言查询详解](03_AI_NLP.md)
- [第4章：智能运维详解](04_AI_Ops.md)
- [第5章：合约生成详解](05_AI_Contract.md)
- [第6章：钱包助手详解](06_AI_Wallet.md)
