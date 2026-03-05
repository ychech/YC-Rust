# YRUST Wallet 完全教程

> 网页版链上钱包 - 助记词、转账、查看资产、连接 DEX

## 📚 目录

1. [功能特性](#功能特性)
2. [助记词和密钥派生](#助记词和密钥派生)
3. [代码实现](#代码实现)
4. [前端界面](#前端界面)
5. [连接 DEX](#连接-dex)
6. [安全最佳实践](#安全最佳实践)

---

## 功能特性

### ✅ 已实现功能

| 功能 | 说明 |
|------|------|
| **助记词生成** | BIP39 标准，12个单词 |
| **密钥派生** | BIP44 路径: m/44'/8888'/0'/0/{index} |
| **多账户** | 一个助记词派生多个地址 |
| **本地加密** | 密码保护私钥 |
| **转账** | 签名并广播交易 |
| **查看资产** | 代币余额和价格 |
| **连接 DEX** | 一键跳转到交易所 |
| **交易历史** | 查看过往交易 |

---

## 助记词和密钥派生

### BIP39 - 助记词

助记词是私钥的人类可读形式。

```
随机熵 (128位) 
    ↓
SHA256 哈希 (取前4位作为校验和)
    ↓
熵 + 校验和 = 132位
    ↓
分成11位一组 (12组)
    ↓
每组对应一个单词 (BIP39词库)
    ↓
12个助记词
```

### BIP44 - 密钥派生路径

```
m / purpose' / coin_type' / account' / change / address_index

示例:
m/44'/8888'/0'/0/0

44'    = BIP44 标准
8888'  = YRUST 币种 (自定义)
0'     = 账户 0
0      = 外部链 (收款)
0      = 第一个地址
```

### 代码实现

```rust
// 生成助记词
let mnemonic = Mnemonic::generate_in(Language::English, 12)?;
let phrase = mnemonic.to_string();
// "abandon ability able about above absent absorb abstract absurd abuse access accident"

// 从助记词派生密钥
let seed = Seed::new(&mnemonic, "");
let path = "m/44'/8888'/0'/0/0";
let (private_key, public_key, address) = derive_keys(seed, path)?;
```

---

## 代码实现

### 1. 创建钱包

```rust
use yrust_wallet::WalletManager;

let mut wallet = WalletManager::new();

// 创建新钱包
let (mnemonic, address) = wallet.create_wallet("password123")?;

println!("助记词: {}", mnemonic);
println!("地址: {}", address);
```

### 2. 恢复钱包

```rust
let mnemonic = "abandon ability able about above absent absorb abstract absurd abuse access accident";

let mut wallet = WalletManager::new();
let address = wallet.restore_wallet(mnemonic, "password123")?;
```

### 3. 创建多账户

```rust
// 账户 1 (默认)
let addr1 = wallet.create_account("Account 1", "password123")?;

// 账户 2
let addr2 = wallet.create_account("Account 2", "password123")?;

// 账户 3
let addr3 = wallet.create_account("Account 3", "password123")?;

// 切换账户
wallet.switch_account(1)?; // 切换到账户 2
```

### 4. 签名交易

```rust
let tx = TransactionRequest {
    from: address.clone(),
    to: "1RecipientAddress...".to_string(),
    amount: 1000_00000000, // 1000 YRU
    gas_price: 1_00000000,
    gas_limit: 21000,
    nonce: 0,
    data: vec![],
};

let signed_tx = wallet.sign_transaction(tx, "password123")?;

// 广播交易
broadcast_transaction(signed_tx).await?;
```

### 5. 导出私钥

```rust
// 导出助记词
let mnemonic = wallet.export_mnemonic("password123")?;

// 导出单个账户私钥
let private_key = wallet.export_private_key(&address, "password123")?;
```

---

## 前端界面

### 界面结构

```
┌─────────────────────────────────────────┐
│  Sidebar    │  Main Content             │
│             │                           │
│  💎 资产     │  ┌─────────────────────┐  │
│  📤 发送     │  │  总资产              │  │
│  📥 接收     │  │  1,234.56 YRU        │  │
│  💱 兑换     │  │  ≈ $12,345.67 USD   │  │
│  📜 历史     │  └─────────────────────┘  │
│  ⚙️ 设置     │                           │
│             │  [发送] [接收] [兑换] [购买]│
│             │                           │
│             │  ┌─────────────────────┐  │
│             │  │ 代币列表             │  │
│             │  │ YRU      1,234.56   │  │
│             │  │ ETH         2.50    │  │
│             │  │ USDC     5,000.00   │  │
│             │  └─────────────────────┘  │
└─────────────────────────────────────────┘
```

### 使用流程

1. **创建钱包**
   - 设置密码
   - 备份助记词（12个单词）
   - 确认备份

2. **查看资产**
   - 总资产价值
   - 各代币余额
   - 钱包地址

3. **发送代币**
   - 输入接收地址
   - 输入金额
   - 选择 Gas 费用
   - 确认交易

4. **接收代币**
   - 显示二维码
   - 复制地址
   - 分享给发送方

5. **连接 DEX**
   - 点击"兑换"按钮
   - 跳转到 DEX 界面
   - 自动连接钱包

---

## 连接 DEX

### 一键连接

```javascript
// 钱包页面
function connectDEX() {
    // 保存当前钱包状态到 sessionStorage
    sessionStorage.setItem('wallet_connected', 'true');
    sessionStorage.setItem('wallet_address', currentAddress);
    
    // 打开 DEX
    window.open('../dex/frontend/index.html', '_blank');
}
```

### DEX 自动识别

```javascript
// DEX 页面
window.onload = function() {
    const connected = sessionStorage.getItem('wallet_connected');
    const address = sessionStorage.getItem('wallet_address');
    
    if (connected && address) {
        // 自动连接钱包
        connectWallet(address);
    }
};
```

### 消息通信

```javascript
// 跨窗口通信
window.addEventListener('message', (event) => {
    if (event.data.type === 'WALLET_CONNECTED') {
        const address = event.data.address;
        // 更新 DEX 界面
        updateWalletUI(address);
    }
});
```

---

## 安全最佳实践

### 1. 助记词安全

✅ **应该做的：**
- 手写在纸上
- 存放在防火防水的地方
- 制作多个备份，存放在不同地点

❌ **不应该做的：**
- 截图保存
- 存储在云端
- 通过邮件/聊天发送
- 告诉任何人

### 2. 密码安全

```rust
// 密码要求
- 至少 12 位字符
- 包含大小写字母
- 包含数字
- 包含特殊字符
- 不要使用常用密码
```

### 3. 私钥加密

```rust
// 使用 AES-256-GCM 加密
fn encrypt_private_key(key: &[u8], password: &str) -> Vec<u8> {
    let salt = generate_random_salt();
    let derived_key = pbkdf2(password, &salt, 100000);
    
    aes_256_gcm_encrypt(key, &derived_key)
}
```

### 4. 防止钓鱼

- 始终检查网址：`https://wallet.yrust.chain`
- 不要点击不明链接
- 确认交易详情后再签名

### 5. 定期备份

```rust
// 导出钱包文件
let wallet_data = wallet.serialize()?;
std::fs::write("wallet_backup.json", wallet_data)?;
```

---

## API 参考

### 创建钱包

```rust
pub fn create_wallet(&mut self, password: &str) -> Result<(String, String), WalletError>
```

返回: `(助记词, 地址)`

### 恢复钱包

```rust
pub fn restore_wallet(&mut self, mnemonic: &str, password: &str) -> Result<String, WalletError>
```

返回: `地址`

### 签名交易

```rust
pub fn sign_transaction(&self, tx: TransactionRequest, password: &str) 
    -> Result<SignedTransaction, WalletError>
```

返回: `签名后的交易`

---

## 故障排查

### 忘记密码

**解决方案：** 使用助记词恢复钱包，设置新密码

### 助记词丢失

**解决方案：** 无法恢复！务必做好备份

### 交易失败

**检查：**
- 余额是否充足
- Gas 费用是否足够
- 接收地址是否正确
- 网络连接是否正常

---

## 总结

YRUST Wallet 特点：

- ✅ BIP39/BIP44 标准兼容
- ✅ 本地加密存储
- ✅ 多账户管理
- ✅ 一键连接 DEX
- ✅ 现代化界面

**安全使用，保护好你的助记词！** 🔐
