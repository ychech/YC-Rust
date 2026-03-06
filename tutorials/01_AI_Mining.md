# 第1章：AI 智能挖矿调度

## 概述

AI 智能挖矿调度模块通过机器学习算法分析历史出块数据，预测最佳挖矿时机，动态调整挖矿策略，最大化矿工收益。

## 核心算法

### 1. 出块时间预测

使用指数加权移动平均 (EWMA) 预测下一个出块时间：

```rust
fn predict_block_time(&self, history: &VecDeque<u64>) -> u64 {
    let mut prediction = 0.0;
    let weights = [0.5, 0.25, 0.125, 0.0625, 0.03125]; // 指数衰减权重
    
    for (i, &value) in history.iter().rev().take(5).enumerate() {
        prediction += value as f64 * weights[i];
    }
    
    prediction as u64
}
```

### 2. 网络算力估计

```rust
fn estimate_hashrate(&mut self) {
    let avg_time = self.calculate_average_block_time();
    if avg_time > 0.0 {
        // 算力 ∝ 难度 / 出块时间
        self.estimated_hashrate = 
            (self.current_difficulty as f64 * 1_000_000.0) / avg_time;
    }
}
```

### 3. 挖矿策略选择

```rust
pub enum MiningStrategy {
    Conservative,  // 保守策略 - 稳定出块
    Aggressive,    // 激进策略 - 最大化收益
    Adaptive,      // 自适应策略 - 根据网络调整
    EcoFriendly,   // 节能策略 - 最小化能耗
}
```

## 使用示例

### 基础使用

```rust
use yrust_chain::ai::mining::SmartMining;

// 创建挖矿调度器
let mut mining = SmartMining::new();

// 记录出块时间
mining.record_block_time(580); // 580秒
mining.record_block_time(620);
mining.record_block_time(595);

// 获取挖矿建议
let advice = mining.get_mining_advice();
println!("建议难度: {}", advice.suggested_difficulty);
println!("预测出块时间: {}秒", advice.predicted_block_time);
println!("收益评分: {}/100", advice.profit_score);
```

### 设置挖矿策略

```rust
use yrust_chain::ai::mining::MiningStrategy;

// 根据网络状况选择策略
mining.set_strategy(MiningStrategy::Adaptive);

// 或手动指定
mining.set_strategy(MiningStrategy::Conservative); // 网络拥堵时
mining.set_strategy(MiningStrategy::Aggressive);   // 网络空闲时
```

### 获取挖矿统计

```rust
let stats = mining.get_stats();
println!("平均出块时间: {:.2}秒", stats.avg_block_time);
println!("网络算力估计: {:.2} H/s", stats.estimated_network_hashrate);
println!("历史记录数: {}", stats.history_count);
```

## API 接口

### HTTP API

```bash
# 获取挖矿建议
curl http://localhost:3000/api/ai/mining/advice

# 响应
{
  "suggested_difficulty": 4,
  "predicted_block_time": 590,
  "optimal_mining_time": 1700000000,
  "profit_score": 85,
  "network_congestion": 30,
  "recommended_strategy": "Adaptive",
  "confidence": 75
}
```

### Rust API

```rust
use yrust_chain::ai::mining::{SmartMining, MiningAdvice, MiningStats};

// 创建实例
let mining = SmartMining::new();

// 获取建议
let advice: MiningAdvice = mining.get_mining_advice();

// 获取统计
let stats: MiningStats = mining.get_stats();

// 分析效率
let report = mining.analyze_efficiency();
```

## 配置参数

```rust
pub struct MiningConfig {
    /// 目标出块时间 (秒)
    pub target_block_time: u64,
    /// 历史记录大小
    pub history_size: usize,
    /// 最小难度
    pub min_difficulty: usize,
    /// 最大难度
    pub max_difficulty: usize,
    /// 学习率
    pub learning_rate: f64,
}

impl Default for MiningConfig {
    fn default() -> Self {
        MiningConfig {
            target_block_time: 600,  // 10分钟
            history_size: 100,
            min_difficulty: 1,
            max_difficulty: 10,
            learning_rate: 0.01,
        }
    }
}
```

