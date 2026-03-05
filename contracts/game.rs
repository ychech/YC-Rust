//! 链游合约 - 简单养成游戏
//! 
//! 功能：
//! - 创建角色
//! - 战斗升级
//! - 装备系统
//! - 排行榜

use std::collections::HashMap;

/// 角色
#[derive(Debug, Clone)]
pub struct Character {
    pub id: u64,
    pub owner: String,
    pub name: String,
    pub level: u32,
    pub exp: u64,
    pub hp: u32,
    pub attack: u32,
    pub defense: u32,
    pub wins: u32,
    pub losses: u32,
    pub created_at: u64,
}

impl Character {
    pub fn new(id: u64, owner: String, name: String, created_at: u64) -> Self {
        Self {
            id,
            owner,
            name,
            level: 1,
            exp: 0,
            hp: 100,
            attack: 10,
            defense: 5,
            wins: 0,
            losses: 0,
            created_at,
        }
    }
    
    /// 升级所需经验
    pub fn exp_to_next_level(&self) -> u64 {
        (self.level as u64) * 100
    }
    
    /// 增加经验
    pub fn add_exp(&mut self, amount: u64) {
        self.exp += amount;
        
        // 检查升级
        while self.exp >= self.exp_to_next_level() {
            self.exp -= self.exp_to_next_level();
            self.level_up();
        }
    }
    
    /// 升级
    fn level_up(&mut self) {
        self.level += 1;
        self.hp += 20;
        self.attack += 5;
        self.defense += 3;
        println!("🎉 {} 升到 {} 级！", self.name, self.level);
    }
    
    /// 计算战斗力
    pub fn power(&self) -> u32 {
        self.hp + self.attack * 10 + self.defense * 5
    }
}

