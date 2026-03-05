//! DAO治理合约
//! 
//! 功能：
//! - 创建提案
//! - 投票（代币权重）
//! - 执行提案
//! - 时间锁

use std::collections::HashMap;

/// 提案状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProposalState {
    Pending,    // 等待执行
    Active,     // 投票中
    Succeeded,  // 通过
    Defeated,   // 失败
    Executed,   // 已执行
    Canceled,   // 已取消
}

/// 提案
#[derive(Debug, Clone)]
pub struct Proposal {
    pub id: u64,
    pub proposer: String,
    pub title: String,
    pub description: String,
    
    // 投票数据
    pub for_votes: u64,
    pub against_votes: u64,
    pub abstain_votes: u64,
    
    // 时间
    pub start_block: u64,
    pub end_block: u64,
    pub eta: u64,  // 可执行时间
    
    // 执行数据
    pub target: String,
    pub value: u64,
    pub call_data: Vec<u8>,
    
    // 状态
    pub executed: bool,
    pub canceled: bool,
    
    // 投票记录
    pub has_voted: HashMap<String, bool>,
}

/// DAO治理合约
pub struct DAOGovernance {
    /// 治理代币
    pub gov_token: String,
    
    /// 提案列表
    pub proposals: HashMap<u64, Proposal>,
    
    /// 下一个提案ID
    pub next_proposal_id: u64,
    
    /// 投票延迟（区块数）
    pub voting_delay: u64,
    
    /// 投票周期（区块数）
    pub voting_period: u64,
    
    /// 提案阈值（需要多少代币才能创建提案）
    pub proposal_threshold: u64,
    
    /// 通过阈值（票数比例，如 400 = 40%）
    pub quorum_votes: u64,
    
    /// 时间锁延迟
    pub timelock_delay: u64,
    
    /// 用户代币余额（模拟）
    pub balances: HashMap<String, u64>,
    
    /// 委托记录
    pub delegates: HashMap<String, String>,
}

impl DAOGovernance {
    pub fn new(gov_token: String) -> Self {
        Self {
            gov_token,
            proposals: HashMap::new(),
            next_proposal_id: 1,
            voting_delay: 1,       // 1个区块后开始投票
            voting_period: 100,    // 投票持续100个区块
            proposal_threshold: 1000,  // 需要1000代币才能提案
            quorum_votes: 400,     // 40%参与率
            timelock_delay: 50,    // 执行前等待50个区块
            balances: HashMap::new(),
            delegates: HashMap::new(),
        }
    }
    
    /// 铸造代币（模拟）
    pub fn mint(&mut self, to: &str, amount: u64) {
        *self.balances.entry(to.to_string()).or_insert(0) += amount;
    }
    
    /// 查询余额
    pub fn balance_of(&self, account: &str) -> u64 {
        *self.balances.get(account).unwrap_or(&0)
    }
    
    /// 委托投票权
    pub fn delegate(&mut self, delegator: &str, delegatee: &str) {
        self.delegates.insert(delegator.to_string(), delegatee.to_string());
        println!("✅ {} 委托投票权给 {}", delegator, delegatee);
    }
    
    /// 获取投票权
    pub fn get_votes(&self, account: &str) -> u64 {
        // 自己的余额 + 被委托的余额
        let own_balance = self.balance_of(account);
        let delegated_balance: u64 = self.delegates
            .iter()
            .filter(|(_, d)| d == account)
            .map(|(delegator, _)| self.balance_of(delegator))
            .sum();
        
        own_balance + delegated_balance
    }
    
    /// 创建提案
    pub fn propose(
        &mut self,
        proposer: &str,
        title: &str,
        description: &str,
        target: &str,
        value: u64,
        call_data: Vec<u8>,
        current_block: u64,
    ) -> Result<u64, String> {
        // 检查提案阈值
        let proposer_votes = self.get_votes(proposer);
        if proposer_votes < self.proposal_threshold {
            return Err(format!(
                "需要 {} 代币才能创建提案，当前只有 {}",
                self.proposal_threshold, proposer_votes
            ));
        }
        
        let proposal_id = self.next_proposal_id;
        
        let proposal = Proposal {
            id: proposal_id,
            proposer: proposer.to_string(),
            title: title.to_string(),
            description: description.to_string(),
            for_votes: 0,
            against_votes: 0,
            abstain_votes: 0,
            start_block: current_block + self.voting_delay,
            end_block: current_block + self.voting_delay + self.voting_period,
            eta: 0,
            target: target.to_string(),
            value,
            call_data,
            executed: false,
            canceled: false,
            has_voted: HashMap::new(),
        };
        
        self.proposals.insert(proposal_id, proposal);
        self.next_proposal_id += 1;
        
        println!("✅ 提案 #{} 创建成功: {}", proposal_id, title);
        println!("   投票开始: 区块 {}", current_block + self.voting_delay);
        println!("   投票结束: 区块 {}", current_block + self.voting_delay + self.voting_period);
        
        Ok(proposal_id)
    }
    
