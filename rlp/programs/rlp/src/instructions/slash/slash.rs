use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;
use crate::errors::RlpError;
use anchor_spl::token::{
    Mint,
    TokenAccount,
    Token,
    transfer,
    Transfer
};
use crate::events::SlashEvent;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SlashArgs {
    liquidity_pool_id: u8,
    amount: u64,
    asset_id: u8,
}

pub fn slash(
    ctx: Context<Slash>,
    args: SlashArgs
) -> Result<()> {
    let token_program = &ctx.accounts.token_program;
    let destination = &ctx.accounts.destination;
    let mint = &ctx.accounts.mint;
    let liquidity_pool = &ctx.accounts.liquidity_pool;
    let liquidity_pool_token_account = &ctx.accounts.liquidity_pool_token_account;

    let SlashArgs {
        amount,
        liquidity_pool_id,
        asset_id: _
    } = args;

    let asset = &ctx.accounts.asset;
    require!(
        liquidity_pool.has_asset(asset.index),
        RlpError::AssetNotWhitelisted
    );

    // Security Fix: Limit slash amount to prevent total fund drainage
    // Maximum slash per transaction is MAX_SLASH_BPS of the pool's token balance
    let max_slash_amount = liquidity_pool_token_account.amount
        .checked_mul(MAX_SLASH_BPS)
        .ok_or(RlpError::MathOverflow)?
        .checked_div(BPS_DENOMINATOR)
        .ok_or(RlpError::MathOverflow)?;
    
    require!(
        amount <= max_slash_amount,
        RlpError::SlashAmountExceedsLimit
    );

    let seeds = &[
        LIQUIDITY_POOL_SEED.as_bytes(),
        &liquidity_pool_id.to_le_bytes(),
        &[liquidity_pool.bump]
    ];
    
    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                authority: liquidity_pool.to_account_info(),
                from: liquidity_pool_token_account.to_account_info(),
                to: destination.to_account_info(),
            },
            &[seeds]
        ),
        amount
    )?;

    emit!(SlashEvent {
        admin: ctx.accounts.signer.key(),
        liquidity_pool: liquidity_pool.key(),
        amount,
        mint: mint.key()
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: SlashArgs
)]
pub struct Slash<'info> {
    #[account(
        mut,
    )]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump = permissions.bump,
        constraint = permissions.can_perform_protocol_action(Action::Slash, &settings.access_control) @ RlpError::PermissionsTooLow,
    )]
    pub permissions: Account<'info, UserPermissions>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump = settings.bump,
        constraint = !settings.access_control.killswitch.is_frozen(&Action::Slash) @ RlpError::Frozen,
    )]
    pub settings: Box<Account<'info, Settings>>,

    #[account(
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            &args.liquidity_pool_id.to_le_bytes()
        ],
        bump = liquidity_pool.bump,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        mut
    )]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [
            ASSET_SEED.as_bytes(),
            &asset.mint.to_bytes()
        ],
        constraint = asset.mint == mint.key(),
        bump = asset.bump,
    )]
    pub asset: Account<'info, Asset>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = liquidity_pool,
    )]
    pub liquidity_pool_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        token::mint = mint,
    )]
    pub destination: Box<Account<'info, TokenAccount>>,

    #[account()]
    pub token_program: Program<'info, Token>,
}