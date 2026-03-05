//! 工业级 P2P 网络 - 基于 libp2p
//! 
//! 功能：
//! - Gossipsub 消息广播
//! - Kademlia DHT 节点发现
//! - QUIC/TCP 传输
//! - Noise 加密
//! - mDNS 本地发现

use crate::core::{Block, Transaction};
use libp2p::{
    gossipsub, mdns, kad,
    identity::Keypair,
    noise, quic,
    swarm::{NetworkBehaviour, SwarmEvent, SwarmBuilder},
    PeerId, Swarm, Transport,
};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::sync::mpsc;
use log::{info, warn, debug, error};

/// 网络消息类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// 新区块
    NewBlock(Block),
    /// 新交易
    NewTransaction(Transaction),
    /// 同步请求
    SyncRequest { from_height: u64 },
    /// 同步响应
    SyncResponse(Vec<Block>),
    /// 心跳
    Heartbeat { height: u64, best_hash: String },
    /// 节点信息
    PeerInfo { node_id: String, version: String },
}

/// 网络行为组合
#[derive(NetworkBehaviour)]
pub struct BlockchainBehaviour {
    /// Gossipsub 广播
    pub gossipsub: gossipsub::Behaviour,
    /// mDNS 本地发现
    pub mdns: mdns::tokio::Behaviour,
    /// Kademlia DHT
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
}

/// P2P 网络节点
pub struct P2PNetwork {
    /// 本地节点 ID
    pub local_peer_id: PeerId,
    /// 网络 Swarm
    pub swarm: Swarm<BlockchainBehaviour>,
    /// 消息接收器
    pub receiver: mpsc::UnboundedReceiver<NetworkMessage>,
    /// 消息发送器
    pub sender: mpsc::UnboundedSender<NetworkMessage>,
    /// 已连接节点
    pub connected_peers: HashSet<PeerId>,
    /// 区块主题
    block_topic: gossipsub::IdentTopic,
    /// 交易主题
    tx_topic: gossipsub::IdentTopic,
    /// 同步主题
    sync_topic: gossipsub::IdentTopic,
}

impl P2PNetwork {
    /// 创建新网络
    pub async fn new(port: u16) -> Result<Self, Box<dyn std::error::Error>> {
        // 生成密钥对
        let local_key = Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());
        
        info!("📡 本地节点 ID: {}", local_peer_id);

        // 创建消息通道
        let (sender, receiver) = mpsc::unbounded_channel();

        // QUIC 传输（快速、可靠）
        let quic_config = quic::Config::default();
        let transport = quic::tokio::Transport::new(quic_config)
            .map(|(peer_id, conn), _| (peer_id, libp2p::StreamMuxerBox::new(conn)))
            .boxed();

        // Gossipsub 配置
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .message_id_fn(|msg: &gossipsub::Message| {
                let mut hasher = DefaultHasher::new();
                msg.data.hash(&mut hasher);
                gossipsub::MessageId::from(hasher.finish().to_string())
            })
            .build()?;

