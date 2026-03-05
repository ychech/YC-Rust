//! 区块链核心逻辑
//! 管理区块、交易、UTXO 集合和挖矿

use super::types::*;
use crate::wallet::Wallet;
use std::collections::{HashMap, HashSet};
use log::{info, warn};

/// 区块链结构
pub struct Blockchain {
    /// 区块存储
    pub chain: Vec<Block>,
    /// UTXO 集合: (tx_id, vout) -> TxOutput
    pub utxo_set: HashMap<(TxId, u32), TxOutput>,
    /// 待处理交易池 (内存池)
    pub mempool: Vec<Transaction>,
    /// 挖矿难度
    pub difficulty: usize,
    /// 挖矿奖励地址
    pub mining_reward_addr: Option<Address>,
    /// 已花费的交易输出 (防止双花)
    spent_outputs: HashSet<(TxId, u32)>,
}

impl Blockchain {
    /// 创建新区块链 (包含创世区块)
    pub fn new() -> Self {
        let genesis = Self::create_genesis_block();
        
        let mut bc = Blockchain {
            chain: vec![genesis],
            utxo_set: HashMap::new(),
            mempool: Vec::new(),
            difficulty: DIFFICULTY,
            mining_reward_addr: None,
            spent_outputs: HashSet::new(),
        };

        // 初始化 UTXO 集合
        bc.rebuild_utxo_set();
        
        info!("✅ 区块链初始化完成，创世区块已创建");
        bc
    }

    /// 创建创世区块
    fn create_genesis_block() -> Block {
        let coinbase = Transaction::new_coinbase(
            "YRustGenesis".to_string(),
            MINING_REWARD,
            0,
        );
        
        let mut block = Block::new(
            0,
            "0".to_string(),
            vec![coinbase],
            0x1d00ffff,
        );
        
        // 创世区块也需要挖矿
        block.mine(DIFFICULTY);
        block
    }

    /// 获取最新区块
    pub fn get_last_block(&self) -> &Block {
        self.chain.last().expect("区块链不能为空")
    }

    /// 获取区块高度
    pub fn get_height(&self) -> u64 {
        self.chain.len() as u64 - 1
    }

    /// 设置挖矿奖励地址
    pub fn set_mining_reward_addr(&mut self, addr: Address) {
        self.mining_reward_addr = Some(addr);
    }

    /// 添加新区块 (带 POW 验证)
    pub fn add_block(&mut self, block: Block) -> Result<(), BlockchainError> {
        // 1. 验证区块高度
        let expected_height = self.get_height() + 1;
        if block.height != expected_height {
            return Err(BlockchainError::InvalidHeight {
                expected: expected_height,
                got: block.height,
            });
        }

        // 2. 验证前一个区块哈希
        let last_block = self.get_last_block();
        if block.header.prev_hash != last_block.hash {
            return Err(BlockchainError::InvalidPreviousHash);
        }

        // 3. 验证区块
        if !block.is_valid(self.difficulty) {
            return Err(BlockchainError::InvalidBlock);
        }

        // 4. 验证交易
        for tx in &block.transactions {
            if !self.verify_transaction(tx)? {
                return Err(BlockchainError::InvalidTransaction(tx.id.clone()));
            }
        }

        // 5. 更新 UTXO 集合
        self.update_utxo_set(&block);

        // 6. 添加区块到链
        self.chain.push(block);
        
        info!("✅ 新区块 #{} 已添加到链上", self.get_height());
        Ok(())
    }

