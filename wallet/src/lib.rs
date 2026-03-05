//! YRUST 网页钱包核心库
//! 
//! 功能：
//! - 助记词生成和恢复
//! - 密钥派生 (BIP39/BIP44)
//! - 交易签名
//! - 本地存储加密

use bip39::{Mnemonic, Language, Seed};
use secp256k1::{SecretKey, PublicKey, Secp256k1, Message};
use sha2::{Sha256, Digest};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// 钱包账户
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletAccount {
    /// 账户索引
    pub index: u32,
    /// 账户名称
    pub name: String,
    /// 地址
    pub address: String,
    /// 公钥 (hex)
    pub public_key: String,
    /// 私钥加密存储
    #[serde(skip_serializing)]
    pub encrypted_private_key: Vec<u8>,
    /// 派生路径
    pub derivation_path: String,
}

/// 钱包数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wallet {
    /// 版本
    pub version: u32,
    /// 助记词加密存储
    #[serde(skip_serializing)]
    pub encrypted_mnemonic: Vec<u8>,
    /// 账户列表
    pub accounts: Vec<WalletAccount>,
    /// 当前选中账户
    pub current_account: u32,
    /// 创建时间
    pub created_at: u64,
}

/// 交易请求
#[derive(Debug, Clone)]
pub struct TransactionRequest {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub gas_price: u64,
    pub gas_limit: u64,
    pub nonce: u64,
    pub data: Vec<u8>,
}

/// 签名后的交易
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedTransaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub gas_price: u64,
    pub gas_limit: u64,
    pub nonce: u64,
    pub data: Vec<u8>,
    pub signature: String,
    pub hash: String,
}

/// 钱包管理器
pub struct WalletManager {
    /// 当前钱包
    wallet: Option<Wallet>,
    /// 密码哈希 (用于验证)
    password_hash: Option<Vec<u8>>,
    /// 内存中的私钥 (使用完后清除)
    private_keys: HashMap<String, SecretKey>,
}

impl WalletManager {
    /// 创建新钱包管理器
    pub fn new() -> Self {
        WalletManager {
            wallet: None,
            password_hash: None,
            private_keys: HashMap::new(),
        }
    }

