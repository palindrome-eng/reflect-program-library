use anchor_lang::prelude::*;

#[error_code]
pub enum SrlErrors {
    
    // Setup
    #[msg("Provided Signer is not the Order Book authority.")]
    UnauthorizedOrderBookAuthority,
    #[msg("There are no fees to claim from this Order Book.")]
    NoOrderBookFeesToClaim,

    // Actions
    #[msg("This Order Book is Locked")]
    OrderBookLocked,
    #[msg("Cannot find the Borrower in the Loan State")]
    BorrowerNotFound,
    #[msg("Cannot find the Lender in the Loan State")]
    LenderNotFound,
    #[msg("Cannot find the Stake Account in the Loan State")]
    StakeAccountNotFound,
    #[msg("Cannot find the Starting Time of the Loan")]
    StartingTimeNotFound,
    #[msg("The current Loan State is unfit for the action you're trying to do")]
    WrongLoanState,
    #[msg("The Stake Account is too small to cover the Loan Amount")]
    NotEnughStakeAmount,
    #[msg("Cannot find the Bump")]
    BumpNotFound,
    #[msg("The Borrower you're using is not the Borrower of the Loan")]
    WrongBorrwer,
    #[msg("The Lender you're using is not the Lender of the Loan")]
    WrongLender,
    #[msg("The Stake Account you're using is not the Stake Account of the Loan")]
    WrongStakeAccount,
    #[msg("The Stake Account deactivation epoch is not the maximum.")]
    WrongStakeAccountDeactivationEpoch,
    #[msg("The Stake Account lockup is in force.")]
    WrongStakeAccountLockupInForce,
    #[msg("The Stake Account withdraw authority is not the right one")]
    WrongStakeAccountWithdrawAuthority,
    #[msg("Cannot find Stake Account's lockup.")]
    StakeAccountLockupNotFound,
    #[msg("Cannot find Stake Account's authorization settings.")]
    StakeAccountAuthorizationNotFound,
    #[msg("Cannot find Stake Account's delegation.")]
    StakeAccountDelegationNotFound,
    #[msg("Provided schema of RemainingAccounts is invalid.")]
    InvalidRemainingAccountsSchema,
    #[msg("Numerical Underflow.")]
    NumericalUnderflow,
    #[msg("Numerical Overflow.")]
    NumericalOverflow,
}
