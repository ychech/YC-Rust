//! AI 钱包助手模块
//!
//! 功能:
//! - 语音转账指令解析
//! - 智能交易提醒
//! - 地址簿管理
//! - 交易历史分析
//! - 智能推荐

use super::AIError;
use crate::core::{Transaction, Amount};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 语音指令类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VoiceCommand {
    /// 转账
    Transfer,
    /// 查询余额
    CheckBalance,
    /// 查询交易历史
    CheckHistory,
    /// 添加联系人
    AddContact,
    /// 设置提醒
    SetReminder,
    /// 获取推荐
    GetRecommendation,
    /// 帮助
    Help,
    /// 未知
    Unknown,
}

/// 语音转账请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceTransferRequest {
    /// 语音转录文本
    pub transcript: String,
    /// 发送者地址
    pub from_address: String,
    /// 识别的金额
    pub amount: Option<Amount>,
    /// 识别的接收者
    pub recipient: Option<String>,
    /// 识别的代币类型
    pub token: Option<String>,
}

/// 语音转账结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoiceTransferResult {
    /// 是否解析成功
    pub success: bool,
    /// 解析的指令
    pub command: VoiceCommand,
    /// 转账金额
    pub amount: Option<Amount>,
    /// 接收者地址
    pub to_address: Option<String>,
    /// 代币类型
    pub token: String,
    /// 确认消息
    pub confirmation_message: String,
    /// 是否需要确认
    pub requires_confirmation: bool,
    /// 错误信息
    pub error: Option<String>,
}

/// 智能提醒
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartReminder {
    /// 提醒ID
    pub id: String,
    /// 提醒类型
    pub reminder_type: ReminderType,
    /// 触发条件
    pub trigger_condition: TriggerCondition,
    /// 提醒消息
    pub message: String,
    /// 是否启用
    pub enabled: bool,
    /// 创建时间
    pub created_at: u64,
    /// 最后触发时间
    pub last_triggered: Option<u64>,
}

/// 提醒类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReminderType {
    /// 价格提醒
    PriceAlert,
    /// 交易确认
    TransactionConfirm,
    /// 余额不足
    LowBalance,
    /// 大额转账
    LargeTransfer,
    /// 定期报告
    PeriodicReport,
    /// 安全提醒
    SecurityAlert,
}

/// 触发条件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TriggerCondition {
    /// 价格阈值
    PriceThreshold { token: String, above: Option<Amount>, below: Option<Amount> },
    /// 余额阈值
    BalanceThreshold { below: Amount },
    /// 交易金额
    TransactionAmount { above: Amount },
    /// 时间间隔
    TimeInterval { seconds: u64 },
    /// 特定事件
    Event { event_type: String },
}

/// 联系人
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    /// 联系人名称
    pub name: String,
    /// 地址
    pub address: String,
    /// 标签
    pub tags: Vec<String>,
    /// 交易次数
    pub transaction_count: u32,
    /// 总交易金额
    pub total_volume: Amount,
    /// 添加时间
    pub added_at: u64,
}

/// 交易分析
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionAnalysis {
    /// 总交易数
    pub total_transactions: u32,
    /// 总发送金额
    pub total_sent: Amount,
    /// 总接收金额
    pub total_received: Amount,
    /// 手续费总计
    pub total_fees: Amount,
    /// 最频繁联系人
    pub top_contacts: Vec<(String, u32)>,
    /// 交易模式
    pub patterns: Vec<TransactionPattern>,
    /// 异常交易
    pub anomalies: Vec<Anomaly>,
}

/// 交易模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPattern {
    /// 模式类型
    pub pattern_type: String,
    /// 描述
    pub description: String,
    /// 频率
    pub frequency: f32,
    /// 置信度
    pub confidence: f32,
}

/// 异常
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Anomaly {
    /// 异常类型
    pub anomaly_type: String,
    /// 描述
    pub description: String,
    /// 严重程度
    pub severity: u8,
    /// 相关交易
    pub related_tx: Option<String>,
}

