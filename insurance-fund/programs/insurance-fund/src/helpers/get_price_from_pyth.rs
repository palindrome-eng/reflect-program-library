use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;
use super::OraclePrice;

pub fn get_price_from_pyth(
    oracle_account: &AccountInfo
) -> Result<OraclePrice> {
    let oracle_account_data = oracle_account.try_borrow_mut_data()?;

    let oracle = PriceUpdateV2::try_deserialize(&mut oracle_account_data.as_ref())?;
    let price = oracle.price_message.price as u64;
    let precision = oracle.price_message.exponent.abs_diff(0);

    Ok(OraclePrice {
        price,
        precision
    })
}