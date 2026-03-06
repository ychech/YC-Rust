//! AI 智能合约生成模块
//!
//! 功能:
//! - 自然语言描述生成合约代码
//! - 合约模板库
//! - 代码安全审计建议
//! - 自动部署接口
//! - 合约验证

use super::AIError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 合约模板类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContractTemplate {
    /// ERC20 代币
    ERC20,
    /// ERC721 NFT
    ERC721,
    /// 多签钱包
    MultiSigWallet,
    /// 时间锁
    Timelock,
    /// 众筹合约
    Crowdfunding,
    /// 投票合约
    Voting,
    /// 质押挖矿
    Staking,
    /// DEX 流动性池
    LiquidityPool,
    /// 保险合约
    Insurance,
    /// 自定义
    Custom,
}

impl ContractTemplate {
    /// 获取模板名称
    pub fn name(&self) -> &'static str {
        match self {
            ContractTemplate::ERC20 => "ERC20 Token",
            ContractTemplate::ERC721 => "ERC721 NFT",
            ContractTemplate::MultiSigWallet => "Multi-Signature Wallet",
            ContractTemplate::Timelock => "Timelock",
            ContractTemplate::Crowdfunding => "Crowdfunding",
            ContractTemplate::Voting => "Voting",
            ContractTemplate::Staking => "Staking",
            ContractTemplate::LiquidityPool => "Liquidity Pool",
            ContractTemplate::Insurance => "Insurance",
            ContractTemplate::Custom => "Custom",
        }
    }

    /// 获取模板描述
    pub fn description(&self) -> &'static str {
        match self {
            ContractTemplate::ERC20 => "标准可替代代币合约，支持转账、授权、查询余额等功能",
            ContractTemplate::ERC721 => "非同质化代币合约，支持唯一资产的创建和转移",
            ContractTemplate::MultiSigWallet => "需要多个签名才能执行交易的多签钱包",
            ContractTemplate::Timelock => "延迟执行交易的时间锁合约",
            ContractTemplate::Crowdfunding => "去中心化众筹合约，支持目标金额和退款机制",
            ContractTemplate::Voting => "链上投票合约，支持提案创建和投票计数",
            ContractTemplate::Staking => "代币质押挖矿合约，支持质押、解押和收益分配",
            ContractTemplate::LiquidityPool => "去中心化交易所流动性池合约",
            ContractTemplate::Insurance => "去中心化保险合约，支持理赔和保费管理",
            ContractTemplate::Custom => "自定义合约，根据需求生成",
        }
    }
}

/// 合约生成请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractGenerationRequest {
    /// 自然语言描述
    pub description: String,
    /// 指定模板 (可选)
    pub template: Option<ContractTemplate>,
    /// 合约名称
    pub name: String,
    /// 符号/缩写
    pub symbol: Option<String>,
    /// 初始参数
    pub parameters: HashMap<String, String>,
    /// 目标语言
    pub language: ContractLanguage,
    /// 安全级别
    pub security_level: SecurityLevel,
}

/// 合约语言
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ContractLanguage {
    /// Rust (WASM)
    Rust,
    /// Solidity (EVM兼容)
    Solidity,
    /// Move
    Move,
}

/// 安全级别
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SecurityLevel {
    /// 基础
    Basic,
    /// 标准
    Standard,
    /// 高安全
    High,
    /// 审计级
    Audited,
}

/// 合约生成结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractGenerationResult {
    /// 生成的合约代码
    pub code: String,
    /// 合约语言
    pub language: ContractLanguage,
    /// ABI 接口
    pub abi: Vec<ContractAbi>,
    /// 字节码 (如果适用)
    pub bytecode: Option<String>,
    /// 部署参数
    pub deploy_params: Vec<DeployParam>,
    /// 安全审计报告
    pub security_report: SecurityReport,
    /// 使用说明
    pub usage_guide: String,
    /// 估计的 Gas 费用
    pub estimated_gas: u64,
}

/// 合约 ABI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAbi {
    /// 函数/事件名称
    pub name: String,
    /// 类型
    pub abi_type: AbiType,
    /// 输入参数
    pub inputs: Vec<AbiParam>,
    /// 输出参数
    pub outputs: Vec<AbiParam>,
    /// 状态可变性
    pub state_mutability: StateMutability,
}

