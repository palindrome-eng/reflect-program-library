use anchor_lang::prelude::*;
use anchor_spl::token::{
    Burn,
    Token, 
    TokenAccount, 
    Transfer,
    burn, 
    transfer,
    Mint,
};
use crate::state::*;

pub fn withdraw(ctx: Context<Withdraw>) -> Result<()> {
    let user = &ctx.accounts.user;
    let vault = &ctx.accounts.vault;
    let lockup = &ctx.accounts.lockup;
    let lockup_receipt_token_account = &ctx.accounts.lockup_receipt_token_account;
    let user_deposit_token_account = &ctx.accounts.user_deposit_token_account;
    let deposit_pool = &ctx.accounts.deposit_pool;
    let reward_pool = &ctx.accounts.reward_pool;
    let receipt_mint = &ctx.accounts.receipt_mint;
    let token_program = &ctx.accounts.token_program;

    // Check if the lockup period has expired
    require!(
        Clock::get()?.unix_timestamp >= lockup.unlock_date,
        crate::errors::CustomError::LockupNotExpired
    );

    // Calculate the user's share of the total supply
    let total_supply = deposit_pool.amount + reward_pool.amount;
    let user_share = lockup.receipt_amount as u128 * total_supply as u128 / vault.total_receipt_supply as u128;
    let user_share = user_share as u64;

    // Calculate the amount to withdraw from each pool
    let deposit_amount = (user_share as u128 * deposit_pool.amount as u128 / total_supply as u128) as u64;
    let reward_amount = (user_share as u128 * reward_pool.amount as u128 / total_supply as u128) as u64;

    burn(
        CpiContext::new(
            token_program.to_account_info(), 
            Burn {
                mint: receipt_mint.to_account_info(),
                from: lockup_receipt_token_account.to_account_info(),
                authority: user.to_account_info(),
            }
        ), 
        lockup.receipt_amount
    )?;

    // Transfer deposit tokens to the user
    if deposit_amount > 0 {
        transfer(
            CpiContext::new(
                token_program.to_account_info(), 
                Transfer {
                    from: deposit_pool.to_account_info(),
                    to: user_deposit_token_account.to_account_info(),
                    authority: vault.to_account_info(),
                }
            ), 
            deposit_amount
        )?;
    }

    // Transfer reward tokens to the user
    if reward_amount > 0 {
        transfer(
            CpiContext::new(
                token_program.to_account_info(), 
                Transfer {
                    from: reward_pool.to_account_info(),
                    to: user_deposit_token_account.to_account_info(),
                    authority: vault.to_account_info(),
                }
            ), 
            reward_amount
        )?;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut, 
        has_one = user,
        has_one = vault,
    )]
    pub lockup: Account<'info, LockupState>,

    #[account(mut)]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        constraint = lockup_receipt_token_account.mint == vault.receipt_token_mint,
        constraint = lockup_receipt_token_account.owner == lockup.key(),
    )]
    pub lockup_receipt_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = user_deposit_token_account.mint == vault.deposit_token_mint,
        constraint = user_deposit_token_account.owner == user.key(),
    )]
    pub user_deposit_token_account: Account<'info, TokenAccount>,

    #[account(
        mut, 
        address = vault.deposit_pool
    )]
    pub deposit_pool: Account<'info, TokenAccount>,

    #[account(
        mut, 
        address = vault.reward_pool
    )]
    pub reward_pool: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = vault.receipt_token_mint
    )]
    pub receipt_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}
