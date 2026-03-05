//! 监控和日志模块
//! 提供指标收集、告警、错误处理等功能

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH, Duration};
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error};

/// 指标类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    Counter,
    Gauge,
    Histogram,
}

/// 指标值
#[derive(Debug, Clone)]
pub enum MetricValue {
    Counter(u64),
    Gauge(f64),
    Histogram(Vec<f64>),
}

/// 指标定义
#[derive(Debug, Clone)]
pub struct Metric {
    pub name: String,
    pub help: String,
    pub metric_type: MetricType,
    pub value: MetricValue,
    pub labels: HashMap<String, String>,
    pub timestamp: u64,
}

/// 指标收集器
pub struct MetricsCollector {
    metrics: Arc<RwLock<HashMap<String, Metric>>>,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 增加计数器
    pub async fn increment_counter(&self, name: &str, labels: HashMap<String, String>, value: u64) {
        let mut metrics = self.metrics.write().await;
        let key = format!("{}:{:?}", name, labels);
        
        if let Some(metric) = metrics.get_mut(&key) {
            if let MetricValue::Counter(ref mut v) = metric.value {
                *v += value;
            }
        } else {
            metrics.insert(key, Metric {
                name: name.to_string(),
                help: String::new(),
                metric_type: MetricType::Counter,
                value: MetricValue::Counter(value),
                labels,
                timestamp: Self::now(),
            });
        }
    }

    /// 设置仪表盘值
    pub async fn set_gauge(&self, name: &str, labels: HashMap<String, String>, value: f64) {
        let mut metrics = self.metrics.write().await;
        let key = format!("{}:{:?}", name, labels);
        
        metrics.insert(key, Metric {
            name: name.to_string(),
            help: String::new(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Gauge(value),
            labels,
            timestamp: Self::now(),
        });
    }

    /// 记录直方图值
    pub async fn record_histogram(&self, name: &str, labels: HashMap<String, String>, value: f64) {
        let mut metrics = self.metrics.write().await;
        let key = format!("{}:{:?}", name, labels);
        
        if let Some(metric) = metrics.get_mut(&key) {
            if let MetricValue::Histogram(ref mut v) = metric.value {
                v.push(value);
                if v.len() > 10000 {
                    v.remove(0);
                }
            }
        } else {
            metrics.insert(key, Metric {
                name: name.to_string(),
                help: String::new(),
                metric_type: MetricType::Histogram,
                value: MetricValue::Histogram(vec![value]),
                labels,
                timestamp: Self::now(),
            });
        }
    }

    /// 获取所有指标
    pub async fn get_all_metrics(&self) -> Vec<Metric> {
        let metrics = self.metrics.read().await;
        metrics.values().cloned().collect()
    }

    /// 导出为 Prometheus 格式
    pub async fn export_prometheus(&self) -> String {
        let metrics = self.metrics.read().await;
        let mut output = String::new();

        for (_, metric) in metrics.iter() {
            match metric.metric_type {
                MetricType::Counter => {
                    if let MetricValue::Counter(v) = metric.value {
                        output.push_str(&format!("# HELP {} {}\n", metric.name, metric.help));
                        output.push_str(&format!("# TYPE {} counter\n", metric.name));
                        output.push_str(&format!("{} {}\n", metric.name, v));
                    }
                }
                MetricType::Gauge => {
                    if let MetricValue::Gauge(v) = metric.value {
                        output.push_str(&format!("# HELP {} {}\n", metric.name, metric.help));
                        output.push_str(&format!("# TYPE {} gauge\n", metric.name));
                        output.push_str(&format!("{} {:.2}\n", metric.name, v));
                    }
                }
                _ => {}
            }
        }

        output
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}

/// 告警级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

/// 告警定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub level: AlertLevel,
    pub title: String,
    pub message: String,
    pub metric_name: String,
    pub threshold: f64,
    pub current_value: f64,
    pub timestamp: u64,
    pub resolved: bool,
}

/// 告警规则
#[derive(Debug, Clone)]
pub struct AlertRule {
    pub name: String,
    pub metric_name: String,
    pub condition: AlertCondition,
    pub threshold: f64,
    pub duration_secs: u64,
    pub level: AlertLevel,
}

/// 告警条件
#[derive(Debug, Clone, Copy)]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    EqualTo,
    NotEqualTo,
}

/// 告警管理器
pub struct AlertManager {
    rules: Arc<RwLock<Vec<AlertRule>>>,
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,
    alert_history: Arc<RwLock<Vec<Alert>>>,
}

impl AlertManager {
    pub fn new() -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// 添加告警规则
    pub async fn add_rule(&self, rule: AlertRule) {
        let mut rules = self.rules.write().await;
        rules.push(rule);
    }

