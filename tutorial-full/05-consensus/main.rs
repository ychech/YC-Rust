//! 05. 共识算法 - Raft实现

use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::Duration;

/// 节点角色
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    Follower,
    Candidate,
    Leader,
}

/// Raft共识状态
pub struct RaftConsensus {
    pub node_id: String,
    pub role: Arc<RwLock<NodeRole>>,
    pub current_term: Arc<RwLock<u64>>,
    pub voted_for: Arc<RwLock<Option<String>>>,
    pub last_heartbeat: Arc<RwLock<u64>>,
}

impl RaftConsensus {
    pub fn new(node_id: String) -> Self {
        Self {
            node_id,
            role: Arc::new(RwLock::new(NodeRole::Follower)),
            current_term: Arc::new(RwLock::new(0)),
            voted_for: Arc::new(RwLock::new(None)),
            last_heartbeat: Arc::new(RwLock::new(current_timestamp())),
        }
    }
    
    /// 启动选举超时检测
    pub async fn start_election_timer(&self) {
        let role = self.role.clone();
        let last_heartbeat = self.last_heartbeat.clone();
        let current_term = self.current_term.clone();
        
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(5)).await;
                
                let last = *last_heartbeat.read().await;
                let now = current_timestamp();
                
                if now - last > 10 && *role.read().await == NodeRole::Follower {
                    println!("⏰ 选举超时，开始选举...");
                    
                    let mut r = role.write().await;
                    *r = NodeRole::Candidate;
                    
                    let mut t = current_term.write().await;
                    *t += 1;
                }
            }
        });
    }
    
    /// 处理心跳
    pub async fn handle_heartbeat(&self, from: &str, term: u64) {
        println!("💓 收到来自 {} 的心跳 (term: {})", from, term);
        
        let mut last = self.last_heartbeat.write().await;
        *last = current_timestamp();
        
        let current_term = *self.current_term.read().await;
        if term > current_term {
            let mut t = self.current_term.write().await;
            *t = term;
            
            let mut role = self.role.write().await;
            *role = NodeRole::Follower;
        }
    }
    
    /// 请求投票
    pub async fn request_vote(&self, candidate: &str, term: u64) -> bool {
        let current_term = *self.current_term.read().await;
        
        if term < current_term {
            return false;
        }
        
        let voted = self.voted_for.read().await;
        if voted.is_none() || voted.as_ref().unwrap() == candidate {
            println!("🗳️  投票给 {}", candidate);
            return true;
        }
        
        false
    }
}

fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[tokio::main]
async fn main() {
    println!("Raft共识演示");
    
    let consensus = RaftConsensus::new("node1".to_string());
    consensus.start_election_timer().await;
    
    // 模拟心跳
    consensus.handle_heartbeat("leader", 1).await;
    
    tokio::time::sleep(Duration::from_secs(3)).await;
}
