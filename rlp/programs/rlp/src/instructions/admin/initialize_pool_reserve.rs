use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::constants::*;
use crate::errors::RlpError;
use crate::states::*;

pub fn initialize_pool_reserve(ctx: Context<InitializePoolReserve>) -> Result<()> {
    let liquidity_pool = &ctx.accounts.liquidity_pool;
    let asset = &ctx.accounts.asset;

    require!(
        liquidity_pool.has_asset(asset.index),
        RlpError::AssetNotWhitelisted
    );

    Ok(())
}

#[derive(Accounts)]
#[instruction(liquidity_pool_id: u8)]
pub struct InitializePoolReserve<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump = permissions.bump,
        constraint = permissions.can_perform_protocol_action(
            Action::InitializeLiquidityPool,
            &settings.access_control
        ) @ RlpError::PermissionsTooLow,
    )]
    pub permissions: Account<'info, UserPermissions>,

    #[account(
        seeds = [SETTINGS_SEED.as_bytes()],
        bump = settings.bump,
    )]
    pub settings: Box<Account<'info, Settings>>,

    #[account(
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            &liquidity_pool_id.to_le_bytes()
        ],
        bump = liquidity_pool.bump,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        seeds = [
            ASSET_SEED.as_bytes(),
            &asset.mint.to_bytes()
        ],
        bump = asset.bump,
    )]
    pub asset: Account<'info, Asset>,

    #[account(
        address = asset.mint
    )]
    pub asset_mint: Box<Account<'info, Mint>>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = asset_mint,
        associated_token::authority = liquidity_pool,
    )]
    pub pool_asset_account: Box<Account<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
