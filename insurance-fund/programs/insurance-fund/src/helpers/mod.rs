
pub struct OraclePrice {
    pub price: u64,
    pub precision: u32,
}

impl OraclePrice {
    pub fn mul(
        &self,
        amount: u64
    ) -> Result<u64, InsuranceFundError> {
        let price: u64 = (amount as u128)
            .checked_mul(self.price.into())
            .ok_or(InsuranceFundError::MathOverflow.into())?
            .checked_div(
                u128::try_from(
                    10_i64
                    .checked_pow(self.precision)
                    .ok_or(InsuranceFundError::MathOverflow.into())?
                )
                .map_err(|_| InsuranceFundError::MathOverflow)?
            )
            .ok_or(InsuranceFundError::MathOverflow.into())?
            .try_into()
            .map_err(|_| InsuranceFundError::MathOverflow.into())?;

        Ok(price)
    }
}

pub mod get_price_from_pyth;
pub use get_price_from_pyth::*;

pub mod get_price_from_switchboard;
pub use get_price_from_switchboard::*;

use crate::errors::InsuranceFundError;