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
pub struct WithdrawArgs {
    pub lockup_id: u64,
    pub deposit_id: u64,
    pub reward_boost_id: Option<u64>,
    pub amount: u64,
}

pub fn withdraw(
    ctx: Context<Withdraw>,
    args: WithdrawArgs
) -> Result<()> {

    let WithdrawArgs {
        amount,
        deposit_id: _,
        lockup_id,
        reward_boost_id: __
    } = args;

    let token_program = &ctx.accounts.token_program;
    let deposit = &mut ctx.accounts.deposit;
    let user = &mut ctx.accounts.user;
    let lockup = &ctx.accounts.lockup;
    let lockup_asset_vault = &ctx.accounts.lockup_asset_vault;
    let asset_mint = &ctx.accounts.asset_mint;
    let user_asset_ata = &ctx.accounts.user_asset_ata;

    let asset_reward_pool = &ctx.accounts.asset_reward_pool;

    let total_rewards = asset_reward_pool.amount;
    let total_lockup = lockup_asset_vault.amount;

    let user_lockup = deposit.amount;
    let user_rewards = total_rewards * user_lockup / total_lockup;

    msg!(
        "deposit share: {:?}",
        (user_lockup as f64) / (total_lockup as f64)
    );

    msg!(
        "rewards share: {:?}",
        (user_rewards as f64) / (total_rewards as f64)
    );

    let seeds = &[
        LOCKUP_SEED.as_bytes(),
        &lockup_id.to_le_bytes(),
        &[ctx.bumps.lockup]
    ];

    // Transfer rewards
    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(), 
            Transfer {
                authority: lockup.to_account_info(),
                from: asset_reward_pool.to_account_info(),
                to: user_asset_ata.to_account_info(),
            }, 
            &[seeds]
        ), 
        user_rewards
    )?;

    // Transfer base amount
    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(), 
            Transfer {
                from: lockup_asset_vault.to_account_info(),
                to: user_asset_ata.to_account_info(),
                authority: lockup.to_account_info()
            }, 
            &[seeds]
        ), 
        amount
    )?;

    deposit.amount -= amount;

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
    args: WithdrawArgs
)]
pub struct Withdraw<'info> {
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
    )]
    pub deposit: Account<'info, Deposit>,

    #[account(
        mut,
        seeds = [
            REWARD_BOOST_SEED.as_bytes(),
            &lockup.key().to_bytes(),
            // Unwrap will panic if reward_boost account is present, but reward_boost_id argument is not.
            &args.reward_boost_id.unwrap().to_le_bytes()
        ],
        bump,
        constraint = reward_boost.min_usd_value <= deposit.initial_usd_value @ InsuranceFundError::BoostNotApplied
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
    pub asset_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = asset_mint
    )]
    pub user_asset_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            VAULT_SEED.as_bytes(),
            lockup.key().as_ref(),
            asset_mint.key().as_ref(),
        ],
        bump,
        constraint = lockup_asset_vault.amount >= args.amount @ InsuranceFundError::NotEnoughFunds,
        token::mint = asset_mint,
        token::authority = lockup
    )]
    pub lockup_asset_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            REWARD_POOL_SEED.as_bytes(),
            lockup.key().as_ref(),
            asset_mint.key().as_ref(),
        ],
        bump,
        token::mint = asset_mint,
        token::authority = lockup,
    )]
    pub asset_reward_pool: Account<'info, TokenAccount>,

    #[account()]
    pub clock: Sysvar<'info, Clock>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub system_program: Program<'info, System>,
}