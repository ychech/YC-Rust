//! 09. 安全加固 - 攻击防护实现

use std::collections::{HashSet, HashMap};
use std::time::{SystemTime, UNIX_EPOCH};

/// 重放保护器
pub struct ReplayProtector {
    processed_txs: HashSet<String>,
    tx_expiration: u64, // 秒
}

impl ReplayProtector {
    pub fn new() -> Self {
        Self {
            processed_txs: HashSet::new(),
            tx_expiration: 300, // 5分钟
        }
    }
    
    /// 检查是否重放
    pub fn is_replay(&self, tx_hash: &str) -> bool {
        self.processed_txs.contains(tx_hash)
    }
    
    /// 标记已处理
    pub fn mark_processed(&mut self, tx_hash: String) {
        self.processed_txs.insert(tx_hash);
    }
    
    /// 验证时间戳
    pub fn validate_timestamp(&self, timestamp: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // 不能来自未来
        if timestamp > now + 60 {
            return false;
        }
        
        // 不能过期
        if now - timestamp > self.tx_expiration {
            return false;
        }
        
        true
    }
}

/// 速率限制器
pub struct RateLimiter {
    requests: HashMap<String, (u32, u64)>, // ip => (count, window_start)
    limit: u32,
    window: u64, // 秒
}

impl RateLimiter {
    pub fn new(limit: u32) -> Self {
        Self {
            requests: HashMap::new(),
            limit,
            window: 60, // 1分钟
        }
    }
    
    /// 检查是否允许请求
    pub fn allow_request(&mut self, ip: &str) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let window_start = now / self.window * self.window;
        
        let entry = self.requests.entry(ip.to_string()).or_insert((0, window_start));
        
        if entry.1 != window_start {
            *entry = (0, window_start);
        }
        
        if entry.0 >= self.limit {
            return false;
        }
        
        entry.0 += 1;
        true
    }
}

/// 攻击防护管理器
pub struct AttackPrevention {
    replay_protector: ReplayProtector,
    rate_limiter: RateLimiter,
    max_peers_per_ip: usize,
    peer_ips: HashMap<String, usize>,
}

impl AttackPrevention {
    pub fn new() -> Self {
        Self {
            replay_protector: ReplayProtector::new(),
            rate_limiter: RateLimiter::new(60),
            max_peers_per_ip: 10,
            peer_ips: HashMap::new(),
        }
    }
    
    /// 验证交易
    pub fn validate_transaction(&self, tx_hash: &str, timestamp: u64) -> Result<(), String> {
        if self.replay_protector.is_replay(tx_hash) {
            return Err("重放攻击检测".to_string());
        }
        
        if !self.replay_protector.validate_timestamp(timestamp) {
            return Err("无效时间戳".to_string());
        }
        
        Ok(())
    }
    
    /// 检查API请求
    pub fn check_api_request(&mut self, ip: &str) -> Result<(), String> {
        if !self.rate_limiter.allow_request(ip) {
            return Err("请求过于频繁".to_string());
        }
        Ok(())
    }
    
    /// 检查节点连接
    pub fn check_peer_connection(&mut self, ip: &str) -> bool {
        let count = self.peer_ips.entry(ip.to_string()).or_insert(0);
        if *count >= self.max_peers_per_ip {
            return false;
        }
        *count += 1;
        true
    }
}

fn main() {
    println!("安全加固演示\n");
    
    let mut security = AttackPrevention::new();
    
    // 重放保护演示
    println!("=== 重放保护 ===");
    let tx_hash = "tx_abc123";
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    match security.validate_transaction(tx_hash, timestamp) {
        Ok(()) => println!("✅ 交易验证通过"),
        Err(e) => println!("❌ {}", e),
    }
    
    // 第二次尝试（重放）
    match security.validate_transaction(tx_hash, timestamp) {
        Ok(()) => println!("✅ 交易验证通过"),
        Err(e) => println!("❌ {} (检测到重放)", e),
    }
    
    // 速率限制演示
    println!("\n=== 速率限制 ===");
    let ip = "192.168.1.1";
    for i in 1..=65 {
        match security.check_api_request(ip) {
            Ok(()) => {
                if i > 58 {
                    println!("请求 {}: ✅", i);
                }
            }
            Err(e) => {
                println!("请求 {}: ❌ {}", i, e);
                break;
            }
        }
    }
}
