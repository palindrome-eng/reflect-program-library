use anchor_lang::prelude::*;
use crate::errors::InsuranceFundError;
use anchor_spl::token::Mint;

#[inline(never)]
pub fn calculate_receipts_on_mint(
    receipt_token_mint: &Account<Mint>,
    deposit: &u64,
    total_deposits: &u64
) -> Result<u64> {
    if receipt_token_mint.supply == 0 { 
        Ok(0) 
    } else {
        let result = deposit
            .checked_mul(*total_deposits)
            .ok_or(InsuranceFundError::MathOverflow)?
            .checked_div(receipt_token_mint.supply)
            .ok_or(InsuranceFundError::MathOverflow)?;

        Ok(result)
    }
}