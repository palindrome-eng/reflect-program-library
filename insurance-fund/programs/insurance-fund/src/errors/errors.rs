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

    #[msg("NotEnoughReceiptTokens")]
    NotEnoughReceiptTokens,

    #[msg("NotEnoughFundsToSlash")]
    NotEnoughFundsToSlash,

    #[msg("DepositsLocked")]
    DepositsLocked,

    #[msg("DepositsOpen")]
    DepositsOpen,

    #[msg("DepositsNotSlashed")]
    DepositsNotSlashed,

    #[msg("AllDepositsSlashed")]
    AllDepositsSlashed,

    #[msg("SlashAmountMismatch")]
    SlashAmountMismatch,

    #[msg("ShareConfigOverflow")]
    ShareConfigOverflow,

    #[msg("Frozen")]
    Frozen,

    #[msg("InvalidOracle")]
    InvalidOracle,

    #[msg("MathOverflow")]
    MathOverflow,

    #[msg("LockupInForce")]
    LockupInForce,

    #[msg("BoostNotApplied")]
    BoostNotApplied,

    #[msg("InvalidSigners")]
    InvalidSigners,

    #[msg("TransferSignatureRequired")]
    TransferSignatureRequired,

    #[msg("ColdWalletNotSlashed")]
    ColdWalletNotSlashed,

    #[msg("PermissionsTooLow")]
    PermissionsTooLow,

    #[msg("WithdrawalThresholdOverflow")]
    WithdrawalThresholdOverflow,

    #[msg("PoolImbalance")]
    PoolImbalance,

    #[msg("InvalidReceiptTokenSetup")]
    InvalidReceiptTokenSetup,

    #[msg("MinimumSuperadminsRequired")]
    MinimumSuperadminsRequired,

    #[msg("IntentValueTooLow")]
    IntentValueTooLow,

    #[msg("WithdrawalNeedsIntent")]
    WithdrawalNeedsIntent,

    #[msg("PriceError")]
    PriceError,

    #[msg("CooldownInForce")]
    CooldownInForce,

    #[msg("SlippageExceeded")]
    SlippageExceeded,
}