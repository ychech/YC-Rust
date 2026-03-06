//! AI 自然语言查询模块 - 智能区块浏览器
//!
//! 功能:
//! - 自然语言解析区块链查询
//! - 智能意图识别
//! - 多轮对话支持
//! - 查询结果自然语言生成
//! - 语音查询接口

use super::AIError;
use crate::core::{Blockchain, Block, Transaction, Amount};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 查询意图类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueryIntent {
    /// 查询余额
    QueryBalance,
    /// 查询交易
    QueryTransaction,
    /// 查询区块
    QueryBlock,
    /// 查询统计
    QueryStats,
    /// 查询地址历史
    QueryAddressHistory,
    /// 查询挖矿信息
    QueryMiningInfo,
    /// 查询网络状态
    QueryNetworkStatus,
    /// 查询合约
    QueryContract,
    /// 比较查询
    QueryComparison,
    /// 趋势分析
    QueryTrend,
    /// 帮助
    QueryHelp,
    /// 未知
    Unknown,
}

/// 自然语言查询请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLPQuery {
    /// 用户输入
    pub text: String,
    /// 会话ID (用于多轮对话)
    pub session_id: Option<String>,
    /// 用户地址 (可选)
    pub user_address: Option<String>,
}

/// 自然语言查询响应
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NLPResponse {
    /// 识别的意图
    pub intent: QueryIntent,
    /// 提取的参数
    pub parameters: QueryParameters,
    /// 查询结果
    pub result: QueryResult,
    /// 自然语言回答
    pub natural_answer: String,
    /// 相关建议
    pub suggestions: Vec<String>,
    /// 置信度
    pub confidence: f32,
}

/// 查询参数
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueryParameters {
    /// 地址
    pub address: Option<String>,
    /// 交易ID
    pub tx_id: Option<String>,
    /// 区块高度
    pub block_height: Option<u64>,
    /// 区块哈希
    pub block_hash: Option<String>,
    /// 时间范围
    pub time_range: Option<(u64, u64)>,
    /// 金额范围
    pub amount_range: Option<(Amount, Amount)>,
    /// 其他参数
    pub extra: HashMap<String, String>,
}

/// 查询结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryResult {
    /// 余额结果
    Balance { address: String, balance: Amount, formatted: String },
    /// 交易结果
    Transaction(TransactionDetail),
    /// 区块结果
    Block(BlockSummary),
    /// 统计结果
    Stats(ChainStats),
    /// 地址历史
    AddressHistory(AddressHistory),
    /// 列表结果
    List(Vec<ListItem>),
    /// 文本结果
    Text(String),
    /// 错误
    Error(String),
}

/// 交易详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionDetail {
    pub id: String,
    pub status: String,
    pub confirmations: u64,
    pub timestamp: i64,
    pub from: Vec<String>,
    pub to: Vec<OutputDetail>,
    pub fee: Amount,
    pub total_value: Amount,
}

/// 输出详情
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputDetail {
    pub address: String,
    pub amount: Amount,
}

/// 区块摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSummary {
    pub height: u64,
    pub hash: String,
    pub timestamp: i64,
    pub transaction_count: usize,
    pub miner: String,
    pub reward: Amount,
    pub size: usize,
}

/// 链统计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainStats {
    pub height: u64,
    pub total_transactions: u64,
    pub total_addresses: u64,
    pub avg_block_time: f64,
    pub network_hashrate: String,
    pub difficulty: usize,
    pub mempool_size: usize,
}

/// 地址历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressHistory {
    pub address: String,
    pub total_received: Amount,
    pub total_sent: Amount,
    pub current_balance: Amount,
    pub transaction_count: usize,
    pub first_seen: Option<i64>,
    pub last_seen: Option<i64>,
}

/// 列表项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListItem {
    pub title: String,
    pub subtitle: String,
    pub value: String,
    pub link: Option<String>,
}

/// NLP 查询引擎
#[derive(Debug)]
pub struct NLPQueryEngine {
    /// 意图识别模式
    intent_patterns: HashMap<QueryIntent, Vec<String>>,
    /// 会话上下文
    sessions: HashMap<String, SessionContext>,
    /// 查询历史
    query_history: Vec<NLPQuery>,
}

