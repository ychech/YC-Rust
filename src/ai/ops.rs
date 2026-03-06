//! AI 智能运维监控模块
//!
//! 功能:
//! - 节点健康监控
//! - 自动故障检测与恢复
//! - 性能指标分析
//! - 智能告警
//! - 资源使用优化建议

use super::AIError;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// 节点健康状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// 健康
    Healthy,
    /// 警告
    Warning,
    /// 严重
    Critical,
    /// 离线
    Offline,
    /// 未知
    Unknown,
}

impl HealthStatus {
    /// 获取状态优先级 (越高越严重)
    pub fn priority(&self) -> u8 {
        match self {
            HealthStatus::Healthy => 0,
            HealthStatus::Unknown => 1,
            HealthStatus::Warning => 2,
            HealthStatus::Critical => 3,
            HealthStatus::Offline => 4,
        }
    }

    /// 从字符串解析
    pub fn from_str(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "healthy" | "健康" => HealthStatus::Healthy,
            "warning" | "警告" => HealthStatus::Warning,
            "critical" | "严重" => HealthStatus::Critical,
            "offline" | "离线" => HealthStatus::Offline,
            _ => HealthStatus::Unknown,
        }
    }
}

/// 监控指标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricType {
    /// CPU 使用率
    CpuUsage,
    /// 内存使用率
    MemoryUsage,
    /// 磁盘使用率
    DiskUsage,
    /// 网络带宽
    NetworkBandwidth,
    /// 区块同步延迟
    BlockSyncDelay,
    /// 对等节点数量
    PeerCount,
    /// 内存池大小
    MempoolSize,
    /// 交易处理速率
    Tps,
    /// 区块处理时间
    BlockProcessTime,
    /// 数据库查询时间
    DbQueryTime,
}

/// 监控指标
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    /// 指标类型
    pub metric_type: MetricType,
    /// 数值
    pub value: f64,
    /// 单位
    pub unit: String,
    /// 时间戳
    pub timestamp: u64,
    /// 标签
    pub labels: HashMap<String, String>,
}

/// 告警规则
#[derive(Debug, Clone)]
pub struct AlertRule {
    /// 规则名称
    pub name: String,
    /// 监控指标
    pub metric_type: MetricType,
    /// 阈值
    pub threshold: f64,
    /// 比较操作 (>, <, =)
    pub operator: String,
    /// 持续时间 (秒)
    pub duration: u64,
    /// 严重级别
    pub severity: AlertSeverity,
    /// 是否启用
    pub enabled: bool,
}

/// 告警严重级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertSeverity {
    /// 信息
    Info,
    /// 警告
    Warning,
    /// 严重
    Critical,
    /// 紧急
    Emergency,
}

/// 告警事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertEvent {
    /// 告警ID
    pub id: String,
    /// 规则名称
    pub rule_name: String,
    /// 严重级别
    pub severity: AlertSeverity,
    /// 告警消息
    pub message: String,
    /// 触发时间
    pub triggered_at: u64,
    /// 恢复时间 (None 表示未恢复)
    pub resolved_at: Option<u64>,
    /// 指标值
    pub metric_value: f64,
    /// 建议操作
    pub suggested_action: String,
}

/// 节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// 节点ID
    pub node_id: String,
    /// 节点地址
    pub address: String,
    /// 节点版本
    pub version: String,
    /// 健康状态
    pub health: HealthStatus,
    /// 最后心跳时间
    pub last_heartbeat: u64,
    /// 区块高度
    pub block_height: u64,
    /// 对等节点数
    pub peer_count: usize,
    /// 运行时间 (秒)
    pub uptime: u64,
    /// 地理位置
    pub location: Option<String>,
}

/// 性能报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceReport {
    /// 报告时间
    pub timestamp: u64,
    /// 整体健康评分 (0-100)
    pub health_score: u8,
    /// 各项指标评分
    pub metric_scores: HashMap<MetricType, u8>,
    /// 瓶颈分析
    pub bottlenecks: Vec<String>,
    /// 优化建议
    pub recommendations: Vec<OptimizationSuggestion>,
    /// 趋势分析
    pub trends: Vec<TrendAnalysis>,
}

/// 优化建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSuggestion {
    /// 建议类别
    pub category: String,
    /// 建议描述
    pub description: String,
    /// 预期效果
    pub expected_improvement: String,
    /// 实施难度 (1-5)
    pub difficulty: u8,
    /// 优先级
    pub priority: u8,
}

/// 趋势分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// 指标类型
    pub metric_type: MetricType,
    /// 趋势方向
    pub direction: TrendDirection,
    /// 变化率 (%)
    pub change_rate: f64,
    /// 预测值
    pub forecast: f64,
}

