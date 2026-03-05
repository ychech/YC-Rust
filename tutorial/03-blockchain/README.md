# 第三章：区块链

## 学习目标

- 理解链式数据结构
- 实现区块链接
- 理解不可篡改性

## 区块链结构

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│  Block #0   │────▶│  Block #1   │────▶│  Block #2   │
│  Genesis    │     │  Prev: #0   │     │  Prev: #1   │
│  Hash: abc  │     │  Hash: def  │     │  Hash: ghi  │
└─────────────┘     └─────────────┘     └─────────────┘
```

每个区块包含前一个区块的哈希，形成链条。

## 核心代码

```rust
struct Blockchain {
    chain: Vec<Block>,
}

impl Blockchain {
    fn new() -> Self {
        // 创世区块
        let genesis = Block::new(0, "Genesis".to_string(), "0".to_string());
        Blockchain { chain: vec![genesis] }
    }
    
    fn add_block(&mut self, data: String) {
        let last = self.get_last_block();
        let new_block = Block::new(
            last.index + 1,
            data,
            last.hash.clone(),  // 链接前一个区块
        );
        self.chain.push(new_block);
    }
}
```

## 验证机制

```rust
fn is_valid(&self) -> bool {
    for i in 1..self.chain.len() {
        let current = &self.chain[i];
        let previous = &self.chain[i - 1];
        
        // 1. 检查哈希
        if current.hash != current.calculate_hash() {
            return false;  // 数据被篡改
        }
        
        // 2. 检查链接
        if current.prev_hash != previous.hash {
            return false;  // 链接断裂
        }
    }
    true
}
```

## 为什么不可篡改？

如果篡改 Block #1：

1. Block #1 的哈希改变
2. Block #2 的 `prev_hash` 不匹配
3. 整个链失效

要篡改一个区块，必须重新计算后面所有区块的哈希，这在计算上不可行。

## 下一章

👉 [第四章：工作量证明](../04-pow/)

学习如何通过计算难题保护区块链安全。
