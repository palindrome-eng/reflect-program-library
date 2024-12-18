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
    Transfer
};

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub enum RequestWithdrawalMode {
    ExactIn(u64), // Exact receipt tokens burned
    ExactOut(u64) // Exact base tokens received
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub struct RequestWithdrawalArgs {
    pub lockup_id: u64,
    pub deposit_id: u64,
    pub reward_boost_id: Option<u64>,
    pub mode: RequestWithdrawalMode,
}

pub fn request_withdrawal(
    ctx: Context<RequestWithdrawal>,
    args: RequestWithdrawalArgs
) -> Result<()> {
    let RequestWithdrawalArgs {
        mode,
        deposit_id,
        lockup_id: _,
        reward_boost_id: __
    } = args;

    let settings = &ctx.accounts.settings;
    let deposit = &mut ctx.accounts.deposit;
    let user = &mut ctx.accounts.user;
    let lockup = &mut ctx.accounts.lockup;
    let asset_mint = &ctx.accounts.asset_mint;
    let cooldown = &mut ctx.accounts.cooldown;
    let lockup_cold_vault = &ctx.accounts.lockup_cold_vault;
    let lockup_hot_vault = &ctx.accounts.lockup_hot_vault;
    let receipt_token_mint = &ctx.accounts.receipt_token_mint;
    let deposit_receipt_token_account = &ctx.accounts.deposit_receipt_token_account;
    let lockup_cooldown_vault = &ctx.accounts.lockup_cooldown_vault;
    let token_program = &ctx.accounts.token_program;

    let total_receipts = receipt_token_mint.supply;
    let total_deposits = lockup_cold_vault.amount
        .checked_add(lockup_hot_vault.amount)
        .ok_or(InsuranceFundError::MathOverflow)?;

    let receipt_to_deposit_exchange_rate_bps = total_deposits
        .checked_mul(10_000)
        .ok_or(InsuranceFundError::MathOverflow)?
        .checked_div(total_receipts)
        .ok_or(InsuranceFundError::MathOverflow)?;

    let (
        base_amount, 
        receipt_amount
    ) = match args.mode {
        RequestWithdrawalMode::ExactIn(receipt_tokens) => {
            (
                receipt_tokens
                    .checked_mul(receipt_to_deposit_exchange_rate_bps)
                    .ok_or(InsuranceFundError::MathOverflow)?
                    .checked_div(10_000)
                    .ok_or(InsuranceFundError::MathOverflow)?,
                receipt_tokens
            )
        },
        RequestWithdrawalMode::ExactOut(base_tokens) => {
            (
                base_tokens,
                base_tokens
                    .checked_mul(10_000)
                    .ok_or(InsuranceFundError::MathOverflow)?
                    .checked_div(receipt_to_deposit_exchange_rate_bps)
                    .ok_or(InsuranceFundError::MathOverflow)?
            )
        }
    };

    // Check if user actually owns enough receipts to process this
    require!(
        receipt_amount <= deposit_receipt_token_account.amount,
        InsuranceFundError::NotEnoughReceiptTokens
    );

    // // Here we check if the amount is not bigger than allowed share, no matter if the
    // // cold-hot wallet ratios are currently balanced.
    // let max_allowed_auto_withdrawal = total_deposits
    //     .checked_mul(settings.shares_config.hot_wallet_share_bps)
    //     .ok_or(InsuranceFundError::MathOverflow)?
    //     .checked_div(10_000)
    //     .ok_or(InsuranceFundError::MathOverflow)?;

    // // If above passes, we check if the pools are actually balanced 
    // // well enough to process the withdrawal.
    // require!(
    //     base_amount <= lockup_hot_vault.amount,
    //     InsuranceFundError::PoolImbalance
    // );

    cooldown.receipt_amount = receipt_amount;
    cooldown.deposit_id = deposit_id;
    cooldown.user = user.key();
    cooldown.lockup_id = lockup.index;

    let rewards = deposit.compute_accrued_rewards(
        // This should be read from lockup, since we need the accumulator here
        lockup.receipt_to_reward_exchange_rate_bps_accumulator, 
        receipt_amount
    )?;

    cooldown.rewards = match lockup.yield_mode {
        YieldMode::Single => {
            CooldownRewards::Single(rewards)
        },
        YieldMode::Dual(_) => {
            CooldownRewards::Dual([rewards, 0])
        }
    };

    cooldown.lock(settings.cooldown_duration)?;

    let lockup_key = lockup.key();
    let seeds = &[
        DEPOSIT_SEED.as_bytes(),
        lockup_key.as_ref(),
        &args.deposit_id.to_le_bytes(),
        &[deposit.bump]
    ];

    // Transfer receipts into cooldown vault
    // We need this to be able to still slash (since slashing is based on the total receipt supply)
    // but not influence others rewards (since reward calculation is based on the total receipt supply - cooldown vault balance)
    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(), 
            Transfer {
                from: deposit_receipt_token_account.to_account_info(),
                to: lockup_cooldown_vault.to_account_info(),
                authority: deposit.to_account_info()
            },
            &[seeds]
        ), 
        receipt_amount
    )?;

    emit!(WithdrawEvent {
        asset: asset_mint.key(),
        from: user.key(),
        base_amount: base_amount,
        reward_amount: rewards
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
        // Cannot request withdraw before unlock timestamp
        constraint = deposit.unlock_ts <= (clock.unix_timestamp as u64) @ InsuranceFundError::LockupInForce,
        // Enforce account ownership
        has_one = user
    )]
    pub deposit: Account<'info, Deposit>,

    #[account(
        mut,
        seeds = [
            // This scheme could be less complex, but keep it this
            // way in case of future updates that would possibly allow 
            // multi-asset deposits with multiple receipts.
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
    pub lockup_hot_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            COLD_VAULT_SEED.as_bytes(),
            lockup.key().as_ref(),
            asset_mint.key().as_ref(),
        ],
        bump,
    )]
    pub lockup_cold_vault: Account<'info, TokenAccount>,

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
    pub lockup_cooldown_vault: Account<'info, TokenAccount>,

    #[account()]
    pub clock: Sysvar<'info, Clock>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub system_program: Program<'info, System>,
}

// impl RequestWithdrawal<'_> {
//     pub fn validate_withdrawal_amounts(
//         &self,
//         mode: &RequestWithdrawalMode
//     ) -> Result<()> {
//         match mode {
//             RequestWithdrawalMode::ExactIn(receipt_tokens) => {
//                 require!(
//                     receipt_tokens <= self.deposit_receipt_token_account.amount,
//                     InsuranceFundError::NotEnoughReceiptTokens
//                 );
//             },
//             RequestWithdrawalMode::ExactOut(tokens_out) => {
//                 let total_receipts = self.receipt_token_mint.supply;
//                 let total_deposited_tokens = self.lockup_hot_vault.amount
//                     .checked_add(self.lockup_cold_vault.amount)
//                     .ok_or(InsuranceFundError::MathOverflow)?;


//             }
//         }
//     }
// }