/// ABI 类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AbiType {
    Function,
    Event,
    Constructor,
    Fallback,
}

/// ABI 参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiParam {
    pub name: String,
    pub param_type: String,
    pub indexed: bool,
}

/// 状态可变性
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum StateMutability {
    Pure,
    View,
    NonPayable,
    Payable,
}

/// 部署参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployParam {
    pub name: String,
    pub param_type: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
}

/// 安全审计报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    /// 风险等级
    pub risk_level: RiskLevel,
    /// 发现的问题
    pub issues: Vec<SecurityIssue>,
    /// 建议
    pub recommendations: Vec<String>,
    /// 检查项总数
    pub total_checks: usize,
    /// 通过检查数
    pub passed_checks: usize,
}

/// 风险等级
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// 安全问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    /// 严重程度
    pub severity: IssueSeverity,
    /// 标题
    pub title: String,
    /// 描述
    pub description: String,
    /// 位置
    pub location: String,
    /// 修复建议
    pub fix_suggestion: String,
}

/// 问题严重程度
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum IssueSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// 合约生成器
#[derive(Debug)]
pub struct ContractGenerator {
    /// AI API 端点
    api_endpoint: Option<String>,
    /// 模板库
    templates: HashMap<ContractTemplate, String>,
    /// 代码片段库
    snippets: HashMap<String, String>,
}

impl ContractGenerator {
    /// 创建新的合约生成器
    pub fn new(api_endpoint: Option<String>) -> Self {
        let mut generator = ContractGenerator {
            api_endpoint,
            templates: HashMap::new(),
            snippets: HashMap::new(),
        };

        generator.initialize_templates();
        generator.initialize_snippets();
        generator
    }

    /// 初始化合约模板
    fn initialize_templates(&mut self) {
        // ERC20 模板
        self.templates.insert(ContractTemplate::ERC20, r#"// ERC20 Token Contract
// Generated by YRust AI Contract Generator

pub struct ERC20Token {
    name: String,
    symbol: String,
    decimals: u8,
    total_supply: u128,
    balances: HashMap<String, u128>,
    allowances: HashMap<(String, String), u128>,
}

impl ERC20Token {
    pub fn new(name: String, symbol: String, decimals: u8, initial_supply: u128) -> Self {
        let mut token = ERC20Token {
            name,
            symbol,
            decimals,
            total_supply: initial_supply,
            balances: HashMap::new(),
            allowances: HashMap::new(),
        };
        token.balances.insert(msg::sender(), initial_supply);
        token
    }

    pub fn transfer(&mut self, to: String, amount: u128) -> Result<(), &'static str> {
        let from = msg::sender();
        let from_balance = self.balances.get(&from).copied().unwrap_or(0);
        
        if from_balance < amount {
            return Err("Insufficient balance");
        }
        
        self.balances.insert(from.clone(), from_balance - amount);
        let to_balance = self.balances.get(&to).copied().unwrap_or(0);
        self.balances.insert(to, to_balance + amount);
        
        emit!(Transfer { from, to, amount });
        Ok(())
    }

    pub fn approve(&mut self, spender: String, amount: u128) {
        let owner = msg::sender();
        self.allowances.insert((owner.clone(), spender.clone()), amount);
        emit!(Approval { owner, spender, amount });
    }

    pub fn transfer_from(&mut self, from: String, to: String, amount: u128) -> Result<(), &'static str> {
        let spender = msg::sender();
        let allowance = self.allowances.get(&(from.clone(), spender.clone())).copied().unwrap_or(0);
        
        if allowance < amount {
            return Err("Insufficient allowance");
        }
        
        let from_balance = self.balances.get(&from).copied().unwrap_or(0);
        if from_balance < amount {
            return Err("Insufficient balance");
        }
        
        self.allowances.insert((from.clone(), spender.clone()), allowance - amount);
        self.balances.insert(from.clone(), from_balance - amount);
        let to_balance = self.balances.get(&to).copied().unwrap_or(0);
        self.balances.insert(to, to_balance + amount);
        
        emit!(Transfer { from, to, amount });
        Ok(())
    }

    pub fn balance_of(&self, account: String) -> u128 {
        self.balances.get(&account).copied().unwrap_or(0)
    }

    pub fn allowance(&self, owner: String, spender: String) -> u128 {
        self.allowances.get(&(owner, spender)).copied().unwrap_or(0)
    }
}

event Transfer {
    from: String,
    to: String,
    amount: u128,
}

event Approval {
    owner: String,
    spender: String,
    amount: u128,
}"#.to_string());

