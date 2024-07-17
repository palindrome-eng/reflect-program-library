use anchor_lang::prelude::*;
use anchor_spl::token::{
    Transfer,
    transfer,
    TokenAccount,
    Token
};
use crate::state::*;
use crate::constants::{
    USER_ACCOUNT_SEED,
    LOCKUP_SEED
};

pub fn lockup(
    ctx: Context<Lockup>, 
    receipt_amount: u64
) -> Result<()> {
    let lockup = &mut ctx.accounts.lockup;
    let lockup_receipt_token_account = &ctx.accounts.lockup_receipt_token_account;
    let user = &ctx.accounts.user;
    let user_receipt_token_account = &ctx.accounts.user_receipt_token_account;
    let vault = &ctx.accounts.vault;
    let token_program = &ctx.accounts.token_program;

    lockup.user = *user.key;
    lockup.vault = *vault.to_account_info().key;
    lockup.receipt_amount = receipt_amount;
    lockup.unlock_date = Clock::get()?.unix_timestamp + vault.min_lockup;

    transfer(
        CpiContext::new(
            token_program.to_account_info(), 
            Transfer {
                from: user_receipt_token_account.to_account_info(),
                to: lockup_receipt_token_account.to_account_info(),
                authority: user.to_account_info(),
            }
        ), 
        receipt_amount
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(receipt_amount: u64)]
pub struct Lockup<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        init_if_needed,
        seeds = [
            USER_ACCOUNT_SEED.as_bytes(),
            user.key().as_ref()
        ],
        space = 8 + 8,
        bump,
        payer = user,
    )]
    pub user_account: Account<'info, UserAccount>,

    #[account(mut)]
    pub vault: Account<'info, Vault>,

    #[account(
        init, 
        payer = user,
        space = LockupState::LEN,
        seeds = [
            LOCKUP_SEED.as_bytes(),
            &user_account.lockups.to_le_bytes()
        ],
        bump,
    )]
    pub lockup: Account<'info, LockupState>,

    #[account(
        mut,
        constraint = user_receipt_token_account.mint == vault.receipt_token_mint,
        constraint = user_receipt_token_account.owner == user.key(),
    )]
    pub user_receipt_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = lockup_receipt_token_account.mint == vault.receipt_token_mint,
        constraint = lockup_receipt_token_account.owner == lockup.key(),
    )]
    pub lockup_receipt_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,

    pub clock: Sysvar<'info, Clock>
}