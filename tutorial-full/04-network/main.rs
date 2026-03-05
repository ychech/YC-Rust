//! 04. P2P网络 - libp2p实现

use std::sync::Arc;
use tokio::sync::mpsc;

/// 网络消息类型
#[derive(Debug, Clone)]
pub enum NetworkMessage {
    NewBlock(Block),
    NewTransaction(Transaction),
    GetBlocks(u64),      // 从指定高度获取区块
    Blocks(Vec<Block>),  // 返回区块列表
    Ping,
    Pong,
}

/// P2P节点
pub struct P2PNode {
    pub node_id: String,
    pub listen_addr: String,
    pub peers: Vec<String>,
}

impl P2PNode {
    pub fn new(port: u16) -> Self {
        Self {
            node_id: format!("node_{}", uuid::Uuid::new_v4()),
            listen_addr: format!("/ip4/0.0.0.0/tcp/{}", port),
            peers: Vec::new(),
        }
    }
    
    /// 广播消息到所有节点
    pub fn broadcast(&self, msg: NetworkMessage) {
        println!("📢 广播消息: {:?}", msg);
        // 实际实现使用libp2p的floodsub或gossipsub
    }
    
    /// 连接到指定节点
    pub async fn connect(&mut self, addr: &str) -> Result<(), String> {
        println!("🔗 连接到: {}", addr);
        self.peers.push(addr.to_string());
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Block {
    pub height: u64,
    pub hash: String,
}

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub from: String,
    pub to: String,
    pub amount: u64,
}

#[tokio::main]
async fn main() {
    println!("P2P网络演示");
    
    let mut node1 = P2PNode::new(8001);
    let mut node2 = P2PNode::new(8002);
    
    // 节点2连接到节点1
    node2.connect(&node1.listen_addr).await.unwrap();
    
    // 广播新区块
    let block = Block {
        height: 1,
        hash: "abc123".to_string(),
    };
    node1.broadcast(NetworkMessage::NewBlock(block));
}
