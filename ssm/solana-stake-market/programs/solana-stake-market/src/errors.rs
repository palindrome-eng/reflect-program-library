//src/errors.rs
use anchor_lang::prelude::*;

#[error_code]
pub enum SsmError {
    #[msg("Could not transfer liquidity to the bid.")]
    TransferFailed,
    #[msg("The deposit amount is insufficient to cover the rate.")]
    UnfundedBid,
    #[msg("Rate defined is below the orderbook secure minimum of 0.6:1")]
    BelowMinimumRate,
    #[msg("Signer is not authorised to modify this account.")]
    Unauthorized,
    #[msg("Bid account has stake_accounts, claim the stake accounts - or withdraw the staked sol to close bid.")]
    Uncloseable,
    #[msg("not enough bids to cover the sale of stake accounts.")]
    InsufficientBids,
    #[msg("Failed to create a public key with the provided seed.")]
    PublicKeyCreationFailed,
    #[msg("Cannot find Stake Account's lockup.")]
    StakeAccountLockupNotFound,
    #[msg("Cannot find Stake Account's authorization settings.")]
    StakeAccountAuthorizationNotFound,
    #[msg("Cannot find Stake Account's delegation.")]
    StakeAccountDelegationNotFound,
    #[msg("Stake is not yet activated.")]
    StakeNotActivated,
    #[msg("Provided schema of RemainingAccounts is invalid.")]
    InvalidRemainingAccountsSchema,
    #[msg("Stake problem will throw error.")]
    StakeProgramWillThrowError,
}
