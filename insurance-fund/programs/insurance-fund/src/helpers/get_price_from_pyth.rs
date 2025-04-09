use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;
use super::OraclePrice;
use crate::constants::*;
use crate::errors::InsuranceFundError;
use anchor_lang::AccountDeserialize;

#[inline(never)]
pub fn get_price_from_pyth(
    oracle_account: &AccountInfo,
    clock: &Clock
) -> Result<OraclePrice> {
    let oracle_account_data = oracle_account.try_borrow_mut_data()?;
    let oracle = PriceUpdateV2
        ::try_deserialize(&mut oracle_account_data.as_ref())?;

    let price = oracle.get_price_no_older_than(
        &clock, 
        ORACLE_MAXIMUM_AGE, 
        &oracle.price_message.feed_id
    ).map_err(|_| InsuranceFundError::PriceError)?;

    let exponent = price.exponent;

    Ok(OraclePrice {
        price: price.price,
        exponent
    })
}