        // 多签钱包模板
        self.templates.insert(ContractTemplate::MultiSigWallet, r#"// Multi-Signature Wallet Contract
// Generated by YRust AI Contract Generator

pub struct MultiSigWallet {
    owners: Vec<String>,
    required_confirmations: u32,
    transactions: Vec<Transaction>,
    confirmations: HashMap<(u64, String), bool>,
}

struct Transaction {
    to: String,
    value: u128,
    data: Vec<u8>,
    executed: bool,
}

impl MultiSigWallet {
    pub fn new(owners: Vec<String>, required: u32) -> Result<Self, &'static str> {
        if owners.is_empty() {
            return Err("Owners required");
        }
        if required == 0 || required > owners.len() as u32 {
            return Err("Invalid required confirmations");
        }
        
        Ok(MultiSigWallet {
            owners,
            required_confirmations: required,
            transactions: Vec::new(),
            confirmations: HashMap::new(),
        })
    }

    pub fn submit_transaction(&mut self, to: String, value: u128, data: Vec<u8>) -> u64 {
        let tx_index = self.transactions.len() as u64;
        self.transactions.push(Transaction {
            to,
            value,
            data,
            executed: false,
        });
        emit!(Submission { transaction_id: tx_index });
        tx_index
    }

    pub fn confirm_transaction(&mut self, tx_id: u64) -> Result<(), &'static str> {
        let sender = msg::sender();
        
        if !self.is_owner(&sender) {
            return Err("Not an owner");
        }
        
        self.confirmations.insert((tx_id, sender), true);
        emit!(Confirmation { sender, transaction_id: tx_id });
        
        if self.is_confirmed(tx_id) {
            self.execute_transaction(tx_id)?;
        }
        
        Ok(())
    }

    fn is_confirmed(&self, tx_id: u64) -> bool {
        let count = self.owners
            .iter()
            .filter(|owner| self.confirmations.get(&(tx_id, owner.to_string())).copied().unwrap_or(false))
            .count() as u32;
        count >= self.required_confirmations
    }

    fn execute_transaction(&mut self, tx_id: u64) -> Result<(), &'static str> {
        let tx = self.transactions.get_mut(tx_id as usize)
            .ok_or("Transaction does not exist")?;
        
        if tx.executed {
            return Err("Already executed");
        }
        
        tx.executed = true;
        // 执行转账逻辑...
        
        emit!(Execution { transaction_id: tx_id });
        Ok(())
    }

    fn is_owner(&self, addr: &str) -> bool {
        self.owners.contains(&addr.to_string())
    }
}

