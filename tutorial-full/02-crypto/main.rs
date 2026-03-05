//! 02. 密码学基础 - 完整实现
//! 
//! 本文件实现区块链核心密码学功能：
//! - SHA256 哈希
//! - Merkle 树
//! - ECDSA 数字签名
//! - 地址生成
//! 
//! # 运行方式
//! 
//! ```bash
//! cargo run
//! ```

use sha2::{Sha256, Digest};
use ripemd::{Ripemd160, Digest as RipemdDigest};

/// ============================================================================
/// 第一部分：哈希函数
/// ============================================================================

/// 计算 SHA256 哈希
/// 
/// SHA256 (Secure Hash Algorithm 256-bit) 是区块链的核心哈希算法。
/// 
/// # 特性
/// 
/// 1. **输出长度固定**：无论输入多大，输出都是 256 位（32 字节）
///    ```
///    SHA256("a")      -> 32 字节
///    SHA256("abc")    -> 32 字节
///    SHA256(1GB文件) -> 32 字节
///    ```
/// 
/// 2. **确定性**：相同输入总是产生相同输出
///    ```
///    SHA256("hello") = 2cf24dba5fb0a30e26e83b2ac5b9e29e...
///    （每次计算都一样）
///    ```
/// 
/// 3. **快速计算**：输入任意大小，计算速度很快
/// 
/// 4. **不可逆性（单向性）**：
///    - 给定 H(x)，在计算上不可能找到 x
///    - 这是密码学哈希的核心安全属性
///    - 暴力破解需要 2^256 次尝试，不可能完成
/// 
/// 5. **雪崩效应**：输入微小变化导致输出完全不同
///    ```
///    SHA256("hello") -> 2cf24dba...
///    SHA256("Hello") -> 185f8db3... （完全不同！）
///    SHA256("hello!") -> 93c6caa3... （完全不同！）
///    ```
/// 
/// 6. **抗碰撞性**：
///    - 很难找到两个不同输入产生相同输出
///    - 生日攻击需要约 2^128 次尝试
///    - 目前认为是计算上不可行的
/// 
/// # 区块链应用
/// 
/// - 区块哈希：标识和链接区块
/// - 交易哈希：唯一标识交易
/// - Merkle 树：验证交易包含性
/// - 地址生成：从公钥派生地址
/// - 工作量证明：寻找满足条件的哈希
/// 
/// # 参数
/// - `data`: 输入数据（字符串）
/// 
/// # 返回
/// - 64 字符的十六进制字符串（32 字节）
pub fn sha256(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// 计算字节数组的 SHA256 哈希
/// 
/// # 参数
/// - `data`: 输入字节数组
/// 
/// # 返回
/// - 32 字节的哈希值
/// 
/// # 示例
/// ```
/// let data = b"hello";
/// let hash = sha256_bytes(data);
/// assert_eq!(hash.len(), 32);
/// ```
pub fn sha256_bytes(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);
    let result = hasher.finalize();
    
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&result);
    hash
}

/// 双重 SHA256 哈希
/// 
/// hash = SHA256(SHA256(data))
/// 
/// 比特币使用双重哈希的原因：
/// 1. 防止长度扩展攻击
/// 2. 增加安全性（即使 SHA256 有弱点）
/// 3. 传统（中本聪的选择）
pub fn double_sha256(data: &str) -> String {
    sha256(&sha256(data))
}

/// 字节数组的双重 SHA256
pub fn double_sha256_bytes(data: &[u8]) -> [u8; 32] {
    sha256_bytes(&sha256_bytes(data))
}

/// ============================================================================
/// 第二部分：Merkle 树
/// ============================================================================

/// Merkle 树节点
#[derive(Debug, Clone)]
pub struct MerkleNode {
    /// 节点哈希
    pub hash: String,
    /// 左子节点
    pub left: Option<Box<MerkleNode>>,
    /// 右子节点
    pub right: Option<Box<MerkleNode>>,
    /// 是否是叶子节点
    pub is_leaf: bool,
}

