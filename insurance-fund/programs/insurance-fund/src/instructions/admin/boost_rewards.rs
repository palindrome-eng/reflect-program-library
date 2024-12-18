use anchor_lang::prelude::*;
use crate::constants::*;
use crate::states::*;
use crate::errors::InsuranceFundError;

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct BoostRewardsArgs {
    pub lockup_id: u64,
    pub min_usd_value: u64,
    pub boost_bps: u64,
}

pub fn boost_rewards(
    ctx: Context<BoostRewards>,
    args: BoostRewardsArgs
) -> Result<()> {

    let reward_boost = &mut ctx.accounts.reward_boost;
    let BoostRewardsArgs {
        boost_bps,
        lockup_id,
        min_usd_value
    } = args;

    reward_boost.boost_bps = boost_bps;
    reward_boost.min_usd_value = min_usd_value;
    reward_boost.lockup = lockup_id;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: BoostRewardsArgs
)]
pub struct BoostRewards<'info> {
    #[account(
        mut
    )]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            ADMIN_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump,
        constraint = admin.has_permissions(Permissions::AssetsAndLockups)
    )]
    pub admin: Account<'info, Admin>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        constraint = !settings.frozen @ InsuranceFundError::Frozen
    )]
    pub settings: Account<'info, Settings>,

    #[account(
        mut,
        seeds = [
            LOCKUP_SEED.as_bytes(),
            &args.lockup_id.to_le_bytes()
        ],
        bump
    )]
    pub lockup: Account<'info, Lockup>,

    #[account(
        init,
        payer = signer,
        seeds = [
            REWARD_BOOST_SEED.as_bytes(),
            &lockup.key().to_bytes(),
            &lockup.reward_boosts.to_le_bytes()
        ],
        bump,
        space = RewardBoost::SIZE
    )]
    pub reward_boost: Account<'info, RewardBoost>,

    #[account()]
    pub system_program: Program<'info, System>,
}