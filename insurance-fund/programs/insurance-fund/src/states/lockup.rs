use anchor_lang::prelude::*;
use crate::errors::InsuranceFundError;

// Arrays are holding rates at which users are rewarded per 1 unit per lockup duration.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum YieldMode {
    Single, // Only offers rUSD yield.
    Dual(u64) // Offers both rUSD and $R yields. 
    // The u64 stores rate at which $R should be minted per 1 unit of deposit per lockup duration.
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct SlashState {
    // How many times was this lockup slashed
    pub index: u64,
    // Total amount slashed
    pub amount: u64,
}

#[account]
pub struct Lockup {
    pub bump: u8,
    pub locked: bool,
    pub index: u64,
    pub asset: Pubkey,
    pub min_deposit: u64,
    pub total_deposits: u64,
    pub duration: u64,
    // Not required, yield will depend on the crank depositing into the pool
    pub yield_bps: u64,
    pub yield_mode: YieldMode,
    pub deposit_cap: Option<u64>,
    pub deposits: u64,
    pub slash_state: SlashState,
    pub reward_boosts: u64,
}

impl Lockup {
    pub const SIZE: usize = 8 + 1 + 1 + 6 * 8 + 1 + 32 + 17 + 16 + 8 + 8;

    pub fn lock(&mut self) {
        self.locked = true;
    } 

    pub fn unlock(&mut self) {
        self.locked = false;
    } 

    pub fn increase_deposits(&mut self, amount: u64) -> Result<()> {
        self.total_deposits = self.total_deposits
            .checked_add(amount)
            .ok_or(InsuranceFundError::MathOverflow)?;

        Ok(())
    }

    pub fn decrease_deposits(&mut self, amount: u64) -> Result<()> {
        self.total_deposits = self.total_deposits
            .checked_sub(amount)
            .ok_or(InsuranceFundError::MathOverflow)?;

        Ok(())
    }
}