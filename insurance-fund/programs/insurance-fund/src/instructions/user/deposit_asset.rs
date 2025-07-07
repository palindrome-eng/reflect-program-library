use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token, transfer, Transfer};
use crate::states::{Asset, LiquidityPool, Settings};
use crate::constants::*;
use crate::helpers::*;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct DepositAssetArgs {
    pub liquidity_pool_index: u64,
    pub amount: u64,
}

pub fn deposit_asset(
    ctx: Context<DepositAsset>,
    args: DepositAssetArgs
) -> Result<()> {
    let DepositAssetArgs { liquidity_pool_index, amount } = args;
    let signer = &ctx.accounts.signer;
    let settings = &mut ctx.accounts.settings;
    let liquidity_pool = &ctx.accounts.liquidity_pool;
    let lp_token = &ctx.accounts.lp_token;
    let token_program = &ctx.accounts.token_program;

    require!(
        amount > 0,
        crate::errors::InsuranceFundError::InvalidInput
    );

    require!(
        liquidity_pool.index == liquidity_pool_index,
        crate::errors::InsuranceFundError::InvalidInput
    );

    require!(
        !settings.frozen,
        crate::errors::InsuranceFundError::Frozen
    );

    let clock = Clock::get()?;

    // Transfer tokens to pool
    liquidity_pool.deposit(
        signer,
        amount,
        &ctx.accounts.user_asset_account,
        &ctx.accounts.pool_asset_account,
        token_program
    )?;

    // Get asset price from oracle
    let asset = &ctx.accounts.asset;
    let oracle = &ctx.accounts.oracle;
    let price = asset.get_price(oracle, &clock)?;

    // Get current pool balance for this asset
    let pool_balance = ctx.accounts.pool_asset_account.amount;

    // Calculate LP tokens to mint based on USD value
    let lp_tokens_to_mint = liquidity_pool.compute_lp_tokens_on_deposit(
        lp_token.supply,
        pool_balance,
        amount,
        price
    )?;

    // Mint LP tokens to user using the liquidity pool as authority
    liquidity_pool.mint_lp_token(
        lp_tokens_to_mint,
        liquidity_pool,
        lp_token,
        &ctx.accounts.user_lp_account,
        token_program
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(args: DepositAssetArgs)]
pub struct DepositAsset<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes(),
        ],
        bump,
    )]
    pub settings: Account<'info, Settings>,

    #[account(
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            &args.liquidity_pool_index.to_le_bytes()
        ],
        bump,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        mut,
        address = liquidity_pool.lp_token
    )]
    pub lp_token: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = lp_token,
        associated_token::authority = signer,
    )]
    pub user_lp_account: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [
            ASSET_SEED.as_bytes(),
            &asset_mint.key().to_bytes()
        ],
        bump,
    )]
    pub asset: Account<'info, Asset>,

    #[account(
        mut,
        address = asset.mint
    )]
    pub asset_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = asset_mint,
        token::authority = signer,
    )]
    pub user_asset_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = asset_mint,
        associated_token::authority = liquidity_pool,
    )]
    pub pool_asset_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: Oracle account for price feed
    #[account(
        constraint = asset.oracle.key() == &oracle.key()
    )]
    pub oracle: UncheckedAccount<'info>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub system_program: Program<'info, System>,
} 