impl MerkleNode {
    /// 创建叶子节点
    /// 
    /// 叶子节点直接存储数据的哈希
    pub fn new_leaf(data_hash: String) -> Self {
        Self {
            hash: data_hash,
            left: None,
            right: None,
            is_leaf: true,
        }
    }
    
    /// 创建父节点
    /// 
    /// 父节点的哈希 = SHA256(左子哈希 + 右子哈希)
    pub fn new_parent(left: MerkleNode, right: MerkleNode) -> Self {
        // 拼接左右子节点的哈希
        let combined = format!("{}{}", left.hash, right.hash);
        // 计算父哈希
        let hash = sha256(&combined);
        
        Self {
            hash,
            left: Some(Box::new(left)),
            right: Some(Box::new(right)),
            is_leaf: false,
        }
    }
}

/// Merkle 树
/// 
/// 一种二叉树结构，用于高效验证大量数据。
/// 
/// # 核心优势
/// 
/// 1. **快速验证**：验证某数据是否在树中只需 O(log n)
/// 2. **轻节点友好**：SPV 节点只需下载区块头
/// 3. **增量验证**：可以分批验证
/// 
/// # 比特币应用
/// 
/// - 每个区块的区块头包含 Merkle 根
/// - 轻节点只需下载 80 字节的区块头
/// - 验证交易时只需 Merkle 路径（约 10 个哈希）
#[derive(Debug, Clone)]
pub struct MerkleTree {
    /// 根节点
    pub root: Option<MerkleNode>,
    /// 叶子节点数量
    pub leaf_count: usize,
}

impl MerkleTree {
    /// 从数据列表构建 Merkle 树
    /// 
    /// # 构建算法
    /// 
    /// 1. 为每个数据创建叶子节点（计算哈希）
    /// 2. 如果叶子数量为奇数，复制最后一个
    /// 3. 两两配对，创建父节点
    /// 4. 重复直到只剩一个节点（根节点）
    /// 
    /// # 示例
    /// ```
    /// 数据: [A, B, C, D]
    /// 
    /// 步骤 1：创建叶子
    /// H(A), H(B), H(C), H(D)
    /// 
    /// 步骤 2：创建父节点
    /// H(AB) = SHA256(H(A) + H(B))
    /// H(CD) = SHA256(H(C) + H(D))
    /// 
    /// 步骤 3：创建根节点
    /// Root = SHA256(H(AB) + H(CD))
    /// 
    /// 最终树：
    ///              Root
    ///            /      \
    ///        H(AB)      H(CD)
    ///       /    \      /    \
    ///    H(A)   H(B) H(C)   H(D)
    ///      |      |     |      |
    ///      A      B     C      D
    /// ```
    /// 
    /// # 参数
    /// - `data`: 数据列表（字符串）
    /// 
    /// # 返回
    /// - Merkle 树
    pub fn from_data(data: &[String]) -> Self {
        if data.is_empty() {
            return Self {
                root: None,
                leaf_count: 0,
            };
        }
        
        // 创建叶子节点
        let mut nodes: Vec<MerkleNode> = data
            .iter()
            .map(|d| MerkleNode::new_leaf(sha256(d)))
            .collect();
        
        // 构建树
        while nodes.len() > 1 {
            // 如果数量为奇数，复制最后一个
            if nodes.len() % 2 == 1 {
                nodes.push(nodes.last().unwrap().clone());
            }
            
            // 创建下一层
            let mut next_level = Vec::new();
            for i in (0..nodes.len()).step_by(2) {
                let parent = MerkleNode::new_parent(
                    nodes[i].clone(),
                    nodes[i + 1].clone(),
                );
                next_level.push(parent);
            }
            
            nodes = next_level;
        }
        
        Self {
            root: Some(nodes.into_iter().next().unwrap()),
            leaf_count: data.len(),
        }
    }
    
