use anchor_lang::prelude::*;
use switchboard_solana::oracle_program;
use crate::errors::InsuranceFundError;
use crate::events::RestakeEvent;
use crate::states::*;
use crate::constants::*;
use anchor_spl::token::{
    Mint,
    Token,
    TokenAccount,
    transfer,
    Transfer
};

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct RestakeArgs {
    pub lockup_id: u64,
    pub amount: u64,
}

pub fn restake(
    ctx: Context<Restake>,
    args: RestakeArgs
) -> Result<()> {
    let RestakeArgs {
        amount,
        lockup_id
    } = args;

    let token_program = &ctx.accounts.token_program;
    let user = &ctx.accounts.user;
    let settings = &mut ctx.accounts.settings;
    let asset = &mut ctx.accounts.asset;
    let asset_mint = &ctx.accounts.asset_mint;
    let user_asset_ata = &ctx.accounts.user_asset_ata;
    let lockup_asset_vault = &ctx.accounts.lockup_asset_vault;
    let cold_wallet_vault = &ctx.accounts.cold_wallet_vault;
    let lockup = &mut ctx.accounts.lockup;
    let deposit = &mut ctx.accounts.deposit;
    let oracle = &ctx.accounts.oracle;

    let clock = Clock::get()?;
    let unix_ts = clock.unix_timestamp as u64;

    deposit.amount = amount;
    deposit.lockup = lockup.key();
    deposit.user = user.key();
    deposit.unlock_ts = unix_ts + lockup.duration;
    deposit.last_slashed = None;
    deposit.amount_slashed = 0;
    
    let oracle_price = asset
        .get_price(oracle)?;

    // Multiply oracle price by amount of tokens, 
    // then divide by token decimals to get value of 1 full token instead of `lamports`.
    let usd_value = oracle_price
        .mul(amount)?
        .checked_div(
            10_u64
            .checked_pow(asset_mint.decimals as u32)
            .ok_or(InsuranceFundError::MathOverflow)?
        )
        .ok_or(InsuranceFundError::MathOverflow)?;

    deposit.initial_usd_value = usd_value;

    let hot_wallet_deposit = settings.calculate_hot_wallet_deposit(amount)?;

    // Transfer to multisig
    transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: user_asset_ata.to_account_info(),
                authority: user.to_account_info(),
                to: cold_wallet_vault.to_account_info()
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
                to: lockup_asset_vault.to_account_info(),
                authority: user.to_account_info()
            }
        ), 
        hot_wallet_deposit
    )?;

    lockup.deposits += 1;
    lockup.increase_deposits(amount)?;

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
        constraint = !lockup.locked @ InsuranceFundError::DepositsLocked,
        constraint = lockup.min_deposit <= args.amount @ InsuranceFundError::DepositTooLow,
    )]
    pub lockup: Account<'info, Lockup>,

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

    // For now it's SystemAccount - should be Squads multisig type?
    #[account(
        mut,
        address = settings.cold_wallet
    )]
    pub cold_wallet: SystemAccount<'info>,

    #[account(
        mut,
        associated_token::mint = asset_mint,
        associated_token::authority = cold_wallet
    )]
    pub cold_wallet_vault: Account<'info, TokenAccount>,

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
        associated_token::mint = asset_mint,
        constraint = user_asset_ata.amount >= args.amount @ InsuranceFundError::NotEnoughFunds,
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
        // Unwrap is safe here since in case of `None` value, only the first statement will be checked
        constraint = lockup.deposit_cap.is_none() || lockup_asset_vault.amount + args.amount <= lockup.deposit_cap.unwrap() @ InsuranceFundError::DepositCapOverflow
    )]
    pub lockup_asset_vault: Account<'info, TokenAccount>,

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