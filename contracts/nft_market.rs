//! NFT市场合约
//! 
//! 功能：
//! - 铸造NFT
//! - 上架出售
//! - 购买NFT
//! - 拍卖功能
//! - 版税分成

use std::collections::HashMap;

/// NFT结构
#[derive(Debug, Clone)]
pub struct NFT {
    pub token_id: u64,
    pub owner: String,
    pub creator: String,      // 创作者（收取版税）
    pub uri: String,          // 元数据URI
    pub royalty_rate: u64,    // 版税率 (100 = 10%)
}

/// 上架信息
#[derive(Debug, Clone)]
pub struct Listing {
    pub seller: String,
    pub price: u64,
    pub listed_at: u64,       // 上架时间
}

/// 拍卖信息
#[derive(Debug, Clone)]
pub struct Auction {
    pub seller: String,
    pub start_price: u64,
    pub highest_bid: u64,
    pub highest_bidder: Option<String>,
    pub end_time: u64,        // 结束时间
    pub started: bool,
}

/// NFT市场
pub struct NFTMarket {
    pub name: String,
    pub symbol: String,
    
    // NFT数据
    pub nfts: HashMap<u64, NFT>,
    pub owner_tokens: HashMap<String, Vec<u64>>,  // owner => token_ids
    
    // 市场数据
    pub listings: HashMap<u64, Listing>,          // token_id => listing
    pub auctions: HashMap<u64, Auction>,          // token_id => auction
    
    // 统计
    pub total_supply: u64,
    pub total_volume: u64,    // 总交易额
    pub next_token_id: u64,
    
    // 平台手续费 (100 = 2.5%)
    pub platform_fee_rate: u64,
    pub platform_owner: String,
}

impl NFTMarket {
    pub fn new(name: String, symbol: String, platform_owner: String) -> Self {
        Self {
            name,
            symbol,
            nfts: HashMap::new(),
            owner_tokens: HashMap::new(),
            listings: HashMap::new(),
            auctions: HashMap::new(),
            total_supply: 0,
            total_volume: 0,
            next_token_id: 1,
            platform_fee_rate: 25,  // 2.5%
            platform_owner,
        }
    }
    
    /// 铸造NFT
    pub fn mint(
        &mut self,
        creator: String,
        to: String,
        uri: String,
        royalty_rate: u64,
    ) -> Result<u64, String> {
        // 版税率不能超过10%
        if royalty_rate > 100 {
            return Err("版税率不能超过10%".to_string());
        }
        
        let token_id = self.next_token_id;
        
        let nft = NFT {
            token_id,
            owner: to.clone(),
            creator: creator.clone(),
            uri,
            royalty_rate,
        };
        
        self.nfts.insert(token_id, nft);
        self.owner_tokens.entry(to).or_default().push(token_id);
        
        self.total_supply += 1;
        self.next_token_id += 1;
        
        println!("✅ 铸造 NFT #{} 给 {}", token_id, to);
        
        Ok(token_id)
    }
    
    /// 转移NFT
    pub fn transfer(&mut self, from: &str, to: &str, token_id: u64) -> Result<(), String> {
        let nft = self.nfts.get_mut(&token_id).ok_or("NFT不存在")?;
        
        if nft.owner != from {
            return Err("无权转移".to_string());
        }
        
        // 更新所有者
        nft.owner = to.to_string();
        
        // 更新索引
        if let Some(tokens) = self.owner_tokens.get_mut(from) {
            tokens.retain(|&id| id != token_id);
        }
        self.owner_tokens.entry(to.to_string()).or_default().push(token_id);
        
        // 取消上架
        self.listings.remove(&token_id);
        
        println!("✅ NFT #{} 从 {} 转移到 {}", token_id, from, to);
        
        Ok(())
    }
    
    /// 上架出售
    pub fn list(&mut self, seller: &str, token_id: u64, price: u64, current_time: u64) -> Result<(), String> {
        let nft = self.nfts.get(&token_id).ok_or("NFT不存在")?;
        
        if nft.owner != seller {
            return Err("无权上架".to_string());
        }
        
        if price == 0 {
            return Err("价格不能为0".to_string());
        }
        
        // 检查是否已在拍卖中
        if self.auctions.contains_key(&token_id) {
            return Err("NFT正在拍卖中".to_string());
        }
        
        let listing = Listing {
            seller: seller.to_string(),
            price,
            listed_at: current_time,
        };
        
        self.listings.insert(token_id, listing);
        
        println!("✅ NFT #{} 上架出售，价格: {}", token_id, price);
        
        Ok(())
    }
    