/// 趋势方向
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TrendDirection {
    /// 上升
    Increasing,
    /// 下降
    Decreasing,
    /// 稳定
    Stable,
    /// 波动
    Fluctuating,
}

/// 运维监控器
pub struct OpsMonitor {
    /// 历史指标数据
    metrics_history: HashMap<MetricType, VecDeque<Metric>>,
    /// 告警规则
    alert_rules: Vec<AlertRule>,
    /// 活跃告警
    active_alerts: HashMap<String, AlertEvent>,
    /// 告警历史
    alert_history: VecDeque<AlertEvent>,
    /// 节点信息
    nodes: HashMap<String, NodeInfo>,
    /// 配置
    config: MonitorConfig,
    /// 自动恢复处理器
    #[allow(dead_code)]
    recovery_handlers: HashMap<String, Box<dyn Fn() -> Result<(), AIError> + Send + Sync>>,
}

/// 监控配置
#[derive(Debug, Clone)]
pub struct MonitorConfig {
    /// 历史数据保留数量
    pub history_size: usize,
    /// 告警冷却时间 (秒)
    pub alert_cooldown: u64,
    /// 是否启用自动恢复
    pub auto_recovery: bool,
    /// 健康检查间隔 (秒)
    pub health_check_interval: u64,
    /// 性能报告间隔 (秒)
    pub report_interval: u64,
}

impl Default for MonitorConfig {
    fn default() -> Self {
        MonitorConfig {
            history_size: 1000,
            alert_cooldown: 300, // 5 分钟
            auto_recovery: true,
            health_check_interval: 30,
            report_interval: 3600, // 1 小时
        }
    }
}

impl OpsMonitor {
    /// 创建新的运维监控器
    pub fn new() -> Self {
        let mut monitor = OpsMonitor {
            metrics_history: HashMap::new(),
            alert_rules: Vec::new(),
            active_alerts: HashMap::new(),
            alert_history: VecDeque::with_capacity(1000),
            nodes: HashMap::new(),
            config: MonitorConfig::default(),
            recovery_handlers: HashMap::new(),
        };

        monitor.initialize_default_rules();
        monitor
    }

    /// 初始化默认告警规则
    fn initialize_default_rules(&mut self) {
        let default_rules = vec![
            AlertRule {
                name: "High CPU Usage".to_string(),
                metric_type: MetricType::CpuUsage,
                threshold: 80.0,
                operator: ">".to_string(),
                duration: 300,
                severity: AlertSeverity::Warning,
                enabled: true,
            },
            AlertRule {
                name: "Critical CPU Usage".to_string(),
                metric_type: MetricType::CpuUsage,
                threshold: 95.0,
                operator: ">".to_string(),
                duration: 60,
                severity: AlertSeverity::Critical,
                enabled: true,
            },
            AlertRule {
                name: "High Memory Usage".to_string(),
                metric_type: MetricType::MemoryUsage,
                threshold: 85.0,
                operator: ">".to_string(),
                duration: 300,
                severity: AlertSeverity::Warning,
                enabled: true,
            },
            AlertRule {
                name: "Low Disk Space".to_string(),
                metric_type: MetricType::DiskUsage,
                threshold: 90.0,
                operator: ">".to_string(),
                duration: 60,
                severity: AlertSeverity::Critical,
                enabled: true,
            },
            AlertRule {
                name: "Block Sync Delay".to_string(),
                metric_type: MetricType::BlockSyncDelay,
                threshold: 300.0,
                operator: ">".to_string(),
                duration: 180,
                severity: AlertSeverity::Warning,
                enabled: true,
            },
            AlertRule {
                name: "No Peers".to_string(),
                metric_type: MetricType::PeerCount,
                threshold: 1.0,
                operator: "<".to_string(),
                duration: 60,
                severity: AlertSeverity::Critical,
                enabled: true,
            },
            AlertRule {
                name: "Low TPS".to_string(),
                metric_type: MetricType::Tps,
                threshold: 1.0,
                operator: "<".to_string(),
                duration: 300,
                severity: AlertSeverity::Warning,
                enabled: true,
            },
        ];

        self.alert_rules = default_rules;
    }

    /// 记录指标
    pub fn record_metric(&mut self, metric: Metric) {
        let history = self
            .metrics_history
            .entry(metric.metric_type)
            .or_insert_with(|| VecDeque::with_capacity(self.config.history_size));

        if history.len() >= self.config.history_size {
            history.pop_front();
        }
        history.push_back(metric.clone());

        // 检查告警规则
        self.check_alert_rules(&metric);
    }