    /// 挖矿 (创建新区块)
    pub fn mine_block(&mut self, miner_addr: Address) -> Result<Block, BlockchainError> {
        // 1. 创建 Coinbase 交易
        let coinbase = Transaction::new_coinbase(
            miner_addr.clone(),
            MINING_REWARD,
            self.get_height(),
        );

        // 2. 准备交易列表 (Coinbase + 内存池中的交易)
        let mut transactions = vec![coinbase];
        
        // 从内存池中选择有效交易 (简单的 FIFO 策略)
        let mut valid_txs = Vec::new();
        let mut remaining_txs = Vec::new();
        
        // 先收集所有交易
        let mempool_txs: Vec<Transaction> = self.mempool.drain(..).collect();
        
        for tx in mempool_txs {
            if valid_txs.len() >= 100 {
                remaining_txs.push(tx);
                continue;
            }
            
            if self.verify_transaction(&tx).unwrap_or(false) {
                valid_txs.push(tx);
            } else {
                remaining_txs.push(tx);
            }
        }
        
        // 将未处理的交易放回内存池
        self.mempool = remaining_txs;
        transactions.extend(valid_txs);

        // 3. 创建新区块
        let last_block = self.get_last_block();
        let mut new_block = Block::new(
            self.get_height() + 1,
            last_block.hash.clone(),
            transactions,
            0x1d00ffff,
        );

        // 4. 执行 POW
        info!("⛏️  开始挖矿区块 #{}...", new_block.height);
        let start_time = std::time::Instant::now();
        new_block.mine(self.difficulty);
        let duration = start_time.elapsed();
        
        info!(
            "✅ 区块 #{} 挖矿完成! 哈希: {}... (耗时: {:?})",
            new_block.height,
            &new_block.hash[..16],
            duration
        );

        // 5. 添加区块到链
        self.add_block(new_block.clone())?;

        Ok(new_block)
    }

    /// 创建并签名交易
    pub fn create_transaction(
        &mut self,
        from_wallet: &Wallet,
        to_address: Address,
        amount: Amount,
    ) -> Result<Transaction, BlockchainError> {
        // 1. 查找可用的 UTXO
        let utxos = self.find_spendable_utxos(&from_wallet.address, amount);
        
        if utxos.is_empty() {
            return Err(BlockchainError::InsufficientBalance);
        }

        // 2. 计算输入总额
        let total_input: Amount = utxos.iter()
            .map(|(_, output)| output.value)
            .sum();

        // 3. 创建输入
        let mut inputs = Vec::new();
        for ((tx_id, vout), _) in &utxos {
            inputs.push(TxInput {
                tx_id: tx_id.clone(),
                vout: *vout,
                script_sig: ScriptSig {
                    signature: vec![],
                    pubkey: hex::decode(&from_wallet.public_key)
                        .map_err(|_| BlockchainError::WalletError)?,
                },
            });
        }

        // 4. 创建输出
        let mut outputs = vec![TxOutput {
            value: amount,
            script_pubkey: to_address,
        }];

        // 5. 找零输出
        let change = total_input - amount;
        if change > 0 {
            outputs.push(TxOutput {
                value: change,
                script_pubkey: from_wallet.address.clone(),
            });
        }

        // 6. 创建未签名交易 (不计算 ID)
        let mut tx = Transaction {
            id: String::new(),
            timestamp: chrono::Utc::now().timestamp(),
            inputs,
            outputs,
            version: 1,
        };

        // 7. 签名交易 (基于未签名的交易数据)
        let tx_data = tx.serialize_for_sign();
        let signature = from_wallet.sign(&tx_data)
            .map_err(|_| BlockchainError::WalletError)?;

        // 8. 更新输入的签名
        let pubkey = hex::decode(&from_wallet.public_key)
            .map_err(|_| BlockchainError::WalletError)?;
        for input in &mut tx.inputs {
            input.script_sig.signature = signature.clone();
            input.script_sig.pubkey = pubkey.clone();
        }

        // 9. 计算交易 ID
        tx.id = tx.calculate_hash();

        Ok(tx)
    }

