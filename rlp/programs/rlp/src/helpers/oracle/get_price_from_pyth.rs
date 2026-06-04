use super::OraclePrice;
use crate::constants::*;
use crate::errors::RlpError;
use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::{PriceUpdateV2, VerificationLevel};

#[inline(never)]
pub fn get_price_from_pyth(
    oracle_account: &AccountInfo,
    clock: &Clock,
    expected_feed_id: &[u8; 32],
) -> Result<OraclePrice> {
    let oracle_account_data = oracle_account.try_borrow_data()?;

    let mut data_slice: &[u8] = &oracle_account_data;
    let oracle = PriceUpdateV2::try_deserialize(&mut data_slice)
        .map_err(|_| RlpError::InvalidOracle)?;

    require!(
        oracle.verification_level == VerificationLevel::Full,
        RlpError::PriceError
    );

    require!(
        &oracle.price_message.feed_id == expected_feed_id,
        RlpError::InvalidOracle
    );

    let price_timestamp = oracle.price_message.publish_time;
    let current_timestamp = clock.unix_timestamp;
    let age = current_timestamp - price_timestamp;

    require!(
        age >= 0 && age <= ORACLE_MAXIMUM_AGE as i64,
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
