//src/errors.rs
use anchor_lang::prelude::*;

#[error_code]
pub enum CustomError {
    #[msg("Insufficient deposit amount.")]
    InsufficientDeposit,

    #[msg("Lockup period has not expired.")]
    LockupNotExpired,

    #[msg("Invalid mint authority. Move mint authority of the receipt token to the vault PDA.")]
    InvalidMintAuthority,

    #[msg("Invalid freeze authority. Move freeze authority of the receipt token to the vault PDA, or remove it completely.")]
    InvalidFreezeAuthority,

    #[msg("Supply of the receipt token has to be 0. Pre-minting is not allowed.")]
    NonZeroReceiptSupply
}
