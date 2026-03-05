# YRUST Chain 生态系统

> 完整的 DeFi 生态系统 - 代币、NFT、DEX、钱包、浏览器

## 🌟 生态系统概览

```
┌─────────────────────────────────────────────────────────────┐
│                     YRUST Chain 生态系统                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐         │
│  │  YRU Token  │  │    NFT      │  │    DEX      │         │
│  │   代币合约   │  │   市场      │  │  去中心化   │         │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘         │
│         │                │                │                │
│         └────────────────┼────────────────┘                │
│                          │                                 │
│              ┌───────────┴───────────┐                    │
│              │    YRUST Chain        │                    │
│              │    核心区块链         │                    │
│              └───────────┬───────────┘                    │
│                          │                                 │
│         ┌────────────────┼────────────────┐               │
│         │                │                │                │
│  ┌──────┴──────┐  ┌──────┴──────┐  ┌──────┴──────┐       │
│  │  Web Wallet │  │  Explorer   │  │   Node      │       │
│  │  网页钱包   │  │  区块浏览器  │  │   节点      │       │
│  └─────────────┘  └─────────────┘  └─────────────┘       │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## 💎 YRU Token (YRUST Token)

### 代币经济学

| 参数 | 值 |
|------|-----|
| 名称 | YRUST Token |
| 符号 | YRU |
| 总供应量 | 21,000,000 YRU |
| 精度 | 8 位小数 |
| 区块奖励 | 50 YRU（每 4 年减半）|
| 初始分配 | 基金会 10%, 团队 5%, 社区 2.5%, 挖矿 82.5% |

### 功能特性

- ✅ ERC-20 兼容接口
- ✅ 转账、授权、查询余额
- ✅ 铸造（仅基金会）
- ✅ 销毁（通缩机制）
- ✅ 手续费销毁（每笔交易销毁 0.1%）

### 使用示例

```rust
use yrust_chain::contracts::token::TokenContract;

// 创建代币
let mut token = TokenContract::new(
    "YRUST Token".to_string(),
    "YRU".to_string(),
    8,
    "foundation_address".to_string(),
);

// 转账
token.transfer("alice", "bob", 100_000_000_000)?; // 1000 YRU

// 查询余额
let balance = token.balance_of("bob");
println!("Bob 余额: {} YRU", balance / 100_000_000);
```

## 🎨 NFT 市场

### 功能特性

- ✅ ERC-721 兼容
- ✅ 铸造、转账、销毁
- ✅ 元数据支持（JSON 标准）
- ✅ 版税机制（创作者获得二次销售 2.5%）
- ✅ 批量铸造

### NFT 元数据标准

```json
{
  "name": "YRUST Genesis #1",
  "description": "YRUST Chain 创世 NFT",
  "image": "https://yrust.chain/nft/1.png",
  "attributes": [
    { "trait_type": "Rarity", "value": "Legendary" },
    { "trait_type": "Power", "value": "100" }
  ]
}
```

### 使用示例

```rust
use yrust_chain::contracts::nft::NFTContract;

// 创建 NFT 合约
let mut nft = NFTContract::new(
    "YRUST Collectibles".to_string(),
    "YRU-NFT".to_string(),
    "creator_address".to_string(),
);

// 铸造 NFT
let token_id = nft.mint(
    "collector_address",
    "https://yrust.chain/nft/1.json",
    "creator_address",
)?;

// 转账
nft.transfer("collector_address", "buyer_address", token_id, "collector_address")?;
```

## 💱 DEX (去中心化交易所)

### 功能特性

- ✅ AMM 自动做市商（恒定乘积公式）
- ✅ 0.3% 交易手续费
- ✅ 流动性挖矿
- ✅ 滑点保护
- ✅ 闪电贷（即将推出）

### 恒定乘积公式

```
x * y = k

其中：
- x = Token A 储备
- y = Token B 储备
- k = 常数

交易后：(x + Δx) * (y - Δy) = k
```

### 使用示例

```rust
use yrust_chain::contracts::dex::{DEXContract, SwapDirection};

// 创建 DEX
let mut dex = DEXContract::new("factory".to_string(), "fee_to".to_string());

// 创建交易对
let pair = dex.create_pool("YRU".to_string(), "ETH".to_string(), "factory")?;