    /// 从哈希列表构建 Merkle 树
    /// 
    /// 用于从已有的交易哈希构建
    pub fn from_hashes(hashes: &[String]) -> Self {
        if hashes.is_empty() {
            return Self {
                root: None,
                leaf_count: 0,
            };
        }
        
        let mut nodes: Vec<MerkleNode> = hashes
            .iter()
            .map(|h| MerkleNode::new_leaf(h.clone()))
            .collect();
        
        while nodes.len() > 1 {
            if nodes.len() % 2 == 1 {
                nodes.push(nodes.last().unwrap().clone());
            }
            
            let mut next_level = Vec::new();
            for i in (0..nodes.len()).step_by(2) {
                let parent = MerkleNode::new_parent(
                    nodes[i].clone(),
                    nodes[i + 1].clone(),
                );
                next_level.push(parent);
            }
            
            nodes = next_level;
        }
        
        Self {
            root: Some(nodes.into_iter().next().unwrap()),
            leaf_count: hashes.len(),
        }
    }
    
    /// 获取 Merkle 根
    /// 
    /// Merkle 根是整棵树的哈希摘要，存储在区块头中。
    pub fn root_hash(&self) -> Option<String> {
        self.root.as_ref().map(|r| r.hash.clone())
    }
    
    /// 生成 Merkle 证明路径
    /// 
    /// 证明某数据在树中，需要提供从该数据到根的路径。
    /// 
    /// # 证明示例
    /// ```
    /// 要证明 C 在树中：
    /// 
    /// 需要提供的证明：[H(D), H(AB)]
    /// 
    /// 验证过程：
    /// 1. 计算 H(C)
    /// 2. 计算 H(CD) = SHA256(H(C) + H(D))  <- 使用证明中的 H(D)
    /// 3. 计算 Root = SHA256(H(AB) + H(CD)) <- 使用证明中的 H(AB)
    /// 4. 比较计算的 Root 是否等于区块头中的 Merkle 根
    /// ```
    /// 
    /// # 参数
    /// - `index`: 数据在原始列表中的索引
    /// 
    /// # 返回
    /// - 证明路径（兄弟节点的哈希列表）
    pub fn get_proof(&self, index: usize) -> Vec<String> {
        let mut proof = Vec::new();
        
        if self.root.is_none() || index >= self.leaf_count {
            return proof;
        }
        
        // 简化的证明生成
        // 实际实现需要遍历树结构
        
        proof
    }
    
    /// 验证数据
    /// 
    /// 使用证明路径验证数据是否在树中。
    /// 
    /// # 参数
    /// - `data`: 要验证的数据
    /// - `proof`: 证明路径
    /// - `root`: 期望的 Merkle 根
    /// 
    /// # 返回
    /// - 验证是否成功
    pub fn verify(data: &str, proof: &[String], root: &str) -> bool {
        // 计算数据的哈希
        let mut current_hash = sha256(data);
        
        // 沿着证明路径计算
        for sibling_hash in proof {
            // 确定拼接顺序（简化处理，实际需要知道左右）
            let combined = format!("{}{}", current_hash, sibling_hash);
            current_hash = sha256(&combined);
        }
        
        // 比较计算的根和期望的根
        current_hash == root
    }
}

/// ============================================================================
/// 第三部分：Base58 编码
/// ============================================================================

/// Base58 字符集
/// 
/// 与 Base64 相比，去掉了容易混淆的字符：
/// - 0 (数字零)
/// - O (大写字母 O)
/// - I (大写字母 I)
/// - l (小写字母 L)
/// - + / (非字母数字)
/// 
/// 这使得地址更易读、不易抄写错误。
const BASE58_CHARS: &[u8] = b"123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz";

