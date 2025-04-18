use std::ops::Mul;

use anchor_lang::prelude::*;
use switchboard_solana::AggregatorAccountData;
use crate::{constants::ORACLE_MAXIMUM_AGE, errors::InsuranceFundError};
use super::OraclePrice;

#[inline(never)]
pub fn get_price_from_switchboard(
    account: &AccountInfo,
    clock: &Clock,
) -> Result<OraclePrice> {
    let account_data = account.try_borrow_mut_data()?;
    let oracle_data = AggregatorAccountData
        ::new_from_bytes(&account_data.as_ref())
        .map_err(|_| InsuranceFundError::PriceError)?;

    let unix_timestamp = clock.unix_timestamp;

    oracle_data.check_staleness(
        unix_timestamp, 
        ORACLE_MAXIMUM_AGE as i64
    ).map_err(|_| InsuranceFundError::PriceError)?;

    let result = oracle_data
        .get_result()
        .map_err(|_| InsuranceFundError::PriceError)?;

    Ok(OraclePrice {
        price: result.try_into().map_err(|_| InsuranceFundError::PriceError)?,
        // result.scale is always decimal places to move to the **LEFT** to yield the actual value
        // since pyth can return both negative or positive scales, we have to add negative sign here
        exponent: (result.scale as i32).mul(-1)
    })
}