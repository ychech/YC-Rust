//! P2P 网络模块 (简化版)
//! 提供基本的节点通信功能

use crate::core::{Block, Transaction};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use log::{info, warn, debug};

/// 网络消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// 握手
    Handshake { version: String, node_id: String },
    /// 握手响应
    HandshakeAck { height: u64, best_hash: String },
    /// 新区块
    NewBlock(Block),
    /// 新交易
    NewTransaction(Transaction),
    /// 请求区块
    GetBlocks { start_height: u64, end_height: u64 },
    /// 响应区块
    BlocksResponse(Vec<Block>),
    /// 请求最新区块哈希
    GetBestBlockHash,
    /// 响应最新区块哈希
    BestBlockHashResponse(String),
    /// 节点地址列表
    PeerList(Vec<String>),
    /// 心跳
    Ping,
    /// 心跳响应
    Pong,
}

/// 网络节点
pub struct P2PNode {
    /// 节点 ID
    pub node_id: String,
    /// 监听地址
    pub listen_addr: SocketAddr,
    /// 已连接节点
    pub connected_peers: HashSet<String>,
    /// 消息接收通道
    pub msg_receiver: mpsc::UnboundedReceiver<NetworkMessage>,
    /// 消息发送通道
    pub msg_sender: mpsc::UnboundedSender<NetworkMessage>,
}

impl P2PNode {
    /// 创建新节点
    pub fn new(port: u16) -> Result<Self, NetworkError> {
        let node_id = format!("node_{}", uuid::Uuid::new_v4());
        let listen_addr = format!("0.0.0.0:{}", port).parse().unwrap();
        
        let (msg_sender, msg_receiver) = mpsc::unbounded_channel();

        Ok(P2PNode {
            node_id,
            listen_addr,
            connected_peers: HashSet::new(),
            msg_receiver,
            msg_sender,
        })
    }

    /// 连接到指定节点
    pub async fn connect(&mut self, addr: &str) -> Result<(), NetworkError> {
        let _stream = TcpStream::connect(addr).await
            .map_err(|e| NetworkError::ConnectionError(e.to_string()))?;
        
        self.connected_peers.insert(addr.to_string());
        info!("✅ 连接到节点: {}", addr);
        
        // 发送握手
        let handshake = NetworkMessage::Handshake {
            version: crate::version().to_string(),
            node_id: self.node_id.clone(),
        };
        
        self.send_message_to_peer(addr, handshake).await?;
        
        Ok(())
    }

    /// 发送消息到指定节点
    async fn send_message_to_peer(
        &self,
        addr: &str,
        msg: NetworkMessage,
    ) -> Result<(), NetworkError> {
        let data = serde_json::to_vec(&msg)
            .map_err(|e| NetworkError::SerializationError(e.to_string()))?;
        
        let mut stream = TcpStream::connect(addr).await
            .map_err(|e| NetworkError::ConnectionError(e.to_string()))?;
        
        // 发送数据长度 + 数据
        let len = data.len() as u32;
        stream.write_all(&len.to_be_bytes()).await
            .map_err(|e| NetworkError::ConnectionError(e.to_string()))?;
        stream.write_all(&data).await
            .map_err(|e| NetworkError::ConnectionError(e.to_string()))?;
        
        Ok(())
    }

    /// 广播新区块
    pub async fn broadcast_block(&self, block: Block) -> Result<(), NetworkError> {
        let msg = NetworkMessage::NewBlock(block);
        
        for peer in &self.connected_peers {
            if let Err(e) = self.send_message_to_peer(peer, msg.clone()).await {
                warn!("发送区块到 {} 失败: {}", peer, e);
            }
        }
        
        debug!("📢 广播新区块到 {} 个节点", self.connected_peers.len());
        Ok(())
    }

    /// 广播新交易
    pub async fn broadcast_transaction(&self, tx: Transaction) -> Result<(), NetworkError> {
        let msg = NetworkMessage::NewTransaction(tx);
        
        for peer in &self.connected_peers {
            if let Err(e) = self.send_message_to_peer(peer, msg.clone()).await {
                warn!("发送交易到 {} 失败: {}", peer, e);
            }
        }
        
        debug!("📢 广播新交易到 {} 个节点", self.connected_peers.len());
        Ok(())
    }

