//! AI 智能挖矿调度模块
//!
//! 功能:
//! - 学习出块时间规律，预测最佳挖矿时机
//! - 动态调整挖矿难度目标
//! - 智能算力分配
//! - 挖矿收益优化

use super::AIError;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

/// 区块时间历史记录
const HISTORY_SIZE: usize = 100;

/// 智能挖矿调度器
#[derive(Debug, Clone)]
pub struct SmartMining {
    /// 出块时间历史 (秒)
    block_time_history: VecDeque<u64>,
    /// 网络算力估计 (哈希/秒)
    estimated_hashrate: f64,
    /// 当前难度
    current_difficulty: usize,
    /// 目标出块时间 (秒)
    target_block_time: u64,
    /// 挖矿策略
    strategy: MiningStrategy,
    /// 预测模型状态
    prediction_model: PredictionModel,
}

/// 挖矿策略
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MiningStrategy {
    /// 保守策略 - 稳定出块
    Conservative,
    /// 激进策略 - 最大化收益
    Aggressive,
    /// 自适应策略 - 根据网络调整
    Adaptive,
    /// 节能策略 - 最小化能耗
    EcoFriendly,
}

impl Default for MiningStrategy {
    fn default() -> Self {
        MiningStrategy::Adaptive
    }
}

/// 预测模型
#[derive(Debug, Clone)]
struct PredictionModel {
    /// 移动平均权重
    weights: Vec<f64>,
    /// 学习率
    learning_rate: f64,
}

impl PredictionModel {
    fn new() -> Self {
        // 初始化指数衰减权重
        let mut weights = Vec::new();
        for i in 0..10 {
            weights.push(0.9_f64.powi(i as i32));
        }
        let sum: f64 = weights.iter().sum();
        weights.iter_mut().for_each(|w| *w /= sum);
        
        PredictionModel {
            weights,
            learning_rate: 0.01,
        }
    }

    /// 预测下一个出块时间
    fn predict(&self, history: &VecDeque<u64>) -> u64 {
        if history.is_empty() {
            return 600; // 默认 10 分钟
        }
        
        let mut prediction = 0.0;
        let recent: Vec<_> = history.iter().rev().take(self.weights.len()).collect();
        
        for (i, &value) in recent.iter().enumerate() {
            if i < self.weights.len() {
                prediction += *value as f64 * self.weights[i];
            }
        }
        
        prediction as u64
    }

    /// 更新模型 (简单的在线学习)
    fn update(&mut self, predicted: u64, actual: u64) {
        let error = (actual as f64 - predicted as f64) / predicted as f64;
        // 简单调整学习率
        self.learning_rate = (self.learning_rate + error.abs() * 0.001).min(0.1);
    }
}

/// 挖矿建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningAdvice {
    /// 建议的难度调整
    pub suggested_difficulty: usize,
    /// 预测的出块时间
    pub predicted_block_time: u64,
    /// 建议的挖矿时机 (Unix 时间戳)
    pub optimal_mining_time: u64,
    /// 预期收益评分 (0-100)
    pub profit_score: u8,
    /// 网络拥堵程度 (0-100)
    pub network_congestion: u8,
    /// 建议的策略
    pub recommended_strategy: MiningStrategy,
    /// 置信度 (0-100)
    pub confidence: u8,
}

/// 挖矿统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningStats {
    /// 平均出块时间
    pub avg_block_time: f64,
    /// 出块时间标准差
    pub block_time_std: f64,
    /// 网络算力估计
    pub estimated_network_hashrate: f64,
    /// 历史记录数量
    pub history_count: usize,
    /// 当前策略
    pub current_strategy: MiningStrategy,
}

impl SmartMining {
    /// 创建新的智能挖矿调度器
    pub fn new() -> Self {
        SmartMining {
            block_time_history: VecDeque::with_capacity(HISTORY_SIZE),
            estimated_hashrate: 0.0,
            current_difficulty: 4,
            target_block_time: 600, // 10 分钟目标
            strategy: MiningStrategy::default(),
            prediction_model: PredictionModel::new(),
        }
    }

