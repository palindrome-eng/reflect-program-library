//src/state/withdraw_request.rs
use anchor_lang::prelude::*;

#[account]
pub struct WithdrawRequest {
    pub user: Pubkey,
    pub vault: Pubkey,
    pub receipt_amount: u64,
    pub unlock_date: i64,
}

impl WithdrawRequest {
    pub const LEN: usize = 8 + // discriminator
        32 + // user
        32 + // vault
        8 + // receipt_amount
        8; // unlock_date
}