    /// 检查告警规则
    fn check_alert_rules(&mut self, metric: &Metric) {
        let triggered_rules: Vec<(String, bool)> = self.alert_rules
            .iter()
            .filter(|rule| rule.enabled && rule.metric_type == metric.metric_type)
            .map(|rule| {
                let triggered = match rule.operator.as_str() {
                    ">" => metric.value > rule.threshold,
                    ">=" => metric.value >= rule.threshold,
                    "<" => metric.value < rule.threshold,
                    "<=" => metric.value <= rule.threshold,
                    "=" => (metric.value - rule.threshold).abs() < f64::EPSILON,
                    _ => false,
                };
                (rule.name.clone(), triggered)
            })
            .collect();

        for (rule_name, triggered) in triggered_rules {
            if triggered {
                if let Some(rule) = self.alert_rules.iter().find(|r| r.name == rule_name) {
                    let rule = rule.clone();
                    self.trigger_alert(&rule, metric);
                }
            } else {
                self.resolve_alert(&rule_name);
            }
        }
    }

    /// 触发告警
    fn trigger_alert(&mut self, rule: &AlertRule, metric: &Metric) {
        let alert_id = format!("{}_{}", rule.name, metric.timestamp);

        // 检查是否已存在相同告警
        if self.active_alerts.contains_key(&alert_id) {
            return;
        }

        // 检查冷却时间
        let now = current_timestamp();
        let recent_similar = self.alert_history.iter().any(|a| {
            a.rule_name == rule.name && now - a.triggered_at < self.config.alert_cooldown
        });

        if recent_similar {
            return;
        }

        let suggested_action = match rule.metric_type {
            MetricType::CpuUsage => "考虑增加CPU资源或优化节点配置".to_string(),
            MetricType::MemoryUsage => "建议增加内存或重启节点释放内存".to_string(),
            MetricType::DiskUsage => "请清理磁盘空间或扩展存储".to_string(),
            MetricType::BlockSyncDelay => "检查网络连接，可能需要重启同步".to_string(),
            MetricType::PeerCount => "检查网络配置和防火墙设置".to_string(),
            _ => "请检查相关配置".to_string(),
        };

        let alert = AlertEvent {
            id: alert_id.clone(),
            rule_name: rule.name.clone(),
            severity: rule.severity,
            message: format!(
                "{} 触发: 当前值 {:.2}, 阈值 {:.2}",
                rule.name, metric.value, rule.threshold
            ),
            triggered_at: now,
            resolved_at: None,
            metric_value: metric.value,
            suggested_action,
        };

        self.active_alerts.insert(alert_id.clone(), alert.clone());
        self.alert_history.push_back(alert);

        // 如果历史记录太多，清理旧的
        while self.alert_history.len() > 1000 {
            self.alert_history.pop_front();
        }

        // 尝试自动恢复
        if self.config.auto_recovery {
            self.attempt_recovery(&alert_id);
        }
    }

    /// 解决告警
    fn resolve_alert(&mut self, rule_name: &str) {
        let now = current_timestamp();
        let to_resolve: Vec<String> = self
            .active_alerts
            .iter()
            .filter(|(_, alert)| alert.rule_name == rule_name)
            .map(|(id, _)| id.clone())
            .collect();

        for id in to_resolve {
            if let Some(alert) = self.active_alerts.get_mut(&id) {
                alert.resolved_at = Some(now);
            }
            self.active_alerts.remove(&id);
        }
    }

    /// 尝试自动恢复
    fn attempt_recovery(&self, alert_id: &str) {
        if let Some(handler) = self.recovery_handlers.get(alert_id) {
            if let Err(e) = handler() {
                eprintln!("自动恢复失败: {:?}", e);
            }
        }
    }

    /// 注册恢复处理器
    pub fn register_recovery_handler<F>(&mut self, alert_type: &str, handler: F)
    where
        F: Fn() -> Result<(), AIError> + Send + Sync + 'static,
    {
        self.recovery_handlers.insert(alert_type.to_string(), Box::new(handler));
    }

    /// 更新节点信息
    pub fn update_node(&mut self, node: NodeInfo) {
        self.nodes.insert(node.node_id.clone(), node);
    }

