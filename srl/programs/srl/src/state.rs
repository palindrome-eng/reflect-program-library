use anchor_lang::prelude::*;

#[account]
pub struct OrderBook {
    pub version: u8,
    pub locked: bool,                           // Protocol Circuit Breaker
    pub tvl: u64,                               // Amount of Sol in the Protocol 
    pub fee_basis_points: u16,                  // Fee Reflect takes in Basis points
    pub authority: Pubkey,                      // Authority
    pub bump: u8,                               // Bump
}

impl OrderBook {
    pub const CURRENT_VERSION: u8 = 0;    
}

impl Space for OrderBook {
    const INIT_SPACE: usize = 8 + 1 + 1 + 8 + 2 + 32 + 1;
}

#[account]
pub struct Loan {
    pub loan_state: LoanState,                  // Loan State
    pub order_book: Pubkey,                     // Order Book
    pub loan_terms: LoanTerms,                  // Loan Terms
    pub borrower: Option<Pubkey>,               // Borrower
    pub lender: Option<Pubkey>,                 // Lender
    pub stake_account: Option<Pubkey>,          // Stake Account
    pub id: u64,                                // Loan ID 
    pub bump: u8,                               // Bump
}

impl Space for Loan {
    const INIT_SPACE: usize = LoanState::INIT_SPACE + 32 + LoanTerms::INIT_SPACE + (1 + 32) + (1 + 32) + (1 + 32) + 8 + 1;
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, InitSpace)]
pub enum LoanState {
    Bid,                                        // 0
    Ask,                                        // 1
    Taken,                                      // 2 
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct LoanTerms {
    pub loan_amount: u64,                       // Amount Borrowed
    pub loan_to_value: u16,                     // Loan to Value
    pub loan_duration: u64,                     // Loan Duration
    pub starting_time: Option<i64>,             // Starting Time
}

impl Space for LoanTerms {
    const INIT_SPACE: usize = 8 + 8 + 2 + 8;
}

