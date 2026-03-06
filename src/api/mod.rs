//! 区块浏览器 API 模块
//! 提供 RESTful API 供前端查询区块链数据

use crate::core::{Block, Blockchain, BlockchainStats, Transaction, Amount};
use crate::wallet::Wallet;
use crate::ai::{AIService, AIConfig};
use axum::{
    extract::{Path, State},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use log::{info, warn};

// AI 模块导入
use crate::ai::nlp::{NLPQuery, NLPResponse};
use crate::ai::mining::MiningAdvice;
use crate::ai::risk::RiskResult;
use crate::ai::contract::{ContractGenerationRequest, ContractGenerationResult};
use crate::ai::wallet::{VoiceTransferRequest, VoiceTransferResult};
use crate::ai::ops::SystemOverview;

/// API 共享状态
pub type ApiState = Arc<RwLock<Blockchain>>;

/// AI 服务状态 (使用 once_cell 或 lazy_static 在运行时初始化)
static AI_SERVICE: once_cell::sync::Lazy<std::sync::Mutex<AIService>> = 
    once_cell::sync::Lazy::new(|| {
        std::sync::Mutex::new(AIService::default())
    });

/// 标准 API 响应
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        ApiResponse {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    pub fn error(msg: String) -> Self {
        ApiResponse {
            success: false,
            data: None,
            error: Some(msg),
        }
    }
}

// ==================== AI API 处理函数 ====================

/// AI 自然语言查询
async fn ai_query(
    Json(query): Json<NLPQuery>,
) -> Json<ApiResponse<NLPResponse>> {
    let mut service = AI_SERVICE.lock().unwrap();
    let response = service.nlp.process_query(query);
    Json(ApiResponse::success(response))
}

/// AI 挖矿建议
async fn ai_mining_advice() -> Json<ApiResponse<MiningAdvice>> {
    let service = AI_SERVICE.lock().unwrap();
    let advice = service.mining.get_mining_advice();
    Json(ApiResponse::success(advice))
}

/// AI 风险检测请求
#[derive(Debug, Deserialize)]
struct RiskCheckRequest {
    tx_id: String,
}

/// AI 风险检测
async fn ai_risk_check(
    Json(_req): Json<RiskCheckRequest>,
) -> Json<ApiResponse<RiskResult>> {
    // 这里简化处理，实际需要查询交易
    let result = RiskResult {
        level: crate::ai::risk::RiskLevel::None,
        score: 0,
        risk_types: vec![],
        details: vec!["交易正常".to_string()],
        recommended_action: crate::ai::risk::RiskAction::Allow,
        timestamp: current_timestamp(),
        confidence: 95,
    };
    Json(ApiResponse::success(result))
}

/// AI 合约生成
async fn ai_contract_generate(
    Json(req): Json<ContractGenerationRequest>,
) -> Json<ApiResponse<ContractGenerationResult>> {
    let service = AI_SERVICE.lock().unwrap();
    match service.contract.generate_contract(req) {
        Ok(result) => Json(ApiResponse::success(result)),
        Err(e) => Json(ApiResponse::error(e.to_string())),
    }
}

/// AI 钱包语音指令
async fn ai_wallet_voice(
    Json(req): Json<VoiceTransferRequest>,
) -> Json<ApiResponse<VoiceTransferResult>> {
    let service = AI_SERVICE.lock().unwrap();
    let result = service.wallet.process_voice_transfer(req);
    Json(ApiResponse::success(result))
}

/// AI 运维状态
async fn ai_ops_status() -> Json<ApiResponse<SystemOverview>> {
    let service = AI_SERVICE.lock().unwrap();
    let overview = service.ops.get_system_overview();
    Json(ApiResponse::success(overview))
}

/// 获取当前时间戳
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// 区块列表响应
#[derive(Debug, Serialize)]
pub struct BlocksResponse {
    pub blocks: Vec<BlockSummary>,
    pub total: u64,
}

/// 区块摘要
#[derive(Debug, Serialize)]
pub struct BlockSummary {
    pub height: u64,
    pub hash: String,
    pub timestamp: i64,
    pub tx_count: usize,
    pub miner: String,
}

/// 交易详情
#[derive(Debug, Serialize)]
pub struct TransactionDetail {
    pub id: String,
    pub timestamp: i64,
    pub inputs: Vec<TxInputDetail>,
    pub outputs: Vec<TxOutputDetail>,
    pub block_height: Option<u64>,
    pub block_hash: Option<String>,
    pub is_coinbase: bool,
}

#[derive(Debug, Serialize)]
pub struct TxInputDetail {
    pub tx_id: String,
    pub vout: u32,
    pub address: String,
    pub value: Amount,
}

#[derive(Debug, Serialize)]
pub struct TxOutputDetail {
    pub index: u32,
    pub address: String,
    pub value: Amount,
}

/// 地址信息
#[derive(Debug, Serialize)]
pub struct AddressInfo {
    pub address: String,
    pub balance: Amount,
    pub tx_count: u64,
    pub received: Amount,
    pub sent: Amount,
}

/// 挖矿请求
#[derive(Debug, Deserialize)]
pub struct MineRequest {
    pub miner_address: String,
}

/// 转账请求
#[derive(Debug, Deserialize)]
pub struct TransferRequest {
    pub from_private_key: String,
    pub to_address: String,
    pub amount: Amount,
}

/// 创建钱包响应
#[derive(Debug, Serialize)]
pub struct WalletResponse {
    pub address: String,
    pub public_key: String,
    pub private_key: String,
}

// ==================== API 处理函数 ====================

/// 获取区块链统计信息
async fn get_stats(State(state): State<ApiState>) -> Json<ApiResponse<BlockchainStats>> {
    let blockchain = state.read().await;
    let stats = blockchain.get_stats();
    Json(ApiResponse::success(stats))
}

/// 获取最新区块
async fn get_latest_block(State(state): State<ApiState>) -> Json<ApiResponse<Block>> {
    let blockchain = state.read().await;
    let block = blockchain.get_last_block().clone();
    Json(ApiResponse::success(block))
}

/// 获取区块列表
async fn get_blocks(
    State(state): State<ApiState>,
) -> Json<ApiResponse<BlocksResponse>> {
    let blockchain = state.read().await;
    
    let blocks: Vec<BlockSummary> = blockchain.chain
        .iter()
        .rev()
        .take(50)
        .map(|b| BlockSummary {
            height: b.height,
            hash: b.hash.clone(),
            timestamp: b.header.timestamp,
            tx_count: b.transactions.len(),
            miner: b.transactions.first()
                .and_then(|tx| tx.outputs.first())
                .map(|o| o.script_pubkey.clone())
                .unwrap_or_default(),
        })
        .collect();

    let response = BlocksResponse {
        total: blockchain.get_height() + 1,
        blocks,
    };

    Json(ApiResponse::success(response))
}

/// 通过高度获取区块
async fn get_block_by_height(
    State(state): State<ApiState>,
    Path(height): Path<u64>,
) -> Json<ApiResponse<Block>> {
    let blockchain = state.read().await;
    
    match blockchain.get_block_by_height(height) {
        Some(block) => Json(ApiResponse::success(block.clone())),
        None => Json(ApiResponse::error(format!("区块 {} 不存在", height))),
    }
}

/// 通过哈希获取区块
async fn get_block_by_hash(
    State(state): State<ApiState>,
    Path(hash): Path<String>,
) -> Json<ApiResponse<Block>> {
    let blockchain = state.read().await;
    
    match blockchain.get_block_by_hash(&hash) {
        Some(block) => Json(ApiResponse::success(block.clone())),
        None => Json(ApiResponse::error(format!("区块 {} 不存在", hash))),
    }
}

/// 通过 ID 获取交易
async fn get_transaction(
    State(state): State<ApiState>,
    Path(tx_id): Path<String>,
) -> Json<ApiResponse<TransactionDetail>> {
    let blockchain = state.read().await;
    
    // 在链上查找
    for block in &blockchain.chain {
        for tx in &block.transactions {
            if tx.id == tx_id {
                let detail = transaction_to_detail(tx, Some(block.height), Some(&block.hash));
                return Json(ApiResponse::success(detail));
            }
        }
    }

    // 在内存池查找
    for tx in &blockchain.mempool {
        if tx.id == tx_id {
            let detail = transaction_to_detail(tx, None, None);
            return Json(ApiResponse::success(detail));
        }
    }

    Json(ApiResponse::error(format!("交易 {} 不存在", tx_id)))
}

/// 获取地址信息
async fn get_address_info(
    State(state): State<ApiState>,
    Path(address): Path<String>,
) -> Json<ApiResponse<AddressInfo>> {
    let blockchain = state.read().await;
    
    if !Wallet::validate_address(&address) {
        return Json(ApiResponse::error("无效的地址格式".to_string()));
    }

    let balance = blockchain.get_balance(&address);
    
    // 统计交易
    let mut tx_count = 0u64;
    let mut received = 0u64;
    let sent;

    for block in &blockchain.chain {
        for tx in &block.transactions {
            let mut is_related = false;
            
            // 检查输入
            for input in &tx.inputs {
                // 这里简化处理，实际需要解析输入地址
                if !input.script_sig.pubkey.is_empty() {
                    is_related = true;
                }
            }
            
            // 检查输出
            for output in &tx.outputs {
                if output.script_pubkey == address {
                    is_related = true;
                    received += output.value;
                }
            }
            
            if is_related {
                tx_count += 1;
            }
        }
    }

    sent = received.saturating_sub(balance);

    let info = AddressInfo {
        address,
        balance,
        tx_count,
        received,
        sent,
    };

    Json(ApiResponse::success(info))
}

/// 获取内存池
async fn get_mempool(State(state): State<ApiState>) -> Json<ApiResponse<Vec<Transaction>>> {
    let blockchain = state.read().await;
    Json(ApiResponse::success(blockchain.mempool.clone()))
}

/// 挖矿
async fn mine_block(
    State(state): State<ApiState>,
    Json(req): Json<MineRequest>,
) -> Json<ApiResponse<Block>> {
    if !Wallet::validate_address(&req.miner_address) {
        return Json(ApiResponse::error("无效的挖矿地址".to_string()));
    }

    let mut blockchain = state.write().await;
    
    match blockchain.mine_block(req.miner_address) {
        Ok(block) => {
            info!("✅ 通过 API 挖出新区块 #{}", block.height);
            Json(ApiResponse::success(block))
        }
        Err(e) => {
            warn!("❌ 挖矿失败: {}", e);
            Json(ApiResponse::error(format!("挖矿失败: {}", e)))
        }
    }
}

/// 创建转账交易
async fn create_transaction(
    State(state): State<ApiState>,
    Json(req): Json<TransferRequest>,
) -> Json<ApiResponse<Transaction>> {
    // 验证地址
    if !Wallet::validate_address(&req.to_address) {
        return Json(ApiResponse::error("无效的目标地址".to_string()));
    }

    // 恢复钱包
    let wallet = match Wallet::from_private_key(&req.from_private_key) {
        Ok(w) => w,
        Err(_) => return Json(ApiResponse::error("无效的私钥".to_string())),
    };

    let mut blockchain = state.write().await;
    
    // 检查余额
    let balance = blockchain.get_balance(&wallet.address);
    if balance < req.amount {
        return Json(ApiResponse::error(
            format!("余额不足: 拥有 {}, 需要 {}", balance, req.amount)
        ));
    }

    // 创建交易
    match blockchain.create_transaction(&wallet, req.to_address, req.amount) {
        Ok(tx) => {
            // 添加到内存池
            if let Err(e) = blockchain.add_to_mempool(tx.clone()) {
                return Json(ApiResponse::error(format!("添加交易到内存池失败: {}", e)));
            }
            
            info!("✅ 创建新交易: {}", &tx.id[..16]);
            Json(ApiResponse::success(tx))
        }
        Err(e) => {
            Json(ApiResponse::error(format!("创建交易失败: {}", e)))
        }
    }
}

/// 创建新钱包
async fn create_wallet() -> Json<ApiResponse<WalletResponse>> {
    let wallet = Wallet::new();
    
    let response = WalletResponse {
        address: wallet.address,
        public_key: wallet.public_key,
        private_key: wallet.private_key,
    };

    Json(ApiResponse::success(response))
}

/// 验证区块链
async fn validate_chain(State(state): State<ApiState>) -> Json<ApiResponse<bool>> {
    let blockchain = state.read().await;
    let valid = blockchain.is_valid();
    Json(ApiResponse::success(valid))
}

/// 获取 UTXO 数量
async fn get_utxo_count(State(state): State<ApiState>) -> Json<ApiResponse<usize>> {
    let blockchain = state.read().await;
    Json(ApiResponse::success(blockchain.get_utxo_count()))
}

/// 辅助函数：转换交易为详情
fn transaction_to_detail(
    tx: &Transaction,
    block_height: Option<u64>,
    block_hash: Option<&str>,
) -> TransactionDetail {
    TransactionDetail {
        id: tx.id.clone(),
        timestamp: tx.timestamp,
        inputs: tx.inputs.iter().map(|i| TxInputDetail {
            tx_id: i.tx_id.clone(),
            vout: i.vout,
            address: "unknown".to_string(), // 需要查找原始交易
            value: 0, // 需要查找原始交易
        }).collect(),
        outputs: tx.outputs.iter().enumerate().map(|(i, o)| TxOutputDetail {
            index: i as u32,
            address: o.script_pubkey.clone(),
            value: o.value,
        }).collect(),
        block_height,
        block_hash: block_hash.map(|s| s.to_string()),
        is_coinbase: tx.is_coinbase(),
    }
}

/// 创建 API 路由
pub fn create_routes(state: ApiState) -> Router {
    Router::new()
        // 区块链信息
        .route("/api/stats", get(get_stats))
        .route("/api/blocks/latest", get(get_latest_block))
        .route("/api/blocks", get(get_blocks))
        .route("/api/blocks/height/:height", get(get_block_by_height))
        .route("/api/blocks/hash/:hash", get(get_block_by_hash))
        
        // 交易
        .route("/api/tx/:tx_id", get(get_transaction))
        .route("/api/mempool", get(get_mempool))
        .route("/api/tx/create", post(create_transaction))
        
        // 地址
        .route("/api/address/:address", get(get_address_info))
        
        // 挖矿
        .route("/api/mine", post(mine_block))
        
        // 钱包
        .route("/api/wallet/create", post(create_wallet))
        
        // 验证
        .route("/api/validate", get(validate_chain))
        .route("/api/utxos/count", get(get_utxo_count))
        
        // AI 功能
        .route("/api/ai/query", post(ai_query))
        .route("/api/ai/mining/advice", get(ai_mining_advice))
        .route("/api/ai/risk/check", post(ai_risk_check))
        .route("/api/ai/contract/generate", post(ai_contract_generate))
        .route("/api/ai/wallet/voice", post(ai_wallet_voice))
        .route("/api/ai/ops/status", get(ai_ops_status))
        
        // 添加 CORS 支持
        .layer(CorsLayer::permissive())
        .with_state(state)
}

/// 启动 API 服务器
pub async fn start_api_server(
    state: ApiState,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_routes(state);
    
    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    info!("🚀 API 服务器启动: http://{}", addr);
    
    axum::serve(listener, app).await?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_api_response() {
        let response: ApiResponse<String> = ApiResponse::success("test".to_string());
        assert!(response.success);
        assert_eq!(response.data, Some("test".to_string()));
    }
}