    /// 获取节点健康状态
    pub fn get_node_health(&self, node_id: &str) -> HealthStatus {
        if let Some(node) = self.nodes.get(node_id) {
            let now = current_timestamp();
            let heartbeat_age = now - node.last_heartbeat;

            if heartbeat_age > 300 {
                return HealthStatus::Offline;
            }

            // 基于指标计算健康状态
            let mut worst_status = HealthStatus::Healthy;

            if let Some(history) = self.metrics_history.get(&MetricType::CpuUsage) {
                if let Some(latest) = history.back() {
                    if latest.value > 95.0 {
                        worst_status = HealthStatus::Critical;
                    } else if latest.value > 80.0 {
                        worst_status = HealthStatus::Warning;
                    }
                }
            }

            worst_status
        } else {
            HealthStatus::Unknown
        }
    }

    /// 生成性能报告
    pub fn generate_performance_report(&self) -> PerformanceReport {
        let timestamp = current_timestamp();
        let mut metric_scores = HashMap::new();
        let mut bottlenecks = Vec::new();
        let mut trends = Vec::new();

        // 计算各项指标评分
        for (metric_type, history) in &self.metrics_history {
            if history.len() < 10 {
                continue;
            }

            let score = self.calculate_metric_score(*metric_type, history);
            metric_scores.insert(*metric_type, score);

            // 识别瓶颈
            if score < 60 {
                bottlenecks.push(format!("{:?} 性能较差 (评分: {})", metric_type, score));
            }

            // 分析趋势
            if let Some(trend) = self.analyze_trend(*metric_type, history) {
                trends.push(trend);
            }
        }

        // 生成优化建议
        let recommendations = self.generate_recommendations(&metric_scores);

        // 计算整体健康评分
        let health_score = if metric_scores.is_empty() {
            100
        } else {
            let total: u32 = metric_scores.values().map(|&v| v as u32).sum();
            (total / metric_scores.len() as u32) as u8
        };

        PerformanceReport {
            timestamp,
            health_score,
            metric_scores,
            bottlenecks,
            recommendations,
            trends,
        }
    }

    /// 计算指标评分
    fn calculate_metric_score(&self, metric_type: MetricType, history: &VecDeque<Metric>) -> u8 {
        let recent: Vec<_> = history.iter().rev().take(10).collect();
        if recent.is_empty() {
            return 100;
        }

        let avg = recent.iter().map(|m| m.value).sum::<f64>() / recent.len() as f64;

        match metric_type {
            MetricType::CpuUsage | MetricType::MemoryUsage | MetricType::DiskUsage => {
                // 使用率越低越好
                (100.0 - avg).max(0.0) as u8
            }
            MetricType::BlockSyncDelay | MetricType::BlockProcessTime | MetricType::DbQueryTime => {
                // 延迟越低越好，使用对数评分
                let normalized = (100.0 - avg.log10() * 10.0).max(0.0);
                normalized.min(100.0) as u8
            }
            MetricType::Tps => {
                // TPS 越高越好
                (avg * 10.0).min(100.0) as u8
            }
            MetricType::PeerCount => {
                // 对等节点数适中最好
                if avg >= 8.0 && avg <= 20.0 {
                    100
                } else {
                    (100.0 - (avg - 14.0).abs() * 5.0).max(0.0) as u8
                }
            }
            _ => 100,
        }
    }

    /// 分析趋势
    fn analyze_trend(&self, metric_type: MetricType, history: &VecDeque<Metric>) -> Option<TrendAnalysis> {
        if history.len() < 20 {
            return None;
        }

        let recent: Vec<_> = history.iter().rev().take(10).collect();
        let older: Vec<_> = history.iter().rev().skip(10).take(10).collect();

        let recent_avg = recent.iter().map(|m| m.value).sum::<f64>() / recent.len() as f64;
        let older_avg = older.iter().map(|m| m.value).sum::<f64>() / older.len() as f64;

        if older_avg == 0.0 {
            return None;
        }

        let change_rate = (recent_avg - older_avg) / older_avg * 100.0;

        let direction = if change_rate > 5.0 {
            TrendDirection::Increasing
        } else if change_rate < -5.0 {
            TrendDirection::Decreasing
        } else if change_rate.abs() < 2.0 {
            TrendDirection::Stable
        } else {
            TrendDirection::Fluctuating
        };

        // 简单线性预测
        let forecast = recent_avg + (recent_avg - older_avg) / 2.0;

        Some(TrendAnalysis {
            metric_type,
            direction,
            change_rate,
            forecast,
        })
    }

