use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Intent {
    pub amount: u64,
    pub lockup: Pubkey,
    pub deposit: Pubkey,
}