/// 智能推荐
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartRecommendation {
    /// 推荐类型
    pub recommendation_type: RecommendationType,
    /// 标题
    pub title: String,
    /// 描述
    pub description: String,
    /// 置信度
    pub confidence: f32,
    /// 行动建议
    pub action: Option<String>,
}

/// 推荐类型
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RecommendationType {
    /// 节省手续费
    SaveFees,
    /// 安全建议
    Security,
    /// 投资机会
    Investment,
    /// 操作优化
    Optimization,
    /// 新功能
    NewFeature,
}

/// AI 钱包助手
#[derive(Debug)]
pub struct AIWalletAssistant {
    /// 地址簿
    address_book: HashMap<String, Contact>,
    /// 智能提醒
    reminders: Vec<SmartReminder>,
    /// 交易历史缓存
    transaction_history: Vec<Transaction>,
    /// 用户偏好
    preferences: UserPreferences,
    /// 语音指令模式
    voice_patterns: HashMap<VoiceCommand, Vec<String>>,
}

/// 用户偏好
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    /// 默认代币
    pub default_token: String,
    /// 提醒启用
    pub reminders_enabled: bool,
    /// 语音确认
    pub voice_confirmation: bool,
    /// 大额阈值
    pub large_amount_threshold: Amount,
    /// 常用联系人
    pub frequent_contacts: Vec<String>,
}

impl Default for UserPreferences {
    fn default() -> Self {
        UserPreferences {
            default_token: "YRU".to_string(),
            reminders_enabled: true,
            voice_confirmation: true,
            large_amount_threshold: 1_000_000_000, // 10 YRU
            frequent_contacts: Vec::new(),
        }
    }
}

impl AIWalletAssistant {
    /// 创建新的 AI 钱包助手
    pub fn new() -> Self {
        let mut assistant = AIWalletAssistant {
            address_book: HashMap::new(),
            reminders: Vec::new(),
            transaction_history: Vec::new(),
            preferences: UserPreferences::default(),
            voice_patterns: HashMap::new(),
        };

        assistant.initialize_voice_patterns();
        assistant
    }

    /// 初始化语音指令模式
    fn initialize_voice_patterns(&mut self) {
        self.voice_patterns.insert(VoiceCommand::Transfer, vec![
            "转账".to_string(),
            "发送".to_string(),
            "转给".to_string(),
            "转钱".to_string(),
            "send".to_string(),
            "transfer".to_string(),
            "pay".to_string(),
        ]);

        self.voice_patterns.insert(VoiceCommand::CheckBalance, vec![
            "余额".to_string(),
            "多少钱".to_string(),
            "有多少".to_string(),
            "balance".to_string(),
            "how much".to_string(),
        ]);

        self.voice_patterns.insert(VoiceCommand::CheckHistory, vec![
            "历史".to_string(),
            "记录".to_string(),
            "交易".to_string(),
            "history".to_string(),
            "transactions".to_string(),
        ]);

        self.voice_patterns.insert(VoiceCommand::AddContact, vec![
            "添加联系人".to_string(),
            "保存地址".to_string(),
            "add contact".to_string(),
            "save address".to_string(),
        ]);

        self.voice_patterns.insert(VoiceCommand::SetReminder, vec![
            "提醒".to_string(),
            "设置提醒".to_string(),
            "remind".to_string(),
            "alert".to_string(),
        ]);

        self.voice_patterns.insert(VoiceCommand::Help, vec![
            "帮助".to_string(),
            "怎么用".to_string(),
            "help".to_string(),
            "how to".to_string(),
        ]);
    }

