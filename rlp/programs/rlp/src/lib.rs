use anchor_lang::prelude::*;

pub mod states;
pub mod constants;
pub mod errors;
pub mod instructions;
pub mod events;
pub mod helpers;

use crate::instructions::*;

declare_id!("rhLMe6vyM1wVLJaxrWUckVmPxSia58nSWZRDtYQow6D");

#[program]
pub mod rlp {
    use super::*;

    pub fn initialize_rlp(
        ctx: Context<InitializeRlp>, 
        args: InitializeRlpArgs
    ) -> Result<()> {
        instructions::initialize_rlp(
            ctx,
            args
        )
    }

    pub fn initialize_lp(
        ctx: Context<InitializeLiquidityPool>,
        args: InitializeLiquidityPoolArgs
    ) -> Result<()> {
        instructions::initialize_lp(ctx, args)
    }

    // TODO: Add asset should be on liquidity pool level
    pub fn add_asset(
        ctx: Context<AddAsset>,
    ) -> Result<()> {
        instructions::add_asset(ctx)
    }

    pub fn deposit_rewards(
        ctx: Context<DepositRewards>,
        args: DepositRewardsArgs
    ) -> Result<()> {
        instructions::deposit_rewards(ctx, args)
    }

    // Freeze should allow for freezing:
    // - liquidity pool
    // - asset
    pub fn manage_freeze(
        ctx: Context<ManageFreeze>,
        args: ManageFreezeArgs
    ) -> Result<()> {
        instructions::manage_freeze(ctx, args)
    }

    // Deposit cap should be on liquidity pool level
    pub fn update_deposit_cap(
        ctx: Context<UpdateDepositCap>,
        args: UpdateDepositCapArgs
    ) -> Result<()> {
        instructions::update_deposit_cap(ctx, args)
    }

    pub fn slash(
        ctx: Context<Slash>,
        args: SlashArgs
    ) -> Result<()> {
        instructions::slash(ctx, args)
    }

    pub fn restake<'a>(
        ctx: Context<'_, '_, 'a, 'a, Restake<'a>>,
        args: RestakeArgs
    ) -> Result<()> {
        instructions::restake(ctx, args)
    }

    pub fn request_withdrawal(
        ctx: Context<RequestWithdrawal>,
        args: RequestWithdrawalArgs
    ) -> Result<()> {
        instructions::request_withdrawal(ctx, args)
    }

    pub fn withdraw<'a>(
        ctx: Context<'_, '_, 'a, 'a, Withdraw<'a>>,
        args: WithdrawArgs
    ) -> Result<()> {
        instructions::withdraw(ctx, args)
    }

    pub fn swap_lp(
        ctx: Context<SwapLp>,
        args: SwapLpArgs
    ) -> Result<()> {
        instructions::swap_lp(ctx, args)
    }
}