    /// 取消上架
    pub fn delist(&mut self, seller: &str, token_id: u64) -> Result<(), String> {
        let listing = self.listings.get(&token_id).ok_or("NFT未上架")?;
        
        if listing.seller != seller {
            return Err("无权取消".to_string());
        }
        
        self.listings.remove(&token_id);
        
        println!("✅ NFT #{} 取消上架", token_id);
        
        Ok(())
    }
    
    /// 购买NFT
    pub fn buy(&mut self, buyer: &str, token_id: u64) -> Result<u64, String> {
        let listing = self.listings.get(&token_id).ok_or("NFT未上架")?;
        let price = listing.price;
        let seller = listing.seller.clone();
        
        let nft = self.nfts.get(&token_id).ok_or("NFT不存在")?;
        let creator = nft.creator.clone();
        let royalty_rate = nft.royalty_rate;
        
        // 计算各方收益
        let platform_fee = price * self.platform_fee_rate / 1000;
        let royalty = price * royalty_rate / 1000;
        let seller_receive = price - platform_fee - royalty;
        
        // 转移NFT所有权
        self.transfer(&seller, buyer, token_id)?;
        
        // 更新统计
        self.total_volume += price;
        
        // 移除上架
        self.listings.remove(&token_id);
        
        println!("✅ NFT #{} 被 {} 以 {} 购买", token_id, buyer, price);
        println!("   卖家获得: {}", seller_receive);
        println!("   创作者版税: {}", royalty);
        println!("   平台手续费: {}", platform_fee);
        
        Ok(seller_receive)
    }
    
    /// 创建拍卖
    pub fn create_auction(
        &mut self,
        seller: &str,
        token_id: u64,
        start_price: u64,
        duration: u64,
        current_time: u64,
    ) -> Result<(), String> {
        let nft = self.nfts.get(&token_id).ok_or("NFT不存在")?;
        
        if nft.owner != seller {
            return Err("无权拍卖".to_string());
        }
        
        if self.listings.contains_key(&token_id) {
            return Err("NFT已上架销售".to_string());
        }
        
        let auction = Auction {
            seller: seller.to_string(),
            start_price,
            highest_bid: 0,
            highest_bidder: None,
            end_time: current_time + duration,
            started: true,
        };
        
        self.auctions.insert(token_id, auction);
        
        println!("✅ NFT #{} 开始拍卖，起拍价: {}，持续时间: {}秒", 
            token_id, start_price, duration);
        
        Ok(())
    }
    
    /// 出价
    pub fn bid(&mut self, bidder: &str, token_id: u64, amount: u64, current_time: u64) -> Result<(), String> {
        let auction = self.auctions.get_mut(&token_id).ok_or("拍卖不存在")?;
        
        if !auction.started {
            return Err("拍卖未开始".to_string());
        }
        
        if current_time > auction.end_time {
            return Err("拍卖已结束".to_string());
        }
        
        let min_bid = if auction.highest_bid == 0 {
            auction.start_price
        } else {
            auction.highest_bid + auction.highest_bid / 20  // 至少加价5%
        };
        
        if amount < min_bid {
            return Err(format!("出价必须 >= {}", min_bid));
        }
        
        // 退还前一个出价者的资金（实际合约中）
        // ...
        
        auction.highest_bid = amount;
        auction.highest_bidder = Some(bidder.to_string());
        
        println!("✅ {} 出价 {} 购买 NFT #{}", bidder, amount, token_id);
        
        Ok(())
    }
    
    /// 结束拍卖
    pub fn end_auction(&mut self, token_id: u64, current_time: u64) -> Result<(), String> {
        let auction = self.auctions.get(&token_id).ok_or("拍卖不存在")?;
        
        if current_time < auction.end_time {
            return Err("拍卖未结束".to_string());
        }
        
        let seller = auction.seller.clone();
        let highest_bid = auction.highest_bid;
        let highest_bidder = auction.highest_bidder.clone();
        
        // 移除拍卖
        self.auctions.remove(&token_id);
        
        if let Some(bidder) = highest_bidder {
            // 有出价，成交
            let nft = self.nfts.get(&token_id).unwrap();
            let creator = nft.creator.clone();
            let royalty_rate = nft.royalty_rate;
            
            let platform_fee = highest_bid * self.platform_fee_rate / 1000;
            let royalty = highest_bid * royalty_rate / 1000;
            let seller_receive = highest_bid - platform_fee - royalty;
            
            self.transfer(&seller, &bidder, token_id)?;
            self.total_volume += highest_bid;
            
            println!("✅ 拍卖结束！NFT #{} 被 {} 以 {} 拍得", 
                token_id, bidder, highest_bid);
            println!("   卖家获得: {}", seller_receive);
        } else {
            // 无人出价，流拍
            println!("✅ 拍卖结束，NFT #{} 流拍", token_id);
        }
        
        Ok(())
    }
    
