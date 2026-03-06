# YRust Chain - 完整 Rust 学习课程

> 通过构建 AI 驱动的区块链，从零掌握 Rust

## 📚 课程概述

本课程包含 **12 个阶段**，从 Rust 基础到高级区块链 + AI 开发，总计约 **200+ 小时**学习内容。

---

## 🎯 阶段 1: Rust 基础 (20小时)

### 1.1 环境搭建与 Hello World
**文件**: `tutorial-full/01-basic/`

```rust
// 第一个 Rust 程序
fn main() {
    println!("Hello, YRust Chain!");
}
```

**学习目标**:
- [ ] 安装 Rust 工具链
- [ ] 理解 Cargo 包管理
- [ ] 掌握基本语法结构
- [ ] 学习变量与可变性

### 1.2 所有权系统
**核心概念**:
```rust
// 所有权转移
let s1 = String::from("hello");
let s2 = s1; // s1 不再有效

// 借用
fn calculate_hash(data: &[u8]) -> String {
    // 只读借用
}

// 可变借用
fn mine_block(block: &mut Block) {
    // 修改区块
}
```

**实践**: 实现简单的字符串哈希

### 1.3 结构体与方法
```rust
// 定义区块结构
pub struct Block {
    height: u64,
    hash: String,
    prev_hash: String,
    timestamp: i64,
}

// 实现方法
impl Block {
    pub fn new(height: u64, prev_hash: String) -> Self {
        Self { height, hash: String::new(), prev_hash, timestamp: 0 }
    }
    
    pub fn mine(&mut self, difficulty: usize) {
        // 挖矿逻辑
    }
}
```

---

## 🔐 阶段 2: 密码学基础 (15小时)

### 2.1 哈希算法
**文件**: `src/core/types.rs`, `tutorial-full/02-crypto/`

```rust
use sha2::{Sha256, Digest};

pub fn calculate_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    hex::encode(result)
}
```

**学习内容**:
- SHA-256 原理
- 哈希指针与区块链
- Merkle 树实现

### 2.2 数字签名
**文件**: `src/wallet/mod.rs`

```rust
use secp256k1::{Secp256k1, Message, SecretKey, PublicKey};

pub struct Wallet {
    secret_key: SecretKey,
    public_key: PublicKey,
}

impl Wallet {
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>, Error> {
        let msg = Message::from_slice(message)?;
        let sig = self.secp.sign_ecdsa(&msg, &self.secret_key);
        Ok(sig.serialize_compact().to_vec())
    }
    
    pub fn verify(
        public_key: &PublicKey,
        message: &[u8],
        signature: &[u8]
    ) -> Result<bool, Error> {
        // 验证逻辑
    }
}
```

---

## ⛓️ 阶段 3: 区块链核心 (25小时)

### 3.1 区块与链
**文件**: `src/core/blockchain.rs`, `tutorial-full/03-blockchain/`

```rust
pub struct Blockchain {
    chain: Vec<Block>,
    utxo_set: HashMap<(String, u32), TxOutput>,
    mempool: Vec<Transaction>,
    difficulty: usize,
}

impl Blockchain {
    pub fn new() -> Self {
        let genesis = Self::create_genesis_block();
        Self {
            chain: vec![genesis],
            utxo_set: HashMap::new(),
            mempool: Vec::new(),
            difficulty: 4,
        }
    }
    
    pub fn mine_block(&mut self, miner_addr: String) -> Result<Block, Error> {
        // 1. 创建 Coinbase 交易
        // 2. 选择内存池交易
        // 3. 执行 POW
        // 4. 验证并添加区块
    }
}
```

### 3.2 UTXO 模型
```rust
pub struct Transaction {
    id: String,
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
}

pub struct TxInput {
    tx_id: String,
    vout: u32,
    script_sig: ScriptSig,
}

pub struct TxOutput {
    value: u64,
    script_pubkey: String,
}
```

### 3.3 共识机制 (POW)
```rust
impl Block {
    pub fn mine(&mut self, difficulty: usize) {
        let target = "0".repeat(difficulty);
        
        loop {
            self.nonce += 1;
            self.hash = self.calculate_hash();
            
            if self.hash.starts_with(&target) {
                break; // 找到有效哈希
            }
        }
    }
}
```

---

## 🌐 阶段 4: 网络编程 (20小时)

### 4.1 P2P 网络基础
**文件**: `src/network/`, `tutorial-full/04-network/`

