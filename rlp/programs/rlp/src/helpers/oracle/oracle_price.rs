use crate::errors::RlpError;

#[derive(Debug)]
pub struct OraclePrice {
    pub price: i64,
    pub exponent: i32,
}

impl OraclePrice {
    
    #[inline(never)]
    pub fn mul(
        &self,
        amount: u64
    ) -> Result<u128, RlpError> {
        let price: u128;

        if self.exponent >= 0 {
            price = (amount as i128)
                .checked_mul(self.price.into())
                .ok_or(RlpError::MathOverflow.into())?
                // If exponent is positive, multiply by power of 10.
                .checked_mul(
                    i128::try_from(
                        10_i64
                            .checked_pow(self.exponent.abs_diff(0))
                            .ok_or(RlpError::MathOverflow.into())?
                    )
                    .map_err(|_| RlpError::MathOverflow)?
                )
                .ok_or(RlpError::MathOverflow.into())?
                .try_into()
                .map_err(|_| RlpError::MathOverflow.into())?;
        } else {
            price = (amount as i128)
                .checked_mul(self.price.into())
                .ok_or(RlpError::MathOverflow.into())?
                // If exponent is negative, divide by power of 10.
                .checked_div(
                    i128::try_from(
                        10_i64
                            .checked_pow(self.exponent.abs_diff(0))
                            .ok_or(RlpError::MathOverflow.into())?
                    )
                    .map_err(|_| RlpError::MathOverflow)?
                )
                .ok_or(RlpError::MathOverflow.into())?
                .try_into()
                .map_err(|_| RlpError::MathOverflow.into())?;
        }

        Ok(price)
    }
}