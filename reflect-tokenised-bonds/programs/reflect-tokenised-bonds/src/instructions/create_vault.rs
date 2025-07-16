use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::state::*;
use crate::errors::ReflectError;
use crate::constants::{
    CONFIG_SEED, 
    VAULT_SEED,
    VAULT_POOL_SEED
};

pub fn create_vault(
    ctx: Context<CreateVault>
) -> Result<()> {
    let CreateVault {
        signer,
        config,
        deposit_mint,
        receipt_mint,
        vault,
        vault_pool: _,
        token_program: __,
        system_program: ___
    } = ctx.accounts;

    vault.set_inner(Vault { 
        bump: ctx.bumps.vault,
        index: config.vaults,
        creator: signer.key(), 
        deposit_token_mint: deposit_mint.key(), 
        receipt_token_mint: receipt_mint.key() 
    });

    config.vaults += 1;

    Ok(())
}

#[derive(Accounts)]
pub struct CreateVault<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            CONFIG_SEED.as_bytes()
        ],
        bump
    )]
    pub config: Account<'info, Config>,

    #[account(
        init,
        seeds = [
            VAULT_SEED.as_bytes(),
            &config.vaults.to_le_bytes()
        ],
        bump,
        payer = signer,
        space = 8 + Vault::INIT_SPACE
    )]
    pub vault: Account<'info, Vault>,

    #[account()]
    pub deposit_mint: Account<'info, Mint>,

    #[account(
        constraint = receipt_mint.supply == 0 @ ReflectError::InvalidReceiptTokenSupply,
        constraint = receipt_mint.mint_authority.unwrap() == vault.key() @ ReflectError::InvalidReceiptTokenMintAuthority,
        constraint = receipt_mint.is_initialized @ ReflectError::InvalidReceiptTokenSetup,
        constraint = receipt_mint.decimals == deposit_mint.decimals @ ReflectError::InvalidReceiptTokenDecimals
    )]
    pub receipt_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = signer,
        token::mint = deposit_mint,
        token::authority = vault,
        seeds = [
            VAULT_POOL_SEED.as_bytes(),
            vault.key().as_ref()
        ],
        bump
    )]
    pub vault_pool: Account<'info, TokenAccount>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub system_program: Program<'info, System>,
}