```rust
use tokio::net::{TcpListener, TcpStream};
use libp2p::{PeerId, Swarm, Multiaddr};

pub struct P2PNode {
    node_id: PeerId,
    swarm: Swarm<Behaviour>,
    peers: HashSet<PeerId>,
}

impl P2PNode {
    pub async fn run(&mut self) -> Result<(), Error> {
        loop {
            tokio::select! {
                event = self.swarm.select_next_some() => {
                    self.handle_event(event).await;
                }
            }
        }
    }
}
```

### 4.2 异步编程
```rust
// 使用 tokio 进行异步网络通信
pub async fn broadcast_block(
    &self,
    block: Block
) -> Result<(), Error> {
    let msg = NetworkMessage::NewBlock(block);
    
    for peer in &self.peers {
        self.send_message(peer, msg.clone()).await?;
    }
    
    Ok(())
}
```

---

## 🤝 阶段 5: 共识算法 (15小时)

### 5.1 工作量证明
**文件**: `src/network/consensus.rs`, `tutorial-full/05-consensus/`

```rust
pub trait Consensus {
    fn validate_block(&self, block: &Block) -> bool;
    fn select_chain(&self, forks: &[Blockchain]) -> usize;
}

pub struct PowConsensus {
    difficulty: usize,
}

impl Consensus for PowConsensus {
    fn validate_block(&self, block: &Block) -> bool {
        // 验证哈希难度
        let target = "0".repeat(self.difficulty);
        block.hash.starts_with(&target)
    }
}
```

---

## 💾 阶段 6: 数据持久化 (15小时)

### 6.1 RocksDB 存储
**文件**: `src/storage/`, `tutorial-full/06-storage/`

```rust
use rocksdb::{DB, Options};

pub struct BlockchainStorage {
    db: DB,
}

impl BlockchainStorage {
    pub fn new(path: &str) -> Result<Self, Error> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path)?;
        Ok(Self { db })
    }
    
    pub fn put_block(&self, block: &Block) -> Result<(), Error> {
        let key = format!("block:{}", block.height);
        let value = serde_json::to_vec(block)?;
        self.db.put(key, value)?;
        Ok(())
    }
}
```

---

## 🔧 阶段 7: WASM 虚拟机 (25小时)

### 7.1 智能合约引擎
**文件**: `src/vm/`, `tutorial-full/07-vm/`

```rust
use wasmtime::{Engine, Module, Store, Instance};

pub struct ContractEngine {
    engine: Engine,
    store: Store<()>,
    contracts: HashMap<String, Contract>,
}

impl ContractEngine {
    pub fn deploy_contract(
        &mut self,
        wasm_code: &[u8],
        abi: ContractAbi,
    ) -> Result<String, Error> {
        let module = Module::new(&self.engine, wasm_code)?;
        let instance = Instance::new(&mut self.store, &module, &[])?;
        
        let addr = generate_contract_address();
        self.contracts.insert(addr.clone(), Contract { module, abi });
        
        Ok(addr)
    }
    
    pub fn call_contract(
        &mut self,
        addr: &str,
        func: &str,
        args: &[Value]
    ) -> Result<Value, Error> {
        let contract = self.contracts.get(addr)?;
        // 执行合约函数
    }
}
```

### 7.2 合约示例
```rust
// ERC20 代币合约
#[no_mangle]
pub extern "C" fn transfer(to: *const u8, amount: u64) -> i32 {
    let sender = env::caller();
    let balance = env::balance_of(&sender);
    
    if balance < amount {
        return -1; // 余额不足
    }
    
    env::transfer(&sender, to, amount);
    0 // 成功
}
```

---

## 📜 阶段 8: 智能合约开发 (20小时)

### 8.1 合约模板
**文件**: `contracts/`, `tutorial-full/08-contracts/`

```rust
// 质押挖矿合约
pub struct StakingContract {
    staked_token: ContractAddress,
    reward_token: ContractAddress,
    reward_per_block: u128,
    pools: HashMap<Address, StakingPool>,
}

impl StakingContract {
    pub fn deposit(&mut self, amount: u128) -> Result<(), Error> {
        let user = env::caller();
        self.update_pool()?;
        
        // 转移代币
        env::transfer_from(&user, &env::contract_address(), amount)?;
        
        // 更新质押记录
        let pool = self.pools.entry(user).or_default();
        pool.amount += amount;
        pool.reward_debt = pool.amount * self.acc_reward_per_share;
        
        Ok(())
    }
    
    pub fn claim(&mut self) -> Result<u128, Error> {
        let user = env::caller();
        let pending = self.pending_reward(&user);
        
        env::transfer(&user, pending)?;
        
        Ok(pending)
    }
}
```

