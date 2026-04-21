use super::OraclePrice;
use crate::constants::DOPPLER_MAX_STALENESS;
use crate::errors::RlpError;
use anchor_lang::prelude::*;

const DOPPLER_ORACLE_LEN: usize = 17;

/// Doppler oracle account layout (17 bytes, no discriminator):
///   [0..8]   slot      (u64 LE)
///   [8..16]  price     (u64 LE)
///   [16]     precision (u8)
#[inline(never)]
pub fn get_price_from_doppler(oracle_account: &AccountInfo) -> Result<OraclePrice> {
    let data = oracle_account.try_borrow_data()?;

    require!(
        data.len() == DOPPLER_ORACLE_LEN,
        RlpError::InvalidOracle
    );

    let posted_slot = u64::from_le_bytes(
        data[0..8].try_into().unwrap(),
    );
    let price = u64::from_le_bytes(
        data[8..16].try_into().unwrap(),
    );
    let precision = data[16];

    let current_slot = Clock::get()?.slot;
    require!(
        current_slot.saturating_sub(posted_slot) <= DOPPLER_MAX_STALENESS,
        RlpError::OracleDataTooStale
    );

    Ok(OraclePrice {
        price: price as i64,
        exponent: -(precision as i32),
    })
}
