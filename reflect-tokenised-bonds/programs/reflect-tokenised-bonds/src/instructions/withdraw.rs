use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer, Burn};
use crate::state::{vault::Vault, withdraw_request::WithdrawRequest};

#[derive(Accounts)]
#[instruction(receipt_amount: u64)]
pub struct RequestWithdraw<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    #[account(init, payer = requester, space = WithdrawRequest::LEN)]
    pub withdraw_request: Account<'info, WithdrawRequest>,
    #[account(mut)]
    pub requester: Signer<'info>,
    #[account(
        mut,
        constraint = receipt_token_account.mint == vault.receipt_token_mint,
        constraint = receipt_token_account.owner == requester.key(),
    )]
    pub receipt_token_account: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct CompleteWithdraw<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    #[account(mut, has_one = user)]
    pub withdraw_request: Account<'info, WithdrawRequest>,
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(
        mut,
        constraint = receipt_token_account.mint == vault.receipt_token_mint,
        constraint = receipt_token_account.owner == user.key(),
    )]
    pub receipt_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = user_deposit_token_account.mint == vault.deposit_token_mint,
        constraint = user_deposit_token_account.owner == user.key(),
    )]
    pub user_deposit_token_account: Account<'info, TokenAccount>,
    #[account(mut, address = vault.deposit_pool)]
    pub deposit_pool: Account<'info, TokenAccount>,
    #[account(mut, address = vault.reward_pool)]
    pub reward_pool: Account<'info, TokenAccount>,
    pub token_program: Program<'info, Token>,
}

pub fn request_withdraw(ctx: Context<RequestWithdraw>, receipt_amount: u64) -> Result<()> {
    let withdraw_request = &mut ctx.accounts.withdraw_request;
    withdraw_request.user = *ctx.accounts.requester.key;
    withdraw_request.vault = *ctx.accounts.vault.to_account_info().key;
    withdraw_request.receipt_amount = receipt_amount;
    withdraw_request.unlock_date = Clock::get()?.unix_timestamp + ctx.accounts.vault.min_lockup;

    // Transfer receipt tokens to the withdraw request account
    let cpi_accounts = Transfer {
        from: ctx.accounts.receipt_token_account.to_account_info(),
        to: ctx.accounts.withdraw_request.to_account_info(),
        authority: ctx.accounts.requester.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_ctx, receipt_amount)?;

    Ok(())
}

pub fn complete_withdraw(ctx: Context<CompleteWithdraw>) -> Result<()> {
    let withdraw_request = &ctx.accounts.withdraw_request;
    let vault = &ctx.accounts.vault;

    // Check if the lockup period has expired
    require!(
        Clock::get()?.unix_timestamp >= withdraw_request.unlock_date,
        crate::errors::CustomError::LockupNotExpired
    );

    // Calculate the user's share of the total supply
    let total_supply = ctx.accounts.deposit_pool.amount + ctx.accounts.reward_pool.amount;
    let user_share = withdraw_request.receipt_amount as u128 * total_supply as u128 / vault.total_receipt_supply as u128;
    let user_share = user_share as u64;

    // Calculate the amount to withdraw from each pool
    let deposit_amount = (user_share as u128 * ctx.accounts.deposit_pool.amount as u128 / total_supply as u128) as u64;
    let reward_amount = (user_share as u128 * ctx.accounts.reward_pool.amount as u128 / total_supply as u128) as u64;

    // Burn receipt tokens from the withdraw request account
    let cpi_accounts = Burn {
        mint: ctx.accounts.receipt_token_account.to_account_info(),
        from: ctx.accounts.withdraw_request.to_account_info(),
        authority: ctx.accounts.user.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    token::burn(cpi_ctx, withdraw_request.receipt_amount)?;

    // Transfer deposit tokens to the user
    if deposit_amount > 0 {
        let cpi_accounts = Transfer {
            from: ctx.accounts.deposit_pool.to_account_info(),
            to: ctx.accounts.user_deposit_token_account.to_account_info(),
            authority: ctx.accounts.vault.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, deposit_amount)?;
    }

    // Transfer reward tokens to the user
    if reward_amount > 0 {
        let cpi_accounts = Transfer {
            from: ctx.accounts.reward_pool.to_account_info(),
            to: ctx.accounts.user_deposit_token_account.to_account_info(),
            authority: ctx.accounts.vault.to_account_info(),
        };
        let cpi_program = ctx.accounts.token_program.to_account_info();
        let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
        token::transfer(cpi_ctx, reward_amount)?;
    }

    Ok(())
}
