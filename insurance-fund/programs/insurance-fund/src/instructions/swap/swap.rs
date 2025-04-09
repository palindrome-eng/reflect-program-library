use std::ops::Div;

use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use anchor_spl::token::{Mint, TokenAccount, transfer, Transfer};
use crate::errors::InsuranceFundError;
use crate::constants::*;
use crate::helpers::get_price_from_pyth;
use crate::helpers::get_price_from_switchboard;
use crate::states::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SwapArgs {
    from_lockup_id: u64,
    to_lockup_id: u64,
    amount_in: u64,
    min_amount_out: Option<u64>
}

pub fn swap(
    ctx: Context<Swap>,
    args: SwapArgs
) -> Result<()> {
    let SwapArgs {
        amount_in,
        from_lockup_id,
        min_amount_out,
        to_lockup_id
    } = args;

    let signer = &ctx.accounts.signer;
    let settings = &ctx.accounts.settings;

    let from_asset = &ctx.accounts.from_asset;
    let from_oracle = &ctx.accounts.from_oracle;
    let from_lockup = &ctx.accounts.from_lockup;
    let from_hot_vault = &ctx.accounts.from_hot_vault;
    let reflect_from_token_account = &ctx.accounts.reflect_from_token_account;

    let to_asset = &ctx.accounts.to_asset;
    let to_oracle = &ctx.accounts.to_oracle;
    let to_lockup = &ctx.accounts.to_lockup;
    let to_hot_vault = &ctx.accounts.to_hot_vault;
    let reflect_to_token_account = &ctx.accounts.reflect_to_token_account;

    let token_program = &ctx.accounts.token_program;

    let clock = Clock::get()?;

    let from_price = from_asset.get_price(from_oracle, &clock)?;
    let to_price = to_asset.get_price(to_oracle, &clock)?;

    let out_amount: u64 = from_price
        .mul(amount_in)?
        .checked_div(
            to_price
                .mul(1)?
        )
        .ok_or(InsuranceFundError::MathOverflow)?
        .try_into()
        .map_err(|_| InsuranceFundError::MathOverflow)?;

    match min_amount_out {
        Some(min_amount_out) => {
            require!(
                out_amount >= min_amount_out,
                InsuranceFundError::SlippageExceeded
            )
        },
        None => {}
    }

    // Only deposit into hot wallet, since we only withdraw from hot wallet, too.
    from_lockup.deposit_hot_wallet(
        amount_in, 
        signer, 
        reflect_from_token_account, 
        from_hot_vault, 
        token_program
    )?;

    to_lockup.withdraw_hot_vault(
        out_amount, 
        to_hot_vault, 
        to_lockup, 
        reflect_to_token_account, 
        token_program
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: SwapArgs
)]
pub struct Swap<'info> {
    #[account()]
    pub signer: Signer<'info>,

    #[account(
        constraint = admin.address == signer.key(),
        constraint = admin.has_permissions(Permissions::Superadmin),
    )]
    pub admin: Account<'info, Admin>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        constraint = !settings.frozen @ InsuranceFundError::Frozen
    )]
    pub settings: Account<'info, Settings>,

    // For the incoming transfer
    #[account(
        constraint = from_lockup.asset_mint == from_token.key()
    )]
    pub from_token: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            LOCKUP_SEED.as_bytes(),
            &args.from_lockup_id.to_le_bytes()
        ],
        bump
    )]
    pub from_lockup: Account<'info, Lockup>,

    #[account(
        constraint = from_asset.mint == from_token.key()
    )]
    pub from_asset: Account<'info, Asset>,

    /// CHECK: Directly validating address
    #[account(
        constraint = from_oracle.key().eq(from_asset.oracle.key())
    )]
    pub from_oracle: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [
            HOT_VAULT_SEED.as_bytes(),
            from_lockup.key().as_ref(),
            from_token.key().as_ref(),
        ],
        bump,
    )]
    pub from_hot_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::authority = signer,
        token::mint = from_token
    )]
    pub reflect_from_token_account: Account<'info, TokenAccount>,

    // For the outgoing transfer
    #[account(
        constraint = to_lockup.asset_mint == to_token.key()
    )]
    pub to_token: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            LOCKUP_SEED.as_bytes(),
            &args.to_lockup_id.to_le_bytes()
        ],
        bump
    )]
    pub to_lockup: Account<'info, Lockup>,

    #[account(
        constraint = to_asset.mint == to_token.key()
    )]
    pub to_asset: Account<'info, Asset>,

    /// CHECK: Directly validating address
    #[account(
        constraint = to_oracle.key().eq(to_asset.oracle.key())
    )]
    pub to_oracle: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [
            HOT_VAULT_SEED.as_bytes(),
            to_lockup.key().as_ref(),
            to_token.key().as_ref(),
        ],
        bump,
    )]
    pub to_hot_vault: Account<'info, TokenAccount>,

    #[account(
        token::authority = signer,
        token::mint = to_token
    )]
    pub reflect_to_token_account: Account<'info, TokenAccount>,

    #[account()]
    pub token_program: Program<'info, Token>,
}