## 策略详解

### Conservative (保守策略)

适用场景：
- 网络拥堵
- 算力不稳定
- 电力成本高

特点：
- 优先保证出块稳定性
- 降低难度目标
- 减少无效计算

### Aggressive (激进策略)

适用场景：
- 网络空闲
- 算力充足
- 追求最大收益

特点：
- 提高难度目标
- 增加计算投入
- 最大化出块奖励

### Adaptive (自适应策略)

适用场景：
- 网络状况变化
- 长期运行
- 自动优化

特点：
- 实时分析网络状态
- 动态调整参数
- 平衡收益与稳定性

### EcoFriendly (节能策略)

适用场景：
- 电力成本高
- 环保要求
- 低优先级挖矿

特点：
- 最小化能耗
- 利用低谷电力
- 智能休眠机制

## 最佳实践

### 1. 初始化配置

```rust
let mut mining = SmartMining::new();
mining.update_difficulty(4);
mining.set_strategy(MiningStrategy::Adaptive);
```

### 2. 定期记录数据

```rust
// 每挖出一个区块
let block_time = measure_block_time();
mining.record_block_time(block_time);
```

### 3. 根据建议调整

```rust
let advice = mining.get_mining_advice();

if advice.confidence > 70 {
    mining.update_difficulty(advice.suggested_difficulty);
    mining.set_strategy(advice.recommended_strategy);
}
```

### 4. 监控效率

```rust
let report = mining.analyze_efficiency();

if report.efficiency_score < 60 {
    println!("警告: 挖矿效率较低");
    println!("建议: {}", report.recommendation);
}
```

## 故障排除

### 预测不准确

**原因：**
- 历史数据不足
- 网络状况突变
- 算力波动大

**解决：**
- 积累更多历史数据
- 降低学习率
- 切换到保守策略

### 收益评分低

**原因：**
- 网络竞争激烈
- 难度设置不当
- 算力不足

**解决：**
- 检查网络算力
- 调整难度目标
- 考虑加入矿池

## 高级用法

### 自定义预测模型

```rust
impl SmartMining {
    pub fn set_prediction_weights(&mut self, weights: Vec<f64>) {
        self.prediction_model.weights = weights;
    }
    
    pub fn set_learning_rate(&mut self, rate: f64) {
        self.prediction_model.learning_rate = rate;
    }
}
```

### 批量分析

```rust
// 分析多个时间段
let block_times = vec![580, 620, 595, 610, 585];
for time in block_times {
    mining.record_block_time(time);
}

let advice = mining.get_mining_advice();
```

### 集成到挖矿程序

```rust
use yrust_chain::ai::mining::SmartMining;
use yrust_chain::core::Blockchain;

async fn ai_miner(mut blockchain: Blockchain) {
    let mut mining = SmartMining::new();
    
    loop {
        // 获取挖矿建议
        let advice = mining.get_mining_advice();
        
        // 调整难度
        blockchain.set_difficulty(advice.suggested_difficulty);
        
        // 等待最佳时机
        if should_mine_now(&advice) {
            match blockchain.mine_block(miner_address.clone()) {
                Ok(block) => {
                    println!("挖出区块 #{}!", block.height);
                    
                    // 记录出块时间
                    let block_time = calculate_block_time(&block);
                    mining.record_block_time(block_time);
                }
                Err(e) => eprintln!("挖矿失败: {}", e),
            }
        }
        
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
```

## 性能指标

| 指标 | 说明 | 目标值 |
|------|------|--------|
| 预测准确率 | 预测出块时间与实际偏差 | < 10% |
| 收益提升 | 相比固定策略的收益增长 | +15-30% |
| 响应时间 | 获取建议的延迟 | < 10ms |
| 内存占用 | 历史数据内存使用 | < 10MB |

## 下一步

- [第2章：AI 风控系统详解](02_AI_Risk.md)
- 返回 [AI 集成概述](00_AI_Integration_Overview.md)
