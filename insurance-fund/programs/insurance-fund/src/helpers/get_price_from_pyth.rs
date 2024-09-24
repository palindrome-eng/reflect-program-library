use anchor_lang::prelude::*;
use pyth_solana_receiver_sdk::price_update::PriceUpdateV2;

use crate::constants::PRICE_PRECISION;

pub fn get_price_from_pyth(
    oracle_account: &AccountInfo
) -> Result<u64> {
    let oracle_account_data = oracle_account.try_borrow_mut_data()?;

    let oracle = PriceUpdateV2::try_deserialize(&mut oracle_account_data.as_ref())?;
    let price = oracle.price_message.price as u64;

    // Only want the first 9 digits
    Ok(price % 10_u64.pow(PRICE_PRECISION as u32))
}