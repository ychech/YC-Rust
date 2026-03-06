# 第2章：AI 风控系统

## 概述

AI 风控系统通过多维度分析交易行为，实时识别和拦截可疑交易，保护区块链网络和用户资产安全。

## 风险类型

### 1. 双花攻击 (DoubleSpending)

检测同一笔资金被多次花费的尝试。

```rust
fn check_double_spending(&self, tx: &Transaction) -> Option<String> {
    for input in &tx.inputs {
        if self.known_double_spends.contains(&input.tx_id) {
            return Some(format!(
                "输入 {}:{} 已被标记为双花", 
                input.tx_id, 
                input.vout
            ));
        }
    }
    None
}
```

### 2. 异常大额交易 (LargeTransaction)

检测超出正常范围的大额转账。

```rust
const LARGE_TX_THRESHOLD: Amount = 1_000_000_000; // 10 YRU

if total_output > self.rules.large_tx_threshold {
    score += 20;
    risks.push(RiskType::LargeTransaction);
}
```

### 3. 快速连续交易 (RapidTransactions)

检测短时间内的大量交易，可能是攻击或机器人行为。

```rust
fn check_rapid_transactions(&self, profile: &AccountRiskProfile) -> bool {
    let now = current_timestamp();
    let recent_count = profile
        .transaction_history
        .iter()
        .filter(|tx| now - tx.timestamp < 60) // 1分钟内
        .count();
    
    recent_count >= 5 // 超过5笔视为异常
}
```

### 4. 洗钱模式 (MoneyLaundering)

检测资金分散转入、集中转出的可疑模式。

```rust
fn detect_money_laundering(&self, tx: &Transaction) -> bool {
    // 多输入少输出可能是混币
    if tx.inputs.len() >= 5 && tx.outputs.len() <= 2 {
        let ratio = tx.inputs.len() as f64 / tx.outputs.len() as f64;
        ratio > 3.0
    } else {
        false
    }
}
```

### 5. 粉尘攻击 (DustAttack)

检测大量极小金额的交易输出。

```rust
fn detect_dust_attack(&self, tx: &Transaction) -> bool {
    const DUST_THRESHOLD: Amount = 546;
    
    let dust_outputs = tx
        .outputs
        .iter()
        .filter(|o| o.value <= DUST_THRESHOLD)
        .count();
    
    dust_outputs >= 10
}
```

## 使用示例

### 基础风险检测

```rust
use yrust_chain::ai::risk::{RiskEngine, RiskLevel, RiskAction};

// 创建风控引擎
let mut risk_engine = RiskEngine::new();

// 检测交易风险
let result = risk_engine.detect_transaction_risk(&transaction);

match result.level {
    RiskLevel::None => {
        println!("✅ 交易安全，可以执行");
    }
    RiskLevel::Low => {
        println!("⚠️  低风险，建议监控");
    }
    RiskLevel::Medium => {
        println!("⚠️  中风险，需要延迟确认");
    }
    RiskLevel::High | RiskLevel::Critical => {
        println!("❌ 高风险，建议操作: {:?}", result.recommended_action);
    }
}
```

### 管理黑名单

```rust
// 添加可疑地址到黑名单
risk_engine.add_to_blacklist("Y1suspicious...");

// 从黑名单移除
risk_engine.remove_from_blacklist("Y1suspicious...");

// 检查地址是否在黑名单
if risk_engine.is_blacklisted(address) {
    println!("地址在黑名单中，拒绝交易");
}
```

### 账户风险画像

```rust
// 更新账户画像
risk_engine.update_account_profile(address, &transaction, is_sender);

// 获取账户风险评分
let score = risk_engine.get_account_risk_score(address);
println!("账户风险评分: {}/100", score);
```

## API 接口

### HTTP API

```bash
# 检测交易风险
curl -X POST http://localhost:3000/api/ai/risk/check \
  -H "Content-Type: application/json" \
  -d '{"tx_id": "transaction_hash"}'

# 响应
{
  "level": "Medium",
  "score": 45,
  "risk_types": ["LargeTransaction"],
  "details": ["大额交易: 1500000000 聪"],
  "recommended_action": "Delay",
  "confidence": 85
}
```

### Rust API

```rust
use yrust_chain::ai::risk::{
    RiskEngine, RiskResult, RiskLevel, 
    RiskType, RiskAction, BlockRiskAnalysis
};

// 创建引擎
let mut engine = RiskEngine::new();

// 检测单笔交易
let result: RiskResult = engine.detect_transaction_risk(&tx);

// 分析整个区块
let analysis: BlockRiskAnalysis = engine.analyze_block_risk(&block);

// 获取风险统计
let stats = engine.get_risk_stats();
```

## 风险评分算法

### 评分规则

```rust
fn calculate_risk_score(&self, tx: &Transaction) -> u8 {
    let mut score = 0u8;
    
    // 双花: +50分 (严重)
    if self.check_double_spending(tx).is_some() {
        score += 50;
    }
    
    // 黑名单: +100分 (直接最高)
    if self.check_blacklist(tx) {
        return 100;
    }
    
    // 大额交易: +20分
    if self.is_large_transaction(tx) {
        score += 20;
    }
    
    // 快速交易: +15分
    if self.check_rapid_transactions(profile) {
        score += 15;
    }
    
    // 洗钱模式: +30分
    if self.detect_money_laundering(tx) {
        score += 30;
    }
    
    // 粉尘攻击: +10分
    if self.detect_dust_attack(tx) {
        score += 10;
    }
    
    score.min(100)
}
```