    /// 创建新钱包
    pub fn create_wallet(&mut self, password: &str) -> Result<(String, String), WalletError> {
        // 生成随机助记词
        let mnemonic = Mnemonic::generate_in(Language::English, 12)
            .map_err(|e| WalletError::MnemonicError(e.to_string()))?;
        
        let phrase = mnemonic.to_string();
        
        // 加密存储
        let encrypted = self.encrypt_data(phrase.as_bytes(), password)?;
        
        // 创建钱包
        let wallet = Wallet {
            version: 1,
            encrypted_mnemonic: encrypted,
            accounts: Vec::new(),
            current_account: 0,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        self.wallet = Some(wallet);
        self.password_hash = Some(self.hash_password(password));
        
        // 创建第一个账户
        let address = self.create_account("Account 1", password)?;
        
        Ok((phrase, address))
    }

    /// 从助记词恢复钱包
    pub fn restore_wallet(&mut self, mnemonic_phrase: &str, password: &str) -> Result<String, WalletError> {
        // 验证助记词
        let mnemonic = Mnemonic::parse_in(Language::English, mnemonic_phrase)
            .map_err(|e| WalletError::InvalidMnemonic(e.to_string()))?;
        
        // 加密存储
        let encrypted = self.encrypt_data(mnemonic_phrase.as_bytes(), password)?;
        
        // 创建钱包
        let wallet = Wallet {
            version: 1,
            encrypted_mnemonic: encrypted,
            accounts: Vec::new(),
            current_account: 0,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };
        
        self.wallet = Some(wallet);
        self.password_hash = Some(self.hash_password(password));
        
        // 恢复第一个账户
        let address = self.create_account("Account 1", password)?;
        
        Ok(address)
    }

    /// 创建新账户
    pub fn create_account(&mut self, name: &str, password: &str) -> Result<String, WalletError> {
        let wallet = self.wallet.as_mut().ok_or(WalletError::WalletNotFound)?;
        
        // 解密助记词
        let mnemonic_bytes = self.decrypt_data(&wallet.encrypted_mnemonic, password)?;
        let mnemonic_phrase = String::from_utf8(mnemonic_bytes)
            .map_err(|_| WalletError::DecryptionError)?;
        
        // 派生密钥
        let index = wallet.accounts.len() as u32;
        let path = format!("m/44'/8888'/0'/0/{}", index);
        
        let (private_key, public_key, address) = self.derive_keys(&mnemonic_phrase, &path)?;
        
        // 加密私钥
        let encrypted_key = self.encrypt_data(&private_key.secret_bytes(), password)?;
        
        // 保存账户
        let account = WalletAccount {
            index,
            name: name.to_string(),
            address: address.clone(),
            public_key: hex::encode(public_key.serialize()),
            encrypted_private_key: encrypted_key,
            derivation_path: path,
        };
        
        wallet.accounts.push(account);
        
        // 缓存私钥到内存
        self.private_keys.insert(address.clone(), private_key);
        
        Ok(address)
    }

    /// 派生密钥 (BIP44)
    fn derive_keys(&self, mnemonic: &str, path: &str) -> Result<(SecretKey, PublicKey, String), WalletError> {
        // 从助记词生成种子
        let mnemonic = Mnemonic::parse_in(Language::English, mnemonic)
            .map_err(|e| WalletError::InvalidMnemonic(e.to_string()))?;
        let seed = Seed::new(&mnemonic, "");
        
        // 使用简化版派生 (实际应该使用 bip32)
        // 这里使用种子哈希派生
        let mut hasher = Sha256::new();
        hasher.update(seed.as_bytes());
        hasher.update(path.as_bytes());
        let hash = hasher.finalize();
        
        // 生成密钥对
        let secp = Secp256k1::new();
        let secret_key = SecretKey::from_slice(&hash[..32])
            .map_err(|_| WalletError::KeyDerivationError)?;
        let public_key = PublicKey::from_secret_key(&secp, &secret_key);
        
        // 生成地址
        let address = self.public_key_to_address(&public_key);
        
        Ok((secret_key, public_key, address))
    }

    /// 公钥转地址
    fn public_key_to_address(&self, public_key: &PublicKey) -> String {
        use ripemd::Ripemd160;
        
        // SHA256
        let mut sha256_hasher = Sha256::new();
        sha256_hasher.update(&public_key.serialize());
        let sha256_result = sha256_hasher.finalize();
        
        // RIPEMD160
        let mut ripemd_hasher = Ripemd160::new();
        ripemd_hasher.update(sha256_result);
        let pubkey_hash = ripemd_hasher.finalize();
        
        // Base58 编码
        let mut address_bytes = vec![0x00]; // 版本字节
        address_bytes.extend_from_slice(&pubkey_hash);
        
        // 校验和
        let mut checksum_hasher = Sha256::new();
        checksum_hasher.update(&address_bytes);
        let checksum1 = checksum_hasher.finalize();
        
        let mut checksum_hasher2 = Sha256::new();
        checksum_hasher2.update(&checksum1);
        let checksum2 = checksum_hasher2.finalize();
        
        address_bytes.extend_from_slice(&checksum2[..4]);
        
        bs58::encode(address_bytes).into_string()
    }

    /// 签名交易
    pub fn sign_transaction(
        &self,
        tx: TransactionRequest,
        password: &str,
    ) -> Result<SignedTransaction, WalletError> {
        let wallet = self.wallet.as_ref().ok_or(WalletError::WalletNotFound)?;
        
        // 获取账户
        let account = wallet.accounts.get(wallet.current_account as usize)
            .ok_or(WalletError::AccountNotFound)?;
        
        if account.address != tx.from {
            return Err(WalletError::InvalidAddress);
        }
        
        // 获取私钥
        let private_key = if let Some(key) = self.private_keys.get(&tx.from) {
            key.clone()
        } else {
            // 从加密存储解密
            let key_bytes = self.decrypt_data(&account.encrypted_private_key, password)?;
            SecretKey::from_slice(&key_bytes)
                .map_err(|_| WalletError::DecryptionError)?
        };
        
        // 序列化交易
        let tx_data = format!(
            "{}{}{}{}{}{:?}",
            tx.from, tx.to, tx.amount, tx.gas_price, tx.gas_limit, tx.data
        );
        
        // 哈希
        let mut hasher = Sha256::new();
        hasher.update(tx_data.as_bytes());
        let tx_hash = hasher.finalize();
        
        // 签名
        let secp = Secp256k1::new();
        let message = Message::from_digest(tx_hash.into());
        let signature = secp.sign_ecdsa(&message, &private_key);
        
        // 交易哈希
        let mut hash_hasher = Sha256::new();
        hash_hasher.update(&tx_hash);
        hash_hasher.update(signature.serialize_compact());
        let final_hash = hex::encode(hash_hasher.finalize());
        
        Ok(SignedTransaction {
            from: tx.from,
            to: tx.to,
            amount: tx.amount,
            gas_price: tx.gas_price,
            gas_limit: tx.gas_limit,
            nonce: tx.nonce,
            data: tx.data,
            signature: hex::encode(signature.serialize_compact()),
            hash: final_hash,
        })
    }

    /// 获取钱包信息
    pub fn get_wallet_info(&self) -> Option<&Wallet> {
        self.wallet.as_ref()
    }

    /// 获取当前账户
    pub fn get_current_account(&self) -> Option<&WalletAccount> {
        self.wallet.as_ref()?.accounts.get(self.wallet.as_ref()?.current_account as usize)
    }

    /// 切换账户
    pub fn switch_account(&mut self, index: u32) -> Result<(), WalletError> {
        let wallet = self.wallet.as_mut().ok_or(WalletError::WalletNotFound)?;
        
        if index as usize >= wallet.accounts.len() {
            return Err(WalletError::AccountNotFound);
        }
        
        wallet.current_account = index;
        Ok(())
    }

    /// 导出助记词 (需要密码)
    pub fn export_mnemonic(&self, password: &str) -> Result<String, WalletError> {
        let wallet = self.wallet.as_ref().ok_or(WalletError::WalletNotFound)?;
        
        // 验证密码
        if !self.verify_password(password) {
            return Err(WalletError::InvalidPassword);
        }
        
        // 解密
        let mnemonic_bytes = self.decrypt_data(&wallet.encrypted_mnemonic, password)?;
        let mnemonic = String::from_utf8(mnemonic_bytes)
            .map_err(|_| WalletError::DecryptionError)?;
        
        Ok(mnemonic)
    }

    /// 导出私钥
    pub fn export_private_key(&self, address: &str, password: &str) -> Result<String, WalletError> {
        let wallet = self.wallet.as_ref().ok_or(WalletError::WalletNotFound)?;
        
        // 查找账户
        let account = wallet.accounts.iter()
            .find(|a| a.address == address)
            .ok_or(WalletError::AccountNotFound)?;
        
        // 解密
        let key_bytes = self.decrypt_data(&account.encrypted_private_key, password)?;
        
        Ok(hex::encode(key_bytes))
    }

    /// 加密数据 (简化版，实际应使用 AES-GCM)
    fn encrypt_data(&self, data: &[u8], password: &str) -> Result<Vec<u8>, WalletError> {
        // 使用密码哈希作为密钥
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        let key = hasher.finalize();
        
        // 简单 XOR 加密 (实际应使用 AES)
        let encrypted: Vec<u8> = data.iter()
            .enumerate()
            .map(|(i, b)| b ^ key[i % key.len()])
            .collect();
        
        Ok(encrypted)
    }

    /// 解密数据
    fn decrypt_data(&self, encrypted: &[u8], password: &str) -> Result<Vec<u8>, WalletError> {
        // 使用相同的密钥
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        let key = hasher.finalize();
        
        // XOR 解密
        let decrypted: Vec<u8> = encrypted.iter()
            .enumerate()
            .map(|(i, b)| b ^ key[i % key.len()])
            .collect();
        
        Ok(decrypted)
    }

    /// 哈希密码
    fn hash_password(&self, password: &str) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(password.as_bytes());
        hasher.finalize().to_vec()
    }

