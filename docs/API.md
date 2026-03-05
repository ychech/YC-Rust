# YRUST Chain API 文档

## 概述

YRUST Chain 提供 RESTful API 和 WebSocket 接口，用于与区块链交互。

- **Base URL**: `http://localhost:8080`
- **Content-Type**: `application/json`

## 认证

部分接口需要 API Key 认证：

```
Authorization: Bearer YOUR_API_KEY
```

## 通用响应格式

```json
{
  "success": true,
  "data": {},
  "error": null
}
```

错误响应：

```json
{
  "success": false,
  "data": null,
  "error": {
    "code": "INVALID_REQUEST",
    "message": "请求参数错误"
  }
}
```

---

## 区块接口

### 获取最新区块

```
GET /api/v1/blocks/latest
```

**响应**：

```json
{
  "success": true,
  "data": {
    "hash": "0000a1b2c3d4e5f6...",
    "height": 1234567,
    "timestamp": 1704067200,
    "transactions": 128,
    "miner": "YR123456789...",
    "reward": 50,
    "difficulty": 4
  }
}
```

### 获取区块详情

```
GET /api/v1/blocks/{height_or_hash}
```

**参数**：
- `height_or_hash`: 区块高度或哈希值

**响应**：

```json
{
  "success": true,
  "data": {
    "hash": "0000a1b2c3d4e5f6...",
    "height": 1234567,
    "prev_hash": "00009a8b7c6d5e4f...",
    "timestamp": 1704067200,
    "nonce": 12345,
    "difficulty": 4,
    "merkle_root": "a1b2c3d4e5f6...",
    "transactions": [
      {
        "hash": "tx1a2b3c4d5e6f7...",
        "from": "YR111111111...",
        "to": "YR222222222...",
        "amount": 100.5,
        "fee": 0.001,
        "timestamp": 1704067195
      }
    ]
  }
}
```

### 获取区块列表

```
GET /api/v1/blocks?page=1&limit=20
```

**参数**：
- `page`: 页码 (默认: 1)
- `limit`: 每页数量 (默认: 20, 最大: 100)

---

## 交易接口

### 发送交易

```
POST /api/v1/transactions
```

**请求体**：

```json
{
  "from": "YR111111111...",
  "to": "YR222222222...",
  "amount": "100.5",
  "fee": "0.001",
  "nonce": 5,
  "timestamp": 1704067200,
  "signature": "0x..."
}
```

**响应**：

```json
{
  "success": true,
  "data": {
    "tx_hash": "tx1a2b3c4d5e6f7...",
    "status": "pending",
    "block_height": null
  }
}
```

### 获取交易详情

```
GET /api/v1/transactions/{tx_hash}
```

**响应**：

```json
{
  "success": true,
  "data": {
    "hash": "tx1a2b3c4d5e6f7...",
    "from": "YR111111111...",
    "to": "YR222222222...",
    "amount": 100.5,
    "fee": 0.001,
    "nonce": 5,
    "timestamp": 1704067200,
    "block_height": 1234567,
    "block_hash": "0000a1b2c3d4e5f6...",
    "confirmations": 12,
    "status": "confirmed"
  }
}
```

### 获取待处理交易

```
GET /api/v1/transactions/pending
```

---

## 账户接口

### 获取账户余额

```
GET /api/v1/accounts/{address}/balance
```

**响应**：

```json
{
  "success": true,
  "data": {
    "address": "YR111111111...",
    "balance": 10000.5,
    "nonce": 5,
    "pending_tx_count": 0
  }
}
```

### 获取账户交易历史

```
GET /api/v1/accounts/{address}/transactions?page=1&limit=20
```

---

## UTXO 接口

### 获取 UTXO 列表

```
GET /api/v1/utxos/{address}
```

**响应**：

```json
{
  "success": true,
  "data": {
    "address": "YR111111111...",
    "utxos": [
      {
        "tx_hash": "tx1a2b3c4d5e6f7...",
        "output_index": 0,
        "amount": 50,
        "confirmations": 100
      }
    ],
    "total_amount": 150
  }
}
```

---

## 智能合约接口

### 部署合约

```
POST /api/v1/contracts/deploy
```

**请求体**：

```json
{
  "from": "YR111111111...",
  "code": "0x0061736d...",
  "abi": {...},
  "gas_limit": 1000000,
  "gas_price": "0.0001",
  "signature": "0x..."
}
```

### 调用合约

```
POST /api/v1/contracts/{contract_address}/call
```

**请求体**：

```json
{
  "from": "YR111111111...",
  "function": "transfer",
  "args": ["YR222222222...", "100"],
  "gas_limit": 100000,
  "gas_price": "0.0001",
  "signature": "0x..."
}
```