event Submission { transaction_id: u64 }
event Confirmation { sender: String, transaction_id: u64 }
event Execution { transaction_id: u64 }"#.to_string());

        // 质押挖矿模板
        self.templates.insert(ContractTemplate::Staking, r#"// Staking Contract
// Generated by YRust AI Contract Generator

pub struct StakingContract {
    staking_token: String,
    reward_token: String,
    reward_per_block: u128,
    total_staked: u128,
    pool_info: PoolInfo,
    user_info: HashMap<String, UserInfo>,
}

struct PoolInfo {
    last_reward_block: u64,
    acc_reward_per_share: u128,
}

struct UserInfo {
    amount: u128,
    reward_debt: u128,
    pending_rewards: u128,
}

impl StakingContract {
    pub fn new(staking_token: String, reward_token: String, reward_per_block: u128) -> Self {
        StakingContract {
            staking_token,
            reward_token,
            reward_per_block,
            total_staked: 0,
            pool_info: PoolInfo {
                last_reward_block: block::number(),
                acc_reward_per_share: 0,
            },
            user_info: HashMap::new(),
        }
    }

    pub fn deposit(&mut self, amount: u128) -> Result<(), &'static str> {
        self.update_pool();
        
        let sender = msg::sender();
        let user = self.user_info.entry(sender.clone()).or_insert(UserInfo {
            amount: 0,
            reward_debt: 0,
            pending_rewards: 0,
        });
        
        if user.amount > 0 {
            let pending = self.pending_reward(&sender);
            user.pending_rewards += pending;
        }
        
        // 转账逻辑...
        user.amount += amount;
        self.total_staked += amount;
        user.reward_debt = user.amount * self.pool_info.acc_reward_per_share / 1e12 as u128;
        
        emit!(Deposit { user: sender, amount });
        Ok(())
    }

    pub fn withdraw(&mut self, amount: u128) -> Result<(), &'static str> {
        self.update_pool();
        
        let sender = msg::sender();
        let user = self.user_info.get_mut(&sender).ok_or("No deposit")?;
        
        if user.amount < amount {
            return Err("Insufficient staked amount");
        }
        
        let pending = self.pending_reward(&sender);
        user.pending_rewards += pending;
        
        user.amount -= amount;
        self.total_staked -= amount;
        user.reward_debt = user.amount * self.pool_info.acc_reward_per_share / 1e12 as u128;
        
        // 转账逻辑...
        
        emit!(Withdraw { user: sender, amount });
        Ok(())
    }

    pub fn claim(&mut self) -> Result<u128, &'static str> {
        self.update_pool();
        
        let sender = msg::sender();
        let user = self.user_info.get_mut(&sender).ok_or("No deposit")?;
        
        let pending = self.pending_reward(&sender) + user.pending_rewards;
        if pending == 0 {
            return Err("No rewards to claim");
        }
        
        user.pending_rewards = 0;
        user.reward_debt = user.amount * self.pool_info.acc_reward_per_share / 1e12 as u128;
        
        // 转账奖励...
        
        emit!(Claim { user: sender, amount: pending });
        Ok(pending)
    }

    fn update_pool(&mut self) {
        let current_block = block::number();
        if current_block <= self.pool_info.last_reward_block {
            return;
        }
        
        if self.total_staked == 0 {
            self.pool_info.last_reward_block = current_block;
            return;
        }
        
        let multiplier = current_block - self.pool_info.last_reward_block;
        let reward = multiplier as u128 * self.reward_per_block;
        self.pool_info.acc_reward_per_share += reward * 1e12 as u128 / self.total_staked;
        self.pool_info.last_reward_block = current_block;
    }

    fn pending_reward(&self, user: &str) -> u128 {
        let user_info = match self.user_info.get(user) {
            Some(u) => u,
            None => return 0,
        };
        
        let acc_reward = self.pool_info.acc_reward_per_share;
        user_info.amount * acc_reward / 1e12 as u128 - user_info.reward_debt
    }
}

event Deposit { user: String, amount: u128 }
event Withdraw { user: String, amount: u128 }
event Claim { user: String, amount: u128 }"#.to_string());
    }

    /// 初始化代码片段
    fn initialize_snippets(&mut self) {
        self.snippets.insert("access_control".to_string(), r#"
// Access Control Modifier
fn only_owner(&self) -> Result<(), &'static str> {
    if msg::sender() != self.owner {
        return Err("Only owner can call this function");
    }
    Ok(())
}
"#.to_string());

        self.snippets.insert("reentrancy_guard".to_string(), r#"
// Reentrancy Guard
static mut LOCKED: bool = false;

fn non_reentrant<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    unsafe {
        require!(!LOCKED, "Reentrant call");
        LOCKED = true;
        let result = f();
        LOCKED = false;
        result
    }
}
"#.to_string());

        self.snippets.insert("safe_math".to_string(), r#"
// Safe Math
fn safe_add(a: u128, b: u128) -> Result<u128, &'static str> {
    a.checked_add(b).ok_or("Addition overflow")
}

fn safe_sub(a: u128, b: u128) -> Result<u128, &'static str> {
    a.checked_sub(b).ok_or("Subtraction underflow")
}

