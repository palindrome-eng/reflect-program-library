use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum CooldownRewards {
    Single(u64),
    Dual([u64; 2])
}

impl CooldownRewards {
    pub const SIZE: usize = 1 + (2 * 8);
}

#[derive(InitSpace)]
#[account]
pub struct Cooldown {
    pub user: Pubkey,
    pub deposit_id: u64,
    pub lockup_id: u64,
    pub receipt_amount: u64,
    pub unlock_ts: u64,
    pub rewards: CooldownRewards,
}

impl Cooldown {
    pub fn is_cooled(&self) -> Result<bool> {
        let clock = Clock::get()?;
        Ok(clock.unix_timestamp as u64 >= self.unlock_ts)
    }

    pub fn lock(&mut self, duration: u64) -> Result<()> {
        let clock = Clock::get()?;

        let now = clock.unix_timestamp;
        self.unlock_ts = (now as u64) + duration;
        
        Ok(())
    }
}