    /// 记录新区块时间
    pub fn record_block_time(&mut self, block_time_seconds: u64) {
        if self.block_time_history.len() >= HISTORY_SIZE {
            self.block_time_history.pop_front();
        }
        self.block_time_history.push_back(block_time_seconds);
        
        // 更新预测模型
        if self.block_time_history.len() >= 2 {
            let predicted = self.prediction_model.predict(&self.block_time_history);
            self.prediction_model.update(predicted, block_time_seconds);
        }
        
        // 更新网络算力估计
        self.update_hashrate_estimate();
    }

    /// 更新网络算力估计
    fn update_hashrate_estimate(&mut self) {
        if self.block_time_history.len() < 10 {
            return;
        }
        
        let avg_time = self.calculate_average_block_time();
        if avg_time > 0.0 {
            // 简化计算: 算力 ∝ 难度 / 出块时间
            self.estimated_hashrate = (self.current_difficulty as f64 * 1_000_000.0) / avg_time;
        }
    }

    /// 计算平均出块时间
    fn calculate_average_block_time(&self) -> f64 {
        if self.block_time_history.is_empty() {
            return self.target_block_time as f64;
        }
        
        let sum: u64 = self.block_time_history.iter().sum();
        sum as f64 / self.block_time_history.len() as f64
    }

    /// 计算标准差
    fn calculate_std_deviation(&self) -> f64 {
        if self.block_time_history.len() < 2 {
            return 0.0;
        }
        
        let avg = self.calculate_average_block_time();
        let variance: f64 = self.block_time_history
            .iter()
            .map(|&x| (x as f64 - avg).powi(2))
            .sum::<f64>() / self.block_time_history.len() as f64;
        
        variance.sqrt()
    }

    /// 获取挖矿建议
    pub fn get_mining_advice(&self) -> MiningAdvice {
        let avg_time = self.calculate_average_block_time();
        let predicted_time = self.prediction_model.predict(&self.block_time_history);
        
        // 计算网络拥堵程度
        let congestion = if avg_time > 0.0 {
            ((avg_time / self.target_block_time as f64 - 1.0) * 50.0 + 50.0).clamp(0.0, 100.0) as u8
        } else {
            50
        };
        
        // 计算收益评分
        let profit_score = self.calculate_profit_score(avg_time, congestion);
        
        // 推荐策略
        let recommended_strategy = self.recommend_strategy(congestion, profit_score);
        
        // 建议的难度调整
        let suggested_difficulty = self.suggest_difficulty_adjustment(avg_time);
        
        // 计算置信度
        let confidence = (self.block_time_history.len() as f64 / HISTORY_SIZE as f64 * 100.0) as u8;
        
        MiningAdvice {
            suggested_difficulty,
            predicted_block_time: predicted_time,
            optimal_mining_time: self.calculate_optimal_mining_time(predicted_time),
            profit_score,
            network_congestion: congestion,
            recommended_strategy,
            confidence: confidence.min(100),
        }
    }

    /// 计算收益评分
    fn calculate_profit_score(&self, avg_time: f64, congestion: u8) -> u8 {
        // 收益评分基于:
        // 1. 出块时间稳定性 (越低越好)
        // 2. 网络拥堵程度 (越低越好)
        // 3. 当前难度 (越低越好)
        
        let time_score = if avg_time > 0.0 {
            (self.target_block_time as f64 / avg_time * 30.0).min(30.0)
        } else {
            15.0
        };
        
        let congestion_score = (100 - congestion) as f64 * 0.4;
        let difficulty_score = (10 - self.current_difficulty.min(10)) as f64 * 3.0;
        
        (time_score + congestion_score + difficulty_score).clamp(0.0, 100.0) as u8
    }

    /// 推荐挖矿策略
    fn recommend_strategy(&self, congestion: u8, profit_score: u8) -> MiningStrategy {
        match (congestion, profit_score) {
            (c, p) if c > 80 && p < 30 => MiningStrategy::Conservative,
            (c, p) if c < 30 && p > 70 => MiningStrategy::Aggressive,
            (c, _) if c > 60 => MiningStrategy::EcoFriendly,
            _ => MiningStrategy::Adaptive,
        }
    }

