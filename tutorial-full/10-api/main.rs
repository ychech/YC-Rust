//! 10. API与部署 - REST API实现

use std::collections::HashMap;

/// 模拟HTTP请求
#[derive(Debug)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub body: Option<String>,
}

/// 模拟HTTP响应
#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub body: String,
}

impl Response {
    pub fn ok(body: String) -> Self {
        Self { status: 200, body }
    }
    
    pub fn not_found() -> Self {
        Self { status: 404, body: "Not Found".to_string() }
    }
    
    pub fn error(msg: String) -> Self {
        Self { status: 500, body: msg }
    }
}

/// API服务器
pub struct ApiServer {
    blockchain: MockBlockchain,
}

impl ApiServer {
    pub fn new() -> Self {
        Self {
            blockchain: MockBlockchain::new(),
        }
    }
    
    /// 处理请求
    pub fn handle(&self, req: Request) -> Response {
        println!("📥 {} {}", req.method, req.path);
        
        match (req.method.as_str(), req.path.as_str()) {
            ("GET", "/api/blocks") => {
                let blocks = self.blockchain.get_blocks();
                Response::ok(format!("Blocks: {:?}", blocks))
            }
            
            ("GET", path) if path.starts_with("/api/blocks/") => {
                let height: u64 = path.split('/').last().unwrap().parse().unwrap_or(0);
                match self.blockchain.get_block(height) {
                    Some(block) => Response::ok(format!("Block: {:?}", block)),
                    None => Response::not_found(),
                }
            }
            
            ("GET", "/api/stats") => {
                let stats = self.blockchain.get_stats();
                Response::ok(format!("Stats: {:?}", stats))
            }
            
            ("POST", "/api/tx") => {
                Response::ok("Transaction accepted".to_string())
            }
            
            _ => Response::not_found(),
        }
    }
}

/// 模拟区块链
#[derive(Debug)]
pub struct MockBlockchain {
    blocks: Vec<Block>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub height: u64,
    pub hash: String,
    pub tx_count: usize,
}

impl MockBlockchain {
    pub fn new() -> Self {
        let blocks = vec![
            Block { height: 0, hash: "genesis".to_string(), tx_count: 1 },
            Block { height: 1, hash: "block1".to_string(), tx_count: 5 },
            Block { height: 2, hash: "block2".to_string(), tx_count: 3 },
        ];
        
        Self { blocks }
    }
    
    pub fn get_blocks(&self) -> Vec<Block> {
        self.blocks.clone()
    }
    
    pub fn get_block(&self, height: u64) -> Option<Block> {
        self.blocks.iter().find(|b| b.height == height).cloned()
    }
    
    pub fn get_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert("height".to_string(), (self.blocks.len() as u64) - 1);
        stats.insert("total_tx".to_string(), self.blocks.iter().map(|b| b.tx_count as u64).sum());
        stats
    }
}

fn main() {
    println!("REST API演示\n");
    
    let server = ApiServer::new();
    
    // 测试各种请求
    let requests = vec![
        Request { method: "GET".to_string(), path: "/api/blocks".to_string(), body: None },
        Request { method: "GET".to_string(), path: "/api/blocks/1".to_string(), body: None },
        Request { method: "GET".to_string(), path: "/api/stats".to_string(), body: None },
        Request { method: "POST".to_string(), path: "/api/tx".to_string(), body: Some("{}".to_string()) },
        Request { method: "GET".to_string(), path: "/api/unknown".to_string(), body: None },
    ];
    
    for req in requests {
        let resp = server.handle(req);
        println!("📤 Status: {} - {}\n", resp.status, resp.body);
    }
}