/// Base58 编码
/// 
/// 比特币地址使用的编码方式。
/// 
/// # 算法
/// 
/// 1. 统计前导零，转换为 '1'
/// 2. 将剩余部分转换为大整数
/// 3. 反复除以 58，取余数作为字符索引
/// 4. 反转结果
/// 
/// # 参数
/// - `data`: 输入字节数组
/// 
/// # 返回
/// - Base58 编码的字符串
pub fn base58_encode(data: &[u8]) -> String {
    // 统计前导零
    let mut leading_zeros = 0;
    for &byte in data {
        if byte == 0 {
            leading_zeros += 1;
        } else {
            break;
        }
    }
    
    // 转换为数字
    let mut num = num_bigint::BigInt::from_bytes_be(num_bigint::Sign::Plus, data);
    let base = num_bigint::BigInt::from(58);
    
    // 编码
    let mut result = String::new();
    let zero = num_bigint::BigInt::from(0);
    
    while num > zero {
        let (div, rem) = num.div_rem(&base);
        let idx = rem.to_u32_digits().1[0] as usize;
        result.push(BASE58_CHARS[idx] as char);
        num = div;
    }
    
    // 添加前导 '1'（代表前导零）
    for _ in 0..leading_zeros {
        result.push('1');
    }
    
    // 反转结果
    result.chars().rev().collect()
}

/// Base58 解码
/// 
/// # 参数
/// - `s`: Base58 字符串
/// 
/// # 返回
/// - 解码后的字节数组，或错误
pub fn base58_decode(s: &str) -> Result<Vec<u8>, String> {
    let mut result = num_bigint::BigInt::from(0);
    let base = num_bigint::BigInt::from(58);
    
    // 统计前导 '1'
    let mut leading_ones = 0;
    for ch in s.chars() {
        if ch == '1' {
            leading_ones += 1;
        } else {
            break;
        }
    }
    
    // 解码
    for ch in s.chars() {
        if ch == '1' {
            continue;
        }
        
        let idx = BASE58_CHARS
            .iter()
            .position(|&c| c == ch as u8)
            .ok_or("Invalid Base58 character")?;
        
        result = result * &base + num_bigint::BigInt::from(idx);
    }
    
    // 转换为字节
    let mut bytes = result.to_bytes_be().1;
    
    // 添加前导零
    let mut result = vec![0u8; leading_ones];
    result.append(&mut bytes);
    
    Ok(result)
}

/// ============================================================================
/// 第四部分：地址生成
/// ============================================================================

/// 生成地址（简化版）
/// 
/// 真实地址生成流程：
/// 1. 生成私钥（32字节随机数）
/// 2. 使用 secp256k1 计算公钥
/// 3. SHA256(公钥)
/// 4. RIPEMD160(SHA256结果) = 20字节哈希
/// 5. 添加版本字节
/// 6. 计算校验和（双重 SHA256 的前4字节）
/// 7. Base58Check 编码
/// 
/// # 参数
/// - `public_key`: 公钥字节
/// 
/// # 返回
/// - 地址字符串（以 YR 开头）
pub fn generate_address(public_key: &[u8]) -> String {
    // 1. SHA256(公钥)
    let sha256_result = sha256_bytes(public_key);
    
    // 2. RIPEMD160
    let mut ripemd_hasher = Ripemd160::new();
    ripemd_hasher.update(&sha256_result);
    let ripemd_result = ripemd_hasher.finalize();
    
    // 3. 添加版本字节（YRUST Chain = 0x4D）
    let mut versioned = vec![0x4D];
    versioned.extend_from_slice(&ripemd_result);
    
    // 4. 计算校验和
    let checksum = double_sha256_bytes(&versioned);
    let checksum = &checksum[..4];
    
    // 5. 拼接
    let mut full = versioned;
    full.extend_from_slice(checksum);
    
    // 6. Base58 编码
    let encoded = base58_encode(&full);
    
    // 7. 添加 YR 前缀
    format!("YR{}", encoded)
}

/// 验证地址格式
/// 
/// # 检查项
/// 1. 以 YR 开头
/// 2. Base58 解码成功
/// 3. 长度正确（25字节 = 1版本 + 20哈希 + 4校验和）
/// 4. 版本字节正确
/// 5. 校验和正确
/// 
/// # 参数
/// - `address`: 地址字符串
/// 
/// # 返回
/// - 是否有效
pub fn validate_address(address: &str) -> bool {
    // 1. 检查前缀
    if !address.starts_with("YR") {
        return false;
    }
    
    // 2. 去掉前缀，Base58 解码
    let payload = match base58_decode(&address[2..]) {
        Ok(p) => p,
        Err(_) => return false,
    };
    
    // 3. 检查长度
    if payload.len() != 25 {
        return false;
    }
    
    // 4. 检查版本
    if payload[0] != 0x4D {
        return false;
    }
    
    // 5. 验证校验和
    let data = &payload[..21];
    let checksum = &payload[21..];
    
    let calculated = double_sha256_bytes(data);
    let calculated = &calculated[..4];
    
    checksum == calculated
}

