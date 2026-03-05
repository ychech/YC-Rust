//! 钱包模块
//! 提供私钥、公钥、地址生成，以及交易签名功能

use secp256k1::{Secp256k1, Message, SecretKey, PublicKey, ecdsa::Signature};
use sha2::{Sha256, Digest};
use ripemd::Ripemd160;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use std::fmt;

/// 钱包结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    /// 私钥 (32 字节，十六进制字符串)
    pub private_key: String,
    /// 公钥 (压缩格式，33 字节，十六进制字符串)
    pub public_key: String,
    /// 地址 (Base58Check 编码)
    pub address: String,
}

/// 网络类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Network {
    Mainnet,
    Testnet,
}

impl Wallet {
    /// 创建新钱包
    pub fn new() -> Self {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        
        let private_key = hex::encode(secret_key.secret_bytes());
        let public_key_bytes = public_key.serialize();
        let public_key_hex = hex::encode(public_key_bytes);
        
        let address = Self::generate_address(&public_key_bytes, Network::Mainnet);

        Wallet {
            private_key,
            public_key: public_key_hex,
            address,
        }
    }

    /// 从私钥恢复钱包
    pub fn from_private_key(private_key_hex: &str) -> Result<Self, WalletError> {
        let secret_key = Self::parse_private_key(private_key_hex)?;
        let secp = Secp256k1::new();
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        
        let public_key_bytes = public_key.serialize();
        let public_key_hex = hex::encode(public_key_bytes);
        let address = Self::generate_address(&public_key_bytes, Network::Mainnet);

        Ok(Wallet {
            private_key: private_key_hex.to_string(),
            public_key: public_key_hex,
            address,
        })
    }

    /// 解析私钥
    fn parse_private_key(hex_str: &str) -> Result<SecretKey, WalletError> {
        let bytes = hex::decode(hex_str)
            .map_err(|_| WalletError::InvalidPrivateKey)?;
        
        if bytes.len() != 32 {
            return Err(WalletError::InvalidPrivateKey);
        }

        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&bytes);
        
