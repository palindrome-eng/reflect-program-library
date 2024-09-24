use anchor_lang::prelude::*;
use switchboard_solana::AggregatorAccountData;

use crate::{constants::PRICE_PRECISION, errors::InsuranceFundError};

pub fn get_price_from_switchboard(
    account: &AccountInfo
) -> Result<u64> {
    let account_data = account.try_borrow_mut_data()?;
    let oracle_data = AggregatorAccountData::new_from_bytes(&account_data.as_ref())?;
    let result = oracle_data.get_result()?;

    // Only want the first 9 digits
    let price: i128 = result.scale_to(PRICE_PRECISION as u32);
    match u64::try_from(price) {
        Ok(price) => Ok(price),
        Err(_) => Err(InsuranceFundError::MathOverflow.into())
    }
}