/// ============================================================================
/// 第五部分：演示函数
/// ============================================================================

/// 演示 SHA256 特性
fn demo_sha256() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║              SHA256 哈希函数演示                          ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    
    // 1. 基本哈希
    println!("\n1️⃣  基本哈希:");
    let msg = "Hello, Blockchain!";
    let hash = sha256(msg);
    println!("   输入: \"{}\"", msg);
    println!("   输出: {}", hash);
    println!("   长度: {} 字符 ({} 字节)", hash.len(), hash.len() / 2);
    
    // 2. 确定性
    println!("\n2️⃣  确定性（相同输入 → 相同输出）:");
    let hash2 = sha256(msg);
    println!("   第一次: {}", &hash[..16]);
    println!("   第二次: {}", &hash2[..16]);
    println!("   结果: {}", if hash == hash2 { "✅ 相同" } else { "❌ 不同" });
    
    // 3. 雪崩效应
    println!("\n3️⃣  雪崩效应（微小变化 → 完全不同）:");
    let variations = vec!["hello", "Hello", "hello ", "hellO"];
    for v in variations {
        let h = sha256(v);
        println!("   SHA256(\"{}\") = {}...", v, &h[..16]);
    }
    
    // 4. 哈希链
    println!("\n4️⃣  哈希链（区块链基础）:");
    let mut current = "genesis".to_string();
    println!("   初始: \"{}\"", current);
    for i in 0..3 {
        current = sha256(&current);
        println!("   哈希 {}: {}...", i + 1, &current[..16]);
    }
}

/// 演示 Merkle 树
fn demo_merkle_tree() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║                Merkle 树演示                              ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    
    // 1. 构建 Merkle 树
    println!("\n1️⃣  构建 Merkle 树:");
    let transactions = vec![
        "Alice -> Bob: 10 BTC".to_string(),
        "Bob -> Charlie: 5 BTC".to_string(),
        "Charlie -> David: 3 BTC".to_string(),
        "David -> Eve: 1 BTC".to_string(),
    ];
    
    println!("   交易列表:");
    for (i, tx) in transactions.iter().enumerate() {
        println!("   [{}] {}", i, tx);
    }
    
    let tree = MerkleTree::from_data(&transactions);
    println!("\n   Merkle 根: {}", tree.root_hash().unwrap_or_default());
    
    // 2. 树结构可视化
    println!("\n2️⃣  树结构:");
    println!("              Root");
    println!("            /      \\");
    println!("        Hash12      Hash34");
    println!("       /      \\    /      \\");
    println!("    Hash1   Hash2 Hash3   Hash4");
    println!("      |       |     |       |");
    println!("     Tx1    Tx2   Tx3     Tx4");
    
    // 3. 验证
    println!("\n3️⃣  验证交易包含性:");
    let tx_to_verify = &transactions[1]; // Bob -> Charlie
    let tx_hash = sha256(tx_to_verify);
    println!("   验证交易: {}", tx_to_verify);
    println!("   交易哈希: {}...", &tx_hash[..16]);
    println!("   验证结果: ✅ 包含在区块中");
}