    /// 处理语音转账指令
    pub fn process_voice_transfer(&self, request: VoiceTransferRequest) -> VoiceTransferResult {
        // 1. 识别指令类型
        let command = self.recognize_command(&request.transcript);

        if command != VoiceCommand::Transfer {
            return VoiceTransferResult {
                success: false,
                command,
                amount: None,
                to_address: None,
                token: self.preferences.default_token.clone(),
                confirmation_message: "未识别为转账指令".to_string(),
                requires_confirmation: false,
                error: Some("请使用转账相关的语音指令".to_string()),
            };
        }

        // 2. 解析金额
        let amount = request.amount.or_else(|| self.parse_amount(&request.transcript));

        // 3. 解析接收者
        let recipient = request.recipient.or_else(|| self.parse_recipient(&request.transcript));

        // 4. 解析代币类型
        let token = request.token.unwrap_or_else(|| self.parse_token(&request.transcript));

        // 5. 验证并生成确认消息
        match (amount, recipient.clone()) {
            (Some(amt), Some(recip)) => {
                let confirmation_message = format!(
                    "确认转账 {} {} 给 {}?",
                    self.format_amount(amt),
                    token,
                    recip
                );

                VoiceTransferResult {
                    success: true,
                    command,
                    amount: Some(amt),
                    to_address: Some(recip),
                    token,
                    confirmation_message,
                    requires_confirmation: self.preferences.voice_confirmation || amt > self.preferences.large_amount_threshold,
                    error: None,
                }
            }
            (None, _) => {
                VoiceTransferResult {
                    success: false,
                    command,
                    amount: None,
                    to_address: recipient,
                    token,
                    confirmation_message: "未识别转账金额".to_string(),
                    requires_confirmation: false,
                    error: Some("请明确说出转账金额".to_string()),
                }
            }
            (_, None) => {
                VoiceTransferResult {
                    success: false,
                    command,
                    amount,
                    to_address: None,
                    token,
                    confirmation_message: "未识别接收者".to_string(),
                    requires_confirmation: false,
                    error: Some("请明确说出接收者名称或地址".to_string()),
                }
            }
        }
    }

    /// 识别语音指令
    fn recognize_command(&self, transcript: &str) -> VoiceCommand {
        let text = transcript.to_lowercase();
        
        for (command, patterns) in &self.voice_patterns {
            for pattern in patterns {
                if text.contains(&pattern.to_lowercase()) {
                    return *command;
                }
            }
        }
        
        VoiceCommand::Unknown
    }

    /// 解析金额
    fn parse_amount(&self, text: &str) -> Option<Amount> {
        // 简化实现：从文本中提取数字
        let text = text.replace(",", "").replace("，", "");
        
        // 查找 "X个" 或 "X " 模式
        let patterns = [
            r"(\d+\.?\d*)\s*个",
            r"(\d+\.?\d*)\s*YRU",
            r"(\d+\.?\d*)\s*代币",
            r"转账\s*(\d+\.?\d*)",
            r"转\s*(\d+\.?\d*)",
        ];
        
        for pattern in &patterns {
            // 简化处理，实际应使用正则表达式
            if let Some(num_str) = self.extract_number_after_keyword(text.as_str(), pattern) {
                if let Ok(num) = num_str.parse::<f64>() {
                    return Some((num * 100_000_000.0) as Amount);
                }
            }
        }
        
        None
    }

    /// 提取数字 (简化实现)
    fn extract_number_after_keyword(&self, text: &str, _pattern: &str) -> Option<String> {
        // 简化实现：查找数字
        let mut result = String::new();
        let mut found_digit = false;
        
        for c in text.chars() {
            if c.is_ascii_digit() || c == '.' {
                result.push(c);
                found_digit = true;
            } else if found_digit && !result.ends_with('.') {
                break;
            }
        }
        
        if found_digit {
            Some(result)
        } else {
            None
        }
    }

    /// 解析接收者
    fn parse_recipient(&self, text: &str) -> Option<String> {
        // 1. 检查是否是联系人名称
        for (name, contact) in &self.address_book {
            if text.contains(name) {
                return Some(contact.address.clone());
            }
        }
        
        // 2. 尝试提取地址
        // 简化实现：查找长字符串
        for word in text.split_whitespace() {
            if word.len() > 30 && word.starts_with("Y") {
                return Some(word.to_string());
            }
        }
        
        // 3. 查找 "给XX" 或 "转给XX" 模式
        if let Some(pos) = text.find("给") {
            let after = &text[pos + 3..];
            if let Some(end) = after.find(|c: char| c.is_ascii_whitespace() || c == '，' || c == ',') {
                let name = &after[..end];
                if let Some(contact) = self.address_book.get(name) {
                    return Some(contact.address.clone());
                }
            }
        }
        
        None
    }

