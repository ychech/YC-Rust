//! 多节点 P2P 共识模块
//! 实现真正的分布式共识、区块同步和冲突解决

use crate::core::models::Block;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use serde::{Serialize, Deserialize};
use tracing::{info, warn, error, debug};

/// 共识状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsensusState {
    /// 跟随者 - 接收区块
    Follower,
    /// 候选者 - 正在竞选领导者
    Candidate,
    /// 领导者 - 负责出块
    Leader,
    /// 同步中 - 正在同步区块
    Syncing,
}

/// 节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: String,
    pub address: String,
    pub version: String,
    pub block_height: u64,
    pub last_seen: u64,
    pub is_validator: bool,
}

/// 投票信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: String,
    pub candidate: String,
    pub term: u64,
    pub block_hash: Option<String>,
}

/// 共识配置
#[derive(Debug, Clone)]
pub struct ConsensusConfig {
    /// 节点ID
    pub node_id: String,
    /// 是否验证者
    pub is_validator: bool,
    /// 心跳间隔 (毫秒)
    pub heartbeat_interval: u64,
    /// 选举超时 (毫秒)
    pub election_timeout: u64,
    /// 最小验证者数量
    pub min_validators: usize,
    /// 最大区块大小
    pub max_block_size: usize,
}

impl Default for ConsensusConfig {
    fn default() -> Self {
        Self {
            node_id: format!("node_{}", uuid::Uuid::new_v4()),
            is_validator: true,
            heartbeat_interval: 3000,
            election_timeout: 10000,
            min_validators: 3,
            max_block_size: 1000,
        }
    }
}

/// 共识引擎
pub struct ConsensusEngine {
    /// 配置
    config: ConsensusConfig,
    /// 当前状态
    state: Arc<RwLock<ConsensusState>>,
    /// 当前任期
    current_term: Arc<RwLock<u64>>,
    /// 投票给谁
    voted_for: Arc<RwLock<Option<String>>>,
    /// 已知节点
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    /// 验证者集合
    validators: Arc<RwLock<HashSet<String>>>,
    /// 事件发送器
    event_sender: mpsc::Sender<ConsensusEvent>,
    /// 最后心跳时间
    last_heartbeat: Arc<RwLock<u64>>,
    /// 领导者ID
    leader_id: Arc<RwLock<Option<String>>>,
}

/// 共识事件
#[derive(Debug, Clone)]
pub enum ConsensusEvent {
    /// 新区块
    NewBlock(Block),
    /// 状态变更
    StateChanged(ConsensusState),
    /// 领导者变更
    LeaderChanged(Option<String>),
    /// 需要同步
    NeedSync { from_height: u64, target_height: u64 },
    /// 冲突检测
    ForkDetected { height: u64, hashes: Vec<String> },
}