### 风险等级划分

| 分数 | 等级 | 建议操作 |
|------|------|----------|
| 0-10 | None | Allow |
| 11-35 | Low | Monitor |
| 36-60 | Medium | Delay |
| 61-85 | High | RequireVerification |
| 86-100 | Critical | Reject/Freeze |

## 配置参数

```rust
pub struct RiskConfig {
    /// 大额交易阈值
    pub large_tx_threshold: Amount,
    /// 快速交易时间窗口 (秒)
    pub rapid_tx_window: u64,
    /// 快速交易数量阈值
    pub rapid_tx_threshold: usize,
    /// 新账户观察期 (秒)
    pub new_account_period: u64,
    /// 洗钱检测深度
    pub laundering_detection_depth: usize,
    /// 粉尘阈值
    pub dust_threshold: Amount,
    /// 是否启用自动拦截
    pub auto_block: bool,
}

impl Default for RiskConfig {
    fn default() -> Self {
        RiskConfig {
            large_tx_threshold: 1_000_000_000, // 10 YRU
            rapid_tx_window: 60,
            rapid_tx_threshold: 5,
            new_account_period: 86400, // 24小时
            laundering_detection_depth: 3,
            dust_threshold: 546,
            auto_block: true,
        }
    }
}
```

## 最佳实践

### 1. 初始化配置

```rust
let mut engine = RiskEngine::new();

// 配置规则
engine.set_large_tx_threshold(5_000_000_000); // 50 YRU
engine.set_rapid_tx_threshold(3);
engine.enable_auto_block(true);
```

### 2. 交易验证流程

```rust
fn validate_transaction(&mut self, tx: &Transaction) -> Result<(), String> {
    // 1. 基础验证
    if !tx.is_valid() {
        return Err("交易格式无效".to_string());
    }
    
    // 2. AI 风险检测
    let risk = self.risk_engine.detect_transaction_risk(tx);
    
    match risk.recommended_action {
        RiskAction::Allow => {
            // 直接通过
            Ok(())
        }
        RiskAction::Monitor => {
            // 记录日志，允许通过
            self.log_suspicious(tx, &risk);
            Ok(())
        }
        RiskAction::Delay => {
            // 延迟确认
            self.delay_confirmation(tx);
            Ok(())
        }
        RiskAction::RequireVerification => {
            // 需要额外验证
            Err("需要额外验证".to_string())
        }
        RiskAction::Reject | RiskAction::Freeze => {
            // 拒绝交易
            Err(format!("交易被拒绝: {:?}", risk.risk_types))
        }
    }
}
```

### 3. 区块风险分析

```rust
// 在打包区块前进行风险分析
let analysis = risk_engine.analyze_block_risk(&block);

if analysis.average_risk_score > 50 {
    println!("警告: 区块平均风险评分较高");
    
    for (tx_id, risk) in &analysis.flagged_transactions {
        println!("  可疑交易 {}: {:?}", tx_id, risk.risk_types);
    }
}
```

### 4. 定期审计

```rust
// 获取风险统计
let stats = risk_engine.get_risk_stats();
println!("监控账户数: {}", stats.total_monitored_accounts);
println!("黑名单地址: {}", stats.blacklisted_addresses);
println!("高风险事件: {}", stats.high_risk_events);

// 查看最近的风险事件
let events = risk_engine.get_recent_risk_events(10);
for event in events {
    println!("{} - {:?}: {}", 
        event.timestamp, 
        event.risk_type, 
        event.description
    );
}
```

## 集成到区块链

```rust
impl Blockchain {
    pub fn add_transaction_with_risk_check(
        &mut self, 
        tx: Transaction,
        risk_engine: &mut RiskEngine
    ) -> Result<(), BlockchainError> {
        // AI 风险检测
        let risk = risk_engine.detect_transaction_risk(&tx);
        
        match risk.level {
            RiskLevel::Critical => {
                return Err(BlockchainError::InvalidTransaction(
                    "高风险交易被拒绝".to_string()
                ));
            }
            RiskLevel::High => {
                // 记录并延迟处理
                risk_engine.record_high_risk_tx(&tx);
            }
            _ => {}
        }
        
        // 添加到内存池
        self.add_to_mempool(tx)?;
        Ok(())
    }
}
```

## 监控面板

### 关键指标

```rust
pub struct RiskDashboard {
    /// 实时风险评分分布
    pub score_distribution: HashMap<RiskLevel, usize>,
    /// 风险类型统计
    pub risk_type_stats: HashMap<RiskType, usize>,
    /// 拦截交易数
    pub blocked_transactions: u64,
    /// 误报率
    pub false_positive_rate: f64,
    /// 平均检测时间
    pub avg_detection_time_ms: f64,
}
```

## 故障排除

### 误报过多

**原因：**
- 阈值设置过低
- 规则过于严格
- 正常用户行为被误判

**解决：**
- 调整阈值参数
- 添加白名单机制
- 优化检测算法

### 漏检攻击

**原因：**
- 新型攻击模式
- 规则覆盖不全
- 检测延迟

**解决：**
- 更新检测规则
- 增加行为分析
- 缩短检测周期

## 下一步

- [第3章：自然语言查询详解](03_AI_NLP.md)
- 返回 [AI 集成概述](00_AI_Integration_Overview.md)
