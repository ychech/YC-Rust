# 02. 密码学基础 - 区块链的安全基石

## 本章目标

理解区块链中使用的核心密码学技术：哈希函数、Merkle树、数字签名、地址生成。

## 为什么需要密码学？

区块链的安全性完全建立在密码学之上：

```
┌─────────────────────────────────────────────────────────────┐
│                    区块链安全架构                            │
├─────────────────────────────────────────────────────────────┤
│  数据完整性  ←  哈希函数 (SHA256)                            │
│  身份认证    ←  数字签名 (ECDSA)                             │
│  隐私保护    ←  公钥加密                                     │
│  不可篡改    ←  哈希链 + 工作量证明                          │
└─────────────────────────────────────────────────────────────┘
```

## 核心概念详解

### 1. 哈希函数

哈希函数是区块链的"指纹生成器"。

#### SHA256 特性

```rust
/// SHA256 核心特性
/// 
/// 1. 确定性：相同输入 → 相同输出
///    SHA256("hello") = "2cf24dba5fb0a30e26e83b2ac5b9e29e..."
/// 
/// 2. 快速计算：输入任意大小，输出固定 256 位
///    输入：1GB 文件 → 输出：32 字节
/// 
/// 3. 不可逆：无法从哈希反推原始数据
///    给定 H(x)，计算上不可行找到 x
/// 
/// 4. 雪崩效应：微小变化导致完全不同输出
///    SHA256("hello")  ≠  SHA256("Hello")
///    SHA256("hello")  ≠  SHA256("hello ")
/// 
/// 5. 抗碰撞：很难找到两个不同输入产生相同输出
///    找到 x ≠ y 使得 H(x) = H(y) 极其困难
```

#### 实际演示

```rust
use sha2::{Sha256, Digest};

fn main() {
    // 示例 1：基本哈希
    let data = "Hello, Blockchain!";
    let hash = sha256(data);
    println!("SHA256(\"{}\") = {}", data, hash);
    // 输出：a8f5f167f44f4964e6c998dee827110c...
    
    // 示例 2：雪崩效应
    let data1 = "hello";
    let data2 = "Hello"; // 只改了大小写
    
    println!("\n雪崩效应演示：");
    println!("SHA256(\"{}\") = {}", data1, sha256(data1));
    println!("SHA256(\"{}\") = {}", data2, sha256(data2));
    // 两个哈希完全不同！
    
    // 示例 3：哈希链
    let mut current = "genesis".to_string();
    println!("\n哈希链演示：");
    for i in 0..5 {
        current = sha256(&current);
        println!("Block {}: {}", i, &current[..16]);
    }
}
```

### 2. Merkle 树

Merkle 树是区块链高效验证的关键数据结构。

#### 为什么需要 Merkle 树？

```
问题：如何验证一笔交易是否在区块中？

方案 1：下载所有交易
- 需要下载 2000 笔交易
- 数据量大，效率低

方案 2：Merkle 证明
- 只需下载区块头（80字节）
- 加上 Merkle 路径（约 10 个哈希）
- 总共几百字节！
```

#### Merkle 树构建过程

```rust
/// Merkle 树构建示例
/// 
/// 假设有 4 笔交易：Tx1, Tx2, Tx3, Tx4
/// 
/// 步骤 1：计算叶子节点哈希
///     H1 = SHA256(Tx1)
///     H2 = SHA256(Tx2)
///     H3 = SHA256(Tx3)
///     H4 = SHA256(Tx4)
/// 
/// 步骤 2：计算父节点
///     H12 = SHA256(H1 + H2)
///     H34 = SHA256(H3 + H4)
/// 
/// 步骤 3：计算根节点
///     Root = SHA256(H12 + H34)
/// 
/// 最终树结构：
/// 
///              Root
///            /      \
///         H12        H34
///        /   \      /   \
///      H1     H2   H3    H4
///      |      |    |     |
///     Tx1   Tx2  Tx3   Tx4
/// 
/// 验证 Tx2 是否在区块中：
/// 1. 计算 H2 = SHA256(Tx2)
/// 2. 获取 H1（来自证明）
/// 3. 计算 H12 = SHA256(H1 + H2)
/// 4. 获取 H34（来自证明）
/// 5. 计算 Root = SHA256(H12 + H34)
/// 6. 比较 Root 是否等于区块头中的 Merkle 根
```

