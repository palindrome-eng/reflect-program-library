use anchor_lang::prelude::*;

pub mod states;
pub mod constants;
pub mod errors;
pub mod instructions;
pub mod events;
pub mod helpers;

use crate::instructions::*;

declare_id!("moCkrLsd1dMvqQgzFgLWSEgYUR7SAMMrNzRwo3TjW2h");

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

    pub fn migrate_settings(
        ctx: Context<MigrateSettings>,
    ) -> Result<()> {
        instructions::migrate_settings(ctx)
    }

    pub fn migrate_dead_shares(
        ctx: Context<MigrateDeadShares>,
        args: MigrateDeadSharesArgs,
    ) -> Result<()> {
        instructions::migrate_dead_shares(ctx, args)
    }

    pub fn force_withdraw_cooldown<'a>(
        ctx: Context<'_, '_, 'a, 'a, ForceWithdrawCooldown<'a>>,
        args: ForceWithdrawCooldownArgs,
    ) -> Result<()> {
        instructions::force_withdraw_cooldown(ctx, args)
    }

    pub fn drain_pool_reserves<'a>(
        ctx: Context<'_, '_, 'a, 'a, DrainPoolReserves<'a>>,
        args: DrainPoolReservesArgs,
    ) -> Result<()> {
        instructions::drain_pool_reserves(ctx, args)
    }
}