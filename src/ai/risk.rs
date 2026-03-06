//! AI 风控检测模块
//!
//! 功能:
//! - 识别恶意交易模式
//! - 检测双花攻击
//! - 识别异常账户行为
//! - 实时风险评分
//! - 自动拦截可疑交易

use super::AIError;
use crate::core::{Transaction, Block, Amount};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

/// 风险等级
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// 无风险
    None,
    /// 低风险
    Low,
    /// 中风险
    Medium,
    /// 高风险
    High,
    /// 严重风险
    Critical,
}

impl RiskLevel {
    /// 获取风险等级数值
    pub fn value(&self) -> u8 {
        match self {
            RiskLevel::None => 0,
            RiskLevel::Low => 25,
            RiskLevel::Medium => 50,
            RiskLevel::High => 75,
            RiskLevel::Critical => 100,
        }
    }

    /// 从数值创建风险等级
    pub fn from_value(value: u8) -> Self {
        match value {
            0..=10 => RiskLevel::None,
            11..=35 => RiskLevel::Low,
            36..=60 => RiskLevel::Medium,
            61..=85 => RiskLevel::High,
            _ => RiskLevel::Critical,
        }
    }
}

/// 风险类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskType {
    /// 双花攻击
    DoubleSpending,
    /// 异常大额交易
    LargeTransaction,
    /// 快速连续交易
    RapidTransactions,
    /// 可疑地址
    SuspiciousAddress,
    /// 洗钱模式
    MoneyLaundering,
    /// 合约漏洞利用
    ContractExploit,
    /// 女巫攻击
    SybilAttack,
    /// 粉尘攻击
    DustAttack,
    /// 时间锁操纵
    TimelockManipulation,
    /// 其他异常
    Other,
}

/// 风险检测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskResult {
    /// 风险等级
    pub level: RiskLevel,
    /// 风险评分 (0-100)
    pub score: u8,
    /// 检测到的风险类型
    pub risk_types: Vec<RiskType>,
    /// 风险详情
    pub details: Vec<String>,
    /// 建议操作
    pub recommended_action: RiskAction,
    /// 检测时间戳
    pub timestamp: u64,
    /// 置信度 (0-100)
    pub confidence: u8,
}

/// 建议操作
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RiskAction {
    /// 允许
    Allow,
    /// 监控
    Monitor,
    /// 延迟确认
    Delay,
    /// 要求额外验证
    RequireVerification,
    /// 拒绝
    Reject,
    /// 冻结账户
    Freeze,
}

/// 账户风险画像
#[derive(Debug, Clone)]
pub struct AccountRiskProfile {
    /// 地址
    pub address: String,
    /// 交易历史
    pub transaction_history: VecDeque<TransactionRecord>,
    /// 风险评分历史
    pub risk_score_history: VecDeque<(u64, u8)>,
    /// 关联地址
    pub related_addresses: HashSet<String>,
    /// 总交易金额
    pub total_volume: Amount,
    /// 异常标记
    pub flags: HashSet<RiskFlag>,
    /// 最后更新时间
    pub last_updated: u64,
}

/// 交易记录
#[derive(Debug, Clone)]
pub struct TransactionRecord {
    pub tx_id: String,
    pub timestamp: u64,
    pub amount: Amount,
    pub is_sender: bool,
    pub counterparty: String,
}

/// 风险标记
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RiskFlag {
    HighFrequency,
    LargeTransfers,
    MixingBehavior,
    ContractInteractions,
    NewAccount,
    Blacklist,
}

/// 风控引擎
#[derive(Debug)]
pub struct RiskEngine {
    /// 账户风险画像
    account_profiles: HashMap<String, AccountRiskProfile>,
    /// 黑名单地址
    blacklist: HashSet<String>,
    /// 灰名单地址 (需要监控)
    greylist: HashSet<String>,
    /// 已知的双花交易
    known_double_spends: HashSet<String>,
    /// 检测规则配置
    rules: RiskRules,
    /// 历史风险事件
    risk_events: VecDeque<RiskEvent>,
    /// 检测阈值
    thresholds: RiskThresholds,
}

/// 风险规则配置
#[derive(Debug, Clone)]
pub struct RiskRules {
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
}

impl Default for RiskRules {
    fn default() -> Self {
        RiskRules {
            large_tx_threshold: 1_000_000_000, // 10 YRU
            rapid_tx_window: 60,               // 1 分钟
            rapid_tx_threshold: 5,
            new_account_period: 86400,         // 24 小时
            laundering_detection_depth: 3,
        }
    }
}