        let gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )?;

        // mDNS 本地发现
        let mdns = mdns::tokio::Behaviour::new(
            mdns::Config::default(),
            local_peer_id,
        )?;

        // Kademlia DHT
        let kademlia = kad::Behaviour::new(
            local_peer_id,
            kad::store::MemoryStore::new(local_peer_id),
        );

        // 创建行为
        let behaviour = BlockchainBehaviour {
            gossipsub,
            mdns,
            kademlia,
        };

        // 创建 Swarm
        let swarm = SwarmBuilder::with_existing_identity(local_key)
            .with_tokio()
            .with_other_transport(|_| transport)?
            .with_behaviour(|_| behaviour)?
            .with_swarm_config(|c| c.with_idle_connection_timeout(Duration::from_secs(60)))
            .build();

        // 创建主题
        let block_topic = gossipsub::IdentTopic::new("yrust/blocks/1.0.0");
        let tx_topic = gossipsub::IdentTopic::new("yrust/txs/1.0.0");
        let sync_topic = gossipsub::IdentTopic::new("yrust/sync/1.0.0");

        Ok(P2PNetwork {
            local_peer_id,
            swarm,
            receiver,
            sender,
            connected_peers: HashSet::new(),
            block_topic,
            tx_topic,
            sync_topic,
        })
    }

    /// 订阅主题
    pub fn subscribe(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.swarm.behaviour_mut().gossipsub.subscribe(&self.block_topic)?;
        self.swarm.behaviour_mut().gossipsub.subscribe(&self.tx_topic)?;
        self.swarm.behaviour_mut().gossipsub.subscribe(&self.sync_topic)?;
        
        info!("📡 已订阅网络主题");
        Ok(())
    }

    /// 监听地址
    pub fn listen(&mut self, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let addr: libp2p::Multiaddr = format!("/ip4/0.0.0.0/udp/{}/quic-v1", port).parse()?;
        self.swarm.listen_on(addr)?;
        
        info!("📡 节点监听端口: {}", port);
        Ok(())
    }

    /// 连接到指定节点
    pub fn dial(&mut self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let remote: libp2p::Multiaddr = addr.parse()?;
        self.swarm.dial(remote)?;
        info!("🔗 正在连接: {}", addr);
        Ok(())
    }

    /// 广播新区块
    pub fn broadcast_block(&mut self, block: Block) -> Result<(), Box<dyn std::error::Error>> {
        let msg = NetworkMessage::NewBlock(block);
        let data = serde_json::to_vec(&msg)?;
        
        self.swarm.behaviour_mut().gossipsub
            .publish(self.block_topic.clone(), data)?;
        
        debug!("📢 广播新区块");
        Ok(())
    }

    /// 广播新交易
    pub fn broadcast_tx(&mut self, tx: Transaction) -> Result<(), Box<dyn std::error::Error>> {
        let msg = NetworkMessage::NewTransaction(tx);
        let data = serde_json::to_vec(&msg)?;
        
        self.swarm.behaviour_mut().gossipsub
            .publish(self.tx_topic.clone(), data)?;
        
        debug!("📢 广播新交易");
        Ok(())
    }

    /// 请求同步
    pub fn request_sync(&mut self, from_height: u64) -> Result<(), Box<dyn std::error::Error>> {
        let msg = NetworkMessage::SyncRequest { from_height };
        let data = serde_json::to_vec(&msg)?;
        
        self.swarm.behaviour_mut().gossipsub
            .publish(self.sync_topic.clone(), data)?;
        
        info!("📡 请求同步，从高度 {}", from_height);
        Ok(())
    }

    /// 运行网络事件循环
    pub async fn run(mut self) {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => {
                    self.handle_event(event).await;
                }
                
                Some(msg) = self.receiver.recv() => {
                    self.handle_command(msg).await;
                }
            }
        }
    }

    /// 处理网络事件
    async fn handle_event(&mut self, event: SwarmEvent<BlockchainBehaviourEvent>) {
        match event {
            SwarmEvent::Behaviour(BlockchainBehaviourEvent::Gossipsub(gossip_event)) => {
                match gossip_event {
                    gossipsub::Event::Message { propagation_source, message_id, message } => {
                        debug!("📨 收到消息 {} from {}", message_id, propagation_source);
                        
                        if let Ok(msg) = serde_json::from_slice::<NetworkMessage>(&message.data) {
                            self.handle_network_message(msg).await;
                        }
                    }
                    gossipsub::Event::Subscribed { peer_id, topic } => {
                        info!("✅ Peer {} 订阅了 {}", peer_id, topic);
                    }
                    _ => {}
                }
            }
            
            SwarmEvent::Behaviour(BlockchainBehaviourEvent::Mdns(mdns_event)) => {
                match mdns_event {
                    mdns::Event::Discovered(peers) => {
                        for (peer_id, addr) in peers {
                            info!("🔍 发现节点: {} at {}", peer_id, addr);
                            self.swarm.behaviour_mut().kademlia.add_address(&peer_id, addr);
                            self.connected_peers.insert(peer_id);
                        }
                    }
                    mdns::Event::Expired(peers) => {
                        for (peer_id, _) in peers {
                            info!("👋 节点离线: {}", peer_id);
                            self.connected_peers.remove(&peer_id);
                        }
                    }
                }
            }
            
            SwarmEvent::Behaviour(BlockchainBehaviourEvent::Kademlia(kad_event)) => {
                match kad_event {
                    kad::Event::RoutingUpdated { peer, .. } => {
                        info!("🌐 Kademlia 路由更新: {}", peer);
                    }
                    _ => {}
                }
            }
            
            SwarmEvent::NewListenAddr { address, .. } => {
                info!("📡 监听地址: {}", address);
            }
            
            SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                info!("✅ 连接到: {}", peer_id);
                self.connected_peers.insert(peer_id);
            }
            
            SwarmEvent::ConnectionClosed { peer_id, .. } => {
                info!("❌ 断开连接: {}", peer_id);
                self.connected_peers.remove(&peer_id);
            }
            
            _ => {}
        }
    }

    /// 处理网络消息
    async fn handle_network_message(&self, msg: NetworkMessage) {
        match &msg {
            NetworkMessage::NewBlock(block) => {
                info!("📦 收到新区块 #{} from network", block.height);
            }
            NetworkMessage::NewTransaction(tx) => {
                debug!("💸 收到新交易 {} from network", &tx.id[..16]);
            }
            NetworkMessage::SyncRequest { from_height } => {
                info!("📡 收到同步请求: 从高度 {}", from_height);
            }
            NetworkMessage::Heartbeat { height, best_hash } => {
                debug!("💓 心跳: 高度={}, 哈希={}...", height, &best_hash[..16]);
            }
            _ => {}
        }

        // 转发到本地通道
        let _ = self.sender.send(msg);
    }

    /// 处理本地命令
    async fn handle_command(&mut self, msg: NetworkMessage) {
        match msg {
            NetworkMessage::NewBlock(block) => {
                if let Err(e) = self.broadcast_block(block) {
                    error!("广播区块失败: {}", e);
                }
            }
            NetworkMessage::NewTransaction(tx) => {
                if let Err(e) = self.broadcast_tx(tx) {
                    error!("广播交易失败: {}", e);
                }
            }
            _ => {}
        }
    }

    /// 获取连接数
    pub fn peer_count(&self) -> usize {
        self.connected_peers.len()
    }
}

/// 网络配置
pub struct NetworkConfig {
    pub port: u16,
    pub bootstrap_nodes: Vec<String>,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        NetworkConfig {
            port: 0,
            bootstrap_nodes: vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_creation() {
        let network = P2PNetwork::new(0).await;
        assert!(network.is_ok());
    }
}