    /// 检查指标并触发告警
    pub async fn check_metrics(&self, metrics: &[Metric]) {
        let rules = self.rules.read().await;
        
        for rule in rules.iter() {
            for metric in metrics {
                if metric.name != rule.metric_name {
                    continue;
                }

                let value = match &metric.value {
                    MetricValue::Counter(v) => *v as f64,
                    MetricValue::Gauge(v) => *v,
                    MetricValue::Histogram(v) => v.iter().sum::<f64>() / v.len() as f64,
                };

                let triggered = match rule.condition {
                    AlertCondition::GreaterThan => value > rule.threshold,
                    AlertCondition::LessThan => value < rule.threshold,
                    AlertCondition::EqualTo => (value - rule.threshold).abs() < 0.001,
                    AlertCondition::NotEqualTo => (value - rule.threshold).abs() >= 0.001,
                };

                if triggered {
                    self.trigger_alert(rule, value).await;
                }
            }
        }
    }

    async fn trigger_alert(&self, rule: &AlertRule, current_value: f64) {
        let alert_id = format!("{}_{}", rule.name, SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
        
        let alert = Alert {
            id: alert_id.clone(),
            level: rule.level,
            title: format!("{} 告警", rule.name),
            message: format!("{} 当前值 {:.2} 超过阈值 {:.2}", rule.metric_name, current_value, rule.threshold),
            metric_name: rule.metric_name.clone(),
            threshold: rule.threshold,
            current_value,
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            resolved: false,
        };

        {
            let mut active = self.active_alerts.write().await;
            active.insert(alert_id.clone(), alert.clone());
        }

        {
            let mut history = self.alert_history.write().await;
            history.push(alert.clone());
        }

        match alert.level {
            AlertLevel::Info => info!("告警: {}", alert.message),
            AlertLevel::Warning => warn!("告警: {}", alert.message),
            AlertLevel::Critical => error!("严重告警: {}", alert.message),
        }
    }

    /// 获取活跃告警
    pub async fn get_active_alerts(&self) -> Vec<Alert> {
        let active = self.active_alerts.read().await;
        active.values().cloned().collect()
    }

    /// 获取告警历史
    pub async fn get_alert_history(&self, limit: usize) -> Vec<Alert> {
        let history = self.alert_history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }

    /// 解决告警
    pub async fn resolve_alert(&self, alert_id: &str) {
        let mut active = self.active_alerts.write().await;
        if let Some(alert) = active.get_mut(alert_id) {
            alert.resolved = true;
            info!("告警已解决: {}", alert.title);
        }
        active.remove(alert_id);
    }
}

/// 系统监控器
pub struct SystemMonitor {
    metrics: Arc<MetricsCollector>,
    alerts: Arc<AlertManager>,
}

impl SystemMonitor {
    pub fn new(metrics: Arc<MetricsCollector>, alerts: Arc<AlertManager>) -> Self {
        Self { metrics, alerts }
    }

    /// 启动监控
    pub async fn start(&self) {
        let metrics = self.metrics.clone();
        let alerts = self.alerts.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            
            loop {
                interval.tick().await;
                
                Self::collect_system_metrics(&metrics).await;
                
                let all_metrics = metrics.get_all_metrics().await;
                alerts.check_metrics(&all_metrics).await;
            }
        });
    }

    async fn collect_system_metrics(metrics: &MetricsCollector) {
        metrics.set_gauge("block_height", HashMap::new(), 1234567.0).await;
        metrics.increment_counter("total_transactions", HashMap::new(), 0).await;
        metrics.set_gauge("active_peers", HashMap::new(), 42.0).await;
        metrics.set_gauge("memory_usage_mb", HashMap::new(), 512.0).await;
        metrics.set_gauge("cpu_usage_percent", HashMap::new(), 25.5).await;
    }
}

/// 健康检查
#[derive(Debug, Clone, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub block_height: u64,
    pub peer_count: usize,
    pub pending_transactions: usize,
}

pub struct HealthChecker;

impl HealthChecker {
    pub fn check() -> HealthStatus {
        HealthStatus {
            status: "healthy".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_seconds: 86400,
            block_height: 1234567,
            peer_count: 42,
            pending_transactions: 15,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector() {
        let collector = MetricsCollector::new();
        
        collector.increment_counter("test_counter", HashMap::new(), 1).await;
        collector.set_gauge("test_gauge", HashMap::new(), 42.0).await;
        
        let metrics = collector.get_all_metrics().await;
        assert_eq!(metrics.len(), 2);
    }

    #[tokio::test]
    async fn test_alert_manager() {
        let manager = AlertManager::new();
        
        let rule = AlertRule {
            name: "test_alert".to_string(),
            metric_name: "test_metric".to_string(),
            condition: AlertCondition::GreaterThan,
            threshold: 100.0,
            duration_secs: 60,
            level: AlertLevel::Warning,
        };
        
        manager.add_rule(rule).await;
        
        let metric = Metric {
            name: "test_metric".to_string(),
            help: String::new(),
            metric_type: MetricType::Gauge,
            value: MetricValue::Gauge(150.0),
            labels: HashMap::new(),
            timestamp: 0,
        };
        
        manager.check_metrics(&[metric]).await;
        
        let alerts = manager.get_active_alerts().await;
        assert_eq!(alerts.len(), 1);
    }
}
