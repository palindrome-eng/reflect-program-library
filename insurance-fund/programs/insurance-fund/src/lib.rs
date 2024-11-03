use anchor_lang::prelude::*;

pub mod states;
pub mod constants;
pub mod errors;
pub mod instructions;
pub mod events;
pub mod helpers;
use crate::instructions::*;

declare_id!("BXopfEhtpSHLxK66tAcxY7zYEUyHL6h91NJtP2nWx54e");

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

    pub fn manage_freeze(
        ctx: Context<ManageFreeze>,
        args: ManageFreezeArgs
    ) -> Result<()> {
        instructions::manage_freeze(ctx, args)
    }

    pub fn initialize_lockup(
        ctx: Context<InitializeLockup>,
        args: InitializeLockupArgs
    ) -> Result<()> {
        instructions::initialize_lockup(ctx, args)
    }

    pub fn manage_lockup_lock(
        ctx: Context<ManageLockupLock>,
        args: ManageLockupLockArgs
    ) -> Result<()> {
        instructions::manage_lockup_lock(ctx, args)
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

    pub fn withdraw(
        ctx: Context<Withdraw>,
        args: WithdrawArgs
    ) -> Result<()> {
        instructions::withdraw(ctx, args)
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

    pub fn slash_cold_wallet(
        ctx: Context<SlashColdWallet>,
        args: SlashColdWalletArgs
    ) -> Result<()> {
        instructions::slash_cold_wallet(ctx, args)
    }

    pub fn deposit_rewards(
        ctx: Context<DepositRewards>,
        args: DepositRewardsArgs
    ) -> Result<()> {
        instructions::deposit_rewards(ctx, args)
    }

    pub fn create_intent(
        ctx: Context<CreateIntent>,
        args: CreateIntentArgs
    ) -> Result<()> {
        instructions::create_intent(ctx, args)
    }

    pub fn process_intent(
        ctx: Context<ProcessIntent>,
        args: ProcessIntentArgs
    ) -> Result<()> {
        instructions::process_intent(ctx, args)
    }
}