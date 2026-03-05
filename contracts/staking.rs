//! 质押挖矿合约
//! 
//! 功能：
//! - 质押代币获得收益
//! - 按区块计算奖励
//! - 随时提取本金和收益

use std::collections::HashMap;

/// 质押池
pub struct StakingPool {
    /// 质押代币地址
    pub staking_token: String,
    /// 奖励代币地址
    pub reward_token: String,
    
    /// 每个区块的奖励数量
    pub reward_per_block: u64,
    /// 开始区块
    pub start_block: u64,
    /// 结束区块
    pub end_block: u64,
    
    /// 总质押量
    pub total_staked: u64,
    /// 累计奖励 per token (精度 1e12)
    pub acc_reward_per_share: u128,
    /// 最后更新区块
    pub last_reward_block: u64,
    
    /// 用户质押信息
    pub users: HashMap<String, UserInfo>,
}

/// 用户质押信息
#[derive(Debug, Clone, Default)]
pub struct UserInfo {
    /// 质押数量
    pub amount: u64,
    /// 已提取的奖励
    pub reward_debt: u128,
    /// 待领取的奖励
    pub pending_reward: u64,
}

impl StakingPool {
    /// 创建质押池
    pub fn new(
        staking_token: String,
        reward_token: String,
        reward_per_block: u64,
        start_block: u64,
        end_block: u64,
    ) -> Self {
        Self {
            staking_token,
            reward_token,
            reward_per_block,
            start_block,
            end_block,
            total_staked: 0,
            acc_reward_per_share: 0,
            last_reward_block: start_block,
            users: HashMap::new(),
        }
    }
    
    /// 更新池子奖励
    /// 
    /// 计算从上次更新到现在的奖励，并更新累计奖励 per share
    pub fn update_pool(&mut self, current_block: u64) {
        if current_block <= self.last_reward_block {
            return;
        }
        
        if self.total_staked == 0 {
            self.last_reward_block = current_block;
            return;
        }
        
        // 计算奖励区块数
        let end = current_block.min(self.end_block);
        let multiplier = end - self.last_reward_block;
        
        // 计算总奖励
        let reward = multiplier as u128 * self.reward_per_block as u128;
        
        // 更新累计奖励 per share (乘以 1e12 精度)
        self.acc_reward_per_share += reward * 1_000_000_000_000 / self.total_staked as u128;
        
        self.last_reward_block = current_block;
    }
    
    /// 质押
    pub fn deposit(&mut self, user: &str, amount: u64, current_block: u64) -> Result<(), String> {
        if amount == 0 {
            return Err("质押数量不能为0".to_string());
        }
        
        // 更新池子
        self.update_pool(current_block);
        
        // 获取或创建用户信息
        let user_info = self.users.entry(user.to_string()).or_default();
        
        // 如果有待领取奖励，先结算
        if user_info.amount > 0 {
            let pending = self.pending_reward(user, current_block);
            user_info.pending_reward += pending;
        }
        
        // 更新质押数量
        user_info.amount += amount;
        user_info.reward_debt = user_info.amount as u128 * self.acc_reward_per_share / 1_000_000_000_000;
        
        // 更新总质押
        self.total_staked += amount;
        
        println!("✅ {} 质押 {}，当前质押: {}", user, amount, user_info.amount);
        
        Ok(())
    }
    
    /// 提取
    pub fn withdraw(&mut self, user: &str, amount: u64, current_block: u64) -> Result<u64, String> {
        let user_info = self.users.get_mut(user).ok_or("用户未质押")?;
        
        if user_info.amount < amount {
            return Err("提取数量超过质押数量".to_string());
        }
        
        // 更新池子
        self.update_pool(current_block);
        
        // 计算待领取奖励
        let pending = self.pending_reward(user, current_block);
        let total_reward = user_info.pending_reward + pending;
        
        // 更新质押数量
        user_info.amount -= amount;
        user_info.reward_debt = user_info.amount as u128 * self.acc_reward_per_share / 1_000_000_000_000;
        user_info.pending_reward = 0;
        
        // 更新总质押
        self.total_staked -= amount;
        
        println!("✅ {} 提取 {}，获得奖励 {}", user, amount, total_reward);
        
        Ok(total_reward)
    }
    
    /// 只提取奖励，不取本金
    pub fn harvest(&mut self, user: &str, current_block: u64) -> Result<u64, String> {
        // 更新池子
        self.update_pool(current_block);
        
        let user_info = self.users.get_mut(user).ok_or("用户未质押")?;
        
        // 计算待领取奖励
        let pending = self.pending_reward(user, current_block);
        let total_reward = user_info.pending_reward + pending;
        
        if total_reward == 0 {
            return Err("没有可领取的奖励".to_string());
        }
        
        // 重置奖励
        user_info.reward_debt = user_info.amount as u128 * self.acc_reward_per_share / 1_000_000_000_000;
        user_info.pending_reward = 0;
        
        println!("✅ {} 领取奖励 {}", user, total_reward);
        
        Ok(total_reward)
    }
    
