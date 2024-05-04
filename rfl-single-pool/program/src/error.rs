//! Error types

use {
    solana_program::{
        decode_error::DecodeError,
        msg,
        program_error::{PrintProgramError, ProgramError},
    },
    thiserror::Error,
};

/// Errors that may be returned by the ReflectPool program.
#[derive(Clone, Debug, Eq, Error, num_derive::FromPrimitive, PartialEq)]
pub enum ReflectPoolError {
    // 0.
    /// Provided pool account has the wrong address for its vote account, is
    /// uninitialized, or otherwise invalid.
    #[error("InvalidPoolAccount")]
    InvalidPoolAccount,
    /// Provided pool stake account does not match address derived from the pool
    /// account.
    #[error("InvalidPoolStakeAccount")]
    InvalidPoolStakeAccount,
    /// Provided pool mint does not match address derived from the pool account.
    #[error("InvalidPoolMint")]
    InvalidPoolMint,
    /// Provided pool stake authority does not match address derived from the
    /// pool account.
    #[error("InvalidPoolStakeAuthority")]
    InvalidPoolStakeAuthority,
    /// Provided pool mint authority does not match address derived from the
    /// pool account.
    #[error("InvalidPoolMintAuthority")]
    InvalidPoolMintAuthority,

    // 5.
    /// Provided pool MPL authority does not match address derived from the pool
    /// account.
    #[error("InvalidPoolMplAuthority")]
    InvalidPoolMplAuthority,
    /// Provided metadata account does not match metadata account derived for
    /// pool mint.
    #[error("InvalidMetadataAccount")]
    InvalidMetadataAccount,
    /// Authorized withdrawer provided for metadata update does not match the
    /// vote account.
    #[error("InvalidMetadataSigner")]
    InvalidMetadataSigner,
    /// Not enough lamports provided for deposit to result in one pool token.
    #[error("DepositTooSmall")]
    DepositTooSmall,
    /// Not enough pool tokens provided to withdraw stake worth one lamport.
    #[error("WithdrawalTooSmall")]
    WithdrawalTooSmall,

    // 10
    /// Not enough stake to cover the provided quantity of pool tokens.
    /// (Generally this should not happen absent user error, but may if the
    /// minimum delegation increases.)
    #[error("WithdrawalTooLarge")]
    WithdrawalTooLarge,
    /// Required signature is missing.
    #[error("SignatureMissing")]
    SignatureMissing,
    /// Stake account is not in the state expected by the program.
    #[error("WrongStakeStake")]
    WrongStakeStake,
    /// Unsigned subtraction crossed the zero.
    #[error("ArithmeticOverflow")]
    ArithmeticOverflow,
    /// A calculation failed unexpectedly.
    /// (This error should never be surfaced; it stands in for failure
    /// conditions that should never be reached.)
    #[error("UnexpectedMathError")]
    UnexpectedMathError,

    // 15
    /// The V0_23_5 vote account type is unsupported and should be upgraded via
    /// `convert_to_current()`.
    #[error("LegacyVoteAccount")]
    LegacyVoteAccount,
    /// Failed to parse vote account.
    #[error("UnparseableVoteAccount")]
    UnparseableVoteAccount,
    /// Incorrect number of lamports provided for rent-exemption when
    /// initializing.
    #[error("WrongRentAmount")]
    WrongRentAmount,
    /// Attempted to deposit from or withdraw to pool stake account.
    #[error("InvalidPoolStakeAccountUsage")]
    InvalidPoolStakeAccountUsage,
    /// Attempted to initialize a pool that is already initialized.
    #[error("PoolAlreadyInitialized")]
    PoolAlreadyInitialized,
}
impl From<ReflectPoolError> for ProgramError {
    fn from(e: ReflectPoolError) -> Self {
        ProgramError::Custom(e as u32)
    }
}
impl<T> DecodeError<T> for ReflectPoolError {
    fn type_of() -> &'static str {
        "Single-Validator Stake Pool Error"
    }
}
impl PrintProgramError for ReflectPoolError {
    fn print<E>(&self)
    where
        E: 'static
            + std::error::Error
            + DecodeError<E>
            + PrintProgramError
            + num_traits::FromPrimitive,
    {
        match self {
            ReflectPoolError::InvalidPoolAccount =>
                msg!("Error: Provided pool account has the wrong address for its vote account, is uninitialized, \
                     or is otherwise invalid."),
            ReflectPoolError::InvalidPoolStakeAccount =>
                msg!("Error: Provided pool stake account does not match address derived from the pool account."),
            ReflectPoolError::InvalidPoolMint =>
                msg!("Error: Provided pool mint does not match address derived from the pool account."),
            ReflectPoolError::InvalidPoolStakeAuthority =>
                msg!("Error: Provided pool stake authority does not match address derived from the pool account."),
            ReflectPoolError::InvalidPoolMintAuthority =>
                msg!("Error: Provided pool mint authority does not match address derived from the pool account."),
            ReflectPoolError::InvalidPoolMplAuthority =>
                msg!("Error: Provided pool MPL authority does not match address derived from the pool account."),
            ReflectPoolError::InvalidMetadataAccount =>
                msg!("Error: Provided metadata account does not match metadata account derived for pool mint."),
            ReflectPoolError::InvalidMetadataSigner =>
                msg!("Error: Authorized withdrawer provided for metadata update does not match the vote account."),
            ReflectPoolError::DepositTooSmall =>
                msg!("Error: Not enough lamports provided for deposit to result in one pool token."),
            ReflectPoolError::WithdrawalTooSmall =>
                msg!("Error: Not enough pool tokens provided to withdraw stake worth one lamport."),
            ReflectPoolError::WithdrawalTooLarge =>
                msg!("Error: Not enough stake to cover the provided quantity of pool tokens. \
                     (Generally this should not happen absent user error, but may if the minimum delegation increases.)"),
            ReflectPoolError::SignatureMissing => msg!("Error: Required signature is missing."),
            ReflectPoolError::WrongStakeStake => msg!("Error: Stake account is not in the state expected by the program."),
            ReflectPoolError::ArithmeticOverflow => msg!("Error: Unsigned subtraction crossed the zero."),
            ReflectPoolError::UnexpectedMathError =>
                msg!("Error: A calculation failed unexpectedly. \
                     (This error should never be surfaced; it stands in for failure conditions that should never be reached.)"),
            ReflectPoolError::UnparseableVoteAccount => msg!("Error: Failed to parse vote account."),
            ReflectPoolError::LegacyVoteAccount =>
                msg!("Error: The V0_23_5 vote account type is unsupported and should be upgraded via `convert_to_current()`."),
            ReflectPoolError::WrongRentAmount =>
                msg!("Error: Incorrect number of lamports provided for rent-exemption when initializing."),
            ReflectPoolError::InvalidPoolStakeAccountUsage =>
                msg!("Error: Attempted to deposit from or withdraw to pool stake account."),
            ReflectPoolError::PoolAlreadyInitialized =>
                msg!("Error: Attempted to initialize a pool that is already initialized."),
        }
    }
}
