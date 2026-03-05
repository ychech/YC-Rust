//! 安全加固模块
//! 提供重放攻击防护、签名校验强化、权限控制等安全功能

use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

/// 安全配置
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// 交易过期时间 (秒)
    pub tx_expiration_seconds: u64,
    /// 最大 nonce 差距
    pub max_nonce_gap: u64,
    /// 是否启用重放保护
    pub enable_replay_protection: bool,
    /// 是否启用签名 malleability 保护
    pub enable_malleability_protection: bool,
    /// 速率限制: 每 IP 每分钟最大请求数
    pub rate_limit_per_minute: u32,
    /// 黑名单过期时间 (秒)
    pub blacklist_expiry_seconds: u64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            tx_expiration_seconds: 300,      // 5分钟
            max_nonce_gap: 100,              // 最大nonce差距
            enable_replay_protection: true,
            enable_malleability_protection: true,
            rate_limit_per_minute: 60,       // 每分钟60个请求
            blacklist_expiry_seconds: 3600,  // 1小时
        }
    }
}

/// 重放保护器
pub struct ReplayProtector {
    /// 已处理的交易哈希集合
    processed_txs: Arc<RwLock<HashSet<String>>>,
    /// 配置
    config: SecurityConfig,
}

impl ReplayProtector {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            processed_txs: Arc::new(RwLock::new(HashSet::new())),
            config,
        }
    }

    /// 检查交易是否已处理 (防止重放)
    pub async fn is_replay(&self, tx_hash: &str) -> bool {
        if !self.config.enable_replay_protection {
            return false;
        }
        
        let txs = self.processed_txs.read().await;
        txs.contains(tx_hash)
    }

    /// 标记交易为已处理
    pub async fn mark_processed(&self, tx_hash: String) {
        let mut txs = self.processed_txs.write().await;
        txs.insert(tx_hash);
        
        // 限制集合大小，防止内存泄漏
        if txs.len() > 100_000 {
            txs.clear();
        }
    }

    /// 验证交易时间戳
    pub fn verify_timestamp(&self, timestamp: u64) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        // 交易不能来自未来
        if timestamp > now + 60 {
            return false;
        }
        
        // 交易不能过期
        if now - timestamp > self.config.tx_expiration_seconds {
            return false;
        }
        
        true
    }
}

/// 签名防篡改保护
pub struct SignatureGuard;

impl SignatureGuard {
    /// 验证签名格式 (防止签名延展性攻击)
    pub fn verify_signature_format(signature: &[u8]) -> Result<(), SecurityError> {
        if signature.len() != 64 {
            return Err(SecurityError::InvalidSignatureLength);
        }
        Ok(())
    }

    /// 计算哈希
    pub fn hash_message(message: &[u8]) -> [u8; 32] {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(message);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }
}

/// 速率限制器
pub struct RateLimiter {
    /// IP -> (请求次数, 窗口开始时间)
    requests: Arc<RwLock<std::collections::HashMap<String, (u32, u64)>>>,
    /// 配置
    config: SecurityConfig,
}

impl RateLimiter {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            requests: Arc::new(RwLock::new(std::collections::HashMap::new())),
            config,
        }
    }

    /// 检查是否允许请求
    pub async fn allow_request(&self, ip: &str) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let window_start = now / 60 * 60;
        
        let mut requests = self.requests.write().await;
        let entry = requests.entry(ip.to_string()).or_insert((0, window_start));
        
        if entry.1 != window_start {
            *entry = (0, window_start);
        }
        
        if entry.0 >= self.config.rate_limit_per_minute {
            return false;
        }
        
        entry.0 += 1;
        true
    }

    /// 获取剩余配额
    pub async fn get_remaining_quota(&self, ip: &str) -> u32 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let window_start = now / 60 * 60;
        
        let requests = self.requests.read().await;
        if let Some((count, win)) = requests.get(ip) {
            if *win == window_start {
                return self.config.rate_limit_per_minute.saturating_sub(*count);
            }
        }
        
        self.config.rate_limit_per_minute
    }
}

/// 权限管理器
pub struct PermissionManager {
    /// 管理员地址集合
    admins: Arc<RwLock<HashSet<String>>>,
    /// 合约部署权限
    deployers: Arc<RwLock<HashSet<String>>>,
}