    /// 添加交易到内存池
    pub fn add_to_mempool(&mut self, tx: Transaction) -> Result<(), BlockchainError> {
        // 1. 验证交易
        match self.verify_transaction(&tx) {
            Ok(true) => {}
            Ok(false) => {
                eprintln!("❌ 交易验证失败: {}", tx.id);
                return Err(BlockchainError::InvalidTransaction(tx.id.clone()));
            }
            Err(e) => {
                eprintln!("❌ 交易验证错误: {:?}", e);
                return Err(BlockchainError::InvalidTransaction(tx.id.clone()));
            }
        }

        // 2. 检查是否已在内存池
        if self.mempool.iter().any(|t| t.id == tx.id) {
            return Err(BlockchainError::DuplicateTransaction);
        }

        // 3. 添加到内存池
        self.mempool.push(tx);
        info!("📥 交易已添加到内存池，当前待处理: {}", self.mempool.len());
        
        Ok(())
    }

    /// 验证交易
    fn verify_transaction(&self, tx: &Transaction) -> Result<bool, BlockchainError> {
        // 1. 基本验证
        if !tx.is_valid() {
            eprintln!("  交易基本验证失败: ID 不匹配");
            return Ok(false);
        }

        // 2. Coinbase 交易特殊处理
        if tx.is_coinbase() {
            // 验证奖励金额
            if let Some(output) = tx.outputs.first() {
                return Ok(output.value == MINING_REWARD);
            }
            return Ok(false);
        }

        // 3. 验证输入
        let mut total_input: Amount = 0;
        
        for (idx, input) in tx.inputs.iter().enumerate() {
            // 检查输入是否已被花费
            if self.spent_outputs.contains(&(input.tx_id.clone(), input.vout)) {
                eprintln!("  输入 #{} 已被花费", idx);
                return Ok(false);
            }

            // 查找 UTXO
            let utxo = match self.utxo_set.get(&(input.tx_id.clone(), input.vout)) {
                Some(u) => u,
                None => {
                    eprintln!("  UTXO 未找到: {}:{}", &input.tx_id[..16], input.vout);
                    return Err(BlockchainError::UtxoNotFound {
                        tx_id: input.tx_id.clone(),
                        vout: input.vout,
                    });
                }
            };

            total_input += utxo.value;

            // 验证签名 (如果提供了签名)
            if !input.script_sig.signature.is_empty() && !input.script_sig.pubkey.is_empty() {
                let tx_data = tx.serialize_for_verify();
                match Wallet::verify_signature(
                    &input.script_sig.pubkey,
                    &tx_data,
                    &input.script_sig.signature,
                ) {
                    Ok(true) => {}
                    Ok(false) => {
                        eprintln!("  签名验证失败");
                        return Ok(false);
                    }
                    Err(e) => {
                        eprintln!("  签名验证错误: {}", e);
                        return Err(BlockchainError::InvalidSignature);
                    }
                }
            } else {
                eprintln!("  签名或公钥为空");
                return Ok(false);
            }
        }

        // 4. 验证输出总额 <= 输入总额
        let total_output: Amount = tx.outputs.iter().map(|o| o.value).sum();
        if total_output > total_input {
            eprintln!("  输出金额 {} 超过输入金额 {}", total_output, total_input);
            return Ok(false);
        }

        Ok(true)
    }

    /// 查找可花费的 UTXO
    fn find_spendable_utxos(
        &self,
        address: &str,
        min_amount: Amount,
    ) -> Vec<((TxId, u32), TxOutput)> {
        let mut result = Vec::new();
        let mut total: Amount = 0;

        for ((tx_id, vout), output) in &self.utxo_set {
            if output.script_pubkey == address {
                // 检查是否已被花费
                if !self.spent_outputs.contains(&(tx_id.clone(), *vout)) {
                    result.push(((tx_id.clone(), *vout), output.clone()));
                    total += output.value;

                    if total >= min_amount {
                        break;
                    }
                }
            }
        }

        if total >= min_amount {
            result
        } else {
            Vec::new()
        }
    }