### 3. 数字签名 (ECDSA)

数字签名证明交易确实由私钥持有者发起。

#### 密钥对生成

```rust
/// 椭圆曲线 secp256k1
/// 
/// 比特币和以太坊使用的椭圆曲线。
/// 
/// 曲线方程：y² = x³ + 7
/// 
/// 参数：
/// - 素数 p = 2^256 - 2^32 - 2^9 - 2^8 - 2^7 - 2^6 - 2^4 - 1
/// - 基点 G（曲线上的一点）
/// - 阶数 n（G 的阶）
/// 
/// 密钥生成：
/// 1. 随机选择私钥 d（1 ≤ d ≤ n-1）
/// 2. 计算公钥 Q = d × G（椭圆曲线点乘）

use secp256k1::{Secp256k1, SecretKey, PublicKey};

fn generate_keypair() -> (SecretKey, PublicKey) {
    let secp = Secp256k1::new();
    
    // 生成随机私钥
    let secret_key = SecretKey::new(&mut rand::thread_rng());
    
    // 计算公钥
    let public_key = PublicKey::from_secret_key(&secp, &secret_key);
    
    (secret_key, public_key)
}
```

#### 签名过程

```rust
/// ECDSA 签名
/// 
/// 签名消息 m：
/// 
/// 1. 计算消息哈希：z = SHA256(m)
/// 2. 随机选择 k（1 ≤ k ≤ n-1）
/// 3. 计算点 R = k × G = (x, y)
/// 4. 令 r = x mod n，如果 r = 0 重新选择 k
/// 5. 计算 s = k^(-1) × (z + r × d) mod n
///    其中 d 是私钥
/// 6. 如果 s = 0 重新选择 k
/// 7. 签名为 (r, s)
/// 
/// 安全性关键：
/// - 每次签名必须使用不同的随机数 k
/// - 如果 k 重复，可以计算出私钥！

fn sign_message(private_key: &SecretKey, message: &[u8]) -> Signature {
    let secp = Secp256k1::new();
    
    // 计算消息哈希
    let message_hash = sha256_bytes(message);
    let msg = Message::from_slice(&message_hash).unwrap();
    
    // 签名
    secp.sign(&msg, private_key)
}
```

#### 验证过程

```rust
/// ECDSA 验证
/// 
/// 验证签名 (r, s) 对消息 m：
/// 
/// 1. 验证：1 ≤ r, s ≤ n-1
/// 2. 计算消息哈希：z = SHA256(m)
/// 3. 计算 w = s^(-1) mod n
/// 4. 计算 u1 = z × w mod n
/// 5. 计算 u2 = r × w mod n
/// 6. 计算点 P = u1 × G + u2 × Q
///    其中 Q 是公钥
/// 7. 如果 P = O（无穷远点），验证失败
/// 8. 令 v = Px mod n
/// 9. 验证 v == r

fn verify_signature(
    public_key: &PublicKey,
    message: &[u8],
    signature: &Signature,
) -> bool {
    let secp = Secp256k1::new();
    
    let message_hash = sha256_bytes(message);
    let msg = Message::from_slice(&message_hash).unwrap();
    
    secp.verify(&msg, signature, public_key).is_ok()
}
```

### 4. 地址生成

地址是公钥的"压缩表示"，便于使用和分享。

#### 地址生成流程

