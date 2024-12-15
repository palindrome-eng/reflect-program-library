use std::collections::HashMap;

type Amount = u128;

// Using 1e6 precision to avoid overflows while maintaining good accuracy
const PRECISION: Amount = 1_000_000u128;

struct UserPosition {
    x_shares: Amount,
    y_ratio_entry_point: Amount,
}

struct Pool {
    total_x: Amount,
    total_x_shares: Amount,
    total_y: Amount,
    // y_share_ratio: Amount,
    positions: HashMap<String, UserPosition>,
}

impl Pool {
    pub fn new() -> Self {
        Pool {
            total_x: 0,
            total_x_shares: 0,
            total_y: 0,
            positions: HashMap::new(),
        }
    }
    
    pub fn deposit(&mut self, user: String, amount_x: Amount) -> Result<(), String> {
        if amount_x == 0 {
            return Err("Cannot deposit zero amount".to_string());
        }

        // Calculate X shares to mint
        let x_shares = if self.total_x_shares == 0 {
            amount_x // First depositor gets 1:1
        } else {
            // (amount_x * total_shares) / total_x
            amount_x.checked_mul(self.total_x_shares)
                .ok_or("Overflow in share calculation")?
                .checked_div(self.total_x)
                .ok_or("Division by zero")?
        };
        
        let position = self.positions.entry(user).or_insert(UserPosition {
            x_shares: 0,
            y_ratio_entry_point: self.total_y * PRECISION / self.total_x_shares,
        });
        
        if position.x_shares == 0 {
            position.y_ratio_entry_point = self.total_y * PRECISION / self.total_x_shares;
        }
        
        position.x_shares = position.x_shares.checked_add(x_shares)
            .ok_or("Overflow in position shares")?;
        
        self.total_x = self.total_x.checked_add(amount_x)
            .ok_or("Overflow in total deposits")?;
        self.total_x_shares = self.total_x_shares.checked_add(x_shares)
            .ok_or("Overflow in total shares")?;
        
        Ok(())
    }
    
    pub fn add_rewards(&mut self, amount_y: Amount) -> Result<(), String> {
        if amount_y == 0 {
            return Err("Cannot add zero rewards".to_string());
        }

        if self.total_x_shares == 0 {
            return Err("No stakers in pool".to_string());
        }
        
        // let ratio_increase = amount_y.checked_mul(PRECISION)
        //     .ok_or("Overflow in ratio calculation")?
        //     .checked_div(self.total_x_shares)
        //     .ok_or("Division by zero")?;
            
        // self.y_share_ratio = self.y_share_ratio.checked_add(ratio_increase)
        //     .ok_or("Overflow in y_share_ratio")?;
            
        self.total_y = self.total_y.checked_add(amount_y)
            .ok_or("Overflow in total_y")?;
        
        Ok(())
    }
    
    pub fn withdraw(&mut self, user: String) -> Result<(Amount, Amount), String> {
        let position = self.positions.get(&user)
            .ok_or("No position found")?;
            
        if position.x_shares == 0 {
            return Err("No shares to withdraw".to_string());
        }
        
        // Calculate X tokens to return
        let x_amount = position.x_shares.checked_mul(self.total_x)
            .ok_or("Overflow in withdrawal calculation")?
            .checked_div(self.total_x_shares)
            .ok_or("Division by zero")?;

        let y_share_ratio = self.total_y * PRECISION / self.total_x_shares;
        
        // Calculate Y rewards
        let ratio_diff = y_share_ratio.checked_sub(position.y_ratio_entry_point)
            .unwrap_or(0);
            
        let y_amount = if ratio_diff == 0 {
            0
        } else {
            position.x_shares.checked_mul(ratio_diff)
                .ok_or("Overflow in reward calculation")?
                .checked_div(PRECISION)
                .ok_or("Division by zero")?
        };
        
        self.total_x = self.total_x.checked_sub(x_amount)
            .ok_or("Insufficient pool balance x")?;
        self.total_x_shares = self.total_x_shares.checked_sub(position.x_shares)
            .ok_or("Insufficient shares")?;
        
        if y_amount > 0 {
            self.total_y = self.total_y.checked_sub(y_amount)
                .ok_or("Insufficient pool balance y")?;
        }
        
        self.positions.remove(&user);
        
        Ok((x_amount, y_amount))
    }
    