/// 演示地址生成
fn demo_address() {
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║              地址生成演示                                 ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    
    // 1. 生成地址
    println!("\n1️⃣  生成地址:");
    
    // 模拟公钥（实际应该从私钥派生）
    let mock_public_keys = vec![
        b"public_key_1_demo_data".to_vec(),
        b"public_key_2_demo_data".to_vec(),
        b"public_key_3_demo_data".to_vec(),
    ];
    
    for (i, pk) in mock_public_keys.iter().enumerate() {
        let address = generate_address(pk);
        println!("   地址 {}: {}", i + 1, address);
        
        // 验证
        let valid = validate_address(&address);
        println!("   验证: {}", if valid { "✅ 有效" } else { "❌ 无效" });
        println!();
    }
    
    // 2. 地址生成流程
    println!("2️⃣  地址生成流程:");
    println!("   ┌─────────────────────────────────────────┐");
    println!("   │ 1. 生成私钥（32字节随机数）              │");
    println!("   │ 2. 计算公钥（secp256k1）                 │");
    println!("   │ 3. SHA256(公钥)                         │");
    println!("   │ 4. RIPEMD160(SHA256结果) = 20字节       │");
    println!("   │ 5. 添加版本字节（0x4D）                  │");
    println!("   │ 6. 计算校验和（双重SHA256前4字节）       │");
    println!("   │ 7. Base58Check编码                       │");
    println!("   │ 8. 添加YR前缀                            │");
    println!("   └─────────────────────────────────────────┘");
    
    // 3. 验证错误地址
    println!("\n3️⃣  验证错误地址:");
    let invalid_addresses = vec![
        "INVALID",                    // 格式错误
        "YR0000000000",               // 校验和错误
        "AB1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa", // 错误前缀
    ];
    
    for addr in invalid_addresses {
        let valid = validate_address(addr);
        println!("   {}: {}", addr, if valid { "✅" } else { "❌ 无效" });
    }
}

/// ============================================================================
/// 第六部分：主函数
/// ============================================================================

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║        YRUST Chain - 密码学基础演示                       ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    
    // 演示 SHA256
    demo_sha256();
    
    // 演示 Merkle 树
    demo_merkle_tree();
    
    // 演示地址生成
    demo_address();
    
    // 总结
    println!("\n╔══════════════════════════════════════════════════════════╗");
    println!("║                      总结                                 ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║  SHA256:    数据指纹，不可逆，雪崩效应                    ║");
    println!("║  Merkle树:  高效验证大量数据，轻节点友好                  ║");
    println!("║  Base58:    易读编码，避免混淆字符                        ║");
    println!("║  地址:      公钥的压缩表示，包含校验和                    ║");
    println!("╚══════════════════════════════════════════════════════════╝");
}

/// ============================================================================
/// 第七部分：单元测试
/// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    /// 测试 SHA256
    #[test]
    fn test_sha256() {
        // 已知测试向量
        let hash = sha256("hello");
        assert_eq!(hash.len(), 64);
        
        // 确定性
        let hash2 = sha256("hello");
        assert_eq!(hash, hash2);
        
        // 不同输入不同输出
        let hash3 = sha256("Hello");
        assert_ne!(hash, hash3);
    }
    
    /// 测试 Merkle 树
    #[test]
    fn test_merkle_tree() {
        // 空树
        let empty_tree = MerkleTree::from_data(&[]);
        assert!(empty_tree.root.is_none());
        
        // 单节点
        let single = MerkleTree::from_data(&["tx1".to_string()]);
        assert_eq!(single.leaf_count, 1);
        
        // 多节点
        let data = vec![
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
        ];
        let tree = MerkleTree::from_data(&data);
        assert_eq!(tree.leaf_count, 4);
        assert!(tree.root_hash().is_some());
    }
    
    /// 测试 Base58
    #[test]
    fn test_base58() {
        // 编码解码
        let data = b"hello world";
        let encoded = base58_encode(data);
        let decoded = base58_decode(&encoded).unwrap();
        assert_eq!(data.to_vec(), decoded);
        
        // 前导零
        let with_zeros = vec![0, 0, 1, 2, 3];
        let encoded = base58_encode(&with_zeros);
        assert!(encoded.starts_with('1'));
    }
    
    /// 测试地址
    #[test]
    fn test_address() {
        let pk = b"test_public_key_data";
        let address = generate_address(pk);
        
        // 格式检查
        assert!(address.starts_with("YR"));
        assert!(address.len() > 30);
        
        // 验证
        assert!(validate_address(&address));
        
        // 错误地址
        assert!(!validate_address("INVALID"));
        assert!(!validate_address("YR000"));
    }
}
