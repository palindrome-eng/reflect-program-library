use anchor_lang::prelude::*;
use crate::errors::InsuranceFundError;
use crate::events::RestakeEvent;
use crate::states::*;
use crate::constants::*;
use anchor_spl::token::{
    Mint,
    Token,
    TokenAccount,
    transfer,
    Transfer,
    mint_to,
    MintTo
};

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct RestakeArgs {
    pub lockup_id: u64,
    pub amount: u64,
}

pub fn restake(
    ctx: Context<Restake>,
    args: RestakeArgs
) -> anchor_lang::prelude::Result<()> {
    let RestakeArgs {
        amount,
        lockup_id: _
    } = args;

    let token_program = &ctx.accounts.token_program;
    let user = &ctx.accounts.user;
    let settings = &mut ctx.accounts.settings;
    let asset = &mut ctx.accounts.asset;
    let asset_mint = &ctx.accounts.asset_mint;
    let user_asset_ata = &ctx.accounts.user_asset_ata;
    let lockup = &mut ctx.accounts.lockup;
    let deposit = &mut ctx.accounts.deposit;
    let oracle = &ctx.accounts.oracle;
    let lockup_cold_vault = &ctx.accounts.lockup_cold_vault;
    let lockup_hot_vault = &ctx.accounts.lockup_hot_vault;
    let receipt_token_mint = &ctx.accounts.receipt_token_mint;
    let deposit_receipt_token_account = &ctx.accounts.deposit_receipt_token_account;

    let clock = Clock::get()?;
    let unix_ts = clock.unix_timestamp as u64;

    let total_deposits = lockup_cold_vault.amount
        .checked_add(lockup_hot_vault.amount)
        .ok_or(InsuranceFundError::MathOverflow)?;

    match lockup.deposit_cap {
        Some(cap) => {
            require!(
                total_deposits + amount <= cap,
                InsuranceFundError::DepositCapOverflow
            );
        },
        None => {}
    }

    let total_receipts = receipt_token_mint.supply;

    let total_receipts_to_mint: u64 = amount
        .checked_mul(total_deposits)
        .ok_or(InsuranceFundError::MathOverflow)?
        .checked_div(total_receipts)
        .ok_or(InsuranceFundError::MathOverflow)?;

    let seeds = &[
        LOCKUP_SEED.as_bytes(),
        &args.lockup_id.to_le_bytes(),
        &[lockup.bump]
    ];

    // Mint receipts to deposit-owned token account.
    mint_to(
        CpiContext::new_with_signer(
            token_program.to_account_info(), 
            MintTo {
                to: deposit_receipt_token_account.to_account_info(),
                mint: receipt_token_mint.to_account_info(),
                authority: lockup.to_account_info()
            }, 
            &[seeds]
        ), 
        total_receipts_to_mint
    )?;

    deposit.lockup = lockup.key();
    deposit.user = user.key();
    deposit.unlock_ts = unix_ts + lockup.duration;
    deposit.last_slashed = None;
    deposit.amount_slashed = 0;

    let receipt_exchange_rate_bps = lockup.receipt_to_reward_exchange_rate_bps_accumulator;
    deposit.initial_receipt_exchange_rate_bps = receipt_exchange_rate_bps;
    
    let oracle_price = asset
        .get_price(oracle, &clock)?;

    // Multiply oracle price by amount of tokens, 
    // then divide by token decimals to get value of 1 full token instead of `lamports`.
    let usd_value_with_decimals = oracle_price
        .mul(amount)?;

    let usd_value: u64 = usd_value_with_decimals
        .checked_div(
            10_u128
            .checked_pow(asset_mint.decimals as u32)
            .ok_or(InsuranceFundError::MathOverflow)?
        )
        .ok_or(InsuranceFundError::MathOverflow)?
        .try_into()
        .map_err(|_| InsuranceFundError::MathOverflow)?;

    deposit.initial_usd_value = usd_value;

    let hot_wallet_deposit = settings
        .calculate_hot_wallet_deposit(amount)?;

    // Transfer to cold wallet
    transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: user_asset_ata.to_account_info(),
                authority: user.to_account_info(),
                to: lockup_cold_vault.to_account_info()
            }
        ),
        amount - hot_wallet_deposit
    )?;

    // Transfer to hot wallet
    transfer(
        CpiContext::new(
            token_program.to_account_info(), 
            Transfer {
                from: user_asset_ata.to_account_info(),
                to: lockup_hot_vault.to_account_info(),
                authority: user.to_account_info()
            }
        ), 
        hot_wallet_deposit
    )?;

    lockup.deposits += 1;

    asset.increase_tvl(amount)?;
    asset.add_deposit()?;

    emit!(RestakeEvent {
        from: user.key(),
        asset: asset.key(),
        amount,
        lockup_ts: deposit.unlock_ts
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: RestakeArgs
)]
pub struct Restake<'info> {
    #[account(
        mut
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
        constraint = lockup.min_deposit <= args.amount @ InsuranceFundError::DepositTooLow,
    )]
    pub lockup: Account<'info, Lockup>,

    #[account(
        mut,
        address = lockup.receipt_mint
    )]
    pub receipt_token_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = user,
        seeds = [
            DEPOSIT_SEED.as_bytes(),
            lockup.key().as_ref(),
            &lockup.deposits.to_le_bytes()
        ],
        bump,
        space = Deposit::LEN
    )]
    pub deposit: Account<'info, Deposit>,

    #[account(
        init,
        payer = user,
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
    pub asset_mint: Account<'info, Mint>,

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
        associated_token::authority = user,
        associated_token::mint = asset_mint,
        constraint = user_asset_ata.amount >= args.amount @ InsuranceFundError::NotEnoughFunds,
    )]
    pub user_asset_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = settings.reward_config.main
    )]
    pub reward_mint: Account<'info, Mint>,

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
    pub reward_pool: Box<Account<'info, TokenAccount>>,

    /// CHECK: Directly checking the address + checking type later.
    #[account(
        mut,
        address = asset.oracle.key().clone()
    )]
    pub oracle: AccountInfo<'info>,

    #[account()]
    pub clock: Sysvar<'info, Clock>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub system_program: Program<'info, System>,
}