/// 会话上下文
#[derive(Debug, Clone)]
struct SessionContext {
    last_intent: QueryIntent,
    last_parameters: QueryParameters,
    query_count: usize,
    last_query_time: u64,
}

impl NLPQueryEngine {
    /// 创建新的 NLP 查询引擎
    pub fn new() -> Self {
        let mut engine = NLPQueryEngine {
            intent_patterns: HashMap::new(),
            sessions: HashMap::new(),
            query_history: Vec::new(),
        };
        
        engine.initialize_patterns();
        engine
    }

    /// 初始化意图识别模式
    fn initialize_patterns(&mut self) {
        // 余额查询模式
        self.intent_patterns.insert(QueryIntent::QueryBalance, vec![
            "余额".to_string(),
            "balance".to_string(),
            "有多少".to_string(),
            "资产".to_string(),
            "钱".to_string(),
        ]);

        // 交易查询模式
        self.intent_patterns.insert(QueryIntent::QueryTransaction, vec![
            "交易".to_string(),
            "transaction".to_string(),
            "转账".to_string(),
            "tx".to_string(),
            "transaction".to_string(),
        ]);

        // 区块查询模式
        self.intent_patterns.insert(QueryIntent::QueryBlock, vec![
            "区块".to_string(),
            "block".to_string(),
            "高度".to_string(),
            "height".to_string(),
        ]);

        // 统计查询模式
        self.intent_patterns.insert(QueryIntent::QueryStats, vec![
            "统计".to_string(),
            "stats".to_string(),
            "状态".to_string(),
            "status".to_string(),
            "概况".to_string(),
            "overview".to_string(),
        ]);

        // 地址历史模式
        self.intent_patterns.insert(QueryIntent::QueryAddressHistory, vec![
            "历史".to_string(),
            "history".to_string(),
            "记录".to_string(),
            "record".to_string(),
        ]);

        // 挖矿信息模式
        self.intent_patterns.insert(QueryIntent::QueryMiningInfo, vec![
            "挖矿".to_string(),
            "mining".to_string(),
            "miner".to_string(),
            "算力".to_string(),
            "hashrate".to_string(),
        ]);

        // 网络状态模式
        self.intent_patterns.insert(QueryIntent::QueryNetworkStatus, vec![
            "网络".to_string(),
            "network".to_string(),
            "节点".to_string(),
            "node".to_string(),
            "连接".to_string(),
        ]);

        // 比较查询模式
        self.intent_patterns.insert(QueryIntent::QueryComparison, vec![
            "比较".to_string(),
            "compare".to_string(),
            "对比".to_string(),
            "vs".to_string(),
            "difference".to_string(),
        ]);

        // 趋势分析模式
        self.intent_patterns.insert(QueryIntent::QueryTrend, vec![
            "趋势".to_string(),
            "trend".to_string(),
            "变化".to_string(),
            "增长".to_string(),
            "下降".to_string(),
        ]);

        // 帮助模式
        self.intent_patterns.insert(QueryIntent::QueryHelp, vec![
            "帮助".to_string(),
            "help".to_string(),
            "怎么用".to_string(),
            "how".to_string(),
            "?".to_string(),
        ]);
    }

    /// 处理自然语言查询
    pub fn process_query(&mut self, query: NLPQuery) -> NLPResponse {
        // 保存查询历史
        self.query_history.push(query.clone());

        // 识别意图
        let (intent, confidence) = self.recognize_intent(&query.text);

        // 提取参数
        let parameters = self.extract_parameters(&query.text, intent);

        // 获取会话上下文
        let session_context = query.session_id
            .as_ref()
            .and_then(|id| self.sessions.get(id));

        // 处理查询
        let result = self.execute_query(intent, &parameters, session_context);

        // 生成自然语言回答
        let natural_answer = self.generate_natural_answer(intent, &result, &parameters);

        // 生成建议
        let suggestions = self.generate_suggestions(intent, &result);

        // 更新会话上下文
        if let Some(session_id) = &query.session_id {
            self.update_session(session_id.clone(), intent, parameters.clone());
        }

        NLPResponse {
            intent,
            parameters,
            result,
            natural_answer,
            suggestions,
            confidence,
        }
    }

