use anchor_lang::prelude::*;

// Arrays are holding rates at which users are rewarded per 1 unit per lockup duration.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum YieldMode {
    Single([u64; 1]), // Only offers rUSD yield.
    Dual([u64; 2]) // Offers both rUSD and $R yields
}

#[account]
pub struct Lockup {
    pub bump: u8,
    pub index: u64,
    pub asset: Pubkey,
    pub min_deposit: u64,
    pub duration: u64,
    pub yield_bps: u64,
    pub yield_mode: YieldMode,
    pub deposit_cap: u64,
    pub deposits: u64,
}

impl Lockup {
    pub const SIZE: usize = 8 + 1 + 6 * 8 + 32 + 17;
}