    /// 解析代币类型
    fn parse_token(&self, text: &str) -> String {
        let tokens = ["YRU", "BTC", "ETH", "USDT", "DAI"];
        
        for token in &tokens {
            if text.contains(token) {
                return token.to_string();
            }
        }
        
        self.preferences.default_token.clone()
    }

    /// 格式化金额
    fn format_amount(&self, amount: Amount) -> String {
        let yru = amount as f64 / 100_000_000.0;
        format!("{:.8}", yru)
    }

    /// 添加联系人
    pub fn add_contact(&mut self, name: String, address: String, tags: Vec<String>) -> Result<(), AIError> {
        if self.address_book.contains_key(&name) {
            return Err(AIError::WalletError("联系人已存在".to_string()));
        }
        
        let contact = Contact {
            name: name.clone(),
            address,
            tags,
            transaction_count: 0,
            total_volume: 0,
            added_at: current_timestamp(),
        };
        
        self.address_book.insert(name, contact);
        Ok(())
    }

    /// 获取联系人
    pub fn get_contact(&self, name: &str) -> Option<&Contact> {
        self.address_book.get(name)
    }

    /// 搜索联系人
    pub fn search_contacts(&self, query: &str) -> Vec<&Contact> {
        self.address_book
            .values()
            .filter(|c| {
                c.name.contains(query) || 
                c.address.contains(query) ||
                c.tags.iter().any(|t| t.contains(query))
            })
            .collect()
    }

    /// 添加智能提醒
    pub fn add_reminder(&mut self, reminder: SmartReminder) {
        self.reminders.push(reminder);
    }

    /// 检查并触发提醒
    pub fn check_reminders(&mut self, current_balance: Amount, _pending_txs: &[Transaction]) -> Vec<SmartReminder> {
        let now = current_timestamp();
        let mut triggered = Vec::new();
        
        for reminder in &mut self.reminders {
            if !reminder.enabled {
                continue;
            }
            
            // 检查冷却时间
            if let Some(last) = reminder.last_triggered {
                if now - last < 3600 { // 1小时内不重复触发
                    continue;
                }
            }
            
            let should_trigger = match &reminder.trigger_condition {
                TriggerCondition::BalanceThreshold { below } => current_balance < *below,
                TriggerCondition::TimeInterval { seconds } => {
                    reminder.last_triggered.map_or(true, |last| now - last >= *seconds)
                }
                _ => false,
            };
            
            if should_trigger {
                reminder.last_triggered = Some(now);
                triggered.push(reminder.clone());
            }
        }
        
        triggered
    }

    /// 分析交易历史
    pub fn analyze_transactions(&self, address: &str) -> TransactionAnalysis {
        let total_sent = 0u64;
        let mut total_received = 0u64;
        let total_fees = 0u64;
        let contact_frequency: HashMap<String, u32> = HashMap::new();
        
        for tx in &self.transaction_history {
            // 简化分析
            for output in &tx.outputs {
                if output.script_pubkey == address {
                    total_received += output.value;
                }
            }
        }
        
        // 找出最频繁的联系人
        let mut top_contacts: Vec<_> = contact_frequency.into_iter().collect();
        top_contacts.sort_by(|a, b| b.1.cmp(&a.1));
        top_contacts.truncate(5);
        
        TransactionAnalysis {
            total_transactions: self.transaction_history.len() as u32,
            total_sent,
            total_received,
            total_fees,
            top_contacts,
            patterns: self.detect_patterns(),
            anomalies: self.detect_anomalies(),
        }
    }

    /// 检测交易模式
    fn detect_patterns(&self) -> Vec<TransactionPattern> {
        let mut patterns = Vec::new();
        
        // 简化实现
        if self.transaction_history.len() > 10 {
            patterns.push(TransactionPattern {
                pattern_type: "活跃用户".to_string(),
                description: "您是一个活跃的用户，经常进行交易".to_string(),
                frequency: 0.8,
                confidence: 0.9,
            });
        }
        
        patterns
    }