/// 风险阈值
#[derive(Debug, Clone)]
pub struct RiskThresholds {
    pub low: u8,
    pub medium: u8,
    pub high: u8,
    pub critical: u8,
}

impl Default for RiskThresholds {
    fn default() -> Self {
        RiskThresholds {
            low: 25,
            medium: 50,
            high: 75,
            critical: 90,
        }
    }
}

/// 风险事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskEvent {
    pub timestamp: u64,
    pub tx_id: String,
    pub risk_type: RiskType,
    pub level: RiskLevel,
    pub description: String,
}

impl RiskEngine {
    /// 创建新的风控引擎
    pub fn new() -> Self {
        RiskEngine {
            account_profiles: HashMap::new(),
            blacklist: HashSet::new(),
            greylist: HashSet::new(),
            known_double_spends: HashSet::new(),
            rules: RiskRules::default(),
            risk_events: VecDeque::with_capacity(1000),
            thresholds: RiskThresholds::default(),
        }
    }

    /// 检测交易风险
    pub fn detect_transaction_risk(&mut self, tx: &Transaction) -> RiskResult {
        let mut score = 0u8;
        let mut detected_risks = Vec::new();
        let mut details = Vec::new();
        let timestamp = current_timestamp();

        // 1. 检查双花
        if let Some(risk) = self.check_double_spending(tx) {
            score = score.saturating_add(50);
            detected_risks.push(RiskType::DoubleSpending);
            details.push(risk);
        }

        // 2. 检查黑名单
        if self.check_blacklist(tx) {
            score = 100; // 直接最高风险
            detected_risks.push(RiskType::SuspiciousAddress);
            details.push("交易涉及黑名单地址".to_string());
        }

        // 3. 检查大额交易
        let total_output: Amount = tx.outputs.iter().map(|o| o.value).sum();
        if total_output > self.rules.large_tx_threshold {
            score = score.saturating_add(20);
            detected_risks.push(RiskType::LargeTransaction);
            details.push(format!("大额交易: {} 聪", total_output));
        }

        // 4. 检查快速连续交易
        for input in &tx.inputs {
            if let Some(profile) = self.account_profiles.get(&input.tx_id) {
                if self.check_rapid_transactions(profile) {
                    score = score.saturating_add(15);
                    detected_risks.push(RiskType::RapidTransactions);
                    details.push("检测到快速连续交易模式".to_string());
                    break;
                }
            }
        }

        // 5. 检查洗钱模式
        if self.detect_money_laundering(tx) {
            score = score.saturating_add(30);
            detected_risks.push(RiskType::MoneyLaundering);
            details.push("疑似洗钱交易模式".to_string());
        }

        // 6. 检查粉尘攻击
        if self.detect_dust_attack(tx) {
            score = score.saturating_add(10);
            detected_risks.push(RiskType::DustAttack);
            details.push("检测到粉尘攻击特征".to_string());
        }

        // 确定风险等级和建议操作
        let level = RiskLevel::from_value(score);
        let action = self.determine_action(level, &detected_risks);

        // 记录风险事件
        if score > 0 {
            self.record_risk_event(tx.id.clone(), &detected_risks, level);
        }

        RiskResult {
            level,
            score: score.min(100),
            risk_types: detected_risks,
            details,
            recommended_action: action,
            timestamp,
            confidence: self.calculate_confidence(tx),
        }
    }

    /// 检查双花
    fn check_double_spending(&self, tx: &Transaction) -> Option<String> {
        // 检查输入是否已被花费
        for input in &tx.inputs {
            if self.known_double_spends.contains(&input.tx_id) {
                return Some(format!("输入 {}:{} 已被标记为双花", input.tx_id, input.vout));
            }
        }
        None
    }

    /// 检查黑名单
    fn check_blacklist(&self, tx: &Transaction) -> bool {
        // 检查输出地址
        for output in &tx.outputs {
            if self.blacklist.contains(&output.script_pubkey) {
                return true;
            }
        }
        false
    }

    /// 检查快速连续交易
    fn check_rapid_transactions(&self, profile: &AccountRiskProfile) -> bool {
        if profile.transaction_history.len() < self.rules.rapid_tx_threshold {
            return false;
        }

        let now = current_timestamp();
        let recent_count = profile
            .transaction_history
            .iter()
            .filter(|tx| now - tx.timestamp < self.rules.rapid_tx_window)
            .count();

        recent_count >= self.rules.rapid_tx_threshold
    }