### 查询合约状态

```
GET /api/v1/contracts/{contract_address}/query?function=balanceOf&args=YR111111111...
```

---

## DEX 接口

### 获取交易对列表

```
GET /api/v1/dex/pairs
```

**响应**：

```json
{
  "success": true,
  "data": {
    "pairs": [
      {
        "token_a": "YRU",
        "token_b": "USDT",
        "reserve_a": 1000000,
        "reserve_b": 500000,
        "lp_token": "YRU-USDT-LP"
      }
    ]
  }
}
```

### 获取价格

```
GET /api/v1/dex/price?token_in=YRU&token_out=USDT&amount_in=100
```

### 添加流动性

```
POST /api/v1/dex/liquidity/add
```

**请求体**：

```json
{
  "from": "YR111111111...",
  "token_a": "YRU",
  "token_b": "USDT",
  "amount_a": 1000,
  "amount_b": 500,
  "signature": "0x..."
}
```

### 交换代币

```
POST /api/v1/dex/swap
```

**请求体**：

```json
{
  "from": "YR111111111...",
  "token_in": "YRU",
  "token_out": "USDT",
  "amount_in": 100,
  "min_amount_out": 49,
  "signature": "0x..."
}
```

---

## 网络接口

### 获取网络状态

```
GET /api/v1/network/status
```

**响应**：

```json
{
  "success": true,
  "data": {
    "peer_count": 42,
    "is_syncing": false,
    "latest_block": 1234567,
    "network_id": "yrust-mainnet-1",
    "version": "1.0.0"
  }
}
```

### 获取节点列表

```
GET /api/v1/network/peers
```

---

## 节点管理接口 (需要认证)

### 获取节点信息

```
GET /api/v1/node/info
```

### 获取内存池

```
GET /api/v1/node/mempool
```

### 添加节点

```
POST /api/v1/node/peers
```

**请求体**：

```json
{
  "peer_id": "12D3KooW...",
  "address": "/ip4/192.168.1.1/tcp/9090"
}
```

---

## WebSocket 接口

连接地址：`ws://localhost:8080/ws`

### 订阅新区块

```json
{
  "action": "subscribe",
  "channel": "blocks"
}
```

### 订阅新交易

```json
{
  "action": "subscribe",
  "channel": "transactions"
}
```

### 订阅地址交易

```json
{
  "action": "subscribe",
  "channel": "address:YR111111111..."
}
```

### 取消订阅

```json
{
  "action": "unsubscribe",
  "channel": "blocks"
}
```

---

## 错误码

| 错误码 | 描述 |
|--------|------|
| `INVALID_REQUEST` | 请求参数错误 |
| `INVALID_SIGNATURE` | 签名无效 |
| `INSUFFICIENT_BALANCE` | 余额不足 |
| `INVALID_NONCE` | Nonce 无效 |
| `TRANSACTION_EXPIRED` | 交易已过期 |
| `RATE_LIMIT_EXCEEDED` | 请求过于频繁 |
| `NOT_FOUND` | 资源不存在 |
| `INTERNAL_ERROR` | 内部错误 |

---

## 速率限制

- 普通接口：60 请求/分钟
- 发送交易：10 请求/分钟
- 节点管理：需要 API Key

---

## SDK 示例

### JavaScript

```javascript
const YRUST = require('yrust-sdk');

const client = new YRUST.Client('http://localhost:8080');

// 获取余额
const balance = await client.getBalance('YR111111111...');

// 发送交易
const tx = await client.sendTransaction({
  from: 'YR111111111...',
  to: 'YR222222222...',
  amount: 100,
  privateKey: '...'
});
```

### Python

```python
from yrust import Client

client = Client('http://localhost:8080')

# 获取最新区块
block = client.get_latest_block()

# 发送交易
tx_hash = client.send_transaction(
    from_addr='YR111111111...',
    to_addr='YR222222222...',
    amount=100,
    private_key='...'
)
```

### Rust

```rust
use yrust_client::Client;

let client = Client::new("http://localhost:8080");

// 获取区块
let block = client.get_block_by_height(1234567).await?;

// 发送交易
let tx_hash = client
    .send_transaction(&tx, &signature)
    .await?;
```

---

## 更新日志

### v1.0.0 (2024-01-01)
- 初始版本发布
- 支持区块、交易、账户查询
- 支持智能合约部署和调用
- 支持 DEX 交易

### v1.1.0 (2024-02-01)
- 添加 WebSocket 实时推送
- 优化查询性能
- 添加批量查询接口
