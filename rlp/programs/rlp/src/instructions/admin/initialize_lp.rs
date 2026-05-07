use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};
use crate::states::*;
use crate::constants::*;
use crate::errors::*;
use crate::events::InitializeLiquidityPoolEvent;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeLiquidityPoolArgs {
    pub cooldown_duration: u64,
    pub deposit_cap: Option<u64>,
    pub assets: Vec<u8>,
}

pub fn initialize_lp(
    ctx: Context<InitializeLiquidityPool>,
    args: InitializeLiquidityPoolArgs
) -> Result<()> {
    let InitializeLiquidityPoolArgs {
        cooldown_duration,
        deposit_cap,
        assets,
    } = args;

    let liquidity_pool = &mut ctx.accounts.liquidity_pool;
    let settings = &mut ctx.accounts.settings;
    let lp_token = &ctx.accounts.lp_token_mint;
    let dead_shares_vault = &ctx.accounts.dead_shares_vault;
    let token_program = &ctx.accounts.token_program;

    require!(
        assets.len() >= 1 && assets.len() <= MAX_POOL_ASSETS,
        RlpError::InvalidInput
    );

    for (i, &asset_index) in assets.iter().enumerate() {
        require!(
            asset_index < settings.assets,
            RlpError::AssetNotWhitelisted
        );
        for j in 0..i {
            require!(assets[j] != asset_index, RlpError::InvalidInput);
        }
    }

    let mut asset_array = [u8::MAX; MAX_POOL_ASSETS];
    for (i, &asset_index) in assets.iter().enumerate() {
        asset_array[i] = asset_index;
    }

    let pool_index = settings.liquidity_pools;

    liquidity_pool.set_inner(LiquidityPool {
        bump: ctx.bumps.liquidity_pool,
        index: pool_index,
        lp_token: lp_token.key(),
        cooldown_duration,
        cooldowns: 0,
        deposit_cap,
        asset_count: assets.len() as u8,
        assets: asset_array,
    });

    let signer_seeds = &[
        LIQUIDITY_POOL_SEED.as_bytes(),
        &pool_index.to_le_bytes(),
        &[ctx.bumps.liquidity_pool]
    ];

    require!(
        lp_token.decimals >= 3,
        RlpError::InvalidReceiptTokenDecimals
    );

    let dead_shares = 10u64
        .checked_pow(lp_token.decimals as u32 - 3)
        .ok_or(RlpError::MathOverflow)?;

    mint_to(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            MintTo {
                mint: lp_token.to_account_info(),
                to: dead_shares_vault.to_account_info(),
                authority: liquidity_pool.to_account_info(),
            },
            &[signer_seeds]
        ),
        dead_shares
    )?;

    settings.liquidity_pools = settings.liquidity_pools
        .checked_add(1)
        .ok_or(RlpError::MathOverflow)?;

    emit!(InitializeLiquidityPoolEvent {
        admin: ctx.accounts.signer.key(),
        liquidity_pool: liquidity_pool.key(),
        lp_token: lp_token.key(),
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
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref(),
        ],
        bump = permissions.bump,
        constraint = permissions.can_perform_protocol_action(
            Action::InitializeLiquidityPool, 
            &settings.access_control
        ) @ RlpError::InvalidSigner,
    )]
    pub permissions: Account<'info, UserPermissions>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump = settings.bump,
        constraint = !settings.access_control.killswitch.is_frozen(&Action::InitializeLiquidityPool) @ RlpError::Frozen,
    )]
    pub settings: Box<Account<'info, Settings>>,

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
        mut,
        constraint = lp_token_mint.supply == 0 @ RlpError::InvalidReceiptTokenSupply,
        constraint = lp_token_mint.mint_authority.unwrap() == liquidity_pool.key() @ RlpError::InvalidReceiptTokenMintAuthority,
        // constraint = lp_token_mint.freeze_authority.is_none() @ RlpError::InvalidReceiptTokenFreezeAuthority,
        constraint = lp_token_mint.is_initialized @ RlpError::InvalidReceiptTokenSetup,
        // constraint = lp_token_mint.decimals == 9 @ RlpError::InvalidReceiptTokenDecimals
    )]
    pub lp_token_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = lp_token_mint,
        associated_token::authority = liquidity_pool,
    )]
    pub dead_shares_vault: Box<Account<'info, TokenAccount>>,

    #[account()]
    pub system_program: Program<'info, System>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub associated_token_program: Program<'info, AssociatedToken>,
}

