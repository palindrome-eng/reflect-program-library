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

    pub fn add_admin(
        ctx: Context<AddAdmin>,
        args: AddAdminArgs
    ) -> Result<()> {
        instructions::add_admin(ctx, args)
    }

    pub fn remove_admin(
        ctx: Context<RemoveAdmin>,
        args: RemoveAdminArgs
    ) -> Result<()> {
        instructions::remove_admin(ctx, args)
    }

    pub fn add_asset(
        ctx: Context<AddAsset>,
    ) -> Result<()> {
        instructions::add_asset(ctx)
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

    pub fn initialize_lockup_vaults(
        ctx: Context<InitializeLockupVaults>,
        lockup_id: u64
    ) -> Result<()> {
        instructions::initialize_lockup_vaults(ctx, lockup_id)
    }

    pub fn update_deposit_cap(
        ctx: Context<UpdateDepositCap>,
        args: UpdateDepositCapArgs
    ) -> Result<()> {
        instructions::update_deposit_cap(ctx, args)
    }

    pub fn rebalance(
        ctx: Context<Rebalance>,
        args: RebalanceArgs
    ) -> Result<()> {
        instructions::rebalance(ctx, args)
    }

    pub fn boost_rewards(
        ctx: Context<BoostRewards>,
        args: BoostRewardsArgs
    ) -> Result<()> {
        instructions::boost_rewards(ctx, args)
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

    pub fn slash(
        ctx: Context<Slash>,
        args: SlashArgs
    ) -> Result<()> {
        instructions::slash(ctx, args)
    }

    pub fn request_withdrawal(
        ctx: Context<RequestWithdrawal>,
        args: RequestWithdrawalArgs
    ) -> Result<()> {
        instructions::request_withdrawal(ctx, args)
    }

    pub fn withdraw(
        ctx: Context<Withdraw>,
        args: WithdrawArgs
    ) -> Result<()> {
        instructions::withdraw(ctx, args)
    }

    pub fn deposit_rewards(
        ctx: Context<DepositRewards>,
        args: DepositRewardsArgs
    ) -> Result<()> {
        instructions::deposit_rewards(ctx, args)
    }

    pub fn get_user_balance_and_reward(
        ctx: Context<GetUserBalanceAndReward>,
        args: GetUserBalanceAndRewardArgs
    ) -> Result<(u64, u64)> {
        instructions::get_user_balance_and_reward(ctx, args)
    }

    pub fn swap(
        ctx: Context<Swap>,
        args: SwapArgs
    ) -> Result<()> {
        instructions::swap(ctx, args)
    }

    pub fn borrow(
        ctx: Context<Borrow>,
        args: BorrowArgs
    ) -> Result<()> {
        instructions::borrow(ctx, args)
    }

    pub fn repay(
        ctx: Context<Repay>,
        args: RepayArgs
    ) -> Result<()> {
        instructions::repay(ctx, args)
    }
}