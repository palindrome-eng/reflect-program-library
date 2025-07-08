use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token};
use crate::states::{lp_lockup, Settings, LpLockup};
use crate::constants::*;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct LockLpTokensArgs {
    pub amount: u64,
    pub duration_seconds: u64,
}

pub fn lock_lp_tokens(
    ctx: Context<LockLpTokens>,
    args: LockLpTokensArgs
) -> Result<()> {
    let LockLpTokensArgs { amount, duration_seconds } = args;
    let signer = &ctx.accounts.signer;
    let lp_lockup = &mut ctx.accounts.lp_lockup;
    let user_lp_account = &ctx.accounts.user_lp_account;
    let lockup_lp_vault = &ctx.accounts.lockup_lp_vault;
    let token_program = &ctx.accounts.token_program;

    require!(
        amount > 0,
        crate::errors::InsuranceFundError::InvalidInput
    );

    require!(
        duration_seconds > 0,
        crate::errors::InsuranceFundError::InvalidInput
    );

    // Lock LP tokens in the lockup vault
    lp_lockup.lock_lp_tokens(
        amount,
        user_lp_account,
        lockup_lp_vault,
        token_program,
        signer
    )?;

    // Update deposit count
    lp_lockup.deposits = lp_lockup.deposits
        .checked_add(1)
        .ok_or(crate::errors::InsuranceFundError::MathOverflow)?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(args: LockLpTokensArgs)]
pub struct LockLpTokens<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [
            SETTINGS_SEED.as_bytes(),
        ],
        bump,
    )]
    pub settings: Account<'info, Settings>,

    #[account(
        mut,
        seeds = [
            LIQUIDITY_POOL_LOCKUP_SEED.as_bytes(),
            &args.duration_seconds.to_le_bytes(),
        ],
        bump,
    )]
    pub lp_lockup: Account<'info, LpLockup>,

    #[account(
        mut,
        associated_token::mint = lp_token,
        associated_token::authority = signer,
    )]
    pub user_lp_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = lp_token,
        associated_token::authority = lp_lockup,
    )]
    pub lockup_lp_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        address = settings.lp_token
    )]
    pub lp_token: Box<Account<'info, Mint>>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub system_program: Program<'info, System>,
} 