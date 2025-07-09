use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token};
use crate::states::*;
use crate::constants::*;
use crate::errors::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeLiquidityPoolArgs {
    pub cooldown_duration: u64,
    pub cooldowns: u64,
}

pub fn initialize_lp(
    ctx: Context<InitializeLiquidityPool>,
    args: InitializeLiquidityPoolArgs
) -> Result<()> {
    let InitializeLiquidityPoolArgs {
        cooldown_duration,
        cooldowns
    } = args;

    let liquidity_pool = &mut ctx.accounts.liquidity_pool;
    let settings = &mut ctx.accounts.settings;
    let lp_token = &ctx.accounts.lp_token_mint;

    liquidity_pool.set_inner(LiquidityPool {
        bump: ctx.bumps.liquidity_pool,
        index: settings.liquidity_pools,
        lp_token: lp_token.key(),
        cooldown_duration,
        cooldowns,
    });

    settings.liquidity_pools += 1;

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeLiquidityPool<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref(),
        ],
        bump,
        constraint = permissions.can_perform_protocol_action(Action::InitializeLiquidityPool, &settings.access_control) @ InsuranceFundError::InvalidSigner,
    )]
    pub permissions: Account<'info, UserPermissions>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        constraint = !settings.frozen @ InsuranceFundError::Frozen
    )]
    pub settings: Account<'info, Settings>,

    #[account(
        init,
        payer = signer,
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            &settings.liquidity_pools.to_le_bytes()
        ],
        bump,
        space = 8 + LiquidityPool::INIT_SPACE
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

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

