use anchor_lang::{prelude::*, solana_program::pubkey};

#[account]
pub struct Deposit {
    pub user: Pubkey,
    pub amount: u64, // Total deposited
    pub amount_slashed: u64, // Amount lost due to slashing
    pub lockup: Pubkey, // Pointer to the lockup
    pub unlock_ts: u64, // Unlock timestamp
    pub last_slashed: Option<u64>, // Index of the last slash
}

impl Deposit {
    pub const LEN: usize = 8 + 2 * 32 + 3 * 8 + (1 + 8);
}