---

## 🛡️ 阶段 9: 安全与加密 (15小时)

### 9.1 安全机制
**文件**: `src/security/`, `tutorial-full/09-security/`

```rust
// 重入攻击防护
pub struct ReentrancyGuard {
    locked: bool,
}

impl ReentrancyGuard {
    pub fn non_reentrant<F, R>(&mut self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        assert!(!self.locked, "Reentrant call");
        self.locked = true;
        let result = f();
        self.locked = false;
        result
    }
}

// 访问控制
pub fn only_owner<T>(state: &State, f: impl FnOnce() -> T) -> T {
    assert_eq!(env::caller(), state.owner, "Not owner");
    f()
}
```

---

## 🌐 阶段 10: Web API 开发 (15小时)

### 10.1 Axum Web 框架
**文件**: `src/api/`, `tutorial-full/10-api/`

```rust
use axum::{
    routing::{get, post},
    Router,
    extract::{Path, State},
    Json,
};

pub fn create_routes(state: ApiState) -> Router {
    Router::new()
        .route("/api/blocks", get(get_blocks))
        .route("/api/tx/create", post(create_transaction))
        .route("/api/mine", post(mine_block))
        .with_state(state)
}

async fn get_blocks(
    State(state): State<ApiState>
) -> Json<ApiResponse<Vec<Block>>> {
    let blockchain = state.read().await;
    Json(ApiResponse::success(blockchain.chain.clone()))
}
```

---

## 🤖 阶段 11: AI 集成 (30小时)

### 11.1 AI 模块架构
**文件**: `src/ai/`

```rust
// AI 服务管理器
pub struct AIService {
    pub mining: SmartMining,
    pub risk: RiskEngine,
    pub nlp: NLPQueryEngine,
    pub ops: OpsMonitor,
    pub contract: ContractGenerator,
    pub wallet: AIWalletAssistant,
}

// 智能挖矿
pub struct SmartMining {
    block_time_history: VecDeque<u64>,
    prediction_model: PredictionModel,
    strategy: MiningStrategy,
}

impl SmartMining {
    pub fn get_mining_advice(&self) -> MiningAdvice {
        let predicted = self.predict_block_time();
        let profit_score = self.calculate_profit_score();
        
        MiningAdvice {
            suggested_difficulty: self.suggest_difficulty(),
            predicted_block_time: predicted,
            profit_score,
            recommended_strategy: self.recommend_strategy(),
        }
    }
}
```

### 11.2 风控系统
```rust
pub struct RiskEngine {
    account_profiles: HashMap<String, AccountRiskProfile>,
    blacklist: HashSet<String>,
    alert_rules: Vec<AlertRule>,
}

impl RiskEngine {
    pub fn detect_transaction_risk(&mut self, tx: &Transaction) -> RiskResult {
        let mut score = 0;
        let mut risks = Vec::new();
        
        // 双花检测
        if self.check_double_spending(tx) {
            score += 50;
            risks.push(RiskType::DoubleSpending);
        }
        
        // 大额交易检测
        if self.is_large_transaction(tx) {
            score += 20;
            risks.push(RiskType::LargeTransaction);
        }
        
        // 洗钱模式检测
        if self.detect_money_laundering(tx) {
            score += 30;
            risks.push(RiskType::MoneyLaundering);
        }
        
        RiskResult {
            level: RiskLevel::from_score(score),
            score,
            risk_types: risks,
            recommended_action: self.determine_action(score),
        }
    }
}
```

### 11.3 自然语言处理
```rust
pub struct NLPQueryEngine {
    intent_patterns: HashMap<QueryIntent, Vec<String>>,
}

impl NLPQueryEngine {
    pub fn process_query(&mut self, query: NLPQuery) -> NLPResponse {
        // 1. 意图识别
        let (intent, confidence) = self.recognize_intent(&query.text);
        
        // 2. 参数提取
        let params = self.extract_parameters(&query.text, intent);
        
        // 3. 执行查询
        let result = self.execute_query(intent, &params);
        
        // 4. 生成自然语言回答
        let answer = self.generate_natural_answer(&result);
        
        NLPResponse {
            intent,
            confidence,
            result,
            natural_answer: answer,
        }
    }
}
```