```rust
/// 地址生成（类似比特币）
/// 
/// 步骤：
/// 1. 生成私钥（32字节随机数）
/// 2. 计算公钥（secp256k1）
/// 3. SHA256(公钥)
/// 4. RIPEMD160(SHA256结果) = 20字节哈希
/// 5. 添加版本字节（0x00 主网，0x6f 测试网）
/// 6. 计算校验和 = SHA256(SHA256(版本+哈希)) 的前4字节
/// 7. 拼接：版本 + 哈希 + 校验和
/// 8. Base58Check 编码
/// 
/// 最终地址格式：
/// - 主网：1...（26-35字符）
/// - 测试网：m... 或 n...
/// 
/// YRUST Chain 地址格式：
/// - 以 "YR" 开头
/// - Base58Check 编码
/// - 包含版本、哈希、校验和

fn generate_address(public_key: &PublicKey) -> String {
    // 1. 公钥转字节
    let pub_key_bytes = public_key.serialize();
    
    // 2. SHA256
    let sha256_result = sha256_bytes(&pub_key_bytes);
    
    // 3. RIPEMD160
    let ripemd160_result = ripemd160(&sha256_result);
    
    // 4. 添加版本字节（YRUST Chain 版本 = 0x4D）
    let mut versioned = vec![0x4D];
    versioned.extend_from_slice(&ripemd160_result);
    
    // 5. 计算校验和
    let checksum = double_sha256_bytes(&versioned);
    let checksum = &checksum[..4];
    
    // 6. 拼接
    let mut full = versioned;
    full.extend_from_slice(checksum);
    
    // 7. Base58 编码
    base58_encode(&full)
}
```

#### 地址验证

```rust
/// 验证地址格式
/// 
/// 检查：
/// 1. 长度合理（26-35字符）
/// 2. Base58 字符集
/// 3. 校验和正确
fn validate_address(address: &str) -> bool {
    // 1. Base58 解码
    let decoded = match base58_decode(address) {
        Ok(d) => d,
        Err(_) => return false,
    };
    
    // 2. 检查长度（1版本 + 20哈希 + 4校验和 = 25字节）
    if decoded.len() != 25 {
        return false;
    }
    
    // 3. 检查版本
    if decoded[0] != 0x4D {
        return false;
    }
    
    // 4. 验证校验和
    let payload = &decoded[..21];
    let checksum = &decoded[21..];
    
    let calculated = double_sha256_bytes(payload);
    let calculated = &calculated[..4];
    
    checksum == calculated
}
```

## 完整代码实现

见 [main.rs](main.rs)

## 运行测试

```bash
cd tutorial-full/02-crypto
cargo run
```

预期输出：
```
SHA256 特性演示:
SHA256("Hello") = 185f8db32271fe25f561a6fc938b2e26...
SHA256("hello") = 2cf24dba5fb0a30e26e83b2ac5b9e29e...

雪崩效应:
SHA256("block") = d63dc... 
SHA256("Block") = 8a5ed...
完全不同！

Merkle 树构建:
交易: [Tx1, Tx2, Tx3, Tx4]
Merkle 根: a3f2b8c1d4e5...

验证 Tx2:
路径: [H1, H34]
计算根: a3f2b8c1d4e5...
验证成功！

密钥对生成:
私钥: 5Kb8kLf9zgWQnogidDA76MzPL6TsZZY36hWXMssSzNydYXYBzKF
公钥: 0478d430274f8c5ec1321338151e9f27f4c676a008bdf8638d07c0b6be9ab35c71a1518063243acd4dfe96b66e3f2ec8013c8e072cd09b3834a19f81f659cc3455
地址: YR1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa

签名验证:
消息: "转账 100 YRU 给 Bob"
签名: 3045022100...
验证结果: ✅ 有效
```

## 练习题

### 练习 1：哈希特性验证
修改输入的每一个字符，观察哈希变化。

### 练习 2：Merkle 证明
实现一个函数，生成某笔交易的 Merkle 证明路径。

### 练习 3：地址生成
生成 10 个地址，验证它们都不相同且格式正确。

### 练习 4：签名安全
尝试用相同的随机数 k 签名两条消息，观察是否安全。

## 挑战任务

🔥 **实现轻节点**
- 只下载区块头（不下载完整区块）
- 验证交易是否包含在区块中
- 使用 Merkle 证明

## 下一章

[03. 区块链核心](../03-blockchain/) - 学习链式结构、POW挖矿、难度调整
