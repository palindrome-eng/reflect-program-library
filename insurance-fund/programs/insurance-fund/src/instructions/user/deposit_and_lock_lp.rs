use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token};
use crate::states::{liquidity_pool, Asset, LiquidityPool};
use crate::constants::*;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct DepositAndLockLpArgs {
    pub token_a_amount: u64,
    pub token_b_amount: u64,
}

pub fn deposit_and_lock_lp(
    ctx: Context<DepositAndLockLp>,
    args: DepositAndLockLpArgs
) -> Result<()> {

    let DepositAndLockLpArgs {
        token_a_amount,
        token_b_amount
    } = args;

    let signer = &ctx.accounts.signer;
    let token_a_user_account = &ctx.accounts.user_token_a_account;
    let token_b_user_account = &ctx.accounts.user_token_b_account;
    let token_a_pool = &ctx.accounts.token_a_pool;
    let token_b_pool = &ctx.accounts.token_b_pool;
    let liquidity_pool = &ctx.accounts.liquidity_pool;
    let lp_token = &ctx.accounts.lp_token;
    let token_program = &ctx.accounts.token_program;

    let clock = &Clock::get()?;

    let token_a_asset = &ctx.accounts.token_a_asset;
    let token_a_oracle = &ctx.accounts.token_a_oracle;
    let token_a_price = token_a_asset.get_price(&token_a_oracle, &clock)?;

    let token_b_asset = &ctx.accounts.token_b_asset;
    let token_b_oracle = &ctx.accounts.token_b_oracle;
    let token_b_price = token_b_asset.get_price(&token_b_oracle, &clock)?;

    liquidity_pool.deposit(
        signer, 
        token_a_amount, 
        token_a_user_account, 
        token_a_pool, 
        token_b_amount, 
        token_b_user_account, 
        token_b_pool, 
        token_program
    )?;

    let lp_tokens = liquidity_pool.compute_lp_tokens_on_deposit(
        lp_token.supply, 
        token_a_pool.amount, 
        token_b_pool.amount, 
        token_a_amount, 
        token_b_amount, 
        token_a_price, 
        token_b_price
    )?;

    liquidity_pool.mint_lp_token(
        lp_tokens, 
        liquidity_pool, 
        lp_token, 
        // TODO: Add creation of per-user specific deposit accounts
        lp_token_lockup_deposit, 
        token_program
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct DepositAndLockLp<'info> {
    #[account()]
    pub signer: Signer<'info>,

    #[account()]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        address = liquidity_pool.lp_token
    )]
    pub lp_token: Account<'info, Mint>,

    #[account(
        address = liquidity_pool.token_a
    )]
    pub token_a: Account<'info, Mint>,

    #[account(
        seeds = [
            ASSET_SEED.as_bytes(),
            &token_a.key().to_bytes()
        ],
        bump
    )]
    pub token_a_asset: Account<'info, Asset>,

    /// CHECK: Directly validating address
    #[account(
        constraint = token_a_asset.oracle.key().eq(&token_a_oracle.key())
    )]
    pub token_a_oracle: UncheckedAccount<'info>,

    #[account(
        seeds = [
            ASSET_SEED.as_bytes(),
            &token_b.key().to_bytes()
        ],
        bump
    )]
    pub token_b_asset: Account<'info, Asset>,

    /// CHECK: Directly validating address
    #[account(
        constraint = token_a_asset.oracle.key().eq(&token_a_oracle.key())
    )]
    pub token_b_oracle: UncheckedAccount<'info>,

    #[account(
        address = liquidity_pool.token_b
    )]
    pub token_b: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = token_a,
        associated_token::authority = liquidity_pool
    )]
    pub token_a_pool: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::mint = token_b,
        associated_token::authority = liquidity_pool
    )]
    pub token_b_pool: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = token_a,
        token::authority = signer,
    )]
    pub user_token_a_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = token_b,
        token::authority = signer,
    )]
    pub user_token_b_account: Account<'info, TokenAccount>,

    #[account()]
    pub token_program: Program<'info, Token>,
}