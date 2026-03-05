//! 07. 虚拟机 - WASM执行环境

/// Gas计量器
pub struct GasMeter {
    pub limit: u64,
    pub used: u64,
}

impl GasMeter {
    pub fn new(limit: u64) -> Self {
        Self { limit, used: 0 }
    }
    
    /// 消耗Gas
    pub fn consume(&mut self, amount: u64) -> Result<(), String> {
        if self.used + amount > self.limit {
            return Err("Out of gas".to_string());
        }
        self.used += amount;
        Ok(())
    }
    
    pub fn remaining(&self) -> u64 {
        self.limit - self.used
    }
}

/// 合约引擎
pub struct ContractEngine {
    gas_meter: GasMeter,
}

impl ContractEngine {
    pub fn new(gas_limit: u64) -> Self {
        Self {
            gas_meter: GasMeter::new(gas_limit),
        }
    }
    
    /// 执行合约
    pub fn execute(&mut self, code: &[u8]) -> Result<String, String> {
        println!("执行合约，代码大小: {} bytes", code.len());
        
        // 模拟Gas消耗
        let gas_cost = code.len() as u64 * 10;
        self.gas_meter.consume(gas_cost)?;
        
        println!("Gas使用: {}/{}", self.gas_meter.used, self.gas_meter.limit);
        
        Ok("执行成功".to_string())
    }
}

/// 简单合约示例
pub struct SimpleContract {
    pub balance: u64,
}

impl SimpleContract {
    pub fn new() -> Self {
        Self { balance: 0 }
    }
    
    pub fn deposit(&mut self, amount: u64) {
        self.balance += amount;
    }
    
    pub fn withdraw(&mut self, amount: u64) -> Result<(), String> {
        if self.balance < amount {
            return Err("余额不足".to_string());
        }
        self.balance -= amount;
        Ok(())
    }
}

fn main() {
    println!("WASM虚拟机演示");
    
    // Gas计量演示
    let mut engine = ContractEngine::new(10000);
    let result = engine.execute(b"simple_contract_code");
    println!("执行结果: {:?}", result);
    println!("剩余Gas: {}", engine.gas_meter.remaining());
    
    // 合约演示
    let mut contract = SimpleContract::new();
    contract.deposit(100);
    println!("存款后余额: {}", contract.balance);
    
    contract.withdraw(30).unwrap();
    println!("取款后余额: {}", contract.balance);
}