    /// 生成优化建议
    fn generate_recommendations(&self, scores: &HashMap<MetricType, u8>) -> Vec<OptimizationSuggestion> {
        let mut recommendations = Vec::new();

        if let Some(&score) = scores.get(&MetricType::CpuUsage) {
            if score < 60 {
                recommendations.push(OptimizationSuggestion {
                    category: "CPU".to_string(),
                    description: "CPU使用率持续偏高，建议升级CPU或优化节点代码".to_string(),
                    expected_improvement: "预计可提升30%的处理能力".to_string(),
                    difficulty: 3,
                    priority: 90 - score,
                });
            }
        }

        if let Some(&score) = scores.get(&MetricType::MemoryUsage) {
            if score < 60 {
                recommendations.push(OptimizationSuggestion {
                    category: "内存".to_string(),
                    description: "内存使用接近上限，建议增加内存或优化缓存策略".to_string(),
                    expected_improvement: "预计可减少50%的内存占用".to_string(),
                    difficulty: 2,
                    priority: 90 - score,
                });
            }
        }

        if let Some(&score) = scores.get(&MetricType::BlockSyncDelay) {
            if score < 60 {
                recommendations.push(OptimizationSuggestion {
                    category: "同步".to_string(),
                    description: "区块同步延迟较大，建议优化网络连接或增加对等节点".to_string(),
                    expected_improvement: "预计可减少70%的同步时间".to_string(),
                    difficulty: 2,
                    priority: 95 - score,
                });
            }
        }

        // 按优先级排序
        recommendations.sort_by(|a, b| b.priority.cmp(&a.priority));
        recommendations.truncate(5);

        recommendations
    }

    /// 获取活跃告警
    pub fn get_active_alerts(&self) -> Vec<&AlertEvent> {
        self.active_alerts.values().collect()
    }

    /// 获取告警历史
    pub fn get_alert_history(&self, limit: usize) -> Vec<&AlertEvent> {
        self.alert_history.iter().rev().take(limit).collect()
    }

    /// 获取节点列表
    pub fn get_nodes(&self) -> Vec<&NodeInfo> {
        self.nodes.values().collect()
    }

    /// 获取系统概览
    pub fn get_system_overview(&self) -> SystemOverview {
        let total_nodes = self.nodes.len();
        let healthy_nodes = self.nodes.values().filter(|n| {
            matches!(n.health, HealthStatus::Healthy)
        }).count();

        let active_alerts = self.active_alerts.len();
        let critical_alerts = self.active_alerts.values().filter(|a| {
            matches!(a.severity, AlertSeverity::Critical | AlertSeverity::Emergency)
        }).count();

        SystemOverview {
            total_nodes,
            healthy_nodes,
            unhealthy_nodes: total_nodes - healthy_nodes,
            active_alerts,
            critical_alerts,
            avg_block_height: self.nodes.values().map(|n| n.block_height).sum::<u64>() / total_nodes.max(1) as u64,
            total_peers: self.nodes.values().map(|n| n.peer_count).sum(),
        }
    }
}

/// 系统概览
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemOverview {
    pub total_nodes: usize,
    pub healthy_nodes: usize,
    pub unhealthy_nodes: usize,
    pub active_alerts: usize,
    pub critical_alerts: usize,
    pub avg_block_height: u64,
    pub total_peers: usize,
}

/// 获取当前时间戳
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl Default for OpsMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ops_monitor_creation() {
        let monitor = OpsMonitor::new();
        assert!(!monitor.alert_rules.is_empty());
    }

    #[test]
    fn test_record_metric() {
        let mut monitor = OpsMonitor::new();
        
        let metric = Metric {
            metric_type: MetricType::CpuUsage,
            value: 75.0,
            unit: "%".to_string(),
            timestamp: current_timestamp(),
            labels: HashMap::new(),
        };
        
        monitor.record_metric(metric);
        
        assert!(monitor.metrics_history.contains_key(&MetricType::CpuUsage));
    }

    #[test]
    fn test_node_health() {
        let mut monitor = OpsMonitor::new();
        
        let node = NodeInfo {
            node_id: "test_node".to_string(),
            address: "127.0.0.1:8080".to_string(),
            version: "1.0.0".to_string(),
            health: HealthStatus::Healthy,
            last_heartbeat: current_timestamp(),
            block_height: 1000,
            peer_count: 10,
            uptime: 3600,
            location: None,
        };
        
        monitor.update_node(node);
        
        let health = monitor.get_node_health("test_node");
        assert!(matches!(health, HealthStatus::Healthy | HealthStatus::Unknown));
    }

    #[test]
    fn test_performance_report() {
        let mut monitor = OpsMonitor::new();
        
        // 添加一些测试指标
        for i in 0..20 {
            monitor.record_metric(Metric {
                metric_type: MetricType::CpuUsage,
                value: 60.0 + i as f64,
                unit: "%".to_string(),
                timestamp: current_timestamp() + i,
                labels: HashMap::new(),
            });
        }
        
        let report = monitor.generate_performance_report();
        assert!(report.health_score <= 100);
    }
}
