use super::OraclePrice;
use crate::constants::*;
use crate::errors::InsuranceFundError;
use anchor_lang::prelude::*;
// use anchor_lang::AccountDeserialize;
use borsh::BorshDeserialize;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

#[inline(never)]
pub fn get_price_from_pyth(oracle_account: &AccountInfo, clock: &Clock) -> Result<OraclePrice> {
    let oracle_account_data = oracle_account.try_borrow_data()?;
    let oracle = PriceUpdateV2::try_from_slice(&oracle_account_data[8..])
        .map_err(|_| InsuranceFundError::InvalidOracle)?;

    let price_timestamp = oracle.price_message.publish_time;
    let current_timestamp = clock.unix_timestamp;
    let age = current_timestamp.saturating_sub(price_timestamp);

    require!(
        age <= ORACLE_MAXIMUM_AGE as i64,
        InsuranceFundError::PriceError
    );

    // let price =
    //     oracle.get_price_no_older_than(clock, ORACLE_MAXIMUM_AGE, &oracle.price_message.feed_id)?;

    // let exponent = price.exponent;

    Ok(OraclePrice {
        price: oracle.price_message.price,
        exponent: oracle.price_message.exponent,
    })
}
