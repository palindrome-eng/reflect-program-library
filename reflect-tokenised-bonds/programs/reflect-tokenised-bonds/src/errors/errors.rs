use anchor_lang::prelude::*;

#[error_code]
pub enum ReflectError {
    #[msg("Invalid transaction signer.")]
    InvalidSigner,

    #[msg("ProgramAccountsMismatch")]
    ProgramAccountsMismatch,

    #[msg("InvalidReceiptTokenSupply")]
    InvalidReceiptTokenSupply,

    #[msg("InvalidReceiptTokenMintAuthority")]
    InvalidReceiptTokenMintAuthority,

    #[msg("InvalidReceiptTokenFreezeAuthority")]
    InvalidReceiptTokenFreezeAuthority,

    #[msg("InvalidReceiptTokenSetup")]
    InvalidReceiptTokenSetup,

    #[msg("InvalidReceiptTokenDecimals")]
    InvalidReceiptTokenDecimals,

    #[msg("ZeroDivision")]
    ZeroDivision,

    #[msg("MathOverflow")]
    MathOverflow,

    #[msg("MissingAccounts")]
    MissingAccounts,

    #[msg("AmountTooLow")]
    AmountTooLow
}
