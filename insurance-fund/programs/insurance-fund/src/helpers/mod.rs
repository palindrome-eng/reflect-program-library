pub mod get_price_from_pyth;
pub use get_price_from_pyth::*;

pub mod get_price_from_switchboard;
pub use get_price_from_switchboard::*;

use crate::errors::InsuranceFundError;

#[derive(Debug)]
pub struct OraclePrice {
    pub price: i64,
    pub exponent: i32,
}

impl OraclePrice {
    pub fn mul(
        &self,
        amount: u64
    ) -> Result<u128, InsuranceFundError> {
        let price: u128;

        if self.exponent >= 0 {
            price = (amount as i128)
                .checked_mul(self.price.into())
                .ok_or(InsuranceFundError::MathOverflow.into())?
                // If exponent is positive, multiply by power of 10.
                .checked_mul(
                    i128::try_from(
                        10_i64
                            .checked_pow(self.exponent.abs_diff(0))
                            .ok_or(InsuranceFundError::MathOverflow.into())?
                    )
                    .map_err(|_| InsuranceFundError::MathOverflow)?
                )
                .ok_or(InsuranceFundError::MathOverflow.into())?
                .try_into()
                .map_err(|_| InsuranceFundError::MathOverflow.into())?;
        } else {
            price = (amount as i128)
                .checked_mul(self.price.into())
                .ok_or(InsuranceFundError::MathOverflow.into())?
                // If exponent is negative, divide by power of 10.
                .checked_div(
                    i128::try_from(
                        10_i64
                            .checked_pow(self.exponent.abs_diff(0))
                            .ok_or(InsuranceFundError::MathOverflow.into())?
                    )
                    .map_err(|_| InsuranceFundError::MathOverflow)?
                )
                .ok_or(InsuranceFundError::MathOverflow.into())?
                .try_into()
                .map_err(|_| InsuranceFundError::MathOverflow.into())?;
        }

        Ok(price)
    }
}