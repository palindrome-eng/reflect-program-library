use anchor_lang::prelude::*;

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
    pub asset_mint: Pubkey,
    pub receipt_mint: Pubkey,
    pub receipt_to_reward_exchange_rate_bps: u64,
    pub deposit_cap: Option<u64>,
    pub min_deposit: u64,
    pub duration: u64,
    pub deposits: u64,
    pub reward_boosts: u64,
    pub yield_mode: YieldMode,
    pub slash_state: SlashState,
}

impl Lockup {
    pub const SIZE: usize = 8 + 1 + 1 + 8 * 6 + 2 * 32 + (1 + 8) + (1 + 8) + (2 * 8);

    pub fn lock(&mut self) {
        self.locked = true;
    } 

    pub fn unlock(&mut self) {
        self.locked = false;
    }
}