    /// 投票
    pub fn cast_vote(
        &mut self,
        voter: &str,
        proposal_id: u64,
        support: u8,  // 0=反对, 1=支持, 2=弃权
        current_block: u64,
    ) -> Result<(), String> {
        let proposal = self.proposals.get_mut(&proposal_id).ok_or("提案不存在")?;
        
        // 检查投票时间
        if current_block < proposal.start_block {
            return Err("投票尚未开始".to_string());
        }
        if current_block > proposal.end_block {
            return Err("投票已结束".to_string());
        }
        
        // 检查是否已投票
        if proposal.has_voted.get(voter).copied().unwrap_or(false) {
            return Err("已经投过票了".to_string());
        }
        
        // 获取投票权重
        let votes = self.get_votes(voter);
        if votes == 0 {
            return Err("没有投票权".to_string());
        }
        
        // 记录投票
        match support {
            0 => proposal.against_votes += votes,
            1 => proposal.for_votes += votes,
            2 => proposal.abstain_votes += votes,
            _ => return Err("无效的投票选项".to_string()),
        }
        
        proposal.has_voted.insert(voter.to_string(), true);
        
        let vote_type = match support {
            0 => "反对",
            1 => "支持",
            2 => "弃权",
            _ => "未知",
        };
        
        println!("✅ {} 投票 '{}'，权重: {}", voter, vote_type, votes);
        
        Ok(())
    }
    
    /// 获取提案状态
    pub fn state(&self, proposal_id: u64, current_block: u64) -> Result<ProposalState, String> {
        let proposal = self.proposals.get(&proposal_id).ok_or("提案不存在")?;
        
        if proposal.canceled {
            return Ok(ProposalState::Canceled);
        }
        
        if proposal.executed {
            return Ok(ProposalState::Executed);
        }
        
        if current_block <= proposal.start_block {
            return Ok(ProposalState::Pending);
        }
        
        if current_block <= proposal.end_block {
            return Ok(ProposalState::Active);
        }
        
        // 投票结束，检查是否通过
        let total_votes = proposal.for_votes + proposal.against_votes + proposal.abstain_votes;
        let quorum = self.quorum_votes;  // 40%
        
        // 检查参与率
        let total_supply: u64 = self.balances.values().sum();
        let participation = if total_supply > 0 {
            (total_votes as u128 * 1000 / total_supply as u128) as u64
        } else {
            0
        };
        
        if participation < quorum {
            return Ok(ProposalState::Defeated);
        }
        
        // 检查支持率
        if proposal.for_votes > proposal.against_votes {
            Ok(ProposalState::Succeeded)
        } else {
            Ok(ProposalState::Defeated)
        }
    }
    
    /// 排队执行（时间锁）
    pub fn queue(&mut self, proposal_id: u64, current_block: u64) -> Result<(), String> {
        let state = self.state(proposal_id, current_block)?;
        
        if state != ProposalState::Succeeded {
            return Err("提案未通过".to_string());
        }
        
        let proposal = self.proposals.get_mut(&proposal_id).unwrap();
        proposal.eta = current_block + self.timelock_delay;
        
        println!("✅ 提案 #{} 进入时间锁，将在区块 {} 后可执行", proposal_id, proposal.eta);
        
        Ok(())
    }
    
    /// 执行提案
    pub fn execute(&mut self, proposal_id: u64, current_block: u64) -> Result<(), String> {
        let proposal = self.proposals.get(&proposal_id).ok_or("提案不存在")?;
        
        if proposal.executed {
            return Err("提案已执行".to_string());
        }
        
        if proposal.eta == 0 {
            return Err("提案未进入时间锁".to_string());
        }
        
        if current_block < proposal.eta {
            return Err("时间锁未到期".to_string());
        }
        
        // 执行提案（模拟）
        let proposal = self.proposals.get_mut(&proposal_id).unwrap();
        proposal.executed = true;
        
        println!("✅ 提案 #{} 已执行！", proposal_id);
        println!("   目标: {}", proposal.target);
        println!("   金额: {}", proposal.value);
        
        Ok(())
    }
    