    pub fn get_pending_rewards(&self, user: &str) -> Amount {
        self.positions.get(user)
            .map(|position| {
                if position.x_shares == 0 {
                    return 0;
                }
                
                let y_share_ratio = self.total_y * PRECISION / self.total_x_shares;
                let ratio_diff = y_share_ratio.saturating_sub(position.y_ratio_entry_point);
                
                if ratio_diff == 0 {
                    0
                } else {
                    position.x_shares.saturating_mul(ratio_diff) / PRECISION
                }
            })
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_flow() {
        let mut pool = Pool::new();
        
        let deposit_amount = 1000 * PRECISION;
        let reward_amount = 100 * PRECISION;
        
        // Alice deposits 1000
        pool.deposit("alice".to_string(), deposit_amount).unwrap();
        assert_eq!(pool.total_x_shares, deposit_amount);
        assert_eq!(pool.total_x, deposit_amount);
        
        // First reward: 100. Alice has 100% of shares
        pool.add_rewards(reward_amount).unwrap();
        
        let alice_pending = pool.get_pending_rewards("alice");
        assert_eq!(alice_pending, reward_amount, "Alice should have all 100 tokens from first reward");
        
        // Bob deposits 1000
        pool.deposit("bob".to_string(), deposit_amount).unwrap();
        assert_eq!(pool.total_x_shares, deposit_amount * 2);
        assert_eq!(pool.total_x, deposit_amount * 2);
        
        let bob_pending = pool.get_pending_rewards("bob");
        assert_eq!(bob_pending, 0, "Bob should have 0 rewards right after deposit");
        
        // Second reward: 100. Both have 50% of shares
        pool.add_rewards(reward_amount).unwrap();
        
        let alice_final = pool.get_pending_rewards("alice");
        let bob_final = pool.get_pending_rewards("bob");
        
        assert_eq!(alice_final, 150 * PRECISION, "Alice should have 150 tokens total");
        assert_eq!(bob_final, 50 * PRECISION, "Bob should have 50 tokens total");
        
        // Test withdrawal
        let (alice_x, alice_y) = pool.withdraw("alice".to_string()).unwrap();
        assert_eq!(alice_x, deposit_amount, "Alice should get back 1000 X tokens");
        assert_eq!(alice_y, 150 * PRECISION, "Alice should get 150 Y tokens");
        
        let (bob_x, bob_y) = pool.withdraw("bob".to_string()).unwrap();
        assert_eq!(bob_x, deposit_amount, "Bob should get back 1000 X tokens");
        assert_eq!(bob_y, 50 * PRECISION, "Bob should get 50 Y tokens");
        
        assert_eq!(pool.total_x, 0);
        assert_eq!(pool.total_x_shares, 0);
        assert_eq!(pool.total_y, 0);
    }

    #[test]
    fn test_multiple_rewards() {
        let mut pool = Pool::new();
        
        let deposit_amount = 1000 * PRECISION;
        let reward_amount = 100 * PRECISION;
        
        // Alice deposits 1000
        pool.deposit("alice".to_string(), deposit_amount).unwrap();
        
        // Add three 100-token rewards while Alice is sole depositor
        for _ in 0..3 {
            pool.add_rewards(reward_amount).unwrap();
        }
        
        let alice_after_three = pool.get_pending_rewards("alice");
        assert_eq!(alice_after_three, 300 * PRECISION, "Alice should have all 300 tokens from first three rewards");
        
        // Bob deposits 1000
        pool.deposit("bob".to_string(), deposit_amount).unwrap();
        assert_eq!(pool.get_pending_rewards("bob"), 0, "Bob should start with 0 rewards");
        
        // Add two more 100-token rewards
        for _ in 0..2 {
            pool.add_rewards(reward_amount).unwrap();
        }
        
        let alice_final = pool.get_pending_rewards("alice");
        let bob_final = pool.get_pending_rewards("bob");
        
        // Alice: 300 (first three rewards) + 100 (50% of last two rewards) = 400
        assert_eq!(alice_final, 400 * PRECISION, "Alice should have 400 tokens total");
        
        // Bob: 0 (from first three rewards) + 100 (50% of last two rewards) = 100
        assert_eq!(bob_final, 100 * PRECISION, "Bob should have 100 tokens total");
    }

    #[test]
    fn test_uneven_deposits() {
        let mut pool = Pool::new();
        
        // Alice deposits 1000
        pool.deposit("alice".to_string(), 1000 * PRECISION).unwrap();
        
        // Add reward of 100
        pool.add_rewards(100 * PRECISION).unwrap();
        
        // Bob deposits 3000 (3x more than Alice)
        pool.deposit("bob".to_string(), 3000 * PRECISION).unwrap();
        
        // Add reward of 200
        pool.add_rewards(200 * PRECISION).unwrap();
        
        let alice_final = pool.get_pending_rewards("alice");
        let bob_final = pool.get_pending_rewards("bob");
        
        // Alice: 100 (first reward) + 50 (25% of second reward) = 150
        assert_eq!(alice_final, 150 * PRECISION, "Alice should have 150 tokens total");
        
        // Bob: 0 (from first reward) + 150 (75% of second reward) = 150
        assert_eq!(bob_final, 150 * PRECISION, "Bob should have 150 tokens total");
    }

    #[test]
    fn test_edge_cases() {
        let mut pool = Pool::new();
        
        // Test zero deposits
        assert!(pool.deposit("alice".to_string(), 0).is_err());
        
        // Test zero rewards
        pool.deposit("alice".to_string(), 100 * PRECISION).unwrap();
        assert!(pool.add_rewards(0).is_err());
        
        // Test rewards with no stakers
        let mut empty_pool = Pool::new();
        assert!(empty_pool.add_rewards(100 * PRECISION).is_err());
        
        // Test withdraw with no position
        assert!(pool.withdraw("bob".to_string()).is_err());
        
        // Test multiple withdraws
        pool.withdraw("alice".to_string()).unwrap();
        assert!(pool.withdraw("alice".to_string()).is_err());
    }
}
