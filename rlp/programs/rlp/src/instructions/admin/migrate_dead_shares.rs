use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{burn, Burn, Mint, Token, TokenAccount};
use crate::constants::*;
use crate::errors::RlpError;
use crate::states::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct MigrateDeadSharesArgs {
    pub pool_id: u8,
}

pub fn migrate_dead_shares(
    ctx: Context<MigrateDeadShares>,
    args: MigrateDeadSharesArgs,
) -> Result<()> {
    let liquidity_pool = &ctx.accounts.liquidity_pool;
    let lp_token = &ctx.accounts.lp_token_mint;
    let dead_shares_vault = &ctx.accounts.dead_shares_vault;

    let correct_dead_shares = 10u64
        .checked_pow(lp_token.decimals as u32 - 3)
        .ok_or(RlpError::MathOverflow)?;

    let current_dead_shares = dead_shares_vault.amount;

    require!(
        current_dead_shares > correct_dead_shares,
        RlpError::InvalidInput
    );

    let burn_amount = current_dead_shares
        .checked_sub(correct_dead_shares)
        .ok_or(RlpError::MathOverflow)?;

    let pool_seeds = &[
        LIQUIDITY_POOL_SEED.as_bytes(),
        &liquidity_pool.index.to_le_bytes(),
        &[liquidity_pool.bump],
    ];

    burn(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: lp_token.to_account_info(),
                from: dead_shares_vault.to_account_info(),
                authority: liquidity_pool.to_account_info(),
            },
            &[pool_seeds],
        ),
        burn_amount,
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(args: MigrateDeadSharesArgs)]
pub struct MigrateDeadShares<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump = admin.bump,
        constraint = admin.is_super_admin() @ RlpError::PermissionsTooLow,
    )]
    pub admin: Account<'info, UserPermissions>,

    #[account(
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            &args.pool_id.to_le_bytes()
        ],
        bump = liquidity_pool.bump,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        mut,
        address = liquidity_pool.lp_token
    )]
    pub lp_token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = lp_token_mint,
        associated_token::authority = liquidity_pool,
    )]
    pub dead_shares_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
