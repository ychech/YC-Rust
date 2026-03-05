//! YRust Chain - 主程序入口
//! 
//! 使用方法:
//!   cargo run -- node        # 启动完整节点 (P2P + API)
//!   cargo run -- miner        # 启动矿工节点
//!   cargo run -- wallet       # 钱包 CLI
//!   cargo run -- demo         # 运行演示
//!   cargo run -- p2p          # 启动 P2P 节点

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::sleep;
use log::info;
use env_logger;

use yrust_chain::core::Blockchain;
use yrust_chain::wallet::Wallet;
use yrust_chain::api::start_api_server;
use yrust_chain::storage::PersistentBlockchain;
use yrust_chain::network::{NetworkManager, NodeConfig, NetworkMessage};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();
    
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("demo");

    match command {
        "node" => run_full_node().await,
        "miner" => run_miner().await,
        "wallet" => run_wallet_cli().await,
        "demo" => run_demo().await,
        "p2p" => run_p2p_node().await,
        _ => {
            println!("用法: cargo run -- [node|miner|wallet|demo|p2p]");
            Ok(())
        }
    }
}

/// 运行完整节点
async fn run_full_node() -> Result<(), Box<dyn std::error::Error>> {
    info!("🚀 启动 YRust Chain 完整节点 v{}", yrust_chain::version());
    
    // 加载或创建区块链
    let persistent = PersistentBlockchain::new("./data/blockchain")?;
    info!("📦 已加载 {} 个区块", persistent.chain.len());
    
    // 创建内存中的区块链用于 API
    let blockchain = Blockchain::new();
    let state = Arc::new(RwLock::new(blockchain));
    
    // 启动 API 服务器
    let api_state = state.clone();
    let api_handle = tokio::spawn(async move {
        if let Err(e) = start_api_server(api_state, 3000).await {
            eprintln!("API 服务器错误: {}", e);
        }
    });

    info!("📡 节点已启动:");
    info!("   API: http://localhost:3000");
    info!("   P2P: 等待连接...");
    
    // 保持运行
    api_handle.await?;
    
    Ok(())
}

