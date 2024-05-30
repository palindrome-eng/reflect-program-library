use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::{create_vault::*, init_vault_pools::*, deposit::*, withdraw::*};

declare_id!("6ZZ1sxKGuXUBL8HSsHqHaYCg92G9VhMNTcJv1gFURCop");
#[program]
mod reflect_tokenised_bonds {
    use super::*;

    pub fn create_vault(
        ctx: Context<CreateVault>,
        deposit_token_mint: Pubkey,
        receipt_token_mint: Pubkey,
        min_deposit: u64,
        min_lockup: i64,
        target_yield_rate: u64,
        vault_seed: u64,
    ) -> Result<()> {
        instructions::create_vault::create_vault(ctx, deposit_token_mint, receipt_token_mint, min_deposit, min_lockup, target_yield_rate, vault_seed)
    }

    pub fn init_vault_pools(
        ctx: Context<InitVaultPools>,
        vault_seed: u64,
    ) -> Result<()> {
        instructions::init_vault_pools::init_vault_pools(ctx, vault_seed)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        instructions::deposit::deposit(ctx, amount)
    }

    pub fn request_withdraw(ctx: Context<RequestWithdraw>, receipt_amount: u64) -> Result<()> {
        instructions::withdraw::request_withdraw(ctx, receipt_amount)
    }

    pub fn complete_withdraw(ctx: Context<CompleteWithdraw>) -> Result<()> {
        instructions::withdraw::complete_withdraw(ctx)
    }
}