fn safe_mul(a: u128, b: u128) -> Result<u128, &'static str> {
    a.checked_mul(b).ok_or("Multiplication overflow")
}

fn safe_div(a: u128, b: u128) -> Result<u128, &'static str> {
    if b == 0 {
        return Err("Division by zero");
    }
    Ok(a / b)
}
"#.to_string());
    }

    /// 生成合约
    pub fn generate_contract(&self, request: ContractGenerationRequest) -> Result<ContractGenerationResult, AIError> {
        // 1. 分析需求并选择模板
        let template = request.template.unwrap_or_else(|| {
            self.infer_template_from_description(&request.description)
        });

        // 2. 获取基础模板代码
        let mut code = self.templates.get(&template)
            .cloned()
            .unwrap_or_else(|| self.generate_custom_contract(&request));

        // 3. 根据安全级别添加保护代码
        code = self.apply_security_features(code, request.security_level);

        // 4. 替换参数
        code = self.replace_parameters(code, &request);

        // 5. 生成 ABI
        let abi = self.generate_abi(&template, &request);

        // 6. 生成部署参数
        let deploy_params = self.generate_deploy_params(&template, &request);

        // 7. 进行安全审计
        let security_report = self.perform_security_audit(&code);

        // 8. 生成使用说明
        let usage_guide = self.generate_usage_guide(&template, &abi);

        // 9. 估算 Gas
        let estimated_gas = self.estimate_gas(&template);

        Ok(ContractGenerationResult {
            code,
            language: request.language,
            abi,
            bytecode: None, // 需要编译
            deploy_params,
            security_report,
            usage_guide,
            estimated_gas,
        })
    }

    /// 从描述推断模板
    fn infer_template_from_description(&self, description: &str) -> ContractTemplate {
        let desc_lower = description.to_lowercase();
        
        if desc_lower.contains("token") || desc_lower.contains("代币") || desc_lower.contains("erc20") {
            ContractTemplate::ERC20
        } else if desc_lower.contains("nft") || desc_lower.contains("erc721") {
            ContractTemplate::ERC721
        } else if desc_lower.contains("multi") || desc_lower.contains("多签") {
            ContractTemplate::MultiSigWallet
        } else if desc_lower.contains("staking") || desc_lower.contains("质押") {
            ContractTemplate::Staking
        } else if desc_lower.contains("vote") || desc_lower.contains("投票") {
            ContractTemplate::Voting
        } else if desc_lower.contains("crowd") || desc_lower.contains("众筹") {
            ContractTemplate::Crowdfunding
        } else if desc_lower.contains("time") || desc_lower.contains("时间锁") {
            ContractTemplate::Timelock
        } else if desc_lower.contains("liquidity") || desc_lower.contains("流动性") {
            ContractTemplate::LiquidityPool
        } else if desc_lower.contains("insurance") || desc_lower.contains("保险") {
            ContractTemplate::Insurance
        } else {
            ContractTemplate::Custom
        }
    }

    /// 生成自定义合约
    fn generate_custom_contract(&self, request: &ContractGenerationRequest) -> String {
        format!(r#"// Custom Contract: {}
// Generated by YRust AI Contract Generator
// Description: {}

use std::collections::HashMap;

pub struct {} {{
    owner: String,
    // TODO: Add your state variables here
}}

impl {} {{
    pub fn new() -> Self {{
        {} {{
            owner: msg::sender(),
        }}
    }}

    // TODO: Add your functions here
    
    fn only_owner(&self) -> Result<(), &'static str> {{
        if msg::sender() != self.owner {{
            return Err("Only owner");
        }}
        Ok(())
    }}
}}
"#, 
            request.name,
            request.description,
            request.name,
            request.name,
            request.name
        )
    }

    /// 应用安全特性
    fn apply_security_features(&self, mut code: String, level: SecurityLevel) -> String {
        match level {
            SecurityLevel::Basic => code,
            SecurityLevel::Standard => {
                // 添加 SafeMath
                if !code.contains("safe_add") {
                    if let Some(safe_math) = self.snippets.get("safe_math") {
                        code = format!("{}\n{}", safe_math, code);
                    }
                }
                code
            }
            SecurityLevel::High | SecurityLevel::Audited => {
                // 添加所有安全特性
                if let Some(safe_math) = self.snippets.get("safe_math") {
                    code = format!("{}\n{}", safe_math, code);
                }
                if let Some(reentrancy) = self.snippets.get("reentrancy_guard") {
                    code = format!("{}\n{}", reentrancy, code);
                }
                code
            }
        }
    }

    /// 替换参数
    fn replace_parameters(&self, code: String, request: &ContractGenerationRequest) -> String {
        let mut result = code;
        
        // 替换合约名称
        result = result.replace("ERC20Token", &request.name);
        result = result.replace("MultiSigWallet", &request.name);
        result = result.replace("StakingContract", &request.name);
        
        // 替换其他参数
        if let Some(symbol) = &request.symbol {
            result = result.replace("symbol: String", &format!("symbol: String = \"{}\".to_string()", symbol));
        }
        
        for (key, value) in &request.parameters {
            result = result.replace(&format!("{{{{{}}}}}" , key), value);
        }
        
        result
    }

    /// 生成 ABI
    fn generate_abi(&self, template: &ContractTemplate, _request: &ContractGenerationRequest) -> Vec<ContractAbi> {
        match template {
            ContractTemplate::ERC20 => vec![
                ContractAbi {
                    name: "transfer".to_string(),
                    abi_type: AbiType::Function,
                    inputs: vec![
                        AbiParam { name: "to".to_string(), param_type: "address".to_string(), indexed: false },
                        AbiParam { name: "amount".to_string(), param_type: "uint256".to_string(), indexed: false },
                    ],
                    outputs: vec![AbiParam { name: "success".to_string(), param_type: "bool".to_string(), indexed: false }],
                    state_mutability: StateMutability::NonPayable,
                },
                ContractAbi {
                    name: "balanceOf".to_string(),
                    abi_type: AbiType::Function,
                    inputs: vec![AbiParam { name: "account".to_string(), param_type: "address".to_string(), indexed: false }],
                    outputs: vec![AbiParam { name: "balance".to_string(), param_type: "uint256".to_string(), indexed: false }],
                    state_mutability: StateMutability::View,
                },
                ContractAbi {
                    name: "Transfer".to_string(),
                    abi_type: AbiType::Event,
                    inputs: vec![
                        AbiParam { name: "from".to_string(), param_type: "address".to_string(), indexed: true },
                        AbiParam { name: "to".to_string(), param_type: "address".to_string(), indexed: true },
                        AbiParam { name: "amount".to_string(), param_type: "uint256".to_string(), indexed: false },
                    ],
                    outputs: vec![],
                    state_mutability: StateMutability::View,
                },
            ],
            _ => vec![],
        }
    }

    /// 生成部署参数
    fn generate_deploy_params(&self, template: &ContractTemplate, request: &ContractGenerationRequest) -> Vec<DeployParam> {
        match template {
            ContractTemplate::ERC20 => vec![
                DeployParam {
                    name: "name".to_string(),
                    param_type: "string".to_string(),
                    description: "代币名称".to_string(),
                    required: true,
                    default_value: Some(request.name.clone()),
                },
                DeployParam {
                    name: "symbol".to_string(),
                    param_type: "string".to_string(),
                    description: "代币符号".to_string(),
                    required: true,
                    default_value: request.symbol.clone(),
                },
                DeployParam {
                    name: "decimals".to_string(),
                    param_type: "uint8".to_string(),
                    description: "小数位数".to_string(),
                    required: false,
                    default_value: Some("18".to_string()),
                },
                DeployParam {
                    name: "initialSupply".to_string(),
                    param_type: "uint256".to_string(),
                    description: "初始供应量".to_string(),
                    required: true,
                    default_value: Some("1000000000000000000000000".to_string()),
                },
            ],
            ContractTemplate::MultiSigWallet => vec![
                DeployParam {
                    name: "owners".to_string(),
                    param_type: "address[]".to_string(),
                    description: "所有者地址列表".to_string(),
                    required: true,
                    default_value: None,
                },
                DeployParam {
                    name: "required".to_string(),
                    param_type: "uint256".to_string(),
                    description: "需要的确认数".to_string(),
                    required: true,
                    default_value: Some("2".to_string()),
                },
            ],
            ContractTemplate::Staking => vec![
                DeployParam {
                    name: "stakingToken".to_string(),
                    param_type: "address".to_string(),
                    description: "质押代币地址".to_string(),
                    required: true,
                    default_value: None,
                },
                DeployParam {
                    name: "rewardToken".to_string(),
                    param_type: "address".to_string(),
                    description: "奖励代币地址".to_string(),
                    required: true,
                    default_value: None,
                },
                DeployParam {
                    name: "rewardPerBlock".to_string(),
                    param_type: "uint256".to_string(),
                    description: "每区块奖励".to_string(),
                    required: true,
                    default_value: Some("1000000000000000000".to_string()),
                },
            ],
            _ => vec![],
        }
    }

    /// 执行安全审计
    fn perform_security_audit(&self, code: &str) -> SecurityReport {
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();
        let mut total_checks = 0;
        let mut passed_checks = 0;

        // 检查 1: 整数溢出
        total_checks += 1;
        if code.contains("checked_add") || code.contains("safe_add") {
            passed_checks += 1;
        } else {
            issues.push(SecurityIssue {
                severity: IssueSeverity::Error,
                title: "整数溢出风险".to_string(),
                description: "未使用安全数学运算".to_string(),
                location: "全局".to_string(),
                fix_suggestion: "使用 safe_add, safe_sub 等安全数学函数".to_string(),
            });
        }

        // 检查 2: 重入攻击
        total_checks += 1;
        if code.contains("non_reentrant") || code.contains("LOCKED") {
            passed_checks += 1;
        } else if code.contains("transfer") {
            issues.push(SecurityIssue {
                severity: IssueSeverity::Warning,
                title: "潜在重入风险".to_string(),
                description: "转账操作可能受到重入攻击".to_string(),
                location: "transfer 函数".to_string(),
                fix_suggestion: "添加重入锁保护".to_string(),
            });
        } else {
            passed_checks += 1;
        }

        // 检查 3: 访问控制
        total_checks += 1;
        if code.contains("only_owner") || code.contains("require") {
            passed_checks += 1;
        } else {
            issues.push(SecurityIssue {
                severity: IssueSeverity::Warning,
                title: "缺少访问控制".to_string(),
                description: "关键函数缺少访问控制检查".to_string(),
                location: "全局".to_string(),
                fix_suggestion: "添加 only_owner 或其他访问控制修饰符".to_string(),
            });
        }

        // 生成建议
        if issues.is_empty() {
            recommendations.push("合约看起来安全，建议进行专业审计".to_string());
        } else {
            recommendations.push("请修复所有安全问题后再部署".to_string());
            recommendations.push("建议聘请专业安全团队进行审计".to_string());
        }

        let risk_level = if issues.iter().any(|i| matches!(i.severity, IssueSeverity::Critical)) {
            RiskLevel::Critical
        } else if issues.iter().any(|i| matches!(i.severity, IssueSeverity::Error)) {
            RiskLevel::High
        } else if !issues.is_empty() {
            RiskLevel::Medium
        } else {
            RiskLevel::Low
        };

        SecurityReport {
            risk_level,
            issues,
            recommendations,
            total_checks,
            passed_checks,
        }
    }

    /// 生成使用说明
    fn generate_usage_guide(&self, template: &ContractTemplate, abi: &[ContractAbi]) -> String {
        let mut guide = format!("# {} 使用说明\n\n", template.name());
        
        guide.push_str("## 部署步骤\n\n");
        guide.push_str("1. 准备部署参数\n");
        guide.push_str("2. 编译合约\n");
        guide.push_str("3. 部署到区块链\n");
        guide.push_str("4. 验证部署\n\n");
        
        guide.push_str("## 函数接口\n\n");
        for item in abi {
            if let AbiType::Function = item.abi_type {
                guide.push_str(&format!("### {}\n", item.name));
                guide.push_str(&format!("- 状态可变性: {:?}\n", item.state_mutability));
                guide.push_str("- 输入参数:\n");
                for input in &item.inputs {
                    guide.push_str(&format!("  - {}: {}\n", input.name, input.param_type));
                }
                guide.push_str("\n");
            }
        }
        
        guide.push_str("## 注意事项\n\n");
        guide.push_str("- 请确保有足够的 Gas 费用\n");
        guide.push_str("- 部署前请在测试网测试\n");
        guide.push_str("- 保存好合约地址和 ABI\n");
        
        guide
    }

    /// 估算 Gas
    fn estimate_gas(&self, template: &ContractTemplate) -> u64 {
        match template {
            ContractTemplate::ERC20 => 500000,
            ContractTemplate::ERC721 => 800000,
            ContractTemplate::MultiSigWallet => 1200000,
            ContractTemplate::Staking => 1500000,
            ContractTemplate::Timelock => 600000,
            ContractTemplate::Crowdfunding => 1000000,
            ContractTemplate::Voting => 900000,
            ContractTemplate::LiquidityPool => 2000000,
            ContractTemplate::Insurance => 1800000,
            ContractTemplate::Custom => 1000000,
        }
    }

    /// 获取可用模板列表
    pub fn get_available_templates(&self) -> Vec<(ContractTemplate, &'static str, &'static str)> {
        vec![
            (ContractTemplate::ERC20, ContractTemplate::ERC20.name(), ContractTemplate::ERC20.description()),
            (ContractTemplate::ERC721, ContractTemplate::ERC721.name(), ContractTemplate::ERC721.description()),
            (ContractTemplate::MultiSigWallet, ContractTemplate::MultiSigWallet.name(), ContractTemplate::MultiSigWallet.description()),
            (ContractTemplate::Timelock, ContractTemplate::Timelock.name(), ContractTemplate::Timelock.description()),
            (ContractTemplate::Crowdfunding, ContractTemplate::Crowdfunding.name(), ContractTemplate::Crowdfunding.description()),
            (ContractTemplate::Voting, ContractTemplate::Voting.name(), ContractTemplate::Voting.description()),
            (ContractTemplate::Staking, ContractTemplate::Staking.name(), ContractTemplate::Staking.description()),
            (ContractTemplate::LiquidityPool, ContractTemplate::LiquidityPool.name(), ContractTemplate::LiquidityPool.description()),
            (ContractTemplate::Insurance, ContractTemplate::Insurance.name(), ContractTemplate::Insurance.description()),
            (ContractTemplate::Custom, ContractTemplate::Custom.name(), ContractTemplate::Custom.description()),
        ]
    }
}

