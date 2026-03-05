# 第一章：区块结构

## 学习目标

- 理解区块的基本组成
- 学习 Rust 结构体定义
- 掌握 Rust 的 Debug 和 Display trait

## 区块是什么？

区块是区块链的基本单位，就像账本中的一页。每个区块包含：

```
┌─────────────────────────────────────┐
│              Block                  │
├─────────────────────────────────────┤
│  Index        区块高度（第几个块）   │
│  Timestamp    创建时间戳            │
│  Data         交易数据              │
│  Prev Hash    前一个区块的哈希      │
│  Hash         本区块的哈希          │
└─────────────────────────────────────┘
```

## 代码实现

### 1. 创建项目

```bash
cargo new block-demo
cd block-demo
```

### 2. 定义区块结构

```rust
// src/main.rs

/// 区块结构
#[derive(Debug)]  // 自动生成调试输出
struct Block {
    index: u64,        // 区块高度
    timestamp: i64,    // 时间戳
    data: String,      // 数据
    prev_hash: String, // 前一个区块哈希
    hash: String,      // 本区块哈希
}

impl Block {
    /// 创建新区块
    fn new(index: u64, data: String, prev_hash: String) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        
        Block {
            index,
            timestamp,
            data,
            prev_hash,
            hash: String::new(), // 暂时为空
        }
    }
}

fn main() {
    // 创建创世区块（第一个区块）
    let genesis = Block::new(
        0,
        "Genesis Block".to_string(),
        "0".to_string(),
    );
    
    println!("创世区块: {:?}", genesis);
}
```

### 3. 添加依赖

```toml
# Cargo.toml
[dependencies]
chrono = "0.4"
```

### 4. 运行

```bash
cargo run
```

输出：
```
创世区块: Block { index: 0, timestamp: 1234567890, data: "Genesis Block", prev_hash: "0", hash: "" }
```

## 知识点讲解

### 1. `#[derive(Debug)]`

自动实现 `Debug` trait，让我们可以用 `{:?}` 打印结构体内容。

### 2. `impl` 块

为结构体实现方法。`new` 是构造函数惯例。

### 3. 所有权

- `String` 拥有数据的所有权
- `&str` 是字符串切片（借用）

### 4. 时间戳

使用 `chrono` 库获取当前 Unix 时间戳（从 1970-01-01 开始的秒数）。

## 练习

1. 为 Block 添加 `display` 方法，美化输出格式
2. 创建第二个区块，链接到创世区块
3. 添加验证方法，检查区块数据是否有效

## 答案

<details>
<summary>点击查看答案</summary>

```rust
use std::fmt;

impl Block {
    // ... new 方法 ...
    
    /// 美化打印
    fn display(&self) {
        println!("╔════════════════════════════════╗");
        println!("║ Block #{:<24} ║", self.index);
        println!("╠════════════════════════════════╣");
        println!("║ Time:  {:<23} ║", self.timestamp);
        println!("║ Data:  {:<23} ║", self.data);
        println!("║ Prev:  {:<23} ║", self.prev_hash);
        println!("╚════════════════════════════════╝");
    }
}

fn main() {
    let genesis = Block::new(0, "Genesis".to_string(), "0".to_string());
    genesis.display();
    
    let block1 = Block::new(1, "Block 1".to_string(), "hash_of_genesis".to_string());
    block1.display();
}
```

</details>

## 下一章

👉 [第二章：哈希计算](../02-hash/)

我们将学习如何计算区块的哈希值，这是区块链不可篡改的基础。
