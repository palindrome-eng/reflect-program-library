use anchor_lang::prelude::*;

#[account]
pub struct LockupState {
    pub id: u64,
    pub user: Pubkey,
    pub vault: Pubkey,
    pub receipt_amount: u64,
    pub unlock_date: i64,
}

impl LockupState {
    pub const LEN: usize = 8 + // discriminator
        8 + // lockup id
        32 + // user
        32 + // vault
        8 + // receipt_amount
        8; // unlock_date
}