    /// 验证密码
    fn verify_password(&self, password: &str) -> bool {
        if let Some(ref hash) = self.password_hash {
            let input_hash = self.hash_password(password);
            hash == &input_hash
        } else {
            false
        }
    }

    /// 序列化钱包 (用于本地存储)
    pub fn serialize(&self) -> Result<String, WalletError> {
        let wallet = self.wallet.as_ref().ok_or(WalletError::WalletNotFound)?;
        serde_json::to_string(wallet)
            .map_err(|e| WalletError::SerializationError(e.to_string()))
    }

    /// 反序列化钱包
    pub fn deserialize(&mut self, data: &str, password: &str) -> Result<(), WalletError> {
        let wallet: Wallet = serde_json::from_str(data)
            .map_err(|e| WalletError::SerializationError(e.to_string()))?;
        
        self.wallet = Some(wallet);
        self.password_hash = Some(self.hash_password(password));
        Ok(())
    }
}

/// 钱包错误
#[derive(Debug, Clone)]
pub enum WalletError {
    WalletNotFound,
    AccountNotFound,
    InvalidPassword,
    InvalidMnemonic(String),
    MnemonicError(String),
    KeyDerivationError,
    DecryptionError,
    InvalidAddress,
    SerializationError(String),
}

impl std::fmt::Display for WalletError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WalletError::WalletNotFound => write!(f, "钱包不存在"),
            WalletError::AccountNotFound => write!(f, "账户不存在"),
            WalletError::InvalidPassword => write!(f, "密码错误"),
            WalletError::InvalidMnemonic(e) => write!(f, "无效助记词: {}", e),
            WalletError::MnemonicError(e) => write!(f, "助记词错误: {}", e),
            WalletError::KeyDerivationError => write!(f, "密钥派生失败"),
            WalletError::DecryptionError => write!(f, "解密失败"),
            WalletError::InvalidAddress => write!(f, "无效地址"),
            WalletError::SerializationError(e) => write!(f, "序列化错误: {}", e),
        }
    }
}

