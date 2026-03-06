//! YRust Chain - AI 驱动的智能公链
//! 
//! 特性:
//! - 工作量证明 (POW) 挖矿
//! - 钱包 / 私钥 / 公钥 / 地址
//! - 交易签名与验签
//! - UTXO 模型
//! - P2P 节点同步
//! - 区块浏览器 API
//! - 数据持久化
//! - 智能合约引擎 (WASM)
//! - AI 智能挖矿调度
//! - AI 风控检测
//! - AI 自然语言查询
//! - AI 智能运维
//! - AI 合约生成
//! - AI 钱包助手

pub mod core;
pub mod wallet;
pub mod network;
pub mod api;
pub mod storage;
pub mod vm;
pub mod ai;

// 重新导出常用类型
pub use core::{Block, Blockchain, Transaction, Amount, Address, Hash};
pub use wallet::Wallet;

/// 版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 获取版本信息
pub fn version() -> &'static str {
    VERSION
}
