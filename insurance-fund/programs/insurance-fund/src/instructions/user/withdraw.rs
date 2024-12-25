use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use crate::errors::InsuranceFundError;
use crate::states::*;
use crate::constants::*;
use anchor_spl::token::{ 
    Mint, 
    TokenAccount, 
    transfer,
    Transfer,
    burn,
    Burn
 };

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub struct WithdrawArgs {
    pub lockup_id: u64,
    pub deposit_id: u64,
}

// Withdraw should
// 1) Take receipt_amount from the cooldown account
// 2) Recalculate amount of the base_amount that corresponds to the receipt_amount
// 3) Burn receipts
// 4) Transfer base_amount
// 5) Get locked user rewards from the cooldown account
// 6) Transfer rewards

pub fn withdraw(
    ctx: Context<Withdraw>,
    args: WithdrawArgs
) -> Result<()> {

    let WithdrawArgs {
        deposit_id: _,
        lockup_id,
    } = args;

    let settings = &ctx.accounts.settings;
    let lockup = &ctx.accounts.lockup;
    let asset_reward_pool = &ctx.accounts.asset_reward_pool;
    let user_reward_ata = &ctx.accounts.user_reward_ata;
    let deposit = &ctx.accounts.deposit;
    let cooldown = &ctx.accounts.cooldown;
    let user_asset_ata = &ctx.accounts.user_asset_ata;
    let lockup_cooldown_vault = &ctx.accounts.lockup_cooldown_vault;
    let lockup_hot_vault = &ctx.accounts.lockup_hot_vault;
    let lockup_cold_vault = &ctx.accounts.lockup_cold_vault;
    let receipt_token_mint = &ctx.accounts.receipt_token_mint;
    let token_program = &ctx.accounts.token_program;

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

    let lockup_seeds = &[
        LOCKUP_SEED.as_bytes(),
        &lockup_id.to_le_bytes(),
        &[lockup.bump]
    ];

    let max_allowed_auto_withdrawal = total_deposits
        .checked_mul(settings.shares_config.hot_wallet_share_bps)
        .ok_or(InsuranceFundError::MathOverflow)?
        .checked_div(10_000)
        .ok_or(InsuranceFundError::MathOverflow)?;

    let reward = match cooldown.rewards {
        CooldownRewards::Single(base) => base,
        CooldownRewards::Dual([base, _]) => base
    };

    // Transfer user rewards
    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(), 
            Transfer {
                authority: lockup.to_account_info(),
                from: asset_reward_pool.to_account_info(),
                to: user_reward_ata.to_account_info()
            }, 
            &[lockup_seeds]
        ), 
        reward
    )?;

    // Burn user receipts
    burn(
        CpiContext::new_with_signer(
            token_program.to_account_info(), 
            Burn {
                authority: lockup.to_account_info(),
                from: lockup_cooldown_vault.to_account_info(),
                mint: receipt_token_mint.to_account_info()
            }, 
            &[lockup_seeds]
        ), 
        receipt_amount
    )?;

    if deposit_to_return > max_allowed_auto_withdrawal {
        return match &mut ctx.accounts.intent {
            Some(intent) => {
                intent.amount = deposit_to_return;
                intent.deposit = deposit.key();
                intent.lockup = lockup.key();

                Ok(())
            },
            None => {
                Err(InsuranceFundError::WithdrawalNeedsIntent.into())
            }
        };
    }

    // Prevent account from initializing if passed and not needed.
    require!(
        ctx.accounts.intent.is_none(),
        InsuranceFundError::IntentValueTooLow
    );

    // Check if the pools are actually balanced 
    // well enough to process the withdrawal.
    require!(
        deposit_to_return <= lockup_hot_vault.amount,
        InsuranceFundError::PoolImbalance
    );

    // Transfer user base_amount
    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(), 
            Transfer {
                from: lockup_hot_vault.to_account_info(),
                to: user_asset_ata.to_account_info(),
                authority: lockup.to_account_info()
            },
            &[lockup_seeds]
        ), 
        deposit_to_return
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: WithdrawArgs
)]
pub struct Withdraw<'info> {
    #[account(mut)]
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

    // All these checks were run during initializing cooldown
    // not sure if we need to check them again, likely not.
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
        mut,
        seeds = [
            COOLDOWN_SEED.as_bytes(),
            &deposit.key().to_bytes(),
        ],
        bump,
        close = user
    )]
    pub cooldown: Account<'info, Cooldown>,

    #[account(
        mut,
        address = lockup.asset_mint,
    )]
    pub asset_mint: Box<Account<'info, Mint>>,

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
        associated_token::authority = user,
        associated_token::mint = asset_mint
    )]
    pub user_asset_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        address = settings.reward_config.main
    )]
    pub reward_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = reward_mint
    )]
    pub user_reward_ata: Box<Account<'info, TokenAccount>>,

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

    #[account(
        init,
        payer = user,
        seeds = [
            INTENT_SEED.as_bytes(),
            deposit.key().as_ref()
        ],
        bump,
        space = Intent::LEN,
    )]
    pub intent: Option<Account<'info, Intent>>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub clock: Sysvar<'info, Clock>,

    #[account()]
    pub system_program: Program<'info, System>,
}