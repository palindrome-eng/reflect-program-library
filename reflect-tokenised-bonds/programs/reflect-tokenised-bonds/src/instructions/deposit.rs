use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount, Transfer, MintTo};
use crate::state::vault::Vault;

#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub vault: Account<'info, Vault>,
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account(
        mut,
        constraint = deposit_token_account.mint == vault.deposit_token_mint,
        constraint = deposit_token_account.owner == depositor.key(),
    )]
    pub deposit_token_account: Account<'info, TokenAccount>,
    #[account(
        mut,
        constraint = receipt_token_account.mint == vault.receipt_token_mint,
        constraint = receipt_token_account.owner == depositor.key(),
    )]
    pub receipt_token_account: Account<'info, TokenAccount>,
    #[account(mut, address = vault.deposit_pool)]
    pub deposit_pool: Account<'info, TokenAccount>,
    #[account(mut)]
    pub receipt_token_mint: Account<'info, Mint>,
    pub token_program: Program<'info, Token>,
}

pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
    require!(amount >= ctx.accounts.vault.min_deposit, crate::errors::CustomError::InsufficientDeposit);

    // Calculate receipt token amount
    let receipt_amount = if ctx.accounts.deposit_pool.amount > 0 {
        amount * ctx.accounts.deposit_pool.amount / (ctx.accounts.deposit_pool.amount + ctx.accounts.reward_pool.amount)
    } else {
        amount
    };

    {
        let vault = &mut ctx.accounts.vault; // Make vault mutable
        vault.total_receipt_supply = vault.total_receipt_supply.checked_add(receipt_amount).ok_or(ProgramError::InvalidInstructionData)?;
    }

    // Transfer deposit tokens to deposit pool
    let cpi_accounts = Transfer {
        from: ctx.accounts.deposit_token_account.to_account_info(),
        to: ctx.accounts.deposit_pool.to_account_info(),
        authority: ctx.accounts.depositor.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    anchor_spl::token::transfer(cpi_ctx, amount)?;

    // Mint receipt tokens
    let cpi_accounts = MintTo {
        mint: ctx.accounts.receipt_token_mint.to_account_info(),
        to: ctx.accounts.receipt_token_account.to_account_info(),
        authority: ctx.accounts.vault.to_account_info(),
    };
    let cpi_program = ctx.accounts.token_program.to_account_info();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);
    anchor_spl::token::mint_to(cpi_ctx, receipt_amount)?;

    Ok(())
}