        SecretKey::from_slice(&key_bytes)
            .map_err(|_| WalletError::InvalidPrivateKey)
    }

    /// 生成地址 (P2PKH 格式)
    /// 
    /// 步骤:
    /// 1. SHA256(公钥)
    /// 2. RIPEMD160(SHA256(公钥)) -> 得到 20 字节的公钥哈希
    /// 3. 添加版本字节 (主网: 0x00, 测试网: 0x6F)
    /// 4. 计算校验和: SHA256(SHA256(版本 + 公钥哈希)) 的前 4 字节
    /// 5. Base58 编码: Base58(版本 + 公钥哈希 + 校验和)
    fn generate_address(public_key: &[u8], network: Network) -> String {
        // 1. SHA256
        let mut sha256_hasher = Sha256::new();
        sha256_hasher.update(public_key);
        let sha256_result = sha256_hasher.finalize();

        // 2. RIPEMD160
        let mut ripemd_hasher = Ripemd160::new();
        ripemd_hasher.update(sha256_result);
        let pubkey_hash = ripemd_hasher.finalize();

        // 3. 添加版本字节
        let version_byte: u8 = match network {
            Network::Mainnet => 0x00,
            Network::Testnet => 0x6F,
        };

        let mut versioned_payload = vec![version_byte];
        versioned_payload.extend_from_slice(&pubkey_hash);

        // 4. 计算校验和 (双重 SHA256)
        let checksum = Self::calculate_checksum(&versioned_payload);

        // 5. 组合并 Base58 编码
        let mut full_payload = versioned_payload;
        full_payload.extend_from_slice(&checksum);

        bs58::encode(full_payload).into_string()
    }

    /// 计算校验和
    fn calculate_checksum(data: &[u8]) -> [u8; 4] {
        let mut hasher1 = Sha256::new();
        hasher1.update(data);
        let hash1 = hasher1.finalize();

        let mut hasher2 = Sha256::new();
        hasher2.update(hash1);
        let hash2 = hasher2.finalize();

        let mut checksum = [0u8; 4];
        checksum.copy_from_slice(&hash2[..4]);
        checksum
    }

    /// 验证地址格式
    pub fn validate_address(address: &str) -> bool {
        // 解码 Base58
        let decoded = match bs58::decode(address).into_vec() {
            Ok(v) => v,
            Err(_) => return false,
        };

        // 检查长度 (1 版本 + 20 公钥哈希 + 4 校验和 = 25)
        if decoded.len() != 25 {
            return false;
        }

        // 分离各部分
        let version = decoded[0];
        let payload = &decoded[..21];
        let checksum = &decoded[21..25];

        // 验证版本字节
        if version != 0x00 && version != 0x6F {
            return false;
        }

        // 验证校验和
        let calculated_checksum = Self::calculate_checksum(payload);
        checksum == calculated_checksum
    }

    /// 签名数据
    pub fn sign(&self, data: &[u8]) -> Result<Vec<u8>, WalletError> {
        let secret_key = Self::parse_private_key(&self.private_key)?;
        let secp = Secp256k1::new();
        
        // 对数据进行双重 SHA256 哈希
        let hash = Self::hash_for_sign(data);
        let message = Message::from_digest(hash);
        
        let signature = secp.sign_ecdsa(&message, &secret_key);
        Ok(signature.serialize_der().to_vec())
    }

    /// 验证签名
    pub fn verify_signature(
        public_key: &[u8],
        data: &[u8],
        signature_bytes: &[u8],
    ) -> Result<bool, WalletError> {
        let secp = Secp256k1::new();
        
        let public_key = PublicKey::from_slice(public_key)
            .map_err(|_| WalletError::InvalidPublicKey)?;
        
        let signature = Signature::from_der(signature_bytes)
            .map_err(|_| WalletError::InvalidSignature)?;
        
        let hash = Self::hash_for_sign(data);
        let message = Message::from_digest(hash);
        
        match secp.verify_ecdsa(&message, &signature, &public_key) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// 为签名准备数据 (双重 SHA256)
    fn hash_for_sign(data: &[u8]) -> [u8; 32] {
        let mut hasher1 = Sha256::new();
        hasher1.update(data);
        let hash1 = hasher1.finalize();

        let mut hasher2 = Sha256::new();
        hasher2.update(hash1);
        let hash2 = hasher2.finalize();

        let mut result = [0u8; 32];
        result.copy_from_slice(&hash2);
        result
    }

    /// 从地址提取公钥哈希
    pub fn address_to_pubkey_hash(address: &str) -> Result<Vec<u8>, WalletError> {
        let decoded = bs58::decode(address)
            .into_vec()
            .map_err(|_| WalletError::InvalidAddress)?;

        if decoded.len() != 25 {
            return Err(WalletError::InvalidAddress);
        }

        // 提取中间的 20 字节公钥哈希
        Ok(decoded[1..21].to_vec())
    }

    /// 导出 WIF 格式私钥 (Wallet Import Format)
    pub fn export_wif(&self, compressed: bool) -> Result<String, WalletError> {
        let secret_key = Self::parse_private_key(&self.private_key)?;
        let mut wif_bytes = vec![0x80]; // 主网版本字节
        wif_bytes.extend_from_slice(&secret_key.secret_bytes());
        
        if compressed {
            wif_bytes.push(0x01);
        }

        let checksum = Self::calculate_checksum(&wif_bytes);
        wif_bytes.extend_from_slice(&checksum);

        Ok(bs58::encode(wif_bytes).into_string())
    }

    /// 从 WIF 导入私钥
    pub fn from_wif(wif: &str) -> Result<Self, WalletError> {
        let decoded = bs58::decode(wif)
            .into_vec()
            .map_err(|_| WalletError::InvalidWif)?;

        // 检查最小长度 (1 版本 + 32 私钥 + 4 校验和 = 37)
        if decoded.len() < 37 {
            return Err(WalletError::InvalidWif);
        }

        // 验证版本字节
        if decoded[0] != 0x80 {
            return Err(WalletError::InvalidWif);
        }

        // 验证校验和
        let payload_len = decoded.len() - 4;
        let checksum = &decoded[payload_len..];
        let calculated_checksum = Self::calculate_checksum(&decoded[..payload_len]);
        
        if checksum != calculated_checksum {
            return Err(WalletError::InvalidWif);
        }

        // 提取私钥
        let private_key_hex = hex::encode(&decoded[1..33]);
        Self::from_private_key(&private_key_hex)
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for Wallet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "╔════════════════════════════════════════════════════════╗")?;
        writeln!(f, "║                    YRust 钱包                           ║")?;
        writeln!(f, "╠════════════════════════════════════════════════════════╣")?;
        writeln!(f, "║ 地址:    {}", self.address)?;
        writeln!(f, "║ 公钥:    {}...", &self.public_key[..64])?;
        writeln!(f, "║ 私钥:    {}...", &self.private_key[..32])?;
        writeln!(f, "╚════════════════════════════════════════════════════════╝")
    }
}

