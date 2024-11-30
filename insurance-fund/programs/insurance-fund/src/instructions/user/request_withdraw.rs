use anchor_lang::prelude::*;
use crate::constants::*;
use crate::events::WithdrawEvent;
use crate::states::*;
use crate::errors::*;
use anchor_spl::token::{
    Mint,
    TokenAccount,
    Token,
    transfer,
    Transfer,
};

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub struct RequestWithdrawalArgs {
    pub lockup_id: u64,
    pub deposit_id: u64,
    pub reward_boost_id: Option<u64>,
    pub amount: u64,
}

pub fn request_withdrawal(
    ctx: Context<RequestWithdrawal>,
    args: RequestWithdrawalArgs
) -> Result<()> {

    let RequestWithdrawalArgs {
        amount,
        deposit_id,
        lockup_id,
        reward_boost_id: __
    } = args;

    let settings = &ctx.accounts.settings;
    let token_program = &ctx.accounts.token_program;
    let deposit = &mut ctx.accounts.deposit;
    let user = &mut ctx.accounts.user;
    let lockup = &mut ctx.accounts.lockup;
    let lockup_asset_vault = &ctx.accounts.lockup_asset_vault;
    let asset_mint = &ctx.accounts.asset_mint;
    let user_asset_ata = &ctx.accounts.user_asset_ata;
    let user_reward_ata = &ctx.accounts.user_reward_ata;
    let asset = &mut ctx.accounts.asset;
    let asset_reward_pool = &ctx.accounts.asset_reward_pool;
    let cooldown = &mut ctx.accounts.cooldown;

    let total_rewards = asset_reward_pool.amount;

    let total_lockup = lockup.total_deposits;
    let user_lockup = deposit.amount;
    let user_share = user_lockup as f64 / total_lockup as f64;
    let user_rewards = (total_rewards as f64 * user_share) as u64;

    // let seeds = &[
    //     LOCKUP_SEED.as_bytes(),
    //     &lockup_id.to_le_bytes(),
    //     &[ctx.bumps.lockup]
    // ];

    // // Transfer rewards
    // transfer(
    //     CpiContext::new_with_signer(
    //         token_program.to_account_info(), 
    //         Transfer {
    //             authority: lockup.to_account_info(),
    //             from: asset_reward_pool.to_account_info(),
    //             to: user_reward_ata.to_account_info(),
    //         }, 
    //         &[seeds]
    //     ), 
    //     user_rewards
    // )?;

    // // Transfer base amount
    // transfer(
    //     CpiContext::new_with_signer(
    //         token_program.to_account_info(), 
    //         Transfer {
    //             from: lockup_asset_vault.to_account_info(),
    //             to: user_asset_ata.to_account_info(),
    //             authority: lockup.to_account_info()
    //         }, 
    //         &[seeds]
    //     ), 
    //     amount
    // )?;

    cooldown.base_amount = amount;
    cooldown.deposit_id = deposit_id;

    cooldown.rewards = match lockup.yield_mode {
        YieldMode::Single => {
            CooldownRewards::Single(user_rewards)
        },
        YieldMode::Dual(rate) => {
            CooldownRewards::Dual([user_rewards, 0])
        }
    };

    cooldown.lock(settings.cooldown_duration)?;

    deposit.amount -= amount;
    asset.decrease_tvl(amount)?;
    lockup.decrease_deposits(amount)?;

    emit!(WithdrawEvent {
        asset: asset_mint.key(),
        from: user.key(),
        base_amount: amount,
        reward_amount: user_rewards
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: RequestWithdrawalArgs
)]
pub struct RequestWithdrawal<'info> {
    #[account(
        mut,
    )]
    pub user: Signer<'info>,

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
        bump,
        constraint = !lockup.locked @ InsuranceFundError::DepositsLocked,
    )]
    pub lockup: Account<'info, Lockup>,

    #[account(
        mut,
        seeds = [
            DEPOSIT_SEED.as_bytes(),
            lockup.key().as_ref(),
            &args.deposit_id.to_le_bytes()
        ],
        bump,
        // Cannot withdraw before unlock timestamp
        constraint = deposit.unlock_ts <= (clock.unix_timestamp as u64) @ InsuranceFundError::LockupInForce,
        // Cannot withdraw more than in deposit
        constraint = deposit.amount >= args.amount @ InsuranceFundError::NotEnoughFunds,
        // Enforce account ownership
        has_one = user
    )]
    pub deposit: Account<'info, Deposit>,

    #[account(
        // If exists, simply means that we're overwriting. 
        // Invoking this IX will always cause the cooldown to be +1 epoch.
        init_if_needed,
        seeds = [
            COOLDOWN_SEED.as_bytes(),
            &deposit.key().to_bytes(),
        ],
        bump,
        payer = user,
        space = Cooldown::SIZE,
    )]
    pub cooldown: Account<'info, Cooldown>,

    #[account(
        mut,
        seeds = [
            REWARD_BOOST_SEED.as_bytes(),
            &lockup.key().to_bytes(),
            // Unwrap will panic if reward_boost account is present, but reward_boost_id argument is not.
            &args.reward_boost_id.unwrap().to_le_bytes()
        ],
        bump,

        constraint = reward_boost
            .validate(deposit.initial_usd_value)
            .is_ok() @ InsuranceFundError::BoostNotApplied,

        constraint = reward_boost.lockup == args.lockup_id
    )]
    pub reward_boost: Option<Account<'info, RewardBoost>>,

    #[account(
        mut,
        seeds = [
            ASSET_SEED.as_bytes(),
            &asset_mint.key().to_bytes()
        ],
        bump,
        constraint = asset.mint == asset_mint.key()
    )]
    pub asset: Account<'info, Asset>,

    #[account(
        mut,
        address = lockup.asset,
    )]
    pub asset_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        address = settings.reward_config.main
    )]
    pub reward_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = asset_mint
    )]
    pub user_asset_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = reward_mint
    )]
    pub user_reward_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [
            VAULT_SEED.as_bytes(),
            lockup.key().as_ref(),
            asset_mint.key().as_ref(),
        ],
        bump,
        // This should never happen
        constraint = lockup_asset_vault.amount >= args.amount @ InsuranceFundError::NotEnoughFunds,
        token::mint = asset_mint,
        token::authority = lockup
    )]
    pub lockup_asset_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [
            REWARD_POOL_SEED.as_bytes(),
            lockup.key().as_ref(),
            reward_mint.key().as_ref(),
        ],
        bump,
        token::mint = reward_mint,
        token::authority = lockup,
    )]
    pub asset_reward_pool: Box<Account<'info, TokenAccount>>,

    #[account()]
    pub clock: Sysvar<'info, Clock>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub system_program: Program<'info, System>,
}