use anchor_lang::prelude::*;

pub mod states;
pub mod constants;
pub mod errors;
pub mod instructions;
pub mod events;
pub mod helpers;

use crate::instructions::*;

#[cfg(feature = "staging")]
declare_id!("GSEYK2FtDLywAoxn2mioXCft9FWH6Zn4VmKUArGyTVj8");

#[cfg(not(feature = "staging"))]
declare_id!("JrXLmS6aYJNJDVxdAfjNJE5wikT8ubf3TA9iL2JA9Av");

#[program]
pub mod rlp {
    use super::*;

    pub fn initialize_rlp(
        ctx: Context<InitializeRlp>,
        args: InitializeRlpArgs,
    ) -> Result<()> {
        instructions::initialize_rlp(ctx, args)
    }

    pub fn initialize_lp(
        ctx: Context<InitializeLiquidityPool>,
        args: InitializeLiquidityPoolArgs
    ) -> Result<()> {
        instructions::initialize_lp(ctx, args)
    }

    pub fn initialize_pool_reserve(
        ctx: Context<InitializePoolReserve>,
        _liquidity_pool_id: u8,
    ) -> Result<()> {
        instructions::initialize_pool_reserve(ctx)
    }

    pub fn add_asset(
        ctx: Context<AddAsset>,
        args: AddAssetArgs
    ) -> Result<()> {
        instructions::add_asset(ctx, args)
    }

    pub fn freeze_functionality(
        ctx: Context<RlpAdminMain>,
        args: FreezeProtocolActionArgs
    ) -> Result<()> {
        instructions::freeze_protocol_action(ctx, args)
    }

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

    pub fn deposit<'a>(
        ctx: Context<'_, '_, 'a, 'a, Deposit<'a>>,
        args: DepositArgs
    ) -> Result<()> {
        instructions::deposit(ctx, args)
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

    pub fn swap(
        ctx: Context<Swap>,
        args: SwapArgs
    ) -> Result<()> {
        instructions::swap(ctx, args)
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

    pub fn update_oracle(
        ctx: Context<UpdateOracle>,
    ) -> Result<()> {
        instructions::update_oracle(ctx)
    }

    pub fn force_remove_asset(
        ctx: Context<ForceRemoveAsset>,
        _liquidity_pool_id: u8,
    ) -> Result<()> {
        instructions::force_remove_asset(ctx)
    }
}