use anchor_lang::prelude::*;

#[error_code]
pub enum InsuranceFundError {
    #[msg("InvalidSigner")]
    InvalidSigner,

    #[msg("InvalidInput")]
    InvalidInput,

    #[msg("AssetNotWhitelisted")]
    AssetNotWhitelisted,

    #[msg("DepositTooLow")]
    DepositTooLow,

    #[msg("DepositCapOverflow")]
    DepositCapOverflow,

    #[msg("NotEnoughFunds")]
    NotEnoughFunds,
}