    /// 获取地址余额
    pub fn get_balance(&self, address: &str) -> Amount {
        self.utxo_set
            .iter()
            .filter(|(_, output)| output.script_pubkey == address)
            .filter(|(key, _)| !self.spent_outputs.contains(key))
            .map(|(_, output)| output.value)
            .sum()
    }

    /// 更新 UTXO 集合
    fn update_utxo_set(&mut self, block: &Block) {
        for tx in &block.transactions {
            // 1. 移除已花费的 UTXO
            for input in &tx.inputs {
                if !tx.is_coinbase() {
                    self.utxo_set.remove(&(input.tx_id.clone(), input.vout));
                    self.spent_outputs.insert((input.tx_id.clone(), input.vout));
                }
            }

            // 2. 添加新的 UTXO
            for (vout, output) in tx.outputs.iter().enumerate() {
                self.utxo_set.insert(
                    (tx.id.clone(), vout as u32),
                    output.clone(),
                );
            }
        }
    }

    /// 重建 UTXO 集合 (从创世区块扫描)
    pub fn rebuild_utxo_set(&mut self) {
        self.utxo_set.clear();
        self.spent_outputs.clear();

        let chain_clone = self.chain.clone();
        for block in chain_clone {
            self.update_utxo_set(&block);
        }
    }

    /// 验证整个区块链
    pub fn is_valid(&self) -> bool {
        // 1. 验证创世区块
        if self.chain.is_empty() {
            return false;
        }

        let genesis = &self.chain[0];
        if genesis.height != 0 || genesis.header.prev_hash != "0" {
            return false;
        }

        // 2. 验证所有区块
        for i in 1..self.chain.len() {
            let current = &self.chain[i];
            let previous = &self.chain[i - 1];

            // 验证区块
            if !current.is_valid(self.difficulty) {
                warn!("❌ 区块 #{} 验证失败", current.height);
                return false;
            }

            // 验证链接
            if current.header.prev_hash != previous.hash {
                warn!("❌ 区块 #{} 链接断裂", current.height);
                return false;
            }

            // 验证高度
            if current.height != previous.height + 1 {
                warn!("❌ 区块 #{} 高度错误", current.height);
                return false;
            }
        }

        info!("✅ 区块链验证通过");
        true
    }

    /// 获取区块通过哈希
    pub fn get_block_by_hash(&self, hash: &str) -> Option<&Block> {
        self.chain.iter().find(|b| b.hash == hash)
    }

    /// 获取区块通过高度
    pub fn get_block_by_height(&self, height: u64) -> Option<&Block> {
        self.chain.get(height as usize)
    }

    /// 获取交易通过 ID
    pub fn get_transaction(&self, tx_id: &str) -> Option<&Transaction> {
        for block in &self.chain {
            for tx in &block.transactions {
                if tx.id == tx_id {
                    return Some(tx);
                }
            }
        }
        None
    }

    /// 获取内存池
    pub fn get_mempool(&self) -> &[Transaction] {
        &self.mempool
    }

    /// 获取 UTXO 集合大小
    pub fn get_utxo_count(&self) -> usize {
        self.utxo_set.len()
    }

    /// 获取区块链统计信息
    pub fn get_stats(&self) -> BlockchainStats {
        let total_txs: usize = self.chain.iter()
            .map(|b| b.transactions.len())
            .sum();

        BlockchainStats {
            height: self.get_height(),
            total_blocks: self.chain.len() as u64,
            total_transactions: total_txs as u64,
            utxo_count: self.utxo_set.len(),
            mempool_size: self.mempool.len(),
            difficulty: self.difficulty,
            best_block_hash: self.get_last_block().hash.clone(),
        }
    }
}

impl Default for Blockchain {
    fn default() -> Self {
        Self::new()
    }
}

/// 区块链统计信息
#[derive(Debug, Clone, serde::Serialize)]
pub struct BlockchainStats {
    pub height: u64,
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub utxo_count: usize,
    pub mempool_size: usize,
    pub difficulty: usize,
    pub best_block_hash: String,
}

