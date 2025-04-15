use std::cmp::Ordering;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::states::*;
use crate::constants::*;
use crate::errors::*;

pub fn initialize_lp(
    ctx: Context<InitializeLiquidityPool>
) -> Result<()> {
    // prevent creation of 2 pools with same tokens (X-Y and Y-X)
    // a > b or error
    ctx.accounts.enforce_token_order()?;

    let liquidity_pool = &mut ctx.accounts.liquidity_pool;
    let token_a = &ctx.accounts.token_a;
    let token_b = &ctx.accounts.token_b;
    let lp_token = &ctx.accounts.lp_token_mint;

    liquidity_pool.set_inner(LiquidityPool {
        token_a: token_a.key(), 
        token_b: token_b.key(), 
        lp_token: lp_token.key(),
        bump: ctx.bumps.liquidity_pool
    });

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeLiquidityPool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            ADMIN_SEED.as_bytes(),
            signer.key().as_ref(),
        ],
        bump,
        constraint = admin.has_permissions(Permissions::Freeze) @ InsuranceFundError::InvalidSigner,
    )]
    pub admin: Account<'info, Admin>,

    #[account(
        init,
        payer = signer,
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            token_a.key().as_ref(),
            token_b.key().as_ref()
        ],
        bump,
        space = 8 + LiquidityPool::INIT_SPACE
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        constraint = token_a_asset.mint.eq(&token_a.key())
    )]
    pub token_a: Box<Account<'info, Mint>>,

    #[account(
        constraint = token_b_asset.mint.eq(&token_b.key())
    )]
    pub token_b: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = token_a,
        associated_token::authority = liquidity_pool,
    )]
    pub token_a_pool: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = token_b,
        associated_token::authority = liquidity_pool,
    )]
    pub token_b_pool: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [
            ASSET_SEED.as_bytes(),
            &token_a.key().to_bytes()
        ],
        bump,
    )]
    pub token_a_asset: Account<'info, Asset>,

    #[account(
        seeds = [
            ASSET_SEED.as_bytes(),
            &token_b.key().to_bytes()
        ],
        bump,
    )]
    pub token_b_asset: Account<'info, Asset>,

    #[account(
        constraint = lp_token_mint.supply == 0 @ InsuranceFundError::InvalidReceiptTokenSupply,
        constraint = lp_token_mint.mint_authority.unwrap() == liquidity_pool.key() @ InsuranceFundError::InvalidReceiptTokenMintAuthority,
        constraint = lp_token_mint.freeze_authority.is_none() @ InsuranceFundError::InvalidReceiptTokenFreezeAuthority,
        constraint = lp_token_mint.is_initialized @ InsuranceFundError::InvalidReceiptTokenSetup,
        constraint = lp_token_mint.decimals == 9 @ InsuranceFundError::InvalidReceiptTokenDecimals
    )]
    pub lp_token_mint: Box<Account<'info, Mint>>,

    #[account()]
    pub system_program: Program<'info, System>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> InitializeLiquidityPool<'info> {
    // need this functionality to enforce alphabetical order of [token_a, token_b]
    // this prevents opening two pools with same tokens
    pub fn enforce_token_order(&self) -> Result<()> {
        let a = self.token_a.key();
        let b = self.token_b.key();

        match a.to_string().cmp(&b.to_string()) {
            Ordering::Greater => Ok(()),
            _ => Err(InsuranceFundError::InvalidTokenOrder.into())
        }
    }
}