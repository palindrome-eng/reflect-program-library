use crate::{constants::*, helpers::action_check_protocol, instructions::admin};
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{transfer, Mint, Token, TokenAccount, Transfer}};
use crate::errors::RlpError;
use crate::events::SwapEvent;
use crate::states::*;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct SwapArgs {
    pub amount_in: u64,
    pub min_out: Option<u64>,
    pub from_asset_id: u8,
    pub to_asset_id: u8,
}

pub fn swap(
    ctx: Context<Swap>,
    args: SwapArgs
) -> Result<()> {
    let SwapArgs {
        min_out,
        amount_in,
        from_asset_id: _,
        to_asset_id: _
    } = args;

    // Input validation
    require!(
        amount_in > 0,
        RlpError::InvalidInput
    );

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

    let admin = &ctx.accounts.admin;
    let settings = &ctx.accounts.settings;

    // If any of the assets are private, require admin permissions.
    if (token_from_asset.access_level == AccessLevel::Private || token_to_asset.access_level == AccessLevel::Private) {
        // Check if PrivateSwap is frozen
        require!(
            !settings.access_control.killswitch.is_frozen(&Action::PrivateSwap),
            RlpError::Frozen
        );
        
        require!(
            admin.is_some() && admin.as_ref().unwrap().can_perform_protocol_action(Action::PrivateSwap, &settings.access_control),
            RlpError::PermissionsTooLow
        );
    } else {
        // Check if PublicSwap is frozen
        require!(
            !settings.access_control.killswitch.is_frozen(&Action::PublicSwap),
            RlpError::Frozen
        );
        
        action_check_protocol(
            Action::PublicSwap,
            admin.as_deref(),
            &settings.access_control
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

    let token_program = &ctx.accounts.token_program;

    let amount_out: u64 = token_from_price
        .mul(amount_in)?
        .checked_mul(
            10_u64
                .checked_pow(token_to.decimals as u32)
                .ok_or(RlpError::MathOverflow)?
                .into()
        )
        .ok_or(RlpError::MathOverflow)?
        .checked_div(token_to_price
            .mul(
                10_u64
                    .checked_pow(token_from.decimals as u32)
                    .ok_or(RlpError::MathOverflow)?
                    .into()
            )?
        )
        .ok_or(RlpError::MathOverflow)?
        .try_into()
        .map_err(|_| RlpError::MathOverflow)?;

    require!(
        token_to_pool.amount >= amount_out,
        RlpError::NotEnoughFunds
    );

    // Slippage protection
    if let Some(min_amount) = min_out {
        require!(
            amount_out >= min_amount,
            RlpError::SlippageExceeded
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

    emit!(SwapEvent {
        signer: signer.key(),
        liquidity_pool: liquidity_pool.key(),
        amount_in,
        amount_out,
        private: token_from_asset.access_level == AccessLevel::Private || token_to_asset.access_level == AccessLevel::Private,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(args: SwapArgs)]
pub struct Swap<'info> {
    #[account(
        mut
    )]
    pub signer: Signer<'info>,

    #[account(
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref(),
        ],
        bump = admin.bump,
    )]
    pub admin: Option<Account<'info, UserPermissions>>,

    #[account(
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump = settings.bump,
    )]
    pub settings: Account<'info, Settings>,

    #[account(
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            &liquidity_pool.index.to_le_bytes()
        ],
        bump = liquidity_pool.bump,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account()]
    pub token_from: Account<'info, Mint>,

    #[account(
        seeds = [
            ASSET_SEED.as_bytes(),
            &args.from_asset_id.to_le_bytes()
        ],
        constraint = token_from_asset.mint == token_from.key(),
        bump = token_from_asset.bump,
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
            &args.to_asset_id.to_le_bytes()
        ],
        constraint = token_to_asset.mint == token_to.key(),
        bump = token_to_asset.bump,
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