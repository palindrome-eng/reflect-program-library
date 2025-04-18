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

    pub fn initialize(
        ctx: Context<Initialize>
    ) -> Result<()> {
        instructions::initialize(ctx)
    }

    pub fn create_vault(
        ctx: Context<CreateVault>,
    ) -> Result<()> {
        instructions::create_vault(ctx)
    }
    
    pub fn deposit(
        ctx: Context<Deposit>,
        args: DepositArgs
    ) -> Result<()> {
        instructions::deposit(ctx, args)
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        args: WithdrawArgs
    ) -> Result<()> {
        instructions::withdraw(ctx, args)
    }
}