    /// 查询待领取奖励
    pub fn pending_reward(&self, user: &str, current_block: u64) -> u64 {
        let user_info = match self.users.get(user) {
            Some(u) => u,
            None => return 0,
        };
        
        if user_info.amount == 0 {
            return user_info.pending_reward;
        }
        
        // 计算从上次更新到现在的奖励
        let end = current_block.min(self.end_block);
        let multiplier = if end > self.last_reward_block {
            end - self.last_reward_block
        } else {
            0
        };
        
        let reward = multiplier as u128 * self.reward_per_block as u128;
        let acc_reward = if self.total_staked > 0 {
            self.acc_reward_per_share + reward * 1_000_000_000_000 / self.total_staked as u128
        } else {
            self.acc_reward_per_share
        };
        
        let pending = user_info.amount as u128 * acc_reward / 1_000_000_000_000 - user_info.reward_debt;
        
        (user_info.pending_reward as u128 + pending) as u64
    }
    
    /// 查询用户质押信息
    pub fn user_info(&self, user: &str) -> Option<&UserInfo> {
        self.users.get(user)
    }
    
    /// 计算APR (年化收益率)
    pub fn calculate_apr(&self) -> f64 {
        if self.total_staked == 0 {
            return 0.0;
        }
        
        // 假设每区块10秒，一年约315万区块
        let blocks_per_year = 3153600u64;
        let yearly_reward = self.reward_per_block as u128 * blocks_per_year as u128;
        
        (yearly_reward as f64 / self.total_staked as f64) * 100.0
    }
}

/// 质押挖矿工厂
pub struct StakingFactory {
    pub pools: Vec<StakingPool>,
}

impl StakingFactory {
    pub fn new() -> Self {
        Self { pools: Vec::new() }
    }
    
    pub fn create_pool(
        &mut self,
        staking_token: String,
        reward_token: String,
        reward_per_block: u64,
        start_block: u64,
        duration_blocks: u64,
    ) -> usize {
        let pool = StakingPool::new(
            staking_token,
            reward_token,
            reward_per_block,
            start_block,
            start_block + duration_blocks,
        );
        
        self.pools.push(pool);
        self.pools.len() - 1
    }
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║           ChengChain 质押挖矿演示                         ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    // 创建质押池：质押 cUSD，获得 cETH 奖励
    let mut pool = StakingPool::new(
        "cUSD".to_string(),      // 质押代币
        "cETH".to_string(),      // 奖励代币
        100,                      // 每区块奖励 100 cETH
        100,                      // 从区块100开始
        100 + 10000,             // 持续10000个区块
    );
    
    println!("1️⃣  创建质押池");
    println!("   质押代币: cUSD");
    println!("   奖励代币: cETH");
    println!("   每区块奖励: {} cETH", pool.reward_per_block);
    println!("   持续区块: {} - {}", pool.start_block, pool.end_block);
    
    // Alice 质押
    println!("\n2️⃣  Alice 质押 10000 cUSD");
    pool.deposit("Alice", 10000, 200).unwrap();
    
    // Bob 质押
    println!("\n3️⃣  Bob 质押 5000 cUSD");
    pool.deposit("Bob", 5000, 300).unwrap();
    
    // 查看收益
    println!("\n4️⃣  查看收益情况 (区块500)");
    let current_block = 500;
    
    let alice_pending = pool.pending_reward("Alice", current_block);
    let bob_pending = pool.pending_reward("Bob", current_block);
    
    println!("   Alice 待领取: {} cETH", alice_pending);
    println!("   Bob 待领取: {} cETH", bob_pending);
    println!("   总质押: {} cUSD", pool.total_staked);
    println!("   预估APR: {:.2}%", pool.calculate_apr());
    
    // Alice 提取奖励
    println!("\n5️⃣  Alice 领取奖励");
    let reward = pool.harvest("Alice", current_block).unwrap();
    println!("   实际领取: {} cETH", reward);
    
    // Alice 退出
    println!("\n6️⃣  Alice 退出质押");
    let (amount, reward) = (10000u64, pool.withdraw("Alice", 10000, 600).unwrap());
    println!("   取回本金: {} cUSD", amount);
    println!("   获得奖励: {} cETH", reward);
    
    // 最终状态
    println!("\n7️⃣  最终状态");
    println!("   总质押: {} cUSD", pool.total_staked);
    println!("   Bob 质押: {} cUSD", pool.user_info("Bob").unwrap().amount);
    
    println!("\n✨ 质押挖矿演示完成！");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_staking() {
        let mut pool = StakingPool::new(
            "STAKE".to_string(),
            "REWARD".to_string(),
            100,
            100,
            1000,
        );
        
        // 质押
        pool.deposit("alice", 1000, 200).unwrap();
        assert_eq!(pool.total_staked, 1000);
        
        // 100个区块后应该有奖励
        let pending = pool.pending_reward("alice", 300);
        assert!(pending > 0);
        
        // 提取
        let reward = pool.withdraw("alice", 1000, 400).unwrap();
        assert!(reward > 0);
        assert_eq!(pool.total_staked, 0);
    }
}
