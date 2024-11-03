use anchor_lang::prelude::*;

#[account]
pub struct Intent {
    pub amount: u64,
    pub lockup: Pubkey,
    pub deposit: Pubkey,
}

impl Intent {
    pub const LEN: usize = 8 + 2 * 32 + 8;
}