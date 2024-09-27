use anchor_lang::prelude::*;

#[account]
pub struct Intent {
    pub amount: u64,
    pub total_deposit: u64,
    pub lockup: Pubkey
}

impl Intent {
    pub const LEN: usize = 8 + 2 * 32 + 2 * 8;
}