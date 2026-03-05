# YRUST DEX 完全教程

> 从零开始理解并构建一个 AMM 去中心化交易所

## 📚 目录

1. [什么是 AMM？](#什么是-amm)
2. [恒定乘积公式](#恒定乘积公式)
3. [代码实现](#代码实现)
4. [前端交互](#前端交互)
5. [部署和运行](#部署和运行)

---

## 什么是 AMM？

**AMM (Automated Market Maker)** = 自动做市商

传统交易所使用**订单簿**：
```
买单:
- 100 USDT @ 0.99
-  99 USDT @ 0.98

卖单:
- 101 USDT @ 1.01
- 102 USDT @ 1.02
```

AMM 使用**流动性池**：
```
池子: 1000 ETH + 2000000 USDT
价格 = 2000000 / 1000 = 2000 USDT/ETH
```

### AMM 的优势

| 特性 | 订单簿 | AMM |
|------|--------|-----|
| 流动性 | 需要做市商 | 任何人提供 |
| 价格发现 | 买卖撮合 | 公式计算 |
| 无常损失 | 无 | 有 |
|  Gas 费 | 低 | 高 |

---

## 恒定乘积公式

### 核心公式

```
x * y = k

x = Token A 数量
y = Token B 数量
k = 常数（流动性不变时）
```

### 交易计算

**买入 Δx，卖出 Δy：**

```
交易前: x * y = k
交易后: (x + Δx) * (y - Δy) = k

解得:
Δy = y * Δx / (x + Δx)
```

### 例子

池子: 100 ETH + 200000 USDT

你想用 10000 USDT 买 ETH：

```
Δy = 100 * 10000 / (200000 + 10000)
   = 1000000 / 210000
   = 4.76 ETH
```

实际价格 = 10000 / 4.76 = 2100 USDT/ETH

**滑点 = (2100 - 2000) / 2000 = 5%**

---

## 代码实现

### 1. 创建流动性池

```rust
// 创建 YRU/ETH 交易对
let pair = amm.create_pair(
    "YRU".to_string(),
    "ETH".to_string(),
    "factory_address"
)?;
```

### 2. 添加流动性

```rust
// 添加 10000 YRU + 2 ETH
let lp_tokens = amm.add_liquidity(
    &pair,
    10000_00000000,  // 10000 YRU
    2_00000000,      // 2 ETH
    "alice"
)?;

// LP Token = sqrt(10000 * 2) = 141.42
```

### 3. 执行交换

```rust
// 用 1000 YRU 买 ETH
let req = SwapRequest {
    pair: pair.clone(),
    token_in: "YRU".to_string(),
    token_out: "ETH".to_string(),
    amount_in: 1000_00000000,
    min_amount_out: 0_19000000,  // 最少 0.19 ETH（滑点保护）
    recipient: "bob".to_string(),
    deadline: now + 3600,  // 1小时内有效
};

let eth_out = amm.swap(req)?;
// 得到约 0.198 ETH
```

### 4. 移除流动性

```rust
// 移除一半流动性
let (yru_out, eth_out) = amm.remove_liquidity(
    &pair,
    lp_tokens / 2,
    "alice"
)?;

// 得到 5000 YRU + 1 ETH
```

---

## 无常损失 (Impermanent Loss)

### 什么是无常损失？

当你提供流动性时，如果代币价格变化，相比单纯持有，你会亏损。

### 计算示例

**初始状态：**
- 池子: 100 ETH + 10000 USDT
- 你提供: 10 ETH + 1000 USDT (10% 份额)
- ETH 价格: 100 USDT

**价格变化后：**
- ETH 涨到 400 USDT
- 新池子: 50 ETH + 20000 USDT (保持 x*y = k)

**你的份额：**
- 取出: 5 ETH + 2000 USDT = 4000 USDT

**如果单纯持有：**
- 10 ETH * 400 + 1000 USDT = 5000 USDT

**无常损失 = 5000 - 4000 = 1000 USDT (20%)**

### 无常损失表

| 价格变化 | 无常损失 |
|----------|----------|
| 1.25x    | 0.6%     |
| 1.50x    | 2.0%     |
| 1.75x    | 3.8%     |
| 2.00x    | 5.7%     |
| 3.00x    | 13.4%    |
| 4.00x    | 20.0%    |
| 5.00x    | 25.5%    |

### 如何减少无常损失？

1. **稳定币对**: USDC/USDT (价格稳定)
2. **相关资产**: ETH/stETH (价格同步)
3. **手续费收入**: 高交易量抵消损失
4. **流动性挖矿**: 额外代币奖励

---

## 前端交互

### 连接钱包

```javascript
async function connectWallet() {
    const provider = new ethers.providers.Web3Provider(window.ethereum);
    await provider.send("eth_requestAccounts", []);
    const signer = provider.getSigner();
    const address = await signer.getAddress();
    console.log("Connected:", address);
}
```

### 获取池子信息

```javascript
async function getPoolInfo(pairAddress) {
    const response = await fetch(`http://localhost:3000/api/dex/pool/${pairAddress}`);
    const data = await response.json();
    
    console.log("Reserve A:", data.reserve_a);
    console.log("Reserve B:", data.reserve_b);
    console.log("Price:", data.reserve_a / data.reserve_b);
}
```

### 执行交换

```javascript
async function swap(fromToken, toToken, amountIn) {
    const response = await fetch('http://localhost:3000/api/dex/swap', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
            pair: "YRU_ETH_PAIR",
            token_in: fromToken,
            token_out: toToken,
            amount_in: amountIn,
            min_amount_out: calculateMinOut(amountIn),
            recipient: userAddress,
            deadline: Date.now() + 3600000
        })
    });
    
    const result = await response.json();
    return result.amount_out;
}
```

---

## 部署和运行

### 1. 启动节点

```bash
cargo run -- node
```

### 2. 创建交易对

```bash
curl -X POST http://localhost:3000/api/dex/create-pair \
  -H "Content-Type: application/json" \
  -d '{
    "token_a": "YRU",
    "token_b": "ETH",
    "caller": "factory"
  }'
