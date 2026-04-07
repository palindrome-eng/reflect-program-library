use anchor_lang::prelude::*;

#[derive(InitSpace, Default)]
#[account]
pub struct Cooldown {
    pub bump: u8,
    pub index: u64,
    pub authority: Pubkey,
    pub liquidity_pool_id: u8,
    pub unlock_ts: u64,
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