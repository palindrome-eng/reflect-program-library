use anchor_lang::prelude::*;

#[account]
pub struct Vault {
    pub admin: Pubkey,
    pub deposit_token_mint: Pubkey,
    pub receipt_token_mint: Pubkey,
    pub min_deposit: u64,
    pub min_lockup: i64,
    pub target_yield_rate: u64,
    pub deposit_pool: Pubkey,
    pub reward_pool: Pubkey,
    pub total_receipt_supply: u64,
}

impl Vault {
    pub const LEN: usize = 8 + // discriminator
        32 + // admin
        32 + // deposit_token_mint
        32 + // receipt_token_mint
        8 + // min_deposit
        8 + // min_lockup
        8 + // target_yield_rate
        32 + // deposit_pool
        32 + // reward_pool
        8; // total_receipt_supply
}
