use anchor_lang::prelude::*;
use switchboard_solana::AggregatorAccountData;

use crate::{constants::PRICE_PRECISION, errors::InsuranceFundError};

pub fn get_price_from_switchboard<'a>(
    account: &'a AccountInfo<'a>
) -> Result<u64> {
    let oracle_data = AggregatorAccountData::new(account)?;
    let result = oracle_data.get_result()?;

    // Only want the first 9 digits
    let price: i128 = result.scale_to(PRICE_PRECISION as u32);
    match u64::try_from(price) {
        Ok(price) => Ok(price),
        Err(_) => Err(InsuranceFundError::MathOverflow.into())
    }
}