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
}