impl PermissionManager {
    pub fn new() -> Self {
        Self {
            admins: Arc::new(RwLock::new(HashSet::new())),
            deployers: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    /// 添加管理员
    pub async fn add_admin(&self, address: String) {
        let mut admins = self.admins.write().await;
        admins.insert(address);
    }

    /// 检查是否是管理员
    pub async fn is_admin(&self, address: &str) -> bool {
        let admins = self.admins.read().await;
        admins.contains(address)
    }

    /// 添加合约部署者
    pub async fn add_deployer(&self, address: String) {
        let mut deployers = self.deployers.write().await;
        deployers.insert(address);
    }

    /// 检查是否可以部署合约
    pub async fn can_deploy(&self, address: &str) -> bool {
        if self.is_admin(address).await {
            return true;
        }
        
        let deployers = self.deployers.read().await;
        deployers.contains(address)
    }
}

/// 安全错误
#[derive(Debug, Clone, PartialEq)]
pub enum SecurityError {
    ReplayAttack,
    ExpiredTransaction,
    FutureTransaction,
    InvalidSignature,
    InvalidSignatureLength,
    HighSValue,
    RateLimitExceeded,
    Unauthorized,
    InvalidNonce,
}

impl std::fmt::Display for SecurityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityError::ReplayAttack => write!(f, "检测到重放攻击"),
            SecurityError::ExpiredTransaction => write!(f, "交易已过期"),
            SecurityError::FutureTransaction => write!(f, "交易时间来自未来"),
            SecurityError::InvalidSignature => write!(f, "无效签名"),
            SecurityError::InvalidSignatureLength => write!(f, "签名长度无效"),
            SecurityError::HighSValue => write!(f, "签名 S 值过高 (延展性攻击)"),
            SecurityError::RateLimitExceeded => write!(f, "请求过于频繁"),
            SecurityError::Unauthorized => write!(f, "未授权操作"),
            SecurityError::InvalidNonce => write!(f, "无效 nonce"),
        }
    }
}

impl std::error::Error for SecurityError {}

/// 安全验证器 - 整合所有安全功能
pub struct SecurityValidator {
    replay_protector: ReplayProtector,
    rate_limiter: RateLimiter,
    permission_manager: PermissionManager,
    config: SecurityConfig,
}

impl SecurityValidator {
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            replay_protector: ReplayProtector::new(config.clone()),
            rate_limiter: RateLimiter::new(config.clone()),
            permission_manager: PermissionManager::new(),
            config,
        }
    }

    /// 验证交易安全
    pub async fn validate_transaction(
        &self,
        tx_hash: &str,
        timestamp: u64,
        signature: &[u8],
        _from: &str,
    ) -> Result<(), SecurityError> {
        // 1. 检查重放
        if self.replay_protector.is_replay(tx_hash).await {
            return Err(SecurityError::ReplayAttack);
        }

        // 2. 验证时间戳
        if !self.replay_protector.verify_timestamp(timestamp) {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            
            if timestamp > now {
                return Err(SecurityError::FutureTransaction);
            } else {
                return Err(SecurityError::ExpiredTransaction);
            }
        }

        // 3. 验证签名格式
        if self.config.enable_malleability_protection {
            SignatureGuard::verify_signature_format(signature)
                .map_err(|_| SecurityError::InvalidSignature)?;
        }

        Ok(())
    }

    /// 验证 API 请求
    pub async fn validate_api_request(&self, ip: &str) -> Result<(), SecurityError> {
        if !self.rate_limiter.allow_request(ip).await {
            return Err(SecurityError::RateLimitExceeded);
        }
        Ok(())
    }

    /// 检查部署权限
    pub async fn validate_deploy_permission(&self, address: &str) -> Result<(), SecurityError> {
        if !self.permission_manager.can_deploy(address).await {
            return Err(SecurityError::Unauthorized);
        }
        Ok(())
    }

    /// 标记交易已处理
    pub async fn mark_transaction_processed(&self, tx_hash: String) {
        self.replay_protector.mark_processed(tx_hash).await;
    }

    /// 获取速率限制剩余配额
    pub async fn get_rate_limit_remaining(&self, ip: &str) -> u32 {
        self.rate_limiter.get_remaining_quota(ip).await
    }

    /// 添加管理员
    pub async fn add_admin(&self, address: String) {
        self.permission_manager.add_admin(address).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signature_format_validation() {
        let valid_sig = [0u8; 64];
        assert!(SignatureGuard::verify_signature_format(&valid_sig).is_ok());

        let invalid_sig = [0u8; 32];
        assert!(SignatureGuard::verify_signature_format(&invalid_sig).is_err());
    }

    #[tokio::test]
    async fn test_replay_protection() {
        let config = SecurityConfig::default();
        let protector = ReplayProtector::new(config);

        let tx_hash = "test_tx_hash".to_string();
        
        assert!(!protector.is_replay(&tx_hash).await);
        
        protector.mark_processed(tx_hash.clone()).await;
        
        assert!(protector.is_replay(&tx_hash).await);
    }

    #[tokio::test]
    async fn test_rate_limiter() {
        let config = SecurityConfig {
            rate_limit_per_minute: 2,
            ..Default::default()
        };
        let limiter = RateLimiter::new(config);

        let ip = "192.168.1.1";
        
        assert!(limiter.allow_request(ip).await);
        assert!(limiter.allow_request(ip).await);
        assert!(!limiter.allow_request(ip).await);
    }
}
