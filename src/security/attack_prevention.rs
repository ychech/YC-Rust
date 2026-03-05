//! 攻击防护模块
//! 实现各种区块链攻击的防护措施

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 攻击类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AttackType {
    MajorityAttack,    // 51%攻击
    DoubleSpend,       // 双花攻击
    SybilAttack,       // Sybil攻击
    EclipseAttack,     // 日食攻击
    DustAttack,        // 粉尘攻击
    Timejacking,       // 时间劫持
    FastMining,        // 快速出块
}

/// 攻击防护配置
#[derive(Debug, Clone)]
pub struct AttackPreventionConfig {
    pub max_time_drift: u64,
    pub min_block_interval: u64,
    pub max_block_size: usize,
    pub max_reorg_depth: u32,
    pub max_peers_per_ip: usize,
}

impl Default for AttackPreventionConfig {
    fn default() -> Self {
        Self {
            max_time_drift: 7200,
            min_block_interval: 60,
            max_block_size: 2_000_000,
            max_reorg_depth: 100,
            max_peers_per_ip: 10,
        }
    }
}

/// 攻击防护管理器
pub struct AttackPreventionManager {
    config: AttackPreventionConfig,
    peer_ips: Arc<RwLock<HashMap<String, usize>>>,
}

impl AttackPreventionManager {
    pub fn new(config: AttackPreventionConfig) -> Self {
        Self {
            config,
            peer_ips: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// 检查IP连接数限制（防Sybil攻击）
    pub async fn check_peer_limit(&self, ip: &str) -> bool {
        let peers = self.peer_ips.read().await;
        if let Some(count) = peers.get(ip) {
            *count < self.config.max_peers_per_ip
        } else {
            true
        }
    }
    
    /// 记录新连接
    pub async fn record_connection(&self, ip: &str) {
        let mut peers = self.peer_ips.write().await;
        *peers.entry(ip.to_string()).or_insert(0) += 1;
    }
    
    /// 验证区块时间（防时间劫持）
    pub fn validate_timestamp(&self, timestamp: u64, prev_timestamp: u64) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // 区块时间不能来自未来
        if timestamp > now + self.config.max_time_drift {
            return false;
        }
        
        // 区块时间不能早于前一区块
        if timestamp < prev_timestamp {
            return false;
        }
        
        true
    }
    
    /// 验证区块大小
    pub fn validate_block_size(&self, size: usize) -> bool {
        size <= self.config.max_block_size
    }
    
    /// 验证重组深度
    pub fn validate_reorg_depth(&self, depth: u32) -> bool {
        depth <= self.config.max_reorg_depth
    }
}