    /// 检测洗钱模式
    fn detect_money_laundering(&self, tx: &Transaction) -> bool {
        // 简化检测: 检查是否有多个小额输入合并成大额输出
        if tx.inputs.len() >= 5 && tx.outputs.len() <= 2 {
            let input_count = tx.inputs.len();
            let output_count = tx.outputs.len();
            
            // 多输入少输出可能是混币或洗钱
            if input_count as f64 / output_count as f64 > 3.0 {
                return true;
            }
        }
        false
    }

    /// 检测粉尘攻击
    fn detect_dust_attack(&self, tx: &Transaction) -> bool {
        // 粉尘攻击特征: 大量极小金额输出
        let dust_threshold = 546; // 标准粉尘阈值
        let dust_outputs = tx
            .outputs
            .iter()
            .filter(|o| o.value <= dust_threshold)
            .count();

        dust_outputs >= 10 // 10个以上的粉尘输出视为攻击
    }

    /// 确定建议操作
    fn determine_action(&self, level: RiskLevel, risk_types: &[RiskType]) -> RiskAction {
        // 如果有双花，直接拒绝
        if risk_types.contains(&RiskType::DoubleSpending) {
            return RiskAction::Reject;
        }

        // 如果涉及黑名单，冻结
        if risk_types.contains(&RiskType::SuspiciousAddress) {
            return RiskAction::Freeze;
        }

        match level {
            RiskLevel::None => RiskAction::Allow,
            RiskLevel::Low => RiskAction::Monitor,
            RiskLevel::Medium => RiskAction::Delay,
            RiskLevel::High => RiskAction::RequireVerification,
            RiskLevel::Critical => RiskAction::Reject,
        }
    }

    /// 计算置信度
    fn calculate_confidence(&self, tx: &Transaction) -> u8 {
        // 基于历史数据量计算置信度
        let mut confidence = 50u8;

        for input in &tx.inputs {
            if let Some(profile) = self.account_profiles.get(&input.tx_id) {
                if profile.transaction_history.len() > 10 {
                    confidence = confidence.saturating_add(5);
                }
            }
        }

        confidence.min(100)
    }

    /// 记录风险事件
    fn record_risk_event(&mut self, tx_id: String, risk_types: &[RiskType], level: RiskLevel) {
        if self.risk_events.len() >= 1000 {
            self.risk_events.pop_front();
        }

        for risk_type in risk_types {
            self.risk_events.push_back(RiskEvent {
                timestamp: current_timestamp(),
                tx_id: tx_id.clone(),
                risk_type: *risk_type,
                level,
                description: format!("{:?} 风险检测", risk_type),
            });
        }
    }

    /// 更新账户画像
    pub fn update_account_profile(&mut self, address: &str, tx: &Transaction, is_sender: bool) {
        let profile = self.account_profiles
            .entry(address.to_string())
            .or_insert_with(|| AccountRiskProfile {
                address: address.to_string(),
                transaction_history: VecDeque::with_capacity(100),
                risk_score_history: VecDeque::with_capacity(50),
                related_addresses: HashSet::new(),
                total_volume: 0,
                flags: HashSet::new(),
                last_updated: current_timestamp(),
            });

        // 添加交易记录
        if profile.transaction_history.len() >= 100 {
            profile.transaction_history.pop_front();
        }

        let amount: Amount = if is_sender {
            tx.inputs.iter().map(|_| 0u64).sum() // 简化处理
        } else {
            tx.outputs.iter().map(|o| o.value).sum()
        };

        profile.transaction_history.push_back(TransactionRecord {
            tx_id: tx.id.clone(),
            timestamp: current_timestamp(),
            amount,
            is_sender,
            counterparty: "unknown".to_string(),
        });

        profile.total_volume += amount;
        profile.last_updated = current_timestamp();

        // 更新标记
        if profile.transaction_history.len() > 50 {
            profile.flags.insert(RiskFlag::HighFrequency);
        }
    }

    /// 添加地址到黑名单
    pub fn add_to_blacklist(&mut self, address: &str) {
        self.blacklist.insert(address.to_string());
    }

    /// 从黑名单移除
    pub fn remove_from_blacklist(&mut self, address: &str) {
        self.blacklist.remove(address);
    }

    /// 添加地址到灰名单
    pub fn add_to_greylist(&mut self, address: &str) {
        self.greylist.insert(address.to_string());
    }