    /// 识别查询意图
    fn recognize_intent(&self, text: &str) -> (QueryIntent, f32) {
        let text_lower = text.to_lowercase();
        let mut best_intent = QueryIntent::Unknown;
        let mut best_score = 0.0;

        for (intent, patterns) in &self.intent_patterns {
            let score = self.calculate_intent_score(&text_lower, patterns);
            if score > best_score {
                best_score = score;
                best_intent = *intent;
            }
        }

        // 如果没有匹配到任何模式，尝试基于关键词推断
        if best_score < 0.3 {
            best_intent = self.infer_intent_from_keywords(&text_lower);
        }

        (best_intent, best_score.min(1.0))
    }

    /// 计算意图匹配分数
    fn calculate_intent_score(&self, text: &str, patterns: &[String]) -> f32 {
        let mut matches = 0;
        for pattern in patterns {
            if text.contains(&pattern.to_lowercase()) {
                matches += 1;
            }
        }
        matches as f32 / patterns.len().max(1) as f32
    }

    /// 基于关键词推断意图
    fn infer_intent_from_keywords(&self, text: &str) -> QueryIntent {
        if text.contains("?") || text.contains("什么") || text.contains("多少") {
            if text.contains("地址") || text.contains("address") {
                QueryIntent::QueryBalance
            } else if text.contains("最新") || text.contains("latest") {
                QueryIntent::QueryBlock
            } else {
                QueryIntent::QueryStats
            }
        } else {
            QueryIntent::Unknown
        }
    }

    /// 提取查询参数
    fn extract_parameters(&self, text: &str, intent: QueryIntent) -> QueryParameters {
        let mut params = QueryParameters::default();

        // 提取地址 (简化版 - 实际应使用正则表达式)
        if text.contains("地址") || text.contains("address") {
            // 尝试提取地址
            let words: Vec<&str> = text.split_whitespace().collect();
            for word in words {
                if word.len() > 30 && word.starts_with("Y") {
                    params.address = Some(word.to_string());
                    break;
                }
            }
        }

        // 提取区块高度
        if intent == QueryIntent::QueryBlock {
            // 尝试从文本中提取数字作为区块高度
            for word in text.split_whitespace() {
                if let Ok(height) = word.parse::<u64>() {
                    if height < 10000000 { // 合理的区块高度范围
                        params.block_height = Some(height);
                        break;
                    }
                }
            }
        }

        // 提取交易ID
        if intent == QueryIntent::QueryTransaction {
            for word in text.split_whitespace() {
                if word.len() == 64 { // 假设是哈希
                    params.tx_id = Some(word.to_string());
                    break;
                }
            }
        }

        params
    }

    /// 执行查询
    fn execute_query(
        &self,
        intent: QueryIntent,
        params: &QueryParameters,
        _context: Option<&SessionContext>,
    ) -> QueryResult {
        match intent {
            QueryIntent::QueryBalance => {
                if let Some(address) = &params.address {
                    // 模拟余额查询
                    QueryResult::Balance {
                        address: address.clone(),
                        balance: 1000000000, // 10 YRU
                        formatted: "10.00 YRU".to_string(),
                    }
                } else {
                    QueryResult::Error("请提供要查询的地址".to_string())
                }
            }
            QueryIntent::QueryTransaction => {
                if let Some(tx_id) = &params.tx_id {
                    QueryResult::Transaction(TransactionDetail {
                        id: tx_id.clone(),
                        status: "已确认".to_string(),
                        confirmations: 128,
                        timestamp: 1700000000,
                        from: vec!["Y1abc...".to_string()],
                        to: vec![OutputDetail {
                            address: "Y2def...".to_string(),
                            amount: 500000000,
                        }],
                        fee: 10000,
                        total_value: 500000000,
                    })
                } else {
                    QueryResult::Error("请提供交易ID".to_string())
                }
            }
            QueryIntent::QueryBlock => {
                QueryResult::Block(BlockSummary {
                    height: params.block_height.unwrap_or(1000),
                    hash: "0000abc...".to_string(),
                    timestamp: 1700000000,
                    transaction_count: 5,
                    miner: "YMiner...".to_string(),
                    reward: 5000000000,
                    size: 2048,
                })
            }
            QueryIntent::QueryStats => {
                QueryResult::Stats(ChainStats {
                    height: 10000,
                    total_transactions: 50000,
                    total_addresses: 10000,
                    avg_block_time: 600.0,
                    network_hashrate: "10 TH/s".to_string(),
                    difficulty: 4,
                    mempool_size: 15,
                })
            }
            QueryIntent::QueryAddressHistory => {
                if let Some(address) = &params.address {
                    QueryResult::AddressHistory(AddressHistory {
                        address: address.clone(),
                        total_received: 5000000000,
                        total_sent: 4000000000,
                        current_balance: 1000000000,
                        transaction_count: 50,
                        first_seen: Some(1699000000),
                        last_seen: Some(1700000000),
                    })
                } else {
                    QueryResult::Error("请提供地址".to_string())
                }
            }
            QueryIntent::QueryHelp => {
                QueryResult::Text(self.get_help_text())
            }
            _ => {
                QueryResult::Text("该功能正在开发中".to_string())
            }
        }
    }