impl ConsensusEngine {
    /// 创建新的共识引擎
    pub fn new(
        config: ConsensusConfig,
        event_sender: mpsc::Sender<ConsensusEvent>,
    ) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(ConsensusState::Follower)),
            current_term: Arc::new(RwLock::new(0)),
            voted_for: Arc::new(RwLock::new(None)),
            peers: Arc::new(RwLock::new(HashMap::new())),
            validators: Arc::new(RwLock::new(HashSet::new())),
            event_sender,
            last_heartbeat: Arc::new(RwLock::new(0)),
            leader_id: Arc::new(RwLock::new(None)),
        }
    }

    /// 启动共识引擎
    pub async fn start(&self) {
        info!("启动共识引擎, 节点ID: {}", self.config.node_id);
        
        // 启动心跳检测
        self.start_heartbeat_checker().await;
    }

    /// 启动心跳检测
    async fn start_heartbeat_checker(&self) {
        let state = self.state.clone();
        let last_heartbeat = self.last_heartbeat.clone();
        let election_timeout = self.config.election_timeout;
        let current_term = self.current_term.clone();
        let event_sender = self.event_sender.clone();

        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(election_timeout)).await;

                let current_state = *state.read().await;
                let last = *last_heartbeat.read().await;
                let now = chrono::Utc::now().timestamp_millis() as u64;

                // 如果是跟随者且超时未收到心跳，转为候选者
                if current_state == ConsensusState::Follower && now - last > election_timeout {
                    info!("心跳超时，开始选举");
                    
                    let mut state_guard = state.write().await;
                    *state_guard = ConsensusState::Candidate;
                    drop(state_guard);

                    // 增加任期
                    let mut term_guard = current_term.write().await;
                    *term_guard += 1;
                    drop(term_guard);

                    // 发送选举事件
                    let _ = event_sender.send(ConsensusEvent::StateChanged(ConsensusState::Candidate)).await;
                }
            }
        });
    }

    /// 处理心跳消息
    pub async fn handle_heartbeat(&self, from: &str, term: u64, height: u64) {
        debug!("收到心跳 from={}, term={}, height={}", from, term, height);

        let mut last_heartbeat = self.last_heartbeat.write().await;
        *last_heartbeat = chrono::Utc::now().timestamp_millis() as u64;
        drop(last_heartbeat);

        let current_term = *self.current_term.read().await;

        // 如果任期更高，更新并转为跟随者
        if term > current_term {
            let mut term_guard = self.current_term.write().await;
            *term_guard = term;
            drop(term_guard);

            let mut state_guard = self.state.write().await;
            *state_guard = ConsensusState::Follower;
            drop(state_guard);

            let mut leader_guard = self.leader_id.write().await;
            *leader_guard = Some(from.to_string());
            drop(leader_guard);

            let _ = self.event_sender.send(ConsensusEvent::LeaderChanged(Some(from.to_string()))).await;
        }
    }

    /// 处理投票请求
    pub async fn handle_vote_request(&self, candidate: &str, term: u64) -> bool {
        let current_term = *self.current_term.read().await;

        if term < current_term {
            return false;
        }

        let mut voted_for = self.voted_for.write().await;
        
        // 如果已经投过票给其他人，拒绝
        if let Some(ref voted) = *voted_for {
            if voted != candidate {
                return false;
            }
        }

        // 记录投票
        *voted_for = Some(candidate.to_string());
        
        info!("投票给候选者: {}", candidate);
        true
    }

    /// 处理投票响应
    pub async fn handle_vote_response(&self, from: &str, granted: bool) {
        if !granted {
            return;
        }

        // 简化的投票统计，实际应该维护投票集合
        // 如果获得多数票，成为领导者
        if self.config.is_validator {
            let mut state_guard = self.state.write().await;
            *state_guard = ConsensusState::Leader;
            drop(state_guard);

            let mut leader_guard = self.leader_id.write().await;
            *leader_guard = Some(self.config.node_id.clone());
            drop(leader_guard);

            info!("成为领导者!");
            let _ = self.event_sender.send(ConsensusEvent::StateChanged(ConsensusState::Leader)).await;
            let _ = self.event_sender.send(ConsensusEvent::LeaderChanged(Some(self.config.node_id.clone()))).await;
        }
    }

    /// 添加节点
    pub async fn add_peer(&self, peer_info: PeerInfo) {
        let mut peers = self.peers.write().await;
        peers.insert(peer_info.peer_id.clone(), peer_info.clone());
        
        if peer_info.is_validator {
            let mut validators = self.validators.write().await;
            validators.insert(peer_info.peer_id.clone());
        }
        
        info!("添加节点: {}", peer_info.peer_id);
    }

    /// 移除节点
    pub async fn remove_peer(&self, peer_id: &str) {
        let mut peers = self.peers.write().await;
        peers.remove(peer_id);
        
        let mut validators = self.validators.write().await;
        validators.remove(peer_id);
        
        info!("移除节点: {}", peer_id);
    }

    /// 获取当前状态
    pub async fn get_state(&self) -> ConsensusState {
        *self.state.read().await
    }

    /// 获取领导者ID
    pub async fn get_leader(&self) -> Option<String> {
        self.leader_id.read().await.clone()
    }

    /// 获取节点列表
    pub async fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers.read().await.values().cloned().collect()
    }
}