/// 装备
#[derive(Debug, Clone)]
pub struct Equipment {
    pub id: u64,
    pub name: String,
    pub equipment_type: EquipmentType,
    pub rarity: Rarity,
    pub hp_bonus: u32,
    pub attack_bonus: u32,
    pub defense_bonus: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EquipmentType {
    Weapon,
    Armor,
    Accessory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rarity {
    Common,     // 普通
    Rare,       // 稀有
    Epic,       // 史诗
    Legendary,  // 传说
}

/// 游戏合约
pub struct GameContract {
    // 角色数据
    pub characters: HashMap<u64, Character>,
    pub owner_characters: HashMap<String, Vec<u64>>,
    pub next_character_id: u64,
    
    // 装备数据
    pub equipments: HashMap<u64, Equipment>,
    pub character_equipments: HashMap<u64, Vec<u64>>, // character_id => equipment_ids
    pub next_equipment_id: u64,
    
    // 游戏代币
    pub token_balances: HashMap<String, u64>,
    
    // 战斗记录
    pub battles: Vec<BattleRecord>,
}

/// 战斗记录
#[derive(Debug, Clone)]
pub struct BattleRecord {
    pub attacker_id: u64,
    pub defender_id: u64,
    pub winner_id: u64,
    pub timestamp: u64,
}

impl GameContract {
    pub fn new() -> Self {
        Self {
            characters: HashMap::new(),
            owner_characters: HashMap::new(),
            next_character_id: 1,
            equipments: HashMap::new(),
            character_equipments: HashMap::new(),
            next_equipment_id: 1,
            token_balances: HashMap::new(),
            battles: Vec::new(),
        }
    }
    
    /// 创建角色
    pub fn create_character(&mut self, owner: &str, name: &str, current_time: u64) -> u64 {
        let id = self.next_character_id;
        
        let character = Character::new(
            id,
            owner.to_string(),
            name.to_string(),
            current_time,
        );
        
        self.characters.insert(id, character);
        self.owner_characters.entry(owner.to_string()).or_default().push(id);
        self.next_character_id += 1;
        
        // 赠送初始代币
        *self.token_balances.entry(owner.to_string()).or_insert(0) += 100;
        
        println!("✅ 角色 '{}' 创建成功！ID: {}", name, id);
        println!("   初始属性: HP={}, 攻击={}, 防御={}", 100, 10, 5);
        println!("   赠送 100 游戏币");
        
        id
    }
    
    /// 获取角色
    pub fn get_character(&self, id: u64) -> Option<&Character> {
        self.characters.get(&id)
    }
    
    /// 获取角色可变引用
    pub fn get_character_mut(&mut self, id: u64) -> Option<&mut Character> {
        self.characters.get_mut(&id)
    }
    
    /// 铸造装备
    pub fn mint_equipment(
        &mut self,
        name: &str,
        equipment_type: EquipmentType,
        rarity: Rarity,
    ) -> u64 {
        let id = self.next_equipment_id;
        
        // 根据稀有度决定属性
        let (hp_bonus, attack_bonus, defense_bonus) = match rarity {
            Rarity::Common => (5, 2, 1),
            Rarity::Rare => (15, 8, 5),
            Rarity::Epic => (40, 20, 12),
            Rarity::Legendary => (100, 50, 30),
        };
        
        let equipment = Equipment {
            id,
            name: name.to_string(),
            equipment_type,
            rarity,
            hp_bonus,
            attack_bonus,
            defense_bonus,
        };
        
        self.equipments.insert(id, equipment);
        self.next_equipment_id += 1;
        
        println!("✨ 装备 '{}' 铸造成功！稀有度: {:?}", name, rarity);
        
        id
    }
    
    /// 装备给角色
    pub fn equip(&mut self, character_id: u64, equipment_id: u64) -> Result<(), String> {
        let character = self.characters.get_mut(&character_id).ok_or("角色不存在")?;
        let equipment = self.equipments.get(&equipment_id).ok_or("装备不存在")?;
        
        // 应用装备属性
        character.hp += equipment.hp_bonus;
        character.attack += equipment.attack_bonus;
        character.defense += equipment.defense_bonus;
        
        // 记录装备
        self.character_equipments.entry(character_id).or_default().push(equipment_id);
        
        println!("⚔️  {} 装备了 {} (+{} 攻击)", character.name, equipment.name, equipment.attack_bonus);
        
        Ok(())
    }
    
    /// 战斗
    pub fn battle(&mut self, attacker_id: u64, defender_id: u64, current_time: u64) -> Result<u64, String> {
        let attacker = self.characters.get(&attacker_id).ok_or("攻击者不存在")?.clone();
        let defender = self.characters.get(&defender_id).ok_or("防御者不存在")?.clone();
        
        println!("\n⚔️  战斗开始: {} vs {}", attacker.name, defender.name);
        println!("   {}: 战力={}, HP={}, 攻击={}", attacker.name, attacker.power(), attacker.hp, attacker.attack);
        println!("   {}: 战力={}, HP={}, 攻击={}", defender.name, defender.power(), defender.hp, defender.attack);
        
        // 简单战斗模拟
        let attacker_power = attacker.power();
        let defender_power = defender.power();
        
        let winner_id = if attacker_power > defender_power {
            println!("   🏆 {} 获胜！", attacker.name);
            attacker_id
        } else if defender_power > attacker_power {
            println!("   🏆 {} 获胜！", defender.name);
            defender_id
        } else {
            // 平局，随机决定
            if current_time % 2 == 0 {
                println!("   🤝 平局！{} 险胜", attacker.name);
                attacker_id
            } else {
                println!("   🤝 平局！{} 险胜", defender.name);
                defender_id
            }
        };
        
        // 更新战斗记录
        let record = BattleRecord {
            attacker_id,
            defender_id,
            winner_id,
            timestamp: current_time,
        };
        self.battles.push(record);
        
        // 更新角色数据
        if let Some(char) = self.characters.get_mut(&attacker_id) {
            if winner_id == attacker_id {
                char.wins += 1;
                char.add_exp(50);
                // 奖励代币
                *self.token_balances.entry(char.owner.clone()).or_insert(0) += 10;
            } else {
                char.losses += 1;
                char.add_exp(10);
            }
        }
        
        if let Some(char) = self.characters.get_mut(&defender_id) {
            if winner_id == defender_id {
                char.wins += 1;
                char.add_exp(50);
                *self.token_balances.entry(char.owner.clone()).or_insert(0) += 10;
            } else {
                char.losses += 1;
                char.add_exp(10);
            }
        }
        
        Ok(winner_id)
    }
    
    /// PVE - 打怪
    pub fn pve_battle(&mut self, character_id: u64, monster_level: u32, current_time: u64) -> Result<bool, String> {
        let character = self.characters.get(&character_id).ok_or("角色不存在")?.clone();
        
        // 生成怪物
        let monster_power = (monster_level * 50) as u32;
        
        println!("\n👹 PVE战斗: {} vs {}级怪物", character.name, monster_level);
        println!("   玩家战力: {}", character.power());
        println!("   怪物战力: {}", monster_power);
        
        let win = character.power() > monster_power;
        
        if win {
            println!("   🎉 胜利！获得 {} 经验", monster_level * 20);
            if let Some(char) = self.characters.get_mut(&character_id) {
                char.add_exp((monster_level * 20) as u64);
                char.wins += 1;
            }
            // 奖励代币
            let reward = monster_level * 5;
            *self.token_balances.entry(character.owner.clone()).or_insert(0) += reward as u64;
            println!("   💰 获得 {} 游戏币", reward);
        } else {
            println!("   💀 失败... 获得 5 经验");
            if let Some(char) = self.characters.get_mut(&character_id) {
                char.add_exp(5);
                char.losses += 1;
            }
        }
        
        Ok(win)
    }
    
    /// 获取排行榜
    pub fn get_leaderboard(&self, limit: usize) -> Vec<&Character> {
        let mut chars: Vec<&Character> = self.characters.values().collect();
        chars.sort_by(|a, b| b.power().cmp(&a.power()));
        chars.into_iter().take(limit).collect()
    }
    
    /// 查询代币余额
    pub fn token_balance(&self, owner: &str) -> u64 {
        *self.token_balances.get(owner).unwrap_or(&0)
    }
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║           ChengChain 链游演示                             ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    let mut game = GameContract::new();
    
    // 创建角色
    println!("1️⃣  玩家创建角色");
    let alice_char = game.create_character("Alice", "勇者爱丽丝", 1000);
    let bob_char = game.create_character("Bob", "战士鲍勃", 1000);
    let charlie_char = game.create_character("Charlie", "法师查理", 1000);
    
    // 铸造装备
    println!("\n2️⃣  铸造装备");
    let sword = game.mint_equipment("屠龙剑", EquipmentType::Weapon, Rarity::Epic);
    let armor = game.mint_equipment("龙鳞甲", EquipmentType::Armor, Rarity::Rare);
    let ring = game.mint_equipment("魔法戒指", EquipmentType::Accessory, Rarity::Legendary);
    
    // 装备
    println!("\n3️⃣  装备角色");
    game.equip(alice_char, sword).unwrap();
    game.equip(alice_char, armor).unwrap();
    game.equip(charlie_char, ring).unwrap();
    
    // PVE战斗
    println!("\n4️⃣  Alice 挑战怪物");
    game.pve_battle(alice_char, 3, 1100).unwrap();
    game.pve_battle(alice_char, 5, 1200).unwrap();
    
    // PVP战斗
    println!("\n5️⃣  PVP对战");
    game.battle(alice_char, bob_char, 1300).unwrap();
    game.battle(bob_char, charlie_char, 1400).unwrap();
    game.battle(alice_char, charlie_char, 1500).unwrap();
    
    // 排行榜
    println!("\n6️⃣  战力排行榜");
    let leaderboard = game.get_leaderboard(10);
    for (i, char) in leaderboard.iter().enumerate() {
        println!("   第{}名: {} - {}级 - 战力{} - {}胜{}负",
            i + 1,
            char.name,
            char.level,
            char.power(),
            char.wins,
            char.losses
        );
    }
    
    // 代币余额
    println!("\n💰 代币余额");
    println!("   Alice: {}", game.token_balance("Alice"));
    println!("   Bob: {}", game.token_balance("Bob"));
    println!("   Charlie: {}", game.token_balance("Charlie"));
    
    println!("\n✨ 链游演示完成！");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_character() {
        let mut game = GameContract::new();
        let id = game.create_character("alice", "Hero", 1000);
        
        let char = game.get_character(id).unwrap();
        assert_eq!(char.name, "Hero");
        assert_eq!(char.level, 1);
    }
    
    #[test]
    fn test_level_up() {
        let mut game = GameContract::new();
        let id = game.create_character("alice", "Hero", 1000);
        
        {
            let char = game.get_character_mut(id).unwrap();
            char.add_exp(150);  // 超过100，应该升级
        }
        
        let char = game.get_character(id).unwrap();
        assert_eq!(char.level, 2);
    }
}