impl Default for ContractGenerator {
    fn default() -> Self {
        Self::new(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_generator_creation() {
        let generator = ContractGenerator::new(None);
        assert!(!generator.templates.is_empty());
    }

    #[test]
    fn test_generate_erc20() {
        let generator = ContractGenerator::new(None);
        
        let request = ContractGenerationRequest {
            description: "创建一个代币".to_string(),
            template: Some(ContractTemplate::ERC20),
            name: "MyToken".to_string(),
            symbol: Some("MTK".to_string()),
            parameters: HashMap::new(),
            language: ContractLanguage::Rust,
            security_level: SecurityLevel::Standard,
        };
        
        let result = generator.generate_contract(request).unwrap();
        assert!(result.code.contains("MyToken"));
        assert!(!result.abi.is_empty());
        assert!(result.estimated_gas > 0);
    }

    #[test]
    fn test_security_audit() {
        let generator = ContractGenerator::new(None);
        
        let safe_code = r#"
            fn safe_add(a: u128, b: u128) -> u128 {
                a.checked_add(b).unwrap()
            }
        "#;
        
        let report = generator.perform_security_audit(safe_code);
        assert!(report.passed_checks > 0);
    }

    #[test]
    fn test_template_inference() {
        let generator = ContractGenerator::new(None);
        
        assert_eq!(
            generator.infer_template_from_description("创建一个代币"),
            ContractTemplate::ERC20
        );
        
        assert_eq!(
            generator.infer_template_from_description("多签钱包"),
            ContractTemplate::MultiSigWallet
        );
        
        assert_eq!(
            generator.infer_template_from_description("质押挖矿"),
            ContractTemplate::Staking
        );
    }
}
