# 03. 区块链核心 - 链式结构与POW挖矿

## 本章目标

理解区块链的链式结构，实现工作量证明（POW）挖矿机制。

## 什么是区块链？

区块链是一个分布式的、不可篡改的账本，由一系列区块通过哈希链接而成。

```
┌─────────┐    ┌─────────┐    ┌─────────┐    ┌─────────┐
│ 区块 0  │───→│ 区块 1  │───→│ 区块 2  │───→│ 区块 3  │
│ (创世)  │    │         │    │         │    │  (最新) │
│ Hash: 0a│    │ Prev: 0a│    │ Prev: 1b│    │ Prev: 2c│
│         │    │ Hash: 1b│    │ Hash: 2c│    │ Hash: 3d│
└─────────┘    └─────────┘    └─────────┘    └─────────┘

每个区块包含前一区块的哈希，形成链条
修改任一区块会导致后续所有区块哈希变化
```

## 核心概念详解

### 1. 链式结构

```rust
/// 区块链结构
/// 
/// 维护一个有序的区块列表，提供链操作功能
pub struct Blockchain {
    /// 区块列表，按高度排序
    /// 索引0是创世区块
    pub blocks: Vec<Block>,
    
    /// UTXO集合 - 快速查询未花费输出
    pub utxo_set: HashMap<String, TxOutput>,
    
    /// 内存池 - 待确认交易
    pub mempool: Vec<Transaction>,
    
    /// 当前挖矿难度
    pub difficulty: u32,
}
```

### 2. 工作量证明（POW）

POW是比特币使用的共识机制：

```
挖矿 = 寻找满足难度要求的nonce

难度4示例：
目标：哈希以 "0000" 开头
尝试：
  nonce=1:   hash=8a7b6c5d... ❌
  nonce=2:   hash=3e4f5a6b... ❌
  ...
  nonce=5234: hash=0000a1b2... ✅ 找到！

计算量：平均需要 16^4 = 65536 次尝试
```

### 3. 难度调整

```rust
/// 难度调整算法
/// 
/// 比特币每2016个区块（约2周）调整一次难度
/// 目标是保持平均出块时间为10分钟
/// 
/// 公式：
/// 新难度 = 旧难度 × (实际时间) / (目标时间)
/// 
/// 限制：单次调整不超过4倍或1/4
pub fn adjust_difficulty(&mut self) {
    let blocks_count = 2016; // 调整周期
    
    if self.blocks.len() % blocks_count == 0 {
        let last_block = self.latest_block();
        let first_block = &self.blocks[self.blocks.len() - blocks_count];
        
        let actual_time = last_block.timestamp - first_block.timestamp;
        let target_time = blocks_count as u64 * 600; // 10分钟 × 2016
        
        // 计算新难度
        let new_difficulty = (self.difficulty as u64 * actual_time) / target_time;
        
        // 限制调整幅度
        let max_difficulty = self.difficulty as u64 * 4;
        let min_difficulty = self.difficulty as u64 / 4;
        
        self.difficulty = (new_difficulty.clamp(min_difficulty, max_difficulty)) as u32;
        self.difficulty = self.difficulty.max(1); // 最小难度为1
    }
}
```

## 代码实现

见 [main.rs](main.rs)

## 练习题

### 练习1：验证链完整性
实现一个函数，验证从创世区块到最新区块的完整性。

### 练习2：模拟难度调整
创建100个区块，观察难度如何随出块速度变化。

### 练习3：链重组
实现一个函数，处理分叉情况下的链重组。

## 挑战任务

🔥 **实现轻节点**
- 只下载区块头（不下载完整区块）
- 使用SPV验证交易

## 下一章

[04. P2P网络](../04-network/) - 学习libp2p、节点发现、消息传播
