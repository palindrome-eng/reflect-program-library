use anchor_lang::prelude::*;
use switchboard_solana::AggregatorAccountData;
use super::OraclePrice;

pub fn get_price_from_switchboard(
    account: &AccountInfo
) -> Result<OraclePrice> {
    let account_data = account.try_borrow_mut_data()?;
    let oracle_data = AggregatorAccountData::new_from_bytes(&account_data.as_ref())?;
    let result = oracle_data.get_result()?;

    Ok(OraclePrice {
        precision: result.scale,
        price: result.try_into()?
    })
}