    /// 查询NFT信息
    pub fn get_nft(&self, token_id: u64) -> Option<&NFT> {
        self.nfts.get(&token_id)
    }
    
    /// 查询用户拥有的NFT
    pub fn get_user_nfts(&self, owner: &str) -> Vec<&NFT> {
        self.owner_tokens
            .get(owner)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.nfts.get(id))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    /// 查询所有上架的NFT
    pub fn get_listings(&self) -> Vec<(u64, &NFT, &Listing)> {
        self.listings
            .iter()
            .filter_map(|(id, listing)| {
                self.nfts.get(id).map(|nft| (*id, nft, listing))
            })
            .collect()
    }
}

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║           ChengChain NFT市场演示                          ║");
    println!("╚══════════════════════════════════════════════════════════╝\n");
    
    let mut market = NFTMarket::new(
        "Cheng NFT".to_string(),
        "CNFT".to_string(),
        "platform".to_string(),
    );
    
    // 1. 铸造NFT
    println!("1️⃣  艺术家铸造NFT");
    let nft1 = market.mint(
        "Artist_A".to_string(),
        "Artist_A".to_string(),
        "https://yrust.chain/nft/1".to_string(),
        50,  // 5%版税
    ).unwrap();
    
    let nft2 = market.mint(
        "Artist_B".to_string(),
        "Artist_B".to_string(),
        "https://yrust.chain/nft/2".to_string(),
        100, // 10%版税
    ).unwrap();
    
    println!("   NFT #{} 创建成功", nft1);
    println!("   NFT #{} 创建成功", nft2);
    
    // 2. 上架出售
    println!("\n2️⃣  艺术家A上架NFT #{}，价格1000", nft1);
    market.list("Artist_A", nft1, 1000, 1000).unwrap();
    
    // 3. 用户购买
    println!("\n3️⃣  Collector购买NFT #{}");
    market.buy("Collector", nft1).unwrap();
    
    // 4. 收藏家转售
    println!("\n4️⃣  Collector转售NFT #{}，价格2000", nft1);
    market.list("Collector", nft1, 2000, 2000).unwrap();
    
    // 5. 另一个人购买（触发版税）
    println!("\n5️⃣  Investor购买NFT #{}（艺术家A获得版税）", nft1);
    market.buy("Investor", nft1).unwrap();
    
    // 6. 创建拍卖
    println!("\n6️⃣  艺术家B拍卖NFT #{}，起拍价500", nft2);
    market.create_auction("Artist_B", nft2, 500, 3600, 3000).unwrap();
    
    // 7. 出价
    println!("\n7️⃣  多人出价");
    market.bid("Bidder1", nft2, 600, 3100).unwrap();
    market.bid("Bidder2", nft2, 800, 3200).unwrap();
    market.bid("Bidder1", nft2, 1000, 3300).unwrap();
    
    // 8. 结束拍卖
    println!("\n8️⃣  拍卖结束");
    market.end_auction(nft2, 6600).unwrap();
    
    // 统计
    println!("\n📊 市场统计");
    println!("   总供应量: {}", market.total_supply);
    println!("   总交易额: {}", market.total_volume);
    println!("   当前上架: {} 个", market.listings.len());
    
    println!("\n✨ NFT市场演示完成！");
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mint_and_transfer() {
        let mut market = NFTMarket::new("Test".to_string(), "TEST".to_string(), "platform".to_string());
        
        let token_id = market.mint("creator", "owner", "uri".to_string(), 50).unwrap();
        assert_eq!(market.total_supply, 1);
        
        market.transfer("owner", "new_owner", token_id).unwrap();
        assert_eq!(market.get_nft(token_id).unwrap().owner, "new_owner");
    }
    
    #[test]
    fn test_buy_with_royalty() {
        let mut market = NFTMarket::new("Test".to_string(), "TEST".to_string(), "platform".to_string());
        
        let token_id = market.mint("creator", "seller", "uri".to_string(), 100).unwrap();
        market.list("seller", token_id, 1000, 100).unwrap();
        
        let seller_receive = market.buy("buyer", token_id).unwrap();
        
        // 1000 - 25(2.5%平台费) - 100(10%版税) = 875
        assert_eq!(seller_receive, 875);
    }
}
