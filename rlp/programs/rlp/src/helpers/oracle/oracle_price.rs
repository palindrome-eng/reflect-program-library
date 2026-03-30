use crate::{constants::PRECISION, errors::RlpError};

#[derive(Debug)]
pub struct OraclePrice {
    pub price: i64,
    pub exponent: i32,
}

impl OraclePrice {
    #[inline(never)]
    pub fn mul(&self, amount: u64, token_decimals: u8) -> Result<u128, RlpError> {
        let decimal_adjustment = PRECISION.saturating_sub(token_decimals as u32);

        let normalized_amount = (amount as u128)
            .checked_mul(10u128.pow(decimal_adjustment))
            .ok_or(RlpError::MathOverflow)?;

        let value = if self.exponent >= 0 {
            (normalized_amount as i128)
                .checked_mul(self.price.into())
                .ok_or(RlpError::MathOverflow.into())?
                .checked_mul(
                    i128::try_from(
                        10_i64
                            .checked_pow(self.exponent.abs_diff(0))
                            .ok_or(RlpError::MathOverflow.into())?,
                    )
                    .map_err(|_| RlpError::MathOverflow)?,
                )
                .ok_or(RlpError::MathOverflow.into())?
                .try_into()
                .map_err(|_| RlpError::MathOverflow.into())?
        } else {
            (normalized_amount as i128)
                .checked_mul(self.price.into())
                .ok_or(RlpError::MathOverflow.into())?
                .checked_div(
                    i128::try_from(
                        10_i64
                            .checked_pow(self.exponent.abs_diff(0))
                            .ok_or(RlpError::MathOverflow.into())?,
                    )
                    .map_err(|_| RlpError::MathOverflow)?,
                )
                .ok_or(RlpError::MathOverflow.into())?
                .try_into()
                .map_err(|_| RlpError::MathOverflow.into())?
        };

        Ok(value)
    }
}