    /// 取消提案
    pub fn cancel(&mut self, proposal_id: u64, caller: &str) -> Result<(), String> {
        let proposal = self.proposals.get(&proposal_id).ok_or("提案不存在")?;
        
        // 只有提案者可以取消
        if proposal.proposer != caller {
            return Err("无权取消".to_string());
        }
        
        if proposal.executed {
            return Err("已执行，无法取消".to_string());
        }
        
        let proposal = self.proposals.get_mut(&proposal_id).unwrap();
        proposal.canceled = true;
        
        println!("✅ 提案 #{} 已取消", proposal_id);
        
        Ok(())
    }
    
    /// 获取提案信息
    pub fn get_proposal(&self, proposal_id: u64) -> Option<&Proposal> {
        self.proposals.get(&proposal_id)
    }
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║           ChengChain DAO 治理演示                         ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    // 创建DAO
    let mut dao = DAOGovernance::new("GOV".to_string());
    
    // 分发治理代币
    println!("1️⃣  分发治理代币");
    dao.mint("Alice", 5000);
    dao.mint("Bob", 3000);
    dao.mint("Charlie", 2000);
    println!("   Alice: 5000 GOV");
    println!("   Bob: 3000 GOV");
    println!("   Charlie: 2000 GOV");
    
    // Alice委托给Bob
    println!("\n2️⃣  Alice 委托投票权给 Bob");
    dao.delegate("Alice", "Bob");
    println!("   Bob 总投票权: {}", dao.get_votes("Bob"));
    
    // 创建提案
    println!("\n3️⃣  Bob 创建提案");
    let proposal_id = dao.propose(
        "Bob",
        "增加出块奖励",
        "将每个区块的挖矿奖励从50增加到100",
        "treasury",
        0,
        vec![0x01, 0x02, 0x03],
        100,
    ).unwrap();
    
    // 投票
    println!("\n4️⃣  社区投票 (区块150)");
    dao.cast_vote("Bob", proposal_id, 1, 150).unwrap();      // 支持 (8000票)
    dao.cast_vote("Charlie", proposal_id, 0, 160).unwrap();  // 反对 (2000票)
    
    // 查看投票结果
    let proposal = dao.get_proposal(proposal_id).unwrap();
    println!("\n   投票结果:");
    println!("   支持: {} 票", proposal.for_votes);
    println!("   反对: {} 票", proposal.against_votes);
    
    // 检查状态
    println!("\n5️⃣  检查提案状态 (区块250)");
    let state = dao.state(proposal_id, 250).unwrap();
    println!("   状态: {:?}", state);
    
    // 排队执行
    if state == ProposalState::Succeeded {
        println!("\n6️⃣  提案通过，进入时间锁");
        dao.queue(proposal_id, 250).unwrap();
        
        // 等待时间锁
        println!("\n7️⃣  等待时间锁到期 (区块350)");
        dao.execute(proposal_id, 350).unwrap();
    }
    
    // 统计
    println!("\n📊 DAO统计");
    println!("   总提案: {}", dao.next_proposal_id - 1);
    println!("   通过阈值: {}%", dao.quorum_votes / 10);
    println!("   投票周期: {} 区块", dao.voting_period);
    println!("   时间锁: {} 区块", dao.timelock_delay);
    
    println!("\n✨ DAO治理演示完成！");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_propose_and_vote() {
        let mut dao = DAOGovernance::new("GOV".to_string());
        dao.mint("alice", 2000);
        
        let id = dao.propose("alice", "test", "desc", "target", 0, vec![], 100).unwrap();
        
        dao.cast_vote("alice", id, 1, 150).unwrap();
        
        let proposal = dao.get_proposal(id).unwrap();
        assert_eq!(proposal.for_votes, 2000);
    }
    
    #[test]
    fn test_proposal_threshold() {
        let mut dao = DAOGovernance::new("GOV".to_string());
        dao.mint("alice", 500);  // 低于阈值
        
        let result = dao.propose("alice", "test", "desc", "target", 0, vec![], 100);
        assert!(result.is_err());
    }
}