    /// 获取账户风险评分
    pub fn get_account_risk_score(&self, address: &str) -> u8 {
        if let Some(profile) = self.account_profiles.get(address) {
            // 基于标记计算风险评分
            let mut score = 0u8;
            
            if profile.flags.contains(&RiskFlag::Blacklist) {
                score = 100;
            }
            if profile.flags.contains(&RiskFlag::MixingBehavior) {
                score = score.saturating_add(30);
            }
            if profile.flags.contains(&RiskFlag::HighFrequency) {
                score = score.saturating_add(15);
            }
            if profile.flags.contains(&RiskFlag::NewAccount) {
                score = score.saturating_add(10);
            }

            return score.min(100);
        }

        // 新账户默认低风险
        10
    }

    /// 获取风险统计
    pub fn get_risk_stats(&self) -> RiskStats {
        RiskStats {
            total_monitored_accounts: self.account_profiles.len(),
            blacklisted_addresses: self.blacklist.len(),
            greylisted_addresses: self.greylist.len(),
            total_risk_events: self.risk_events.len(),
            high_risk_events: self.risk_events
                .iter()
                .filter(|e| matches!(e.level, RiskLevel::High | RiskLevel::Critical))
                .count(),
        }
    }

    /// 获取最近的风险事件
    pub fn get_recent_risk_events(&self, limit: usize) -> Vec<&RiskEvent> {
        self.risk_events.iter().rev().take(limit).collect()
    }

    /// 批量检测区块风险
    pub fn analyze_block_risk(&mut self, block: &Block) -> BlockRiskAnalysis {
        let mut total_score = 0u32;
        let mut risk_txs = Vec::new();
        let mut risk_distribution = HashMap::new();

        for tx in &block.transactions {
            let result = self.detect_transaction_risk(tx);
            
            // 统计风险类型分布
            for risk_type in &result.risk_types {
                *risk_distribution.entry(*risk_type).or_insert(0) += 1;
            }
            
            if result.score > 0 {
                total_score += result.score as u32;
                risk_txs.push((tx.id.clone(), result));
            }
        }

        let avg_score = if block.transactions.is_empty() {
            0
        } else {
            (total_score / block.transactions.len() as u32) as u8
        };

        BlockRiskAnalysis {
            block_hash: block.hash.clone(),
            block_height: block.height,
            average_risk_score: avg_score,
            high_risk_transactions: risk_txs.len(),
            risk_distribution,
            flagged_transactions: risk_txs,
        }
    }
}

/// 区块风险分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockRiskAnalysis {
    pub block_hash: String,
    pub block_height: u64,
    pub average_risk_score: u8,
    pub high_risk_transactions: usize,
    pub risk_distribution: HashMap<RiskType, usize>,
    pub flagged_transactions: Vec<(String, RiskResult)>,
}

/// 风险统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskStats {
    pub total_monitored_accounts: usize,
    pub blacklisted_addresses: usize,
    pub greylisted_addresses: usize,
    pub total_risk_events: usize,
    pub high_risk_events: usize,
}

/// 获取当前时间戳
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl Default for RiskEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_risk_level() {
        assert_eq!(RiskLevel::from_value(5), RiskLevel::None);
        assert_eq!(RiskLevel::from_value(30), RiskLevel::Low);
        assert_eq!(RiskLevel::from_value(55), RiskLevel::Medium);
        assert_eq!(RiskLevel::from_value(80), RiskLevel::High);
        assert_eq!(RiskLevel::from_value(95), RiskLevel::Critical);
    }

    #[test]
    fn test_risk_engine_creation() {
        let engine = RiskEngine::new();
        assert!(engine.account_profiles.is_empty());
        assert!(engine.blacklist.is_empty());
    }

    #[test]
    fn test_blacklist() {
        let mut engine = RiskEngine::new();
        let address = "test_address_123";
        
        engine.add_to_blacklist(address);
        assert!(engine.blacklist.contains(address));
        
        engine.remove_from_blacklist(address);
        assert!(!engine.blacklist.contains(address));
    }

    #[test]
    fn test_dust_attack_detection() {
        let engine = RiskEngine::new();
        
        // 创建模拟粉尘交易
        let mut tx = Transaction {
            id: "test".to_string(),
            timestamp: 0,
            inputs: vec![],
            outputs: vec![],
            version: 1,
        };
        
        // 添加 15 个粉尘输出
        for _ in 0..15 {
            tx.outputs.push(crate::core::TxOutput {
                value: 500, // 小于粉尘阈值
                script_pubkey: "addr".to_string(),
            });
        }
        
        assert!(engine.detect_dust_attack(&tx));
    }
}
