use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction;
use anchor_spl::token::{
    Mint, 
    Token, 
    TokenAccount, 
    Transfer,
    MintTo,
    transfer,
    mint_to
};
use crate::state::*;
use crate::errors::*;
use crate::constants::*;

pub fn deposit(
    ctx: Context<Deposit>, 
    amount: u64,
    vault_id: u64
) -> Result<()> {
    let user = &ctx.accounts.user;
    let vault = &mut ctx.accounts.vault;
    let deposit_pool = &ctx.accounts.deposit_pool;
    let reward_pool = &ctx.accounts.reward_pool;

    let token_program = &ctx.accounts.token_program;
    let deposit_token_account = &ctx.accounts.deposit_token_account;
    let receipt_token_mint = &ctx.accounts.receipt_token_mint;
    let receipt_token_account = &ctx.accounts.receipt_token_account;

    require!(
        amount >= vault.min_deposit, 
        CustomError::InsufficientDeposit
    );

    // Calculate receipt token amount
    let receipt_amount = if deposit_pool.amount > 0 {
        amount * deposit_pool.amount / (deposit_pool.amount + reward_pool.amount)
    } else {
        amount
    };

    vault.total_receipt_supply = vault
        .total_receipt_supply
        .checked_add(receipt_amount)
        .ok_or(ProgramError::InvalidInstructionData)?;
    
    transfer(
        CpiContext::new(
            token_program.to_account_info(), 
            Transfer {
                from: deposit_token_account.to_account_info(),
                to: deposit_pool.to_account_info(),
                authority: user.to_account_info(),
            }
        ), 
        amount
    )?;
    
    mint_to(
        CpiContext::new_with_signer(
            token_program.to_account_info(), 
            MintTo {
                mint: receipt_token_mint.to_account_info(),
                to: receipt_token_account.to_account_info(),
                authority: vault.to_account_info(),
            },
            &[&[
                VAULT_SEED.as_bytes(),
                &vault_id.to_le_bytes(),
                &[ctx.bumps.vault]
            ]]
        ), 
        receipt_amount
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    amount: u64,
    vault_id: u64
)]
pub struct Deposit<'info> {
    #[account(
        mut
    )]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [
            VAULT_SEED.as_bytes(),
            vault_id.to_le_bytes().as_ref()
        ],
        bump,
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        constraint = deposit_token_account.mint == vault.deposit_token_mint,
        constraint = deposit_token_account.owner == user.key(),
    )]
    pub deposit_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        constraint = receipt_token_account.mint == vault.receipt_token_mint,
        constraint = receipt_token_account.owner == user.key(),
    )]
    pub receipt_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = vault.reward_pool
    )]
    pub reward_pool: Account<'info, TokenAccount>,

    #[account(
        mut, 
        address = vault.deposit_pool
    )]
    pub deposit_pool: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = vault.receipt_token_mint
    )]
    pub receipt_token_mint: Account<'info, Mint>,

    pub token_program: Program<'info, Token>,
}
