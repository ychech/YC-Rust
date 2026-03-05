# YRUST Chain 白皮书

## 摘要

YRUST Chain 是一个基于 Rust 构建的高性能、安全、可扩展的区块链平台。采用工作量证明（POW）共识机制，支持智能合约和去中心化应用（DApp）开发。通过创新的架构设计和先进的密码学技术，YRUST Chain 实现了每秒 1000+ 笔交易的处理能力，同时保持去中心化和安全性。

---

## 1. 引言

### 1.1 背景

区块链技术自比特币诞生以来，已经发展成为一个重要的技术领域。然而，现有区块链平台面临着以下挑战：

- **性能瓶颈**：比特币 7 TPS，以太坊 15 TPS
- **高昂费用**：高峰期交易费用可达数十美元
- **复杂性**：智能合约开发门槛高，安全问题频发
- **能源消耗**：传统 POW 机制消耗大量电力

### 1.2 愿景

YRUST Chain 旨在构建一个：
- ⚡ **高性能**：1000+ TPS，秒级确认
- 💰 **低成本**：平均交易费用低于 $0.001
- 🛠️ **开发者友好**：使用 Rust 开发智能合约
- 🌱 **节能环保**：优化挖矿算法，降低能耗

---

## 2. 技术架构

### 2.1 系统架构

```
┌─────────────────────────────────────────────────────────────┐
│                      应用层 (DApps)                          │
│         DeFi    NFT    Game    Social    DAO               │
├─────────────────────────────────────────────────────────────┤
│                      合约层 (Smart Contracts)                │
│              Token    NFT    DEX    Custom                 │
├─────────────────────────────────────────────────────────────┤
│                      执行层 (WASM VM)                        │
│         Contract Engine    Gas Metering    Storage         │
├─────────────────────────────────────────────────────────────┤
│                      核心层 (Core)                           │
│    Blockchain    Consensus    P2P Network    Mempool       │
├─────────────────────────────────────────────────────────────┤
│                      数据层 (Storage)                        │
│         RocksDB    UTXO Set    State    Index              │
├─────────────────────────────────────────────────────────────┤
│                      安全层 (Security)                       │
│    Crypto    Replay Protection    Access Control           │
└─────────────────────────────────────────────────────────────┘
```

### 2.2 核心特性

#### 2.2.1 双模型支持

YRUST Chain 同时支持两种账户模型：

**UTXO 模型（比特币风格）**
- 优点：并行处理能力强，隐私性好
- 适用：转账、支付场景

**账户模型（以太坊风格）**
- 优点：合约开发直观，状态管理简单
- 适用：智能合约、DApp

#### 2.2.2 WASM 虚拟机

- **高性能**：接近原生代码执行速度
- **多语言**：支持 Rust、C++、AssemblyScript
- **安全**：沙箱执行，内存隔离
- **标准**：WebAssembly 行业标准

#### 2.2.3 libp2p 网络

- **模块化**：可插拔传输协议
- **高效**：QUIC 传输，减少延迟
- **去中心化**：Kademlia DHT 节点发现
- **健壮**：Gossipsub 消息传播

---

## 3. 共识机制

### 3.1 工作量证明（POW）

YRUST Chain 采用优化的 POW 机制：

```
挖矿算法：SHA256（双重哈希）
区块时间：约 10 秒
难度调整：每 2016 个区块（约 6 小时）
初始奖励：50 YRU
减半周期：每 210,000 个区块（约 4 年）
```

### 3.2 难度调整算法

```rust
新难度 = 旧难度 × (实际时间) / (目标时间)

目标时间 = 2016 个区块 × 10 秒 = 20160 秒
```

调整限制：单次调整不超过 4 倍或 1/4

### 3.3 安全性分析

**51% 攻击成本**：
- 假设网络总算力为 H
- 攻击者需要 > 0.5H 的算力
- 成本 = 硬件成本 + 电力成本

**双花防护**：
- 6 个确认后，双花概率 < 0.1%
- UTXO 模型天然防止双花

---

## 4. 密码学方案

### 4.1 哈希算法

**SHA256**：用于区块哈希、交易哈希、Merkle 树

**双重哈希**：`hash = SHA256(SHA256(data))`
- 防止长度扩展攻击
- 比特币传统，经过时间考验

### 4.2 数字签名

**算法**：ECDSA（secp256k1）

**密钥生成**：
```
私钥：32 字节随机数
公钥：33 字节（压缩）或 65 字节（未压缩）
地址：Base58Check 编码
```

**签名流程**：
1. 计算消息哈希
2. 生成随机数 k
3. 计算 R = k × G
4. 计算 s = k^(-1) × (z + r × d)
5. 输出 (r, s)

### 4.3 地址格式

```
主网地址：YR1...
测试网地址：YR0...

生成流程：
1. 公钥 → SHA256 → RIPEMD160 = 20 字节哈希
2. 添加版本字节（0x4D）
3. 计算校验和（双重 SHA256 前 4 字节）
4. Base58 编码
```

---

## 5. 智能合约

### 5.1 合约模型

**基于 WASM**：
- 编译目标：wasm32-unknown-unknown
- 执行环境：wasmtime
- 计量单位：Gas

**Gas 机制**：
- 每指令消耗固定 Gas
- 存储按字节计费
- 调用按复杂度计费

### 5.2 合约示例