/// 运行矿工节点
async fn run_miner() -> Result<(), Box<dyn std::error::Error>> {
    info!("⛏️  启动 YRust Chain 矿工节点");
    
    let mut blockchain = Blockchain::new();
    
    // 创建矿工钱包
    let miner_wallet = Wallet::new();
    println!("{}", miner_wallet);
    
    // 持续挖矿
    loop {
        println!("\n⛏️  正在挖矿...");
        
        match blockchain.mine_block(miner_wallet.address.clone()) {
            Ok(block) => {
                println!("✅ 挖出新区块!");
                println!("{}", block);
                
                let balance = blockchain.get_balance(&miner_wallet.address);
                println!("💰 矿工余额: {} 聪 ({} YRU)", 
                    balance, 
                    balance as f64 / 100_000_000.0
                );
            }
            Err(e) => {
                eprintln!("❌ 挖矿失败: {}", e);
            }
        }

        // 每挖 5 个区块展示一次统计
        if blockchain.get_height() % 5 == 0 {
            println!("\n{}", blockchain.get_stats());
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

/// 钱包 CLI
async fn run_wallet_cli() -> Result<(), Box<dyn std::error::Error>> {
    println!("╔══════════════════════════════════════════╗");
    println!("║         YRust Chain 钱包 CLI              ║");
    println!("╚══════════════════════════════════════════╝");
    
    loop {
        println!("\n选项:");
        println!("  1. 创建新钱包");
        println!("  2. 导入私钥");
        println!("  3. 导出 WIF");
        println!("  4. 签名消息");
        println!("  5. 验证签名");
        println!("  0. 退出");
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        
        match input.trim() {
            "1" => {
                let wallet = Wallet::new();
                println!("\n✅ 新钱包已创建!");
                println!("{}", wallet);
                println!("\n⚠️  请妥善保存私钥，丢失后无法恢复!");
            }
            "2" => {
                println!("请输入私钥 (十六进制):");
                let mut key = String::new();
                std::io::stdin().read_line(&mut key)?;
                
                match Wallet::from_private_key(key.trim()) {
                    Ok(wallet) => {
                        println!("\n✅ 钱包已恢复!");
                        println!("{}", wallet);
                    }
                    Err(e) => {
                        println!("\n❌ 恢复失败: {}", e);
                    }
                }
            }
            "3" => {
                println!("请输入私钥:");
                let mut key = String::new();
                std::io::stdin().read_line(&mut key)?;
                
                match Wallet::from_private_key(key.trim()) {
                    Ok(wallet) => {
                        match wallet.export_wif(true) {
                            Ok(wif) => println!("\nWIF 格式: {}", wif),
                            Err(e) => println!("\n❌ 导出失败: {}", e),
                        }
                    }
                    Err(e) => println!("\n❌ 无效的私钥: {}", e),
                }
            }
            "4" => {
                println!("请输入私钥:");
                let mut key = String::new();
                std::io::stdin().read_line(&mut key)?;
                
                println!("请输入要签名的消息:");
                let mut msg = String::new();
                std::io::stdin().read_line(&mut msg)?;
                
                match Wallet::from_private_key(key.trim()) {
                    Ok(wallet) => {
                        match wallet.sign(msg.trim().as_bytes()) {
                            Ok(sig) => println!("\n签名: {}", hex::encode(&sig)),
                            Err(e) => println!("\n❌ 签名失败: {}", e),
                        }
                    }
                    Err(e) => println!("\n❌ 无效的私钥: {}", e),
                }
            }
            "5" => {
                println!("请输入公钥 (十六进制):");
                let mut key = String::new();
                std::io::stdin().read_line(&mut key)?;
                
                println!("请输入消息:");
                let mut msg = String::new();
                std::io::stdin().read_line(&mut msg)?;
                
                println!("请输入签名 (十六进制):");
                let mut sig = String::new();
                std::io::stdin().read_line(&mut sig)?;
                
                match (hex::decode(key.trim()), hex::decode(sig.trim())) {
                    (Ok(pubkey), Ok(signature)) => {
                        match Wallet::verify_signature(&pubkey, msg.trim().as_bytes(), &signature) {
                            Ok(true) => println!("\n✅ 签名有效!"),
                            Ok(false) => println!("\n❌ 签名无效!"),
                            Err(e) => println!("\n❌ 验证错误: {}", e),
                        }
                    }
                    _ => println!("\n❌ 无效的十六进制数据"),
                }
            }
            "0" => {
                println!("再见!");
                break;
            }
            _ => println!("无效选项"),
        }
    }
    
    Ok(())
}

/// 运行演示
async fn run_demo() -> Result<(), Box<dyn std::error::Error>> {
    use yrust_chain::vm::{ContractEngine, create_counter_contract};

    println!("╔════════════════════════════════════════════════════════╗");
    println!("║                                                        ║");
    println!("║           🚀 YRust Chain 区块链演示 🚀                  ║");
    println!("║                                                        ║");
    println!("║  高性能 Rust 原生区块链实现                             ║");
    println!("║  POW | UTXO | P2P | WASM合约                          ║");
    println!("║                                                        ║");
    println!("╚════════════════════════════════════════════════════════╝");

    // 1. 创建区块链
    println!("\n📦 初始化区块链...");
    let mut blockchain = Blockchain::new();
    println!("✅ 创世区块已创建");

    // 2. 创建钱包
    println!("\n👛 创建钱包...");
    let alice = Wallet::new();
    let bob = Wallet::new();
    let charlie = Wallet::new();

    println!("✅ Alice 地址: {}...", &alice.address[..20]);
    println!("✅ Bob 地址:   {}...", &bob.address[..20]);
    println!("✅ Charlie 地址: {}...", &charlie.address[..20]);

    // 3. 挖矿获得奖励
    println!("\n⛏️  Alice 开始挖矿...");
    let block1 = blockchain.mine_block(alice.address.clone())?;
    println!("✅ 挖出区块 #{}! 哈希: {}...", block1.height, &block1.hash[..20]);

    let alice_balance = blockchain.get_balance(&alice.address);
    println!("💰 Alice 余额: {} 聪 ({} YRU)", 
        alice_balance, 
        alice_balance as f64 / 100_000_000.0
    );

    // 4. 创建转账交易
    println!("\n💸 Alice 转账给 Bob 0.5 YRU...");
    let transfer_amount = 50_000_000; // 0.5 YRU
    
    let tx = blockchain.create_transaction(&alice, bob.address.clone(), transfer_amount)?;
    println!("✅ 交易创建成功: {}...", &tx.id[..20]);
    
    // 添加到内存池
    blockchain.add_to_mempool(tx.clone())?;
    println!("📥 交易已加入内存池");

    // 5. 挖矿确认交易
    println!("\n⛏️  Bob 挖矿确认交易...");
    let block2 = blockchain.mine_block(bob.address.clone())?;
    println!("✅ 挖出区块 #{}! 包含 {} 笔交易", block2.height, block2.transactions.len());

    // 6. 查看余额
    println!("\n💰 当前余额:");
    println!("   Alice:  {} 聪 ({} YRU)", 
        blockchain.get_balance(&alice.address),
        blockchain.get_balance(&alice.address) as f64 / 100_000_000.0
    );
    println!("   Bob:    {} 聪 ({} YRU)", 
        blockchain.get_balance(&bob.address),
        blockchain.get_balance(&bob.address) as f64 / 100_000_000.0
    );
    println!("   Charlie:{} 聪 ({} YRU)", 
        blockchain.get_balance(&charlie.address),
        blockchain.get_balance(&charlie.address) as f64 / 100_000_000.0
    );

    // 7. 多笔转账
    println!("\n💸 Bob 同时转账给 Alice 和 Charlie...");
    let tx2 = blockchain.create_transaction(&bob, alice.address.clone(), 10_000_000)?;
    let tx3 = blockchain.create_transaction(&bob, charlie.address.clone(), 10_000_000)?;
    
    blockchain.add_to_mempool(tx2)?;
    blockchain.add_to_mempool(tx3)?;
    println!("📥 2 笔交易已加入内存池");

    // 8. 继续挖矿
    println!("\n⛏️  Charlie 挖矿确认交易...");
    let block3 = blockchain.mine_block(charlie.address.clone())?;
    println!("✅ 挖出区块 #{}!", block3.height);

    // 9. 最终余额
    println!("\n💰 最终余额:");
    println!("   Alice:  {} 聪 ({} YRU)", 
        blockchain.get_balance(&alice.address),
        blockchain.get_balance(&alice.address) as f64 / 100_000_000.0
    );
    println!("   Bob:    {} 聪 ({} YRU)", 
        blockchain.get_balance(&bob.address),
        blockchain.get_balance(&bob.address) as f64 / 100_000_000.0
    );
    println!("   Charlie:{} 聪 ({} YRU)", 
        blockchain.get_balance(&charlie.address),
        blockchain.get_balance(&charlie.address) as f64 / 100_000_000.0
    );

    // 10. 验证区块链
    println!("\n🔍 验证区块链完整性...");
    if blockchain.is_valid() {
        println!("✅ 区块链验证通过！数据未被篡改");
    } else {
        println!("❌ 区块链验证失败！");
    }

    // 11. 统计信息
    println!("\n{}", blockchain.get_stats());

    // 12. 显示区块详情
    println!("\n📦 区块详情:");
    for block in &blockchain.chain {
        println!("{}", block);
    }

    // 13. 演示签名验证
    println!("\n🔐 签名验证演示:");
    let message = b"Hello, YRust Chain!";
    let signature = alice.sign(message)?;
    println!("✅ Alice 签名消息");
    
    let alice_pubkey = hex::decode(&alice.public_key)?;
    let valid = Wallet::verify_signature(&alice_pubkey, message, &signature)?;
    println!("{} 签名验证结果: {}", 
        if valid { "✅" } else { "❌" },
        if valid { "有效" } else { "无效" }
    );

    // 14. 智能合约演示
    println!("\n📜 智能合约引擎演示:");
    let mut engine = ContractEngine::new()?;
    let (wasm, abi) = create_counter_contract()?;
    
    let contract_addr = engine.deploy_contract(
        wasm,
        abi,
        alice.address.clone(),
        0,
    )?;
    
    println!("✅ 计数器合约已部署: {}...", &contract_addr[..20]);
    println!("   合约函数: get(), increment(), decrement()");

    // 15. API 服务器启动提示
    println!("\n🌐 启动区块浏览器 API...");
    println!("   你可以通过以下方式启动完整节点:");
    println!("   cargo run -- node");
    println!("\n   API 端点:");
    println!("   - GET  /api/stats              区块链统计");
    println!("   - GET  /api/blocks             区块列表");
    println!("   - GET  /api/blocks/height/:h   通过高度获取区块");
    println!("   - GET  /api/tx/:id             获取交易");
    println!("   - GET  /api/address/:addr      获取地址信息");
    println!("   - POST /api/mine               挖矿");
    println!("   - POST /api/tx/create          创建交易");
    println!("   - POST /api/wallet/create      创建钱包");

    println!("\n✨ 演示完成！感谢使用 YRust Chain ✨");

    Ok(())
}

/// 运行 P2P 节点
async fn run_p2p_node() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 启动 P2P 节点演示\n");
    
    // 解析参数
    let args: Vec<String> = std::env::args().collect();
    let mut port = 8000u16;
    let mut peer: Option<String> = None;
    
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--port" => {
                if i + 1 < args.len() {
                    port = args[i + 1].parse()?;
                    i += 1;
                }
            }
            "--peer" => {
                if i + 1 < args.len() {
                    peer = Some(args[i + 1].clone());
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    
    println!("📡 节点配置:");
    println!("   监听端口: {}", port);
    if let Some(ref p) = peer {
        println!("   连接节点: {}", p);
    }
    
    // 创建区块链
    let mut blockchain = Blockchain::new();
    println!("\n📦 区块链初始化完成，高度: {}", blockchain.get_height());
    
    // 创建网络管理器
    let mut network = NetworkManager::new(port).await?;
    
    // 连接指定节点
    if let Some(ref peer_addr) = peer {
        println!("\n🔗 正在连接到 {}...", peer_addr);
        match network.node.connect(peer_addr).await {
            Ok(_) => println!("✅ 连接成功!"),
            Err(e) => println!("❌ 连接失败: {}", e),
        }
    }
    
    println!("\n📡 P2P 节点运行中!");
    println!("   节点ID: {}", network.node.node_id);
    println!("   监听地址: {}", network.node.listen_addr);
    println!("\n可用命令:");
    println!("   mine         - 挖一个新块");
    println!("   status       - 查看区块链状态");
    println!("   peers        - 查看连接节点数");
    println!("   broadcast    - 广播测试消息");
    println!("   help         - 显示帮助");
    println!("   exit/quit    - 退出\n");
    
    // 创建矿工钱包
    let miner = Wallet::new();
    
    // 主循环
    loop {
        print!("> ");
        std::io::Write::flush(&mut std::io::stdout())?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let cmd = input.trim();
        
        match cmd {
            "mine" => {
                println!("⛏️  开始挖矿...");
                match blockchain.mine_block(miner.address.clone()) {
                    Ok(block) => {
                        println!("✅ 挖出区块 #{}!", block.height);
                        println!("   哈希: {}...", &block.hash[..20]);
                        
                        // 广播新区块
                        let msg = NetworkMessage::NewBlock(block);
                        network.node.send_message(msg);
                        println!("📢 区块已广播到网络");
                    }
                    Err(e) => println!("❌ 挖矿失败: {:?}", e),
                }
            }
            "status" => {
                let stats = blockchain.get_stats();
                println!("\n{}", stats);
            }
            "peers" => {
                println!("📡 已连接节点数: {}", network.node.peer_count());
                for peer in &network.node.connected_peers {
                    println!("   - {}", peer);
                }
            }
            "broadcast" => {
                let msg = NetworkMessage::Ping;
                network.node.send_message(msg);
                println!("📢 测试消息已发送");
            }
            "help" => {
                println!("可用命令:");
                println!("   mine         - 挖一个新块");
                println!("   status       - 查看区块链状态");
                println!("   peers        - 查看连接节点数");
                println!("   broadcast    - 广播测试消息");
                println!("   help         - 显示帮助");
                println!("   exit/quit    - 退出");
            }
            "exit" | "quit" => {
                println!("👋 关闭节点...");
                break;
            }
            "" => {}
            _ => println!("未知命令: {}，输入 help 查看帮助", cmd),
        }
    }
    
    Ok(())
}
