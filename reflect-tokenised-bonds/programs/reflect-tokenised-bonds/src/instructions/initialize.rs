use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::vault::Vault;

#[derive(Accounts)]
#[instruction(vault_seed: u64)]
pub struct Initialize<'info> {
    #[account(
        init,
        seeds = [admin.key().as_ref(), vault_seed.to_le_bytes().as_ref()],
        bump,
        payer = admin,
        space = Vault::LEN
    )]
    pub vault: Box<Account<'info, Vault>>,
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(
        init,
        payer = admin,
        seeds = [vault.key().as_ref(), b"deposit_pool"],
        bump,
        token::mint = deposit_token_mint,
        token::authority = vault,
    )]
    pub deposit_pool: Box<Account<'info, TokenAccount>>,
    #[account(
        init,
        payer = admin,
        seeds = [vault.key().as_ref(), b"reward_pool"],
        bump,
        token::mint = deposit_token_mint,
        token::authority = vault,
    )]
    pub reward_pool: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub deposit_token_mint: Box<Account<'info, Mint>>,
    #[account(mut)]
    pub receipt_token_mint: Box<Account<'info, Mint>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn initialize(
    ctx: Context<Initialize>,
    deposit_token_mint: Pubkey,
    receipt_token_mint: Pubkey,
    min_deposit: u64,
    min_lockup: i64,
    target_yield_rate: u64,
    _vault_seed: u64,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;
    vault.admin = *ctx.accounts.admin.key;
    vault.deposit_token_mint = deposit_token_mint;
    vault.receipt_token_mint = receipt_token_mint;
    vault.min_deposit = min_deposit;
    vault.min_lockup = min_lockup;
    vault.target_yield_rate = target_yield_rate;
    vault.deposit_pool = *ctx.accounts.deposit_pool.to_account_info().key;
    vault.reward_pool = *ctx.accounts.reward_pool.to_account_info().key;
    vault.total_receipt_supply = 0; // Initialize total receipt supply
    Ok(())
}