```rust
// Token 合约
#[contract]
mod token {
    #[storage]
    struct Token {
        total_supply: u64,
        balances: Map<Address, u64>,
    }
    
    #[init]
    fn init(total_supply: u64) {
        let caller = msg::caller();
        storage.balances.insert(caller, total_supply);
        storage.total_supply = total_supply;
    }
    
    #[call]
    fn transfer(to: Address, amount: u64) -> bool {
        let from = msg::caller();
        let from_balance = storage.balances.get(&from).unwrap_or(0);
        
        require(from_balance >= amount, "Insufficient balance");
        
        storage.balances.insert(from, from_balance - amount);
        let to_balance = storage.balances.get(&to).unwrap_or(0);
        storage.balances.insert(to, to_balance + amount);
        
        true
    }
}
```

### 5.3 合约标准

**YRC-20**：同质化代币标准
- `total_supply()`
- `balance_of(address)`
- `transfer(to, amount)`
- `approve(spender, amount)`
- `transfer_from(from, to, amount)`

**YRC-721**：非同质化代币标准
- `owner_of(token_id)`
- `transfer_from(from, to, token_id)`
- `approve(to, token_id)`

---

## 6. 经济模型

### 6.1 代币分配

**总供应量**：21,000,000 YRU（与比特币相同）

**分配方案**：
- 挖矿奖励：90%（18,900,000 YRU）
- 开发者基金：5%（1,050,000 YRU）
- 社区激励：3%（630,000 YRU）
- 早期投资者：2%（420,000 YRU）

### 6.2 挖矿奖励

| 周期 | 区块高度 | 奖励（YRU） | 累计产出 |
|------|----------|-------------|----------|
| 1 | 0 - 210,000 | 50 | 10,500,000 |
| 2 | 210,001 - 420,000 | 25 | 15,750,000 |
| 3 | 420,001 - 630,000 | 12.5 | 18,375,000 |
| 4 | 630,001 - 840,000 | 6.25 | 19,687,500 |
| ... | ... | ... | ... |

### 6.3 交易费用

**费用计算**：
```
总费用 = Gas 使用量 × Gas 价格

Gas 价格由市场决定
矿工优先打包高费用交易
```

**费用分配**：
- 100% 归矿工
- 销毁机制（可选）：部分费用销毁

---

## 7. 治理机制

### 7.1 链上治理

**提案流程**：
1. 抵押代币提交提案
2. 社区讨论期（2周）
3. 投票期（1周）
4. 执行（如通过）

**投票权重**：
- 1 YRU = 1 票
- 可委托投票

### 7.2 协议升级

**软分叉**：
- 向后兼容
- 矿工信号激活（BIP9风格）

**硬分叉**：
- 不向后兼容
- 需要社区广泛共识

---

## 8. 路线图

### Phase 1: 基础（2024 Q1）
- ✅ 核心区块链实现
- ✅ POW 共识
- ✅ 基础钱包
- ✅ 区块浏览器

### Phase 2: 智能合约（2024 Q2）
- ✅ WASM 虚拟机
- ✅ 合约部署/调用
- ✅ Token/NFT 合约
- ✅ DEX 合约

### Phase 3: 生态（2024 Q3）
- 🔄 跨链桥接
- 🔄 隐私交易
- 🔄 Layer 2 扩展
- 🔄 移动端钱包

### Phase 4: 规模化（2024 Q4）
- ⏳ 分片技术
- ⏳ 零知识证明
- ⏳ 企业级解决方案
- ⏳ 全球节点网络

---

## 9. 团队与社区

### 9.1 核心团队

- **创始人**：区块链领域 10 年经验
- **技术负责人**：Rust 核心贡献者
- **密码学专家**：博士，多篇顶会论文
- **产品经理**：前以太坊生态负责人

### 9.2 社区建设

- GitHub：开源代码，欢迎贡献
- Discord：实时交流，技术支持
- Twitter：最新动态，公告发布
- 论坛：深度讨论，提案提交

---

## 10. 风险提示

### 10.1 技术风险

- **智能合约漏洞**：尽管经过审计，仍可能存在未知漏洞
- **网络攻击**：51% 攻击、DDoS 攻击等
- **代码缺陷**：软件 bug 可能导致资金损失

### 10.2 市场风险

- **价格波动**：加密货币价格波动剧烈
- **监管风险**：各国监管政策不确定
- **竞争风险**：区块链行业竞争激烈

### 10.3 使用建议

- 不要投入无法承受损失的资金
- 妥善保管私钥
- 关注官方安全公告
- 使用多重签名保护大额资产

---

## 11. 总结

YRUST Chain 是一个创新的区块链平台，通过 Rust 的安全性和性能优势，结合先进的密码学技术，为去中心化应用提供高性能、低成本的基础设施。我们相信，YRUST Chain 将成为下一代互联网的重要组成部分。

---

## 附录

### A. 技术参数

| 参数 | 值 |
|------|-----|
| 区块时间 | ~10 秒 |
| 区块大小 | 2 MB |
| 交易确认 | 6 个区块 |
| 总供应量 | 21,000,000 YRU |
| 最小单位 | 1 YRU = 10^8 聪 |
| 哈希算法 | SHA256 |
| 签名算法 | ECDSA (secp256k1) |
| 地址格式 | Base58Check |

### B. 相关链接

- 官网：https://yrust.chain
- GitHub：https://github.com/yrust/yrust-chain
- 文档：https://docs.yrust.chain
- 浏览器：https://explorer.yrust.chain

### C. 参考资料

1. Bitcoin: A Peer-to-Peer Electronic Cash System (Satoshi Nakamoto)
2. Ethereum White Paper (Vitalik Buterin)
3. Rust Programming Language
4. WebAssembly Specification
5. libp2p Documentation

---

**版本**：v1.0  
**日期**：2024-03-05  
**版权**：YRUST Chain Foundation
