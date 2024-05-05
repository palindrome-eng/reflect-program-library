//src/errors.rs
use anchor_lang::prelude::*;

#[error_code]
pub enum SsmError {
    #[msg("The provided rate is not acceptable.")]
    InvalidRate,
    #[msg("Could not transfer liquidity to the bid.")]
    TransferFailed,
}
