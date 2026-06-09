//! Oracle conversions that mirror the proxy program's raw-unit math.
//!
//! Proxy stores `principal` as USDC raw units (u64). Its vault holds USDC+ raw
//! units. The proxy's `crank` computes `vault_usdc_value = vault_balance ×
//! oracle_price × 10^exponent` and compares against `principal + commission`.
//!
//! To stay consistent with the proxy's accounting we use the same raw-unit
//! conversion here, not the `PRECISION`-scaled `OraclePrice::mul` used
//! elsewhere in RLP (which targets 18-decimal precision for LP share math).

use crate::errors::RlpError;

/// `value × price × 10^exponent`. Mirrors proxy's `mul_oracle`.
pub fn mul_oracle_raw(value: u128, price: u128, exponent: i32) -> Result<u128, RlpError> {
    let raw = value.checked_mul(price).ok_or(RlpError::MathOverflow)?;
    if exponent >= 0 {
        let scale = 10u128
            .checked_pow(exponent as u32)
            .ok_or(RlpError::MathOverflow)?;
        raw.checked_mul(scale).ok_or(RlpError::MathOverflow)
    } else {
        let scale = 10u128
            .checked_pow((-exponent) as u32)
            .ok_or(RlpError::MathOverflow)?;
        raw.checked_div(scale).ok_or(RlpError::MathOverflow)
    }
}

/// `value / (price × 10^exponent)`. Mirrors proxy's `div_oracle`. Floor.
pub fn div_oracle_raw(value: u128, price: u128, exponent: i32) -> Result<u128, RlpError> {
    if exponent >= 0 {
        let scale = 10u128
            .checked_pow(exponent as u32)
            .ok_or(RlpError::MathOverflow)?;
        let divisor = price.checked_mul(scale).ok_or(RlpError::MathOverflow)?;
        value.checked_div(divisor).ok_or(RlpError::MathOverflow)
    } else {
        let scale = 10u128
            .checked_pow((-exponent) as u32)
            .ok_or(RlpError::MathOverflow)?;
        value
            .checked_mul(scale)
            .ok_or(RlpError::MathOverflow)?
            .checked_div(price)
            .ok_or(RlpError::MathOverflow)
    }
}
