use anchor_lang::prelude::*;
use crate::errors::InsuranceFundError;
use anchor_spl::token::TokenAccount;

#[inline(never)]
pub fn calculate_total_deposits(
    cold_wallet: &Account<TokenAccount>,
    hot_wallet: &Account<TokenAccount>
) -> Result<u64> {
    let result = cold_wallet.amount
        .checked_add(hot_wallet.amount)
        .ok_or(InsuranceFundError::MathOverflow)?;

    Ok(result)
}