/// 钱包错误类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WalletError {
    InvalidPrivateKey,
    InvalidPublicKey,
    InvalidAddress,
    InvalidSignature,
    InvalidWif,
    SigningError,
}

impl fmt::Display for WalletError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            WalletError::InvalidPrivateKey => write!(f, "无效的私钥"),
            WalletError::InvalidPublicKey => write!(f, "无效的公钥"),
            WalletError::InvalidAddress => write!(f, "无效的地址"),
            WalletError::InvalidSignature => write!(f, "无效的签名"),
            WalletError::InvalidWif => write!(f, "无效的 WIF 格式"),
            WalletError::SigningError => write!(f, "签名错误"),
        }
    }
}

impl std::error::Error for WalletError {}

/// 密钥存储管理
pub struct KeyStore {
    wallets: std::collections::HashMap<String, Wallet>,
}

impl KeyStore {
    pub fn new() -> Self {
        KeyStore {
            wallets: std::collections::HashMap::new(),
        }
    }

    /// 创建新钱包
    pub fn create_wallet(&mut self) -> &Wallet {
        let wallet = Wallet::new();
        self.wallets.insert(wallet.address.clone(), wallet);
        self.wallets.get(&Wallet::new().address).unwrap()
    }

    /// 导入钱包
    pub fn import_wallet(&mut self, private_key: &str) -> Result<&Wallet, WalletError> {
        let wallet = Wallet::from_private_key(private_key)?;
        let address = wallet.address.clone();
        self.wallets.insert(address.clone(), wallet);
        Ok(self.wallets.get(&address).unwrap())
    }

    /// 获取钱包
    pub fn get_wallet(&self, address: &str) -> Option<&Wallet> {
        self.wallets.get(address)
    }

    /// 获取所有钱包地址
    pub fn get_addresses(&self) -> Vec<&String> {
        self.wallets.keys().collect()
    }

    /// 签名交易数据
    pub fn sign_transaction(
        &self,
        address: &str,
        tx_data: &[u8],
    ) -> Result<Vec<u8>, WalletError> {
        let wallet = self.wallets.get(address)
            .ok_or(WalletError::InvalidAddress)?;
        wallet.sign(tx_data)
    }
}

impl Default for KeyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new();
        assert!(!wallet.private_key.is_empty());
        assert!(!wallet.public_key.is_empty());
        assert!(!wallet.address.is_empty());
        assert!(Wallet::validate_address(&wallet.address));
    }

    #[test]
    fn test_address_validation() {
        let wallet = Wallet::new();
        assert!(Wallet::validate_address(&wallet.address));
        assert!(!Wallet::validate_address("invalid_address"));
    }

    #[test]
    fn test_sign_and_verify() {
        let wallet = Wallet::new();
        let data = b"Hello, YRust Chain!";
        
        let signature = wallet.sign(data).unwrap();
        let public_key = hex::decode(&wallet.public_key).unwrap();
        
        assert!(Wallet::verify_signature(&public_key, data, &signature).unwrap());
    }

    #[test]
    fn test_wif_import_export() {
        let wallet = Wallet::new();
        let wif = wallet.export_wif(true).unwrap();
        let imported = Wallet::from_wif(&wif).unwrap();
        
        assert_eq!(wallet.private_key, imported.private_key);
        assert_eq!(wallet.address, imported.address);
    }

    #[test]
    fn test_recover_from_private_key() {
        let wallet1 = Wallet::new();
        let wallet2 = Wallet::from_private_key(&wallet1.private_key).unwrap();
        
        assert_eq!(wallet1.public_key, wallet2.public_key);
        assert_eq!(wallet1.address, wallet2.address);
    }
}
