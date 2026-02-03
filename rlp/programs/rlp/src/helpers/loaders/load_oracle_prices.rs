use anchor_lang::prelude::*;
use crate::helpers::OraclePrice;
use crate::states::*;
use crate::errors::RlpError;
use crate::helpers::get_price_from_pyth;

#[inline(never)]
pub fn load_oracle_prices(
    clock: &Clock,
    assets: &Vec<&Asset>,
    remaining_accounts: &[AccountInfo],
) -> Result<Vec<OraclePrice>> {
    let remaining_accounts_iter = &mut remaining_accounts.iter();
    let mut prices: Vec<OraclePrice> = Vec::with_capacity(assets.len() as usize);

    for asset in assets.iter() {
        let oracle_key  = asset.oracle.key();

        let maybe_account = remaining_accounts_iter
                .find(|account| account.key().eq(&oracle_key));

        let result = match maybe_account {
            Some(account_info) => {
                match asset.oracle {
                    Oracle::Pyth(_) => {
                        get_price_from_pyth(account_info, clock)
                    },
                }
            },
            None => Err(RlpError::InvalidInput.into())
        }?;

        prices.push(result);
    }

    Ok(prices)
}