    /// 建议难度调整
    fn suggest_difficulty_adjustment(&self, avg_time: f64) -> usize {
        if avg_time == 0.0 {
            return self.current_difficulty;
        }
        
        let ratio = avg_time / self.target_block_time as f64;
        let adjustment = if ratio > 1.2 {
            // 出块太慢，降低难度
            -1
        } else if ratio < 0.8 {
            // 出块太快，增加难度
            1
        } else {
            0
        };
        
        ((self.current_difficulty as i32 + adjustment).max(1) as usize)
            .min(10) // 最大难度 10
    }

    /// 计算最佳挖矿时机
    fn calculate_optimal_mining_time(&self, predicted_time: u64) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        // 简单策略: 如果预测时间小于目标时间，立即开始
        // 否则等待一段时间
        if predicted_time <= self.target_block_time {
            now
        } else {
            now + (predicted_time - self.target_block_time) / 2
        }
    }

    /// 设置挖矿策略
    pub fn set_strategy(&mut self, strategy: MiningStrategy) {
        self.strategy = strategy;
    }

    /// 更新当前难度
    pub fn update_difficulty(&mut self, difficulty: usize) {
        self.current_difficulty = difficulty;
    }

    /// 获取挖矿统计
    pub fn get_stats(&self) -> MiningStats {
        MiningStats {
            avg_block_time: self.calculate_average_block_time(),
            block_time_std: self.calculate_std_deviation(),
            estimated_network_hashrate: self.estimated_hashrate,
            history_count: self.block_time_history.len(),
            current_strategy: self.strategy,
        }
    }

    /// 分析挖矿效率
    pub fn analyze_efficiency(&self) -> EfficiencyReport {
        let stats = self.get_stats();
        let target = self.target_block_time as f64;
        let actual = stats.avg_block_time;
        
        let efficiency = if actual > 0.0 {
            (target / actual * 100.0).min(150.0) // 最高 150%
        } else {
            0.0
        };
        
        let stability = if stats.block_time_std > 0.0 {
            (100.0 - (stats.block_time_std / actual * 100.0)).max(0.0)
        } else {
            100.0
        };
        
        EfficiencyReport {
            efficiency_score: efficiency as u8,
            stability_score: stability as u8,
            recommendation: self.generate_recommendation(efficiency, stability),
        }
    }

    /// 生成优化建议
    fn generate_recommendation(&self, efficiency: f64, stability: f64) -> String {
        match (efficiency, stability) {
            (e, s) if e > 90.0 && s > 80.0 => "挖矿效率优秀，保持当前策略".to_string(),
            (e, s) if e < 70.0 && s > 80.0 => "建议增加算力投入以提高效率".to_string(),
            (e, s) if e > 90.0 && s < 60.0 => "出块时间波动较大，建议切换到保守策略".to_string(),
            _ => "建议启用自适应策略以优化整体表现".to_string(),
        }
    }
}

/// 效率报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EfficiencyReport {
    /// 效率评分 (0-150)
    pub efficiency_score: u8,
    /// 稳定性评分 (0-100)
    pub stability_score: u8,
    /// 优化建议
    pub recommendation: String,
}

impl Default for SmartMining {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smart_mining_creation() {
        let mining = SmartMining::new();
        assert_eq!(mining.current_difficulty, 4);
        assert_eq!(mining.target_block_time, 600);
    }

    #[test]
    fn test_record_block_time() {
        let mut mining = SmartMining::new();
        mining.record_block_time(600);
        mining.record_block_time(580);
        mining.record_block_time(620);
        
        assert_eq!(mining.block_time_history.len(), 3);
    }

    #[test]
    fn test_mining_advice() {
        let mut mining = SmartMining::new();
        
        // 添加一些历史数据
        for _ in 0..20 {
            mining.record_block_time(600);
        }
        
        let advice = mining.get_mining_advice();
        assert!(advice.confidence > 0);
        assert!(advice.profit_score <= 100);
    }

    #[test]
    fn test_prediction_model() {
        let mut model = PredictionModel::new();
        let mut history = VecDeque::new();
        
        for i in 0..15 {
            history.push_back(500 + i * 10);
        }
        
        let prediction = model.predict(&history);
        assert!(prediction > 0);
    }
}
