use anchor_lang::prelude::*;
use crate::errors::InsuranceFundError;
use anchor_spl::token::Mint;

#[inline(never)]
pub fn calculate_receipts_on_mint(
    receipt_token_mint: &Account<Mint>,
    deposit: u64,
    total_deposits: u64
) -> Result<u64> {
    if receipt_token_mint.supply == 0 { 
        Ok(
            deposit
                .try_into()
                .map_err(|_| InsuranceFundError::MathOverflow)?
        )
    } else {
        msg!("total_deposits: {:?}", total_deposits);
        msg!("receipt supply: {:?}", receipt_token_mint.supply);
        msg!("deposit: {:?}", deposit);

        let result = (deposit as u128)
            .checked_mul(total_deposits as u128)
            .ok_or(InsuranceFundError::MathOverflow)?
            .checked_div(receipt_token_mint.supply as u128)
            .ok_or(InsuranceFundError::MathOverflow)?;

        Ok(
            result
                .try_into()
                .map_err(|_| InsuranceFundError::MathOverflow)?
        )
    }
}