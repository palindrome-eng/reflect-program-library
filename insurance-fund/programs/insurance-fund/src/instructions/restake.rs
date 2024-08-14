use anchor_lang::prelude::*;
use crate::borsh::*;
use crate::errors::InsuranceFundError;
use crate::states::*;
use crate::constants::*;
use anchor_spl::token::{
    Mint,
    Token,
    TokenAccount,
    transfer,
    Transfer
};

#[derive(BorshDeserialize, BorshSerialize)]
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
    let user_asset_ata = &ctx.accounts.user_asset_ata;
    let lockup_asset_vault = &ctx.accounts.lockup_asset_vault;
    let lockup = &mut ctx.accounts.lockup;
    let deposit = &mut ctx.accounts.deposit;

    let clock = Clock::get()?;
    let unix_ts = clock.unix_timestamp as u64;

    deposit.amount = amount;
    deposit.lockup = lockup.key();
    deposit.user = user.key();
    deposit.unlock_ts = unix_ts + lockup.duration;
    deposit.last_slashed = None;
    deposit.amount_slashed = 0;

    transfer(
        CpiContext::new(
            token_program.to_account_info(), 
            Transfer {
                from: user_asset_ata.to_account_info(),
                to: lockup_asset_vault.to_account_info(),
                authority: user.to_account_info()
            }
        ), 
        amount
    )?;

    lockup.deposits += 1;
    settings.tvl += amount;

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
        constraint = !settings.deposits_locked @ InsuranceFundError::DepositsLocked
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
        mut,
        address = lockup.asset
    )]
    pub asset_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = asset_mint,
        constraint = user_asset_ata.amount >= args.amount @ InsuranceFundError::NotEnoughFunds
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
        constraint = lockup_asset_vault.amount + args.amount <= lockup.deposit_cap @ InsuranceFundError::DepositCapOverflow
    )]
    pub lockup_asset_vault: Account<'info, TokenAccount>,

    #[account()]
    pub clock: Sysvar<'info, Clock>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub system_program: Program<'info, System>,
}