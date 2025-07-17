use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Transfer};
use crate::errors::InsuranceFundError;
use crate::constants::*;
use super::swap::*;

pub fn swap_public(
    ctx: Context<Swap>,
    args: SwapArgs
) -> Result<()> {
    let SwapArgs {
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
        .checked_div(token_to_price
            .mul(1)?
        )
        .ok_or(InsuranceFundError::MathOverflow)?
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