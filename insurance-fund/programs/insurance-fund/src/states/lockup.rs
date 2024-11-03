use anchor_lang::prelude::*;

// Arrays are holding rates at which users are rewarded per 1 unit per lockup duration.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum YieldMode {
    Single([u64; 1]), // Only offers rUSD yield.
    Dual([u64; 2]) // Offers both rUSD and $R yields
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
    pub const SIZE: usize = 8 + 1 + 1 + 6 * 8 + 1 + 32 + 17 + 16 + 8;

    pub fn lock(&mut self) {
        self.locked = true;
    } 

    pub fn unlock(&mut self) {
        self.locked = false;
    } 
}