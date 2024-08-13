use anchor_lang::{prelude::*, solana_program::pubkey};

#[account]
pub struct Deposit {
    pub user: Pubkey,
    pub amount: u64,
    pub lockup: Pubkey,
    pub unlock_ts: u64, // Unlock timestamp
}

impl Deposit {
    pub const LEN: usize = 8 + 8 + 2 * 32;
}