```

### 3. 添加流动性

```bash
curl -X POST http://localhost:3000/api/dex/add-liquidity \
  -H "Content-Type: application/json" \
  -d '{
    "pair": "PAIR_ADDRESS",
    "amount_a": "1000000000000",
    "amount_b": "200000000",
    "provider": "alice"
  }'
```

### 4. 执行交换

```bash
curl -X POST http://localhost:3000/api/dex/swap \
  -H "Content-Type: application/json" \
  -d '{
    "pair": "PAIR_ADDRESS",
    "token_in": "YRU",
    "token_out": "ETH",
    "amount_in": "10000000000",
    "min_amount_out": "1900000",
    "recipient": "bob",
    "deadline": 1704067200
  }'
```

### 5. 打开 DEX 界面

```bash
open dex/frontend/index.html
```

---

## 高级特性

### 闪电贷

```rust
// 借出 1000 ETH，无需抵押
amm.flash_loan(
    &pair,
    0,
    1000_00000000,
    |engine| {
        // 在这里使用借来的 ETH
        // 套利、清算等操作
        
        // 必须在回调结束前归还 + 0.3% 手续费
        Ok(())
    }
)?;
```

### 价格预言机

```rust
// 获取时间加权平均价格 (TWAP)
let price = amm.get_twap_price(&pair, 3600)?; // 1小时平均价
```

### 多跳交换

```rust
// YRU -> ETH -> BTC
let eth = amm.swap(yru_to_eth)?;
let btc = amm.swap(eth_to_btc)?;
```

---

## 总结

YRUST DEX 特点：

- ✅ 恒定乘积 AMM
- ✅ 0.3% 手续费
- ✅ 闪电贷支持
- ✅ 价格预言机
- ✅ 现代化界面

下一步：
1. 部署到测试网
2. 添加更多交易对
3. 启动流动性挖矿
4. 集成更多钱包

**Happy Trading! 🚀**