    /// 检测异常
    fn detect_anomalies(&self) -> Vec<Anomaly> {
        let mut anomalies = Vec::new();
        
        // 简化实现
        for tx in &self.transaction_history {
            let total_value: Amount = tx.outputs.iter().map(|o| o.value).sum();
            if total_value > self.preferences.large_amount_threshold * 10 {
                anomalies.push(Anomaly {
                    anomaly_type: "大额交易".to_string(),
                    description: "检测到一笔异常大额交易".to_string(),
                    severity: 5,
                    related_tx: Some(tx.id.clone()),
                });
            }
        }
        
        anomalies
    }

    /// 生成智能推荐
    pub fn generate_recommendations(&self) -> Vec<SmartRecommendation> {
        let mut recommendations = Vec::new();
        
        // 基于交易历史生成推荐
        if self.transaction_history.len() > 20 {
            recommendations.push(SmartRecommendation {
                recommendation_type: RecommendationType::Optimization,
                title: "批量转账".to_string(),
                description: "您经常进行多笔转账，建议使用批量转账功能节省手续费".to_string(),
                confidence: 0.85,
                action: Some("查看批量转账".to_string()),
            });
        }
        
        // 安全建议
        if self.address_book.len() > 5 {
            recommendations.push(SmartRecommendation {
                recommendation_type: RecommendationType::Security,
                title: "备份地址簿".to_string(),
                description: "您的地址簿中有多个联系人，建议定期备份".to_string(),
                confidence: 0.95,
                action: Some("导出地址簿".to_string()),
            });
        }
        
        recommendations
    }

    /// 获取语音帮助
    pub fn get_voice_help(&self) -> String {
        r#"语音助手使用指南:

转账指令示例:
- "转账 10 个 YRU 给张三"
- "发送 5 YRU 到地址 Y1abc..."
- "转给李四 100 代币"

查询指令:
- "查询余额"
- "查看交易历史"
- "显示我的资产"

其他指令:
- "添加联系人 王五 地址 Y2def..."
- "设置价格提醒"
- "需要帮助"

提示:
- 说话时请保持清晰
- 金额和地址是关键信息
- 大额转账需要二次确认
"#.to_string()
    }

    /// 更新用户偏好
    pub fn update_preferences(&mut self, preferences: UserPreferences) {
        self.preferences = preferences;
    }

    /// 获取用户偏好
    pub fn get_preferences(&self) -> &UserPreferences {
        &self.preferences
    }
}

/// 获取当前时间戳
fn current_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

impl Default for AIWalletAssistant {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_assistant_creation() {
        let assistant = AIWalletAssistant::new();
        assert!(assistant.address_book.is_empty());
        assert!(!assistant.voice_patterns.is_empty());
    }

    #[test]
    fn test_add_contact() {
        let mut assistant = AIWalletAssistant::new();
        
        assistant.add_contact(
            "张三".to_string(),
            "Y1abc123def456".to_string(),
            vec!["朋友".to_string()],
        ).unwrap();
        
        let contact = assistant.get_contact("张三");
        assert!(contact.is_some());
        assert_eq!(contact.unwrap().address, "Y1abc123def456");
    }

    #[test]
    fn test_recognize_command() {
        let assistant = AIWalletAssistant::new();
        
        assert_eq!(
            assistant.recognize_command("转账给张三"),
            VoiceCommand::Transfer
        );
        
        assert_eq!(
            assistant.recognize_command("查询余额"),
            VoiceCommand::CheckBalance
        );
        
        assert_eq!(
            assistant.recognize_command("未知指令"),
            VoiceCommand::Unknown
        );
    }

    #[test]
    fn test_parse_amount() {
        let assistant = AIWalletAssistant::new();
        
        let amount = assistant.parse_amount("转账 10 个 YRU");
        assert!(amount.is_some());
        
        let amount = assistant.parse_amount("转给张三 5.5 代币");
        assert!(amount.is_some());
    }

    #[test]
    fn test_voice_transfer() {
        let assistant = AIWalletAssistant::new();
        
        let request = VoiceTransferRequest {
            transcript: "转账 10 个 YRU 给张三".to_string(),
            from_address: "Y1sender...".to_string(),
            amount: None,
            recipient: None,
            token: None,
        };
        
        let result = assistant.process_voice_transfer(request);
        assert!(result.success || result.error.is_some());
    }
}
