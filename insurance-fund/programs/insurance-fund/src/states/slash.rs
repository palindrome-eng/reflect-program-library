use anchor_lang::prelude::*;

#[account]
pub struct Slash {
    pub index: u64,
    pub target_accounts: u64,
    pub slashed_accounts: u64,
    pub target_amount: u64,
    pub slashed_amount: u64
}

impl Slash {
    pub const LEN: usize = 8 + 5 * 8;
}