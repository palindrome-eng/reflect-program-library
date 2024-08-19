use anchor_lang::prelude::*;

pub mod states;
pub mod constants;
pub mod errors;
pub mod instructions;
use crate::instructions::*;

declare_id!("CPW6gyeGhh7Kt3LYwjF7yXTYgbcNfT7dYBSRDz7TH5YB");

#[program]
pub mod insurance_fund {
    use super::*;

    pub fn initialize_insurance_fund(
        ctx: Context<InitializeInsuranceFund>, 
        args: InitializeInsuranceFundArgs
    ) -> Result<()> {
        instructions::initialize_insurance_fund(
            ctx,
            args
        )
    }

    pub fn initialize_lockup(
        ctx: Context<InitializeLockup>,
        args: InitializeLockupArgs
    ) -> Result<()> {
        instructions::initialize_lockup(ctx, args)
    }

    pub fn add_asset(
        ctx: Context<AddAsset>,
    ) -> Result<()> {
        instructions::add_asset(ctx)
    }

    pub fn restake(
        ctx: Context<Restake>,
        args: RestakeArgs
    ) -> Result<()> {
        instructions::restake(
            ctx,
            args
        )
    }

    pub fn initialize_slash(
        ctx: Context<InitializeSlash>,
        args: InitializeSlashArgs
    ) -> Result<()> {
        instructions::initialize_slash(ctx, args)
    }
    
    pub fn slash_deposits(
        ctx: Context<SlashDeposits>,
        args: SlashDepositsArgs
    ) -> Result<()> {
        instructions::slash_deposits(ctx, args)
    }

    pub fn slash_pool(
        ctx: Context<SlashPool>,
        args: SlashPoolArgs
    ) -> Result<()> {
        instructions::slash_pool(ctx, args)
    }
}