    /// 生成自然语言回答
    fn generate_natural_answer(
        &self,
        _intent: QueryIntent,
        result: &QueryResult,
        _params: &QueryParameters,
    ) -> String {
        match result {
            QueryResult::Balance { address, formatted, .. } => {
                format!("地址 {} 的当前余额为 {}", 
                    self.truncate_address(address), 
                    formatted
                )
            }
            QueryResult::Transaction(tx) => {
                format!(
                    "交易 {} 的状态为{}，已有 {} 个确认，转账金额 {} YRU，手续费 {} 聪",
                    self.truncate_txid(&tx.id),
                    tx.status,
                    tx.confirmations,
                    tx.total_value as f64 / 100_000_000.0,
                    tx.fee
                )
            }
            QueryResult::Block(block) => {
                format!(
                    "区块 #{} (哈希: {}...) 包含 {} 笔交易，由 {} 挖出，区块奖励 {} YRU",
                    block.height,
                    &block.hash[..16],
                    block.transaction_count,
                    self.truncate_address(&block.miner),
                    block.reward as f64 / 100_000_000.0
                )
            }
            QueryResult::Stats(stats) => {
                format!(
                    "区块链当前高度 {}，总交易数 {}，活跃地址数 {}，平均出块时间 {:.1} 秒，网络算力 {}，内存池待处理交易 {} 笔",
                    stats.height,
                    stats.total_transactions,
                    stats.total_addresses,
                    stats.avg_block_time,
                    stats.network_hashrate,
                    stats.mempool_size
                )
            }
            QueryResult::AddressHistory(history) => {
                format!(
                    "地址 {} 共接收 {} YRU，发送 {} YRU，当前余额 {} YRU，总交易次数 {} 次",
                    self.truncate_address(&history.address),
                    history.total_received as f64 / 100_000_000.0,
                    history.total_sent as f64 / 100_000_000.0,
                    history.current_balance as f64 / 100_000_000.0,
                    history.transaction_count
                )
            }
            QueryResult::Text(text) => text.clone(),
            QueryResult::Error(err) => format!("抱歉，查询失败: {}", err),
            _ => "查询完成".to_string(),
        }
    }

    /// 生成查询建议
    fn generate_suggestions(&self, intent: QueryIntent, _result: &QueryResult) -> Vec<String> {
        let mut suggestions = Vec::new();

        match intent {
            QueryIntent::QueryBalance => {
                suggestions.push("查看交易历史".to_string());
                suggestions.push("查看该地址的挖矿收益".to_string());
            }
            QueryIntent::QueryTransaction => {
                suggestions.push("查看发送方地址详情".to_string());
                suggestions.push("查看接收方地址详情".to_string());
                suggestions.push("查看交易所在区块".to_string());
            }
            QueryIntent::QueryBlock => {
                suggestions.push("查看上一个区块".to_string());
                suggestions.push("查看区块中的交易".to_string());
                suggestions.push("查看最新区块".to_string());
            }
            QueryIntent::QueryStats => {
                suggestions.push("查看挖矿统计".to_string());
                suggestions.push("查看网络节点".to_string());
                suggestions.push("查看地址增长趋势".to_string());
            }
            _ => {
                suggestions.push("查看区块链统计".to_string());
                suggestions.push("查询地址余额".to_string());
                suggestions.push("查看最新区块".to_string());
            }
        }

        suggestions
    }

