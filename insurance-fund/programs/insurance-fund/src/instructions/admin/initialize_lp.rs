use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token;
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;
use crate::states::*;
use crate::constants::*;
use crate::errors::*;

pub fn initialize_lp(
    ctx: Context<InitializeLiquidityPool>
) -> Result<()> {
    let liquidity_pool = &mut ctx.accounts.liquidity_pool;
    let token_a = &ctx.accounts.token_a;
    let token_b = &ctx.accounts.token_b;
    let lp_token = &ctx.accounts.lp_token_mint;

    liquidity_pool.set_inner(LiquidityPool {
        token_a: token_a.key(), 
        token_b: token_b.key(), 
        lp_token: lp_token.key()
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
            // TODO: Sort out seeds
        ],
        bump,
        space = 8 + LiquidityPool::INIT_SPACE
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        constraint = token_a_asset.mint.eq(&token_a.key())
    )]
    pub token_a: Account<'info, Mint>,

    #[account(
        constraint = token_b_asset.mint.eq(&token_b.key())
    )]
    pub token_b: Account<'info, Mint>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = token_a,
        associated_token::authority = liquidity_pool,
    )]
    pub token_a_pool: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = token_b,
        associated_token::authority = liquidity_pool,
    )]
    pub token_b_pool: Account<'info, TokenAccount>,

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
        constraint = lp_token_mint.supply == 0 &&
            lp_token_mint.mint_authority.unwrap() == liquidity_pool.key() &&
            lp_token_mint.freeze_authority.is_none() &&
            lp_token_mint.is_initialized &&
            lp_token_mint.decimals == 9 @ InsuranceFundError::InvalidReceiptTokenSetup
    )]
    pub lp_token_mint: Account<'info, Mint>,

    #[account()]
    pub system_program: Program<'info, System>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub associated_token_program: Program<'info, AssociatedToken>,
}