    /// 请求区块
    pub async fn request_blocks(
        &self,
        addr: &str,
        start_height: u64,
        end_height: u64,
    ) -> Result<(), NetworkError> {
        let msg = NetworkMessage::GetBlocks { start_height, end_height };
        self.send_message_to_peer(addr, msg).await?;
        info!("📡 向 {} 请求区块 {}-{}", addr, start_height, end_height);
        Ok(())
    }

    /// 获取已连接节点数
    pub fn peer_count(&self) -> usize {
        self.connected_peers.len()
    }

    /// 运行网络服务
    pub async fn run(&mut self) -> Result<(), NetworkError> {
        let listener = TcpListener::bind(self.listen_addr).await
            .map_err(|e| NetworkError::ConnectionError(e.to_string()))?;
        
        info!("📡 P2P 节点监听: {}", self.listen_addr);

        loop {
            tokio::select! {
                // 接受新连接
                Ok((stream, addr)) = listener.accept() => {
                    info!("📡 新连接: {}", addr);
                    self.handle_connection(stream).await;
                }
            }
        }
    }

    /// 处理连接
    async fn handle_connection(&self, mut stream: TcpStream) {
        let mut buf = [0u8; 4];
        
        loop {
            // 读取消息长度
            match stream.read_exact(&mut buf).await {
                Ok(_) => {
                    let len = u32::from_be_bytes(buf) as usize;
                    let mut data = vec![0u8; len];
                    
                    // 读取消息数据
                    match stream.read_exact(&mut data).await {
                        Ok(_) => {
                            if let Ok(msg) = serde_json::from_slice::<NetworkMessage>(&data) {
                                self.handle_message(msg).await;
                            }
                        }
                        Err(e) => {
                            warn!("读取消息失败: {}", e);
                            break;
                        }
                    }
                }
                Err(_) => break,
            }
        }
    }

    /// 处理网络消息
    async fn handle_message(&self, msg: NetworkMessage) {
        match &msg {
            NetworkMessage::Handshake { version, node_id } => {
                info!("🤝 收到握手: 节点 {} (版本 {})", node_id, version);
            }
            NetworkMessage::NewBlock(block) => {
                info!("📦 收到新区块 #{} from network", block.height);
            }
            NetworkMessage::NewTransaction(tx) => {
                debug!("💸 收到新交易 {} from network", &tx.id[..16]);
            }
            NetworkMessage::GetBlocks { start_height, end_height } => {
                info!("📡 收到区块请求: {}-{}", start_height, end_height);
            }
            NetworkMessage::BlocksResponse(blocks) => {
                info!("📦 收到 {} 个区块", blocks.len());
            }
            NetworkMessage::Ping => {
                debug!("💓 Ping");
            }
            NetworkMessage::Pong => {
                debug!("💓 Pong");
            }
            _ => {}
        }

        // 转发到本地通道
        let _ = self.msg_sender.send(msg);
    }

    /// 发送消息到本地通道
    pub fn send_message(&self, msg: NetworkMessage) {
        let _ = self.msg_sender.send(msg);
    }
}

/// 网络管理器
pub struct NetworkManager {
    pub node: P2PNode,
}

impl NetworkManager {
    pub async fn new(port: u16) -> Result<Self, NetworkError> {
        let node = P2PNode::new(port)?;
        Ok(NetworkManager { node })
    }

    pub async fn run(mut self) -> Result<(), NetworkError> {
        self.node.run().await
    }
}

/// 节点配置
#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub port: u16,
    pub bootstrap_nodes: Vec<String>,
}

impl Default for NodeConfig {
    fn default() -> Self {
        NodeConfig {
            port: 0,
            bootstrap_nodes: vec![],
        }
    }
}

/// 网络错误类型
#[derive(Debug, Clone)]
pub enum NetworkError {
    ConnectionError(String),
    SerializationError(String),
    InvalidMessage,
}

impl std::fmt::Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkError::ConnectionError(e) => write!(f, "连接错误: {}", e),
            NetworkError::SerializationError(e) => write!(f, "序列化错误: {}", e),
            NetworkError::InvalidMessage => write!(f, "无效的消息"),
        }
    }
}

impl std::error::Error for NetworkError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation() {
        let node = P2PNode::new(0);
        assert!(node.is_ok());
    }

    #[test]
    fn test_message_serialization() {
        let msg = NetworkMessage::Ping;
        let data = serde_json::to_vec(&msg).unwrap();
        let decoded: NetworkMessage = serde_json::from_slice(&data).unwrap();
        
        match decoded {
            NetworkMessage::Ping => {}
            _ => panic!("消息反序列化失败"),
        }
    }
}
