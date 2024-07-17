use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;
pub mod constants;

use instructions::*;

declare_id!("6ZZ1sxKGuXUBL8HSsHqHaYCg92G9VhMNTcJv1gFURCop");

#[program]
mod reflect_tokenised_bonds {
    use super::*;

    // Required to initialize main protocol account,
    // which keeps track of vault seed.
    pub fn initialize_protocol(
        ctx: Context<InitializeProtocol>
    ) -> Result<()> {
        instructions::initialize_protocol(ctx)
    }

    pub fn create_vault(
        ctx: Context<CreateVault>,
        min_deposit: u64,
        min_lockup: i64,
        target_yield_rate: u64,
        vault_seed: u64,
    ) -> Result<()> {
        instructions::create_vault(
            ctx,
            min_deposit,
            min_lockup, 
            target_yield_rate, 
            vault_seed
        )
    }

    pub fn init_vault_pools(
        ctx: Context<InitVaultPools>,
        vault_seed: u64,
    ) -> Result<()> {
        instructions::init_vault_pools(ctx, vault_seed)
    }

    pub fn deposit(
        ctx: Context<Deposit>, 
        amount: u64,
        vault_id: u64,
    ) -> Result<()> {
        instructions::deposit(
            ctx, 
            amount,
            vault_id
        )
    }

    pub fn lockup(ctx: Context<Lockup>, receipt_amount: u64) -> Result<()> {
        instructions::lockup(ctx, receipt_amount)
    }

    pub fn complete_withdraw(ctx: Context<Withdraw>) -> Result<()> {
        instructions::withdraw(ctx)
    }
}
