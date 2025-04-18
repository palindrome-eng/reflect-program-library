use anchor_lang::prelude::*;
use crate::errors::InsuranceFundError;
use crate::states::*;
use crate::constants::*;
use anchor_spl::token::{
    Token,
    TokenAccount,
    Mint
};

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub struct GetUserBalanceAndRewardArgs {
    pub lockup_id: u64,
    pub deposit_id: u64,
    pub reward_boost_id: Option<u64>,
    pub receipt_tokens: u64
}

pub fn get_user_balance_and_reward(
    ctx: Context<GetUserBalanceAndReward>,
    args: GetUserBalanceAndRewardArgs
) -> Result<(u64, u64)> {
    let GetUserBalanceAndRewardArgs {
        deposit_id,
        lockup_id,
        receipt_tokens,
        reward_boost_id
    } = args;

    let settings = &ctx.accounts.settings;
    let lockup = &ctx.accounts.lockup;
    let asset_reward_pool = &ctx.accounts.asset_reward_pool;
    let deposit = &ctx.accounts.deposit;
    let cooldown = &ctx.accounts.cooldown;
    let lockup_cooldown_vault = &ctx.accounts.lockup_cooldown_vault;
    let lockup_hot_vault = &ctx.accounts.lockup_hot_vault;
    let lockup_cold_vault = &ctx.accounts.lockup_cold_vault;
    let receipt_token_mint = &ctx.accounts.receipt_token_mint;
    let token_program = &ctx.accounts.token_program;
    let reward_boost = &ctx.accounts.reward_boost;

    let receipt_amount = cooldown.receipt_amount;
    let total_receipts = receipt_token_mint.supply;
    let total_deposits = lockup_hot_vault.amount
        .checked_add(lockup_cold_vault.amount)
        .ok_or(InsuranceFundError::MathOverflow)?;

    let receipt_to_deposit_exchange_rate_bps = total_deposits
        .checked_mul(10_000)
        .ok_or(InsuranceFundError::MathOverflow)?
        .checked_div(total_receipts)
        .ok_or(InsuranceFundError::MathOverflow)?;

    let deposit_to_return = receipt_amount
        .checked_mul(receipt_to_deposit_exchange_rate_bps)
        .ok_or(InsuranceFundError::MathOverflow)?
        .checked_div(10_000)
        .ok_or(InsuranceFundError::MathOverflow)?;

    let total_receipts = receipt_token_mint.supply;
    let total_deposits = lockup_cold_vault.amount
        .checked_add(lockup_hot_vault.amount)
        .ok_or(InsuranceFundError::MathOverflow)?;

    let receipt_to_deposit_exchange_rate_bps = total_deposits
        .checked_mul(10_000)
        .ok_or(InsuranceFundError::MathOverflow)?
        .checked_div(total_receipts)
        .ok_or(InsuranceFundError::MathOverflow)?;
    
    let base_amount = receipt_tokens
        .checked_mul(receipt_to_deposit_exchange_rate_bps)
        .ok_or(InsuranceFundError::MathOverflow)?
        .checked_div(10_000)
        .ok_or(InsuranceFundError::MathOverflow)?;

    match reward_boost {
        Some(account) => {
            require!(
                account
                    .validate(deposit.initial_usd_value)
                    .is_ok(),
                InsuranceFundError::BoostNotApplied
            );

            require!(
                account.lockup == lockup_id,
                InsuranceFundError::BoostNotApplied
            );
        },
        None => {}
    }

    // Take reward boost into calculation
    let rewards = deposit.compute_accrued_rewards(
        lockup.receipt_to_reward_exchange_rate_bps_accumulator, 
        receipt_amount
    )?;

    Ok((base_amount, rewards))
}

#[derive(Accounts)]
#[instruction(
    args: GetUserBalanceAndRewardArgs
)]
pub struct GetUserBalanceAndReward<'info> {
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
        bump
    )]
    pub lockup: Account<'info, Lockup>,

    #[account(
        mut,
        address = lockup.receipt_mint
    )]
    pub receipt_token_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            DEPOSIT_SEED.as_bytes(),
            lockup.key().as_ref(),
            &args.deposit_id.to_le_bytes()
        ],
        bump,
        // Enforce account ownership
        has_one = user
    )]
    pub deposit: Account<'info, Deposit>,

    #[account(
        mut,
        seeds = [
            DEPOSIT_RECEIPT_VAULT_SEED.as_bytes(),
            deposit.key().as_ref(),
            receipt_token_mint.key().as_ref(),
        ],
        bump,
        token::mint = receipt_token_mint,
        token::authority = deposit,
    )]
    pub deposit_receipt_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        seeds = [
            COOLDOWN_SEED.as_bytes(),
            &deposit.key().to_bytes(),
        ],
        bump,
        payer = user,
        space = 8 + Cooldown::INIT_SPACE,
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
        bump
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
        address = lockup.asset_mint,
    )]
    pub asset_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        address = settings.reward_config.main
    )]
    pub reward_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [
            HOT_VAULT_SEED.as_bytes(),
            lockup.key().as_ref(),
            asset_mint.key().as_ref(),
        ],
        bump,
    )]
    pub lockup_hot_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [
            COLD_VAULT_SEED.as_bytes(),
            lockup.key().as_ref(),
            asset_mint.key().as_ref(),
        ],
        bump,
    )]
    pub lockup_cold_vault: Box<Account<'info, TokenAccount>>,

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

    #[account(
        mut,
        seeds = [
            COOLDOWN_VAULT_SEED.as_bytes(),
            lockup.key().as_ref(),
            receipt_token_mint.key().as_ref(),
        ],
        bump,
        token::mint = receipt_token_mint,
        token::authority = lockup,
    )]
    pub lockup_cooldown_vault: Box<Account<'info, TokenAccount>>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub system_program: Program<'info, System>,
}