# 第二章：哈希计算

## 学习目标

- 理解哈希函数的作用
- 学习 SHA256 算法
- 掌握 Rust 的十六进制编码

## 什么是哈希？

哈希函数是把任意长度的数据转换成固定长度的"指纹"。

```
输入: "Hello, World!"
      ↓ SHA256
输出: "dffd6021bb2bd5b0af676290809ec3a53191dd81c7f70a4b28688a362182986f"
```

哈希的特点：
1. **确定性**：相同输入 → 相同输出
2. **不可逆**：无法从哈希反推原文
3. **抗碰撞**：很难找到两个不同的输入有相同的哈希
4. **雪崩效应**：输入微小变化 → 输出巨大变化

## 代码实现

### 1. 添加依赖

```toml
[dependencies]
sha2 = "0.10"    # SHA256 算法
hex = "0.4"      # 十六进制编码
chrono = "0.4"
```

### 2. 计算区块哈希

```rust
use sha2::{Sha256, Digest};

impl Block {
    /// 计算区块哈希
    fn calculate_hash(&self) -> String {
        // 1. 把区块数据拼接成字符串
        let data = format!(
            "{}{}{}{}",
            self.index, self.timestamp, self.data, self.prev_hash
        );
        
        // 2. 创建 SHA256 哈希器
        let mut hasher = Sha256::new();
        
        // 3. 写入数据
        hasher.update(data.as_bytes());
        
        // 4. 获取结果（字节数组）
        let result = hasher.finalize();
        
        // 5. 转成十六进制字符串
        hex::encode(result)
    }
}
```

### 3. 完整代码

```rust
use sha2::{Sha256, Digest};
use std::fmt;

#[derive(Debug, Clone)]
struct Block {
    index: u64,
    timestamp: i64,
    data: String,
    prev_hash: String,
    hash: String,
}

impl Block {
    fn new(index: u64, data: String, prev_hash: String) -> Self {
        let timestamp = chrono::Utc::now().timestamp();
        
        let mut block = Block {
            index,
            timestamp,
            data,
            prev_hash,
            hash: String::new(),
        };
        
        // 计算哈希
        block.hash = block.calculate_hash();
        
        block
    }
    
    fn calculate_hash(&self) -> String {
        let data = format!(
            "{}{}{}{}",
            self.index, self.timestamp, self.data, self.prev_hash
        );
        
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        
        hex::encode(result)
    }
}
```

## 运行结果

```
创世区块:
  Index: 0
  Hash:  a3f5c2e8d9b1... (64位十六进制)
```

## 知识点

### 1. SHA256

- 输出长度：256 位（32 字节）
- 编码后：64 位十六进制字符串
- 用途：数据完整性验证、区块链、密码学

### 2. 十六进制编码

```
二进制:  1101 1111 1101 1110
十六进制:   d    f    d    e
```

### 3. 雪崩效应演示

```rust
let hash1 = sha256("abc");
let hash2 = sha256("abd"); // 只改了一个字母

// hash1 和 hash2 完全不同！
```

## 练习

1. 验证同一个区块计算出的哈希是否相同
2. 修改区块数据，观察哈希变化
3. 实现一个简单的工作量证明（找前导零）

## 答案

<details>
<summary>点击查看答案</summary>

```rust
/// 验证哈希
fn verify_hash(&self) -> bool {
    self.hash == self.calculate_hash()
}

/// 简单的工作量证明：找前导零
fn mine(&mut self, difficulty: usize) {
    let target = "0".repeat(difficulty);
    
    loop {
        self.hash = self.calculate_hash();
        if self.hash.starts_with(&target) {
            break;
        }
        // 改变 nonce 或数据重新计算
        self.timestamp = chrono::Utc::now().timestamp();
    }
}
```

</details>

## 下一章

👉 [第三章：区块链](../03-blockchain/)

将多个区块链接起来，形成不可篡改的链条。