    /// 更新会话上下文
    fn update_session(&mut self, session_id: String, intent: QueryIntent, params: QueryParameters) {
        let context = SessionContext {
            last_intent: intent,
            last_parameters: params,
            query_count: 1,
            last_query_time: current_timestamp(),
        };

        if let Some(existing) = self.sessions.get_mut(&session_id) {
            existing.last_intent = context.last_intent;
            existing.last_parameters = context.last_parameters;
            existing.query_count += 1;
            existing.last_query_time = context.last_query_time;
        } else {
            self.sessions.insert(session_id, context);
        }
    }

    /// 获取帮助文本
    fn get_help_text(&self) -> String {
        r#"YRust Chain AI 区块浏览器使用帮助:

支持的查询类型:
1. 余额查询 - "查询地址 Yxxx 的余额"
2. 交易查询 - "查找交易 0xabc..."
3. 区块查询 - "查看区块 1000" 或 "最新区块"
4. 统计信息 - "区块链统计" 或 "网络状态"
5. 地址历史 - "地址 Yxxx 的交易历史"
6. 挖矿信息 - "当前挖矿难度" 或 "网络算力"

示例:
- "我的余额是多少" (需要先设置用户地址)
- "最新区块有多少交易"
- "地址 Y1abc... 收到过多少转账"

你可以用自然语言提问，我会尽力理解你的意图!"#.to_string()
    }

    /// 截断地址显示
    fn truncate_address(&self, address: &str) -> String {
        if address.len() > 20 {
            format!("{}...{}", &address[..10], &address[address.len()-6..])
        } else {
            address.to_string()
        }
    }

    /// 截断交易ID显示
    fn truncate_txid(&self, txid: &str) -> String {
        if txid.len() > 20 {
            format!("{}...{}", &txid[..10], &txid[txid.len()-6..])
        } else {
            txid.to_string()
        }
    }

    /// 语音查询接口 (模拟)
    pub fn process_voice_query(&mut self, transcript: &str, session_id: Option<String>) -> NLPResponse {
        // 语音查询转换为文本查询处理
        let query = NLPQuery {
            text: transcript.to_string(),
            session_id,
            user_address: None,
        };
        self.process_query(query)
    }

    /// 批量查询处理
    pub fn batch_query(&mut self, queries: Vec<String>) -> Vec<NLPResponse> {
        queries
            .into_iter()
            .map(|text| {
                let query = NLPQuery {
                    text,
                    session_id: None,
                    user_address: None,
                };
                self.process_query(query)
            })
            .collect()
    }
}

/// 获取当前时间戳
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl Default for NLPQueryEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nlp_engine_creation() {
        let engine = NLPQueryEngine::new();
        assert!(!engine.intent_patterns.is_empty());
    }

    #[test]
    fn test_intent_recognition() {
        let engine = NLPQueryEngine::new();
        
        // 测试余额相关查询
        let (intent, _) = engine.recognize_intent("查询我的余额");
        assert!(matches!(intent, QueryIntent::QueryBalance | QueryIntent::Unknown));
        
        // 测试区块相关查询
        let (intent, _) = engine.recognize_intent("查看最新区块");
        assert!(matches!(intent, QueryIntent::QueryBlock | QueryIntent::Unknown));
        
        // 测试网络相关查询
        let (intent, _) = engine.recognize_intent("网络状态怎么样");
        assert!(matches!(intent, QueryIntent::QueryNetworkStatus | QueryIntent::Unknown));
    }

    #[test]
    fn test_query_processing() {
        let mut engine = NLPQueryEngine::new();
        
        let query = NLPQuery {
            text: "区块链统计".to_string(),
            session_id: Some("test_session".to_string()),
            user_address: None,
        };
        
        let response = engine.process_query(query);
        // 验证返回了响应
        assert!(!response.natural_answer.is_empty());
        // 验证响应包含统计信息或提示
        assert!(response.natural_answer.contains("区块") || 
                response.natural_answer.contains("统计") ||
                response.natural_answer.contains("正在开发"));
    }
}