// 添加流动性
let lp_tokens = dex.add_liquidity(&pair, 10000, 20000, "alice")?;

// 交换（用 1000 YRU 买 ETH）
let eth_out = dex.swap(&pair, 1000, 0, SwapDirection::AtoB, "bob")?;

// 移除流动性
let (yru_out, eth_out) = dex.remove_liquidity(&pair, lp_tokens / 2, "alice")?;
```

## 👛 Web 钱包

### 功能特性

- ✅ 创建/导入钱包
- ✅ 转账 YRU
- ✅ 查看 NFT
- ✅ 连接 DEX
- ✅ 交易历史

### 界面预览

![Wallet Preview](docs/wallet-preview.png)

### 安全特性

- 私钥本地加密存储
- 助记词备份
- 硬件钱包支持（Ledger、Trezor）
- 多签钱包

## 🔍 区块浏览器

### 功能特性

- ✅ 实时区块/交易展示
- ✅ 地址查询
- ✅ 代币追踪
- ✅ NFT 展示
- ✅ 图表统计
- ✅ API 接口

### 访问地址

- 主网: https://yrust.chain
- 测试网: https://testnet.yrust.chain

## 🚀 快速开始

### 1. 安装钱包

```bash
# 访问网页钱包
open https://yrust.chain/wallet

# 创建新钱包
# 保存助记词！
```

### 2. 获取 YRU

```bash
# 方式 1: 挖矿
cargo run -- miner

# 方式 2: 从 DEX 购买
open https://yrust.chain/dex

# 方式 3: 水龙头（测试网）
curl https://faucet.yrust.chain/claim?address=YOUR_ADDRESS
```

### 3. 铸造 NFT

```bash
# 访问 NFT 市场
open https://yrust.chain/nft

# 上传图片
# 设置元数据
# 支付 YRU 铸造
```

### 4. 交易代币

```bash
# 访问 DEX
open https://yrust.chain/dex

# 连接钱包
# 选择交易对
# 输入金额
# 确认交易
```

## 📊 经济模型

### 代币分配

```
总供应量: 21,000,000 YRU

基金会:     2,100,000 (10%)  - 生态建设
团队:       1,050,000 (5%)   - 4 年解锁
社区:         525,000 (2.5%) - 空投、活动
挖矿:      17,325,000 (82.5%) - 100 年挖出
```

### 减半机制

| 周期 | 区块高度 | 区块奖励 | 累计产出 |
|------|----------|----------|----------|
| 1 | 0 - 2,100,000 | 50 YRU | 105,000,000 |
| 2 | 2,100,001 - 4,200,000 | 25 YRU | 157,500,000 |
| 3 | 4,200,001 - 6,300,000 | 12.5 YRU | 183,750,000 |
| ... | ... | ... | ... |

### 手续费分配

- 70% 给流动性提供者
- 20% 销毁（通缩）
- 10% 给基金会

## 🔧 开发者文档

### 部署合约

```rust
use yrust_chain::vm::ContractEngine;

let mut engine = ContractEngine::new()?;

// 编译合约
let wasm_code = compile_contract("my_contract.rs")?;

// 部署
let address = engine.deploy(
    wasm_code,
    abi,
    "creator_address".to_string(),
    0,
)?;

println!("合约部署地址: {}", address);
```

### 调用合约

```javascript
// Web3.js 风格
const contract = new web3.Contract(abi, address);

// 调用函数
const result = await contract.methods.transfer(to, amount).send({
    from: myAddress,
    gas: 100000,
});
```

## 🌐 网络信息

### 主网 (Mainnet)

- Network ID: `yrust-mainnet-1`
- Chain ID: 8888
- RPC: https://yrust.chain/api
- P2P: /dns4/bootstrap.yrust.chain/tcp/8000

### 测试网 (Testnet)

- Network ID: `yrust-testnet-1`
- Chain ID: 8889
- RPC: https://testnet.yrust.chain/api
- Faucet: https://faucet.yrust.chain

## 🤝 社区

- Discord: https://discord.gg/yrust
- Twitter: https://twitter.com/yrustchain
- GitHub: https://github.com/yrust/yrust-chain
- Forum: https://forum.yrust.chain

## 📜 许可证

MIT License

---

**YRUST Chain - 构建未来的去中心化金融生态系统** 🚀
