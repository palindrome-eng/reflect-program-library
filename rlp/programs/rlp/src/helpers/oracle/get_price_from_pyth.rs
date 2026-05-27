use super::OraclePrice;
use crate::constants::*;
use crate::errors::RlpError;
use anchor_lang::prelude::*;
use borsh::BorshDeserialize;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

#[inline(never)]
pub fn get_price_from_pyth(oracle_account: &AccountInfo, clock: &Clock) -> Result<OraclePrice> {
    let oracle_account_data = oracle_account.try_borrow_data()?;

    let mut data_slice = &oracle_account_data[8..];
    let oracle = PriceUpdateV2::deserialize(&mut data_slice).map_err(|_| {
        RlpError::InvalidOracle
    })?;

    let price_timestamp = oracle.price_message.publish_time;
    let current_timestamp = clock.unix_timestamp;
    let age = current_timestamp.saturating_sub(price_timestamp);

    require!(
        age <= ORACLE_MAXIMUM_AGE as i64,
        RlpError::PriceError
    );

    let price = oracle.price_message.price;
    let conf = oracle.price_message.conf;

    require!(price > 0, RlpError::PriceError);
    require!(
        conf.checked_mul(MAX_ORACLE_CONFIDENCE_RATIO)
            .map_or(false, |c| c <= price as u64),
        RlpError::PriceError
    );

    Ok(OraclePrice {
        price,
        exponent: oracle.price_message.exponent,
    })
}
