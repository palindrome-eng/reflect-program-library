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

    pub fn initialize_lp_token_account(
        ctx: Context<InitializeLpTokenAccount>,
        args: InitializeLpTokenAccountArgs
    ) -> Result<()> {
        instructions::initialize_lp_token_account(ctx, args)
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

    pub fn freeze_functionality(
        ctx: Context<RlpAdminMain>,
        args: FreezeProtocolActionArgs
    ) -> Result<()> {
        instructions::freeze_protocol_action(ctx, args)
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

    pub fn create_permission_account(
        ctx: Context<RlpUserPermissionsInit>,
        new_admin: Pubkey
    ) -> Result<()> {
        instructions::create_permission_account(ctx, new_admin)
    }

    pub fn update_action_role(
        ctx: Context<RlpAdminMain>,
        args: UpdateActionRoleArgs
    ) -> Result<()> {
        instructions::update_action_role_protocol(ctx, args)
    }

    pub fn update_role_holder(
        ctx: Context<RlpAdminRoleUpdate>,
        args: UpdateRoleHolderArgs
    ) -> Result<()> {
        instructions::update_role_holder_protocol(ctx, args)
    }
}