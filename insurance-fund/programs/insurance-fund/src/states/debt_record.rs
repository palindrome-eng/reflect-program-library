use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct DebtRecord {
    pub amount: u64,
    pub asset: Pubkey,
    pub lockup: u64,
    pub timestamp: u64,
}