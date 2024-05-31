use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::vault::Vault;
use crate::errors::CustomError;

#[derive(Accounts)]
#[instruction(vault_seed: u64)]
pub struct InitVaultPools<'info> {
    #[account(mut, has_one = admin)]
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

pub fn init_vault_pools(
    ctx: Context<InitVaultPools>,
    _vault_seed: u64,
) -> Result<()> {
    let receipt_token_mint = &mut ctx.accounts.receipt_token_mint;
    let vault = &mut ctx.accounts.vault;

    require!(
        receipt_token_mint.supply == 0,
        CustomError::NonZeroReceiptSupply
    );

    require!(
        receipt_token_mint.mint_authority.is_some() && 
        receipt_token_mint.mint_authority.unwrap() == vault.key(),
        CustomError::InvalidMintAuthority
    );

    require!(
        receipt_token_mint.freeze_authority.is_none() ||
        receipt_token_mint.freeze_authority.unwrap() == vault.key(),
        CustomError::InvalidFreezeAuthority
    );

    vault.deposit_pool = *ctx.accounts.deposit_pool.to_account_info().key;
    vault.reward_pool = *ctx.accounts.reward_pool.to_account_info().key;

    Ok(())
}
