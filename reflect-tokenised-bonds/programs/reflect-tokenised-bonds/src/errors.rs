//src/errors.rs
use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Insufficient deposit amount.")]
    InsufficientDeposit,
    #[msg("Lockup period has not expired.")]
    LockupNotExpired,
}
