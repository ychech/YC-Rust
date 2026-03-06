//! AI 智能模块 - 为 YRust Chain 提供 AI 能力
//!
//! 功能模块:
//! - mining: AI 智能挖矿调度
//! - risk: AI 风控检测
//! - nlp: 自然语言查询
//! - ops: 智能运维监控
//! - contract: AI 合约生成
//! - wallet: AI 钱包助手

pub mod mining;
pub mod risk;
pub mod nlp;
pub mod ops;
pub mod contract;
pub mod wallet;

use serde::{Deserialize, Serialize};

/// AI 模块配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    /// 启用 AI 挖矿调度
    pub enable_smart_mining: bool,
    /// 启用 AI 风控
    pub enable_risk_detection: bool,
    /// 启用自然语言查询
    pub enable_nlp_query: bool,
    /// 启用智能运维
    pub enable_smart_ops: bool,
    /// 启用 AI 合约生成
    pub enable_ai_contract: bool,
    /// 启用 AI 钱包助手
    pub enable_ai_wallet: bool,
    /// AI 服务 API 端点 (豆包/其他 LLM)
    pub ai_api_endpoint: Option<String>,
    /// API 密钥
    pub ai_api_key: Option<String>,
}

impl Default for AIConfig {
    fn default() -> Self {
        AIConfig {
            enable_smart_mining: true,
            enable_risk_detection: true,
            enable_nlp_query: true,
            enable_smart_ops: true,
            enable_ai_contract: true,
            enable_ai_wallet: true,
            ai_api_endpoint: None,
            ai_api_key: None,
        }
    }
}

/// AI 服务管理器
pub struct AIService {
    pub config: AIConfig,
    pub mining: mining::SmartMining,
    pub risk: risk::RiskEngine,
    pub nlp: nlp::NLPQueryEngine,
    pub ops: ops::OpsMonitor,
    pub contract: contract::ContractGenerator,
    pub wallet: wallet::AIWalletAssistant,
}

impl AIService {
    /// 创建新的 AI 服务
    pub fn new(config: AIConfig) -> Self {
        AIService {
            config: config.clone(),
            mining: mining::SmartMining::new(),
            risk: risk::RiskEngine::new(),
            nlp: nlp::NLPQueryEngine::new(),
            ops: ops::OpsMonitor::new(),
            contract: contract::ContractGenerator::new(config.ai_api_endpoint),
            wallet: wallet::AIWalletAssistant::new(),
        }
    }

    /// 创建默认 AI 服务
    pub fn default() -> Self {
        Self::new(AIConfig::default())
    }
}

/// AI 功能统一错误类型
#[derive(Debug, Clone)]
pub enum AIError {
    MiningError(String),
    RiskError(String),
    NLPError(String),
    OpsError(String),
    ContractError(String),
    WalletError(String),
    APIError(String),
    ConfigError(String),
}

impl std::fmt::Display for AIError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AIError::MiningError(e) => write!(f, "AI挖矿错误: {}", e),
            AIError::RiskError(e) => write!(f, "AI风控错误: {}", e),
            AIError::NLPError(e) => write!(f, "NLP错误: {}", e),
            AIError::OpsError(e) => write!(f, "运维错误: {}", e),
            AIError::ContractError(e) => write!(f, "合约生成错误: {}", e),
            AIError::WalletError(e) => write!(f, "钱包助手错误: {}", e),
            AIError::APIError(e) => write!(f, "API错误: {}", e),
            AIError::ConfigError(e) => write!(f, "配置错误: {}", e),
        }
    }
}

impl std::error::Error for AIError {}
