use crate::errors::RlpError;
use crate::events::SwapEvent;
use crate::states::*;
use crate::{constants::*, helpers::action_check_protocol};
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount, Transfer},
};

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct SwapArgs {
    pub amount_in: u64,
    pub min_out: Option<u64>,
}

pub fn swap(ctx: Context<Swap>, args: SwapArgs) -> Result<()> {
    let SwapArgs { min_out, amount_in } = args;

    // Input validation
    require!(amount_in > 0, RlpError::InvalidInput);

    let clock = &Clock::get()?;

    let signer = &ctx.accounts.signer;
    let liquidity_pool = &ctx.accounts.liquidity_pool;

    let token_from = &ctx.accounts.token_from;
    let token_to = &ctx.accounts.token_to;

    // Prevent swapping the same token
    require!(
        token_from.key() != token_to.key(),
        RlpError::InvalidInput
    );

    let token_from_asset = &ctx.accounts.token_from_asset;
    let token_to_asset = &ctx.accounts.token_to_asset;

    // Validate both assets are whitelisted for this pool
    require!(
        liquidity_pool.has_asset(token_from_asset.index),
        RlpError::AssetNotWhitelisted
    );
    require!(
        liquidity_pool.has_asset(token_to_asset.index),
        RlpError::AssetNotWhitelisted
    );

    let admin = &ctx.accounts.admin;
    let settings = &ctx.accounts.settings;

    // Check if Swap is frozen
    require!(
        !settings
            .access_control
            .killswitch
            .is_frozen(&Action::Swap),
        RlpError::Frozen
    );

    // If any of the assets are private, require admin permissions.
    if token_from_asset.access_level == AccessLevel::Private
        || token_to_asset.access_level == AccessLevel::Private
    {
        require!(
            admin.is_some()
                && admin
                    .as_ref()
                    .unwrap()
                    .can_perform_protocol_action(Action::Swap, &settings.access_control),
            RlpError::PermissionsTooLow
        );
    } else {
        action_check_protocol(
            Action::Swap,
            admin.as_deref(),
            &settings.access_control,
        )?;
    }

    let token_from_oracle = &ctx.accounts.token_from_oracle;
    let token_to_oracle = &ctx.accounts.token_to_oracle;

    let token_from_price = token_from_asset.get_price(token_from_oracle, clock)?;
    let token_to_price = token_to_asset.get_price(token_to_oracle, clock)?;

    let token_from_signer_account = &ctx.accounts.token_from_signer_account;
    let token_to_signer_account = &ctx.accounts.token_to_signer_account;

    let token_from_pool = &ctx.accounts.token_from_pool;
    let token_to_pool = &ctx.accounts.token_to_pool;

    let token_to_decimals = &ctx.accounts.token_to.decimals;
    let token_from_decimals = &ctx.accounts.token_from.decimals;

    let token_program = &ctx.accounts.token_program;

    let fee = &ctx.accounts.settings.swap_fee_bps;
    let reserve_from_amount = token_from_pool.amount;

    msg!("[swap] fee {}", fee);

    // impact_factor = x / x + a;
    let impact_factor = (amount_in as u128)
        .checked_mul(BPS_PRECISION)
        .ok_or(RlpError::MathOverflow)?
        .checked_div(
            (reserve_from_amount as u128)
                .checked_add(amount_in as u128)
                .ok_or(RlpError::MathOverflow)?,
        )
        .ok_or(RlpError::MathOverflow)?;

    // calculate oracle based amount out (oracle_amount_out = amount_in * y / x)
    let oracle_amount_out: u64 = token_from_price
        .mul(amount_in, *token_from_decimals)?
        .checked_div(token_to_price.mul(1, *token_to_decimals)?)
        .ok_or(RlpError::MathOverflow)?
        .try_into()
        .map_err(|_| RlpError::MathOverflow)?;

    msg!("[swap] oracle_amount_out {}", oracle_amount_out);

    // calculate amount after impact: oracle_amount_out * (1 - impact_factor)
    let impact_complement = BPS_PRECISION
        .checked_sub(impact_factor)
        .ok_or(RlpError::MathOverflow)?;
    let amount_after_impact = (oracle_amount_out as u128)
        .checked_mul(impact_complement)
        .ok_or(RlpError::MathOverflow)?
        .checked_div(BPS_PRECISION)
        .ok_or(RlpError::MathOverflow)?;

    msg!("[swap] amount_after_impact {}", amount_after_impact);

    // apply fee
    // amount_out = amount_after_impact * (1 - fee)
    let fee_complement = BPS_PRECISION
        .checked_sub(*fee as u128)
        .ok_or(RlpError::MathOverflow)?;

    msg!("[swap] fee_complement {}", fee_complement);

    let amount_out = amount_after_impact
        .checked_mul(fee_complement)
        .ok_or(RlpError::MathOverflow)?
        .checked_div(BPS_PRECISION)
        .ok_or(RlpError::MathOverflow)?;

    msg!("[swap] amount_out {}", amount_out);

    require!(
        token_to_pool.amount as u128 >= amount_out,
        RlpError::NotEnoughFunds
    );

    // Slippage protection
    if let Some(min_amount) = min_out {
        require!(
            amount_out as u128 >= min_amount as u128,
            RlpError::SlippageExceeded
        );
    }

    let lp_seeds = &[
        LIQUIDITY_POOL_SEED.as_bytes(),
        &liquidity_pool.index.to_le_bytes(),
        &[liquidity_pool.bump],
    ];

    transfer(
        CpiContext::new(
            token_program.to_account_info(),
            Transfer {
                from: token_from_signer_account.to_account_info(),
                to: token_from_pool.to_account_info(),
                authority: signer.to_account_info(),
            },
        ),
        amount_in,
    )?;

    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                from: token_to_pool.to_account_info(),
                to: token_to_signer_account.to_account_info(),
                authority: liquidity_pool.to_account_info(),
            },
            &[lp_seeds],
        ),
        amount_out as u64,
    )?;

    emit!(SwapEvent {
        signer: signer.key(),
        liquidity_pool: liquidity_pool.key(),
        amount_in,
        amount_out: amount_out as u64,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref(),
        ],
        bump,
    )]
    pub admin: Option<Account<'info, UserPermissions>>,

    #[account(
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
    )]
    pub settings: Box<Account<'info, Settings>>,

    #[account(
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            &liquidity_pool.index.to_le_bytes()
        ],
        bump,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account()]
    pub token_from: Box<Account<'info, Mint>>,

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
        constraint = token_from_oracle.key() == *token_from_asset.oracle.key() @ RlpError::InvalidOracle
    )]
    pub token_from_oracle: AccountInfo<'info>,

    #[account()]
    pub token_to: Box<Account<'info, Mint>>,

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
        constraint = token_to_oracle.key() == *token_to_asset.oracle.key() @ RlpError::InvalidOracle
    )]
    pub token_to_oracle: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::authority = liquidity_pool,
        associated_token::mint = token_from
    )]
    pub token_from_pool: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::authority = liquidity_pool,
        associated_token::mint = token_to
    )]
    pub token_to_pool: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = token_from,
        token::authority = signer,
    )]
    pub token_from_signer_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = token_to,
        token::authority = signer,
    )]
    pub token_to_signer_account: Box<Account<'info, TokenAccount>>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub associated_token_program: Program<'info, AssociatedToken>,
}