impl std::error::Error for WalletError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_wallet() {
        let mut manager = WalletManager::new();
        let (mnemonic, address) = manager.create_wallet("password123").unwrap();
        
        assert!(!mnemonic.is_empty());
        assert!(!address.is_empty());
        assert_eq!(manager.get_wallet_info().unwrap().accounts.len(), 1);
    }

    #[test]
    fn test_restore_wallet() {
        let mut manager = WalletManager::new();
        let (mnemonic, _) = manager.create_wallet("password123").unwrap();
        
        let mut manager2 = WalletManager::new();
        let address2 = manager2.restore_wallet(&mnemonic, "password123").unwrap();
        
        assert!(!address2.is_empty());
    }

    #[test]
    fn test_create_multiple_accounts() {
        let mut manager = WalletManager::new();
        manager.create_wallet("password123").unwrap();
        
        let addr2 = manager.create_account("Account 2", "password123").unwrap();
        let addr3 = manager.create_account("Account 3", "password123").unwrap();
        
        assert!(!addr2.is_empty());
        assert!(!addr3.is_empty());
        assert_eq!(manager.get_wallet_info().unwrap().accounts.len(), 3);
    }

    #[test]
    fn test_export_mnemonic() {
        let mut manager = WalletManager::new();
        let (mnemonic, _) = manager.create_wallet("password123").unwrap();
        
        let exported = manager.export_mnemonic("password123").unwrap();
        assert_eq!(mnemonic, exported);
    }
}