impl fmt::Display for BlockchainStats {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "╔══════════════════════════════════════════╗")?;
        writeln!(f, "║         YRust 区块链统计信息              ║")?;
        writeln!(f, "╠══════════════════════════════════════════╣")?;
        writeln!(f, "║ 区块高度:       {:<25} ║", self.height)?;
        writeln!(f, "║ 总区块数:       {:<25} ║", self.total_blocks)?;
        writeln!(f, "║ 总交易数:       {:<25} ║", self.total_transactions)?;
        writeln!(f, "║ UTXO 数量:      {:<25} ║", self.utxo_count)?;
        writeln!(f, "║ 内存池大小:     {:<25} ║", self.mempool_size)?;
        writeln!(f, "║ 挖矿难度:       {:<25} ║", self.difficulty)?;
        writeln!(f, "║ 最新区块:       {}... ║", &self.best_block_hash[..20])?;
        writeln!(f, "╚══════════════════════════════════════════╝")
    }
}

/// 区块链错误类型
#[derive(Debug, Clone)]
pub enum BlockchainError {
    InvalidHeight { expected: u64, got: u64 },
    InvalidPreviousHash,
    InvalidBlock,
    InvalidTransaction(String),
    InvalidSignature,
    InvalidAddress,
    InsufficientBalance,
    UtxoNotFound { tx_id: String, vout: u32 },
    DuplicateTransaction,
    WalletError,
}

impl fmt::Display for BlockchainError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BlockchainError::InvalidHeight { expected, got } => {
                write!(f, "无效的区块高度: 期望 {}, 实际 {}", expected, got)
            }
            BlockchainError::InvalidPreviousHash => write!(f, "无效的前一个区块哈希"),
            BlockchainError::InvalidBlock => write!(f, "无效的区块"),
            BlockchainError::InvalidTransaction(id) => write!(f, "无效的交易: {}", id),
            BlockchainError::InvalidSignature => write!(f, "无效的签名"),
            BlockchainError::InvalidAddress => write!(f, "无效的地址"),
            BlockchainError::InsufficientBalance => write!(f, "余额不足"),
            BlockchainError::UtxoNotFound { tx_id, vout } => {
                write!(f, "UTXO 未找到: {}:{}", tx_id, vout)
            }
            BlockchainError::DuplicateTransaction => write!(f, "重复的交易"),
            BlockchainError::WalletError => write!(f, "钱包错误"),
        }
    }
}

impl std::error::Error for BlockchainError {}

use std::fmt;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blockchain_creation() {
        let bc = Blockchain::new();
        assert_eq!(bc.get_height(), 0);
        assert_eq!(bc.chain.len(), 1);
    }

    #[test]
    fn test_mining() {
        let mut bc = Blockchain::new();
        let wallet = Wallet::new();
        
        let block = bc.mine_block(wallet.address.clone()).unwrap();
        assert_eq!(bc.get_height(), 1);
        assert!(block.is_valid(DIFFICULTY));
    }

    #[test]
    fn test_transaction() {
        let mut bc = Blockchain::new();
        let wallet1 = Wallet::new();
        let wallet2 = Wallet::new();
        
        // 挖矿获得奖励
        bc.mine_block(wallet1.address.clone()).unwrap();
        
        // 创建转账交易
        let tx = bc.create_transaction(&wallet1, wallet2.address.clone(), 50_000_000).unwrap();
        assert!(bc.verify_transaction(&tx).unwrap());
    }

    #[test]
    fn test_balance() {
        let mut bc = Blockchain::new();
        let wallet = Wallet::new();
        
        bc.mine_block(wallet.address.clone()).unwrap();
        
        let balance = bc.get_balance(&wallet.address);
        assert_eq!(balance, MINING_REWARD);
    }
}
