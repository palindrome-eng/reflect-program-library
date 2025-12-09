use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token, transfer, Transfer};
use spl_math::precise_number::PreciseNumber;
use crate::errors::RlpError;
use crate::states::{Asset, LiquidityPool, Settings, UserPermissions, Action};
use crate::constants::*;
use anchor_spl::associated_token::AssociatedToken;
use crate::helpers::action_check_protocol;
use crate::helpers::loaders::{
    load_assets,
    load_reserves,
    load_oracle_prices
};
use crate::events::RestakeEvent;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct RestakeArgs {
    pub liquidity_pool_index: u8,
    pub amount: u64,
    pub min_lp_tokens: u64,
    pub asset_id: u8,
}

pub fn restake<'a>(
    ctx: Context<'_, '_, 'a, 'a, Restake<'a>>,
    args: RestakeArgs
) -> Result<()> {
    let RestakeArgs { 
        liquidity_pool_index: _, 
        amount, 
        min_lp_tokens,
        asset_id: __
    } = args;

    let settings = &ctx.accounts.settings;
    let permissions = &ctx.accounts.permissions;

    action_check_protocol(
        Action::Restake,
        permissions.as_deref(),
        &settings.access_control
    )?;

    let signer = &ctx.accounts.signer;
    let liquidity_pool = &ctx.accounts.liquidity_pool;
    let lp_token = &ctx.accounts.lp_token;
    let token_program = &ctx.accounts.token_program;

    require!(
        amount > 0,
        crate::errors::RlpError::InvalidInput
    );

    let clock = Clock::get()?;

    msg!("asset: {:?}", ctx.accounts.asset.key());

    require!(
        ctx.remaining_accounts.len() == (settings.assets as usize) * 3,
        RlpError::InvalidInput
    );

    for account in ctx.remaining_accounts {
        msg!("remaining account: {:?}", account.key());
    }

    msg!("loading accounts");

    let assets = load_assets(settings, &ctx.remaining_accounts)?;
    let assets_datas = assets.iter().map(|(_, asset)| asset).collect::<Vec<&Asset>>();

    msg!("loaded assets");

    let reserves = load_reserves(liquidity_pool, &assets_datas, &ctx.remaining_accounts)?;
    let reserves_datas = reserves.iter().map(|(_, reserve)| reserve).collect::<Vec<&TokenAccount>>();

    msg!("loaded reserves");

    let oracle_prices = load_oracle_prices(&clock, &assets_datas, &ctx.remaining_accounts)?;

    msg!("loaded oracle prices");

    let total_pool_value_before = liquidity_pool.calculate_total_pool_value(
        &reserves_datas,
        &oracle_prices
    )?;

    liquidity_pool.deposit(
        signer,
        amount,
        &ctx.accounts.user_asset_account,
        &ctx.accounts.pool_asset_account,
        token_program
    )?;

    let asset = &ctx.accounts.asset;
    let oracle = &ctx.accounts.oracle;
    let deposit_asset_price = asset.get_price(oracle, &clock)?;

    let deposit_value = PreciseNumber::new(
        deposit_asset_price.mul(amount)?
    ).ok_or(RlpError::MathOverflow)?;

    if liquidity_pool.deposit_cap.is_some() {
        let new_pool_value = total_pool_value_before
        .checked_add(&deposit_value)
        .ok_or(RlpError::MathOverflow)?;

        require!(
            new_pool_value
                .less_than(&PreciseNumber::new(liquidity_pool.deposit_cap.unwrap() as u128).unwrap()),
            RlpError::DepositCapOverflow
        );
    }

    let lp_tokens_to_mint = liquidity_pool
        .calculate_lp_tokens_on_deposit(
            lp_token,
            total_pool_value_before,
            deposit_value
        )?;

    require!(
        min_lp_tokens <= lp_tokens_to_mint,
        RlpError::SlippageExceeded
    );

    liquidity_pool.mint_lp_token(
        lp_tokens_to_mint,
        liquidity_pool,
        lp_token,
        &ctx.accounts.user_lp_account,
        token_program
    )?;

    emit!(RestakeEvent {
        amount,
        from: signer.key(),
        asset: asset.key()
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(args: RestakeArgs)]
pub struct Restake<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes(),
        ],
        bump = settings.bump,
        constraint = !settings.access_control.killswitch.is_frozen(&Action::Restake) @ RlpError::Frozen,
    )]
    pub settings: Box<Account<'info, Settings>>,

    #[account(
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump = permissions.bump
    )]
    pub permissions: Option<Account<'info, UserPermissions>>,

    #[account(
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            &args.liquidity_pool_index.to_le_bytes()
        ],
        bump = liquidity_pool.bump,
        constraint = liquidity_pool.index == args.liquidity_pool_index,
    )]
    pub liquidity_pool: Box<Account<'info, LiquidityPool>>,

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
            &args.asset_id.to_le_bytes()
        ],
        bump = asset.bump,
    )]
    pub asset: Box<Account<'info, Asset>>,

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

    /// CHECK: Directly checking the address
    #[account(
        constraint = asset.oracle.key() == &oracle.key()
    )]
    pub oracle: AccountInfo<'info>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account()]
    pub system_program: Program<'info, System>,
}