---

## 🎓 阶段 12: 高级主题与优化 (20小时)

### 12.1 性能优化
```rust
// 使用通道进行并发处理
use crossbeam_channel::{bounded, Sender, Receiver};

pub struct Miner {
    work_sender: Sender<Work>,
    result_receiver: Receiver<Block>,
}

impl Miner {
    pub fn start_parallel_mining(&self, num_threads: usize) {
        for i in 0..num_threads {
            let receiver = self.work_sender.clone();
            let sender = self.result_receiver.clone();
            
            thread::spawn(move || {
                while let Ok(work) = receiver.recv() {
                    if let Some(block) = Self::mine(work) {
                        sender.send(block).ok();
                    }
                }
            });
        }
    }
}
```

### 12.2 内存管理
```rust
// 使用 arena 分配器优化内存
use bumpalo::Bump;

pub struct BlockArena {
    arena: Bump,
}

impl BlockArena {
    pub fn allocate_block(&self, txs: Vec<Transaction>) -> &mut Block {
        self.arena.alloc(Block::new(txs))
    }
}
```

---

## 📋 学习路径建议

### 初学者路径 (无编程经验)
1. 阶段 1: Rust 基础 (20h)
2. 阶段 2: 密码学基础 (15h)
3. 阶段 3: 区块链核心 (25h)
4. 阶段 10: Web API (15h)
5. 阶段 11: AI 集成 (30h)

**总计**: 105 小时 → 可以构建基础 AI 区块链

### 进阶路径 (有其他语言经验)
1. 阶段 1: Rust 基础 (10h - 加速)
2. 阶段 3-5: 区块链核心 (60h)
3. 阶段 7-8: 智能合约 (45h)
4. 阶段 11: AI 集成 (30h)

**总计**: 145 小时 → 完整区块链开发者

### 专家路径 (有区块链经验)
1. 阶段 1: Rust 快速入门 (5h)
2. 阶段 7-9: WASM + 安全 (55h)
3. 阶段 11-12: AI + 优化 (50h)

**总计**: 110 小时 → 区块链架构师

---

## 🛠️ 实践项目

### 项目 1: 简单区块链 (阶段 1-3)
- 实现 POW 挖矿
- 创建交易和区块
- 本地节点运行

### 项目 2: 加密货币钱包 (阶段 1-2, 10)
- 生成密钥对
- 签名交易
- Web 界面

### 项目 3: DEX 交易所 (阶段 7-8, 10)
- AMM 流动性池
- 代币交换
- 前端界面

### 项目 4: AI 智能链 (阶段 11)
- 集成 AI 挖矿
- 实现风控系统
- 自然语言查询

### 项目 5: 完整生态 (所有阶段)
- DeFi + GameFi + DAO
- AI 全功能集成
- 主网部署

---

## 📚 学习资源

### 官方文档
- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust by Example](https://doc.rust-lang.org/rust-by-example/)
- [Cargo 手册](https://doc.rust-lang.org/cargo/)

### 本项目资源
- `tutorial-full/` - 完整教程代码
- `tutorials/` - AI 模块教程
- `docs/` - 架构文档
- `contracts/` - 合约示例

### 推荐书籍
- 《Programming Rust》
- 《Rust in Action》
- 《Mastering Blockchain》

---

## ✅ 学习检查清单

### Rust 基础
- [ ] 理解所有权和借用
- [ ] 掌握生命周期
- [ ] 使用泛型和 trait
- [ ] 错误处理 (Result/Option)
- [ ] 并发编程 (async/await)

### 区块链核心
- [ ] 理解哈希指针
- [ ] 实现 Merkle 树
- [ ] POW 共识算法
- [ ] UTXO 模型
- [ ] P2P 网络

### 智能合约
- [ ] WASM 基础
- [ ] 合约状态管理
- [ ] 跨合约调用
- [ ] 安全最佳实践

### AI 集成
- [ ] 机器学习基础
- [ ] 自然语言处理
- [ ] 风险模型
- [ ] 预测算法

---

## 🎓 认证项目

完成以下项目获得认证：

1. **Rust 开发者认证** - 完成阶段 1-3
2. **区块链工程师认证** - 完成阶段 1-8
3. **AI 区块链架构师认证** - 完成所有阶段

---

**开始你的 Rust + 区块链 + AI 之旅吧！** 🚀
