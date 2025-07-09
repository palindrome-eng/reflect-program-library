use std::ops::Div;
use crate::constants::{PERMISSIONS_SEED, SETTINGS_SEED};
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{transfer, Mint, Token, TokenAccount, Transfer}};
use crate::{constants::{ASSET_SEED, LIQUIDITY_POOL_SEED}, errors::InsuranceFundError, states::{UserPermissions, Asset, LiquidityPool, Settings, Action}};

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct SwapLpArgs {
    pub amount_in: u64,
    pub min_out: Option<u64>
}

pub fn swap_lp(
    ctx: Context<SwapLp>,
    args: SwapLpArgs
) -> Result<()> {
    let SwapLpArgs {
        min_out,
        amount_in
    } = args;

    // Input validation
    require!(
        amount_in > 0,
        InsuranceFundError::InvalidInput
    );

    let clock = &Clock::get()?;

    let signer = &ctx.accounts.signer;
    let liquidity_pool = &ctx.accounts.liquidity_pool;

    let token_from = &ctx.accounts.token_from;
    let token_to = &ctx.accounts.token_to;

    // Prevent swapping the same token
    require!(
        token_from.key() != token_to.key(),
        InsuranceFundError::InvalidInput
    );

    let token_from_asset = &ctx.accounts.token_from_asset;
    let token_to_asset = &ctx.accounts.token_to_asset;

    let token_from_oracle = &ctx.accounts.token_from_oracle;
    let token_to_oracle = &ctx.accounts.token_to_oracle;

    let token_from_price = token_from_asset.get_price(token_from_oracle, clock)?;
    let token_to_price = token_to_asset.get_price(token_to_oracle, clock)?;

    let token_from_signer_account = &ctx.accounts.token_from_signer_account;
    let token_to_signer_account = &ctx.accounts.token_to_signer_account;

    let token_from_pool = &ctx.accounts.token_from_pool;
    let token_to_pool = &ctx.accounts.token_to_pool;

    let token_program = &ctx.accounts.token_program;

    // Check if pool has sufficient balance for the swap
    let amount_out: u64 = token_from_price
        .mul(amount_in)?
        .div(token_to_price
            .mul(1)?
        )
        .try_into()
        .map_err(|_| InsuranceFundError::MathOverflow)?;

    require!(
        token_to_pool.amount >= amount_out,
        InsuranceFundError::NotEnoughFunds
    );

    // Slippage protection
    if let Some(min_amount) = min_out {
        require!(
            amount_out >= min_amount,
            InsuranceFundError::SlippageExceeded
        );
    }

    let lp_seeds = &[
        LIQUIDITY_POOL_SEED.as_bytes(),
        &liquidity_pool.index.to_le_bytes(),
        &[liquidity_pool.bump]
    ];

    transfer(
        CpiContext::new(
            token_program.to_account_info(), 
            Transfer { 
                from: token_from_signer_account.to_account_info(), 
                to: token_from_pool.to_account_info(), 
                authority: signer.to_account_info() 
            }
        ),
        amount_in
    )?;

    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(), 
            Transfer { 
                from: token_to_pool.to_account_info(), 
                to: token_to_signer_account.to_account_info(), 
                authority: liquidity_pool.to_account_info()
            }, 
            &[lp_seeds]
        ), 
        amount_out
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct SwapLp<'info> {
    #[account(
        mut
    )]
    pub signer: Signer<'info>,

    #[account(
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref(),
        ],
        bump,
        constraint = admin.can_perform_protocol_action(Action::Swap, &settings.access_control) @ InsuranceFundError::PermissionsTooLow,
    )]
    pub admin: Account<'info, UserPermissions>,

    #[account(
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        constraint = !settings.frozen @ InsuranceFundError::Frozen,
    )]
    pub settings: Account<'info, Settings>,

    #[account(
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            &liquidity_pool.index.to_le_bytes()
        ],
        bump,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account()]
    pub token_from: Account<'info, Mint>,

    #[account(
        seeds = [
            ASSET_SEED.as_bytes(),
            token_from.key().as_ref()
        ],
        bump
    )]
    pub token_from_asset: Account<'info, Asset>,

    /// CHECK: Directly checking the address
    #[account(
        address = *token_from_asset.oracle.key()
    )]
    pub token_from_oracle: AccountInfo<'info>,

    #[account()]
    pub token_to: Account<'info, Mint>,

    #[account(
        seeds = [
            ASSET_SEED.as_bytes(),
            token_to.key().as_ref()
        ],
        bump
    )]
    pub token_to_asset: Account<'info, Asset>,

    /// CHECK: Directly checking the address
    #[account(
        address = *token_to_asset.oracle.key()
    )]
    pub token_to_oracle: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::authority = liquidity_pool,
        associated_token::mint = token_from
    )]
    pub token_from_pool: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::authority = liquidity_pool,
        associated_token::mint = token_to
    )]
    pub token_to_pool: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = token_from,
        token::authority = signer,
    )]
    pub token_from_signer_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = token_to,
        token::authority = signer,
    )]
    pub token_to_signer_account: Account<'info, TokenAccount>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub associated_token_program: Program<'info, AssociatedToken>,
}