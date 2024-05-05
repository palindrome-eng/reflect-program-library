// src/states/bid.rs
use anchor_lang::prelude::*;

#[account]
pub struct Bid {
    pub amount: u64,      // Amount of SOL offered
    pub bid_rate: u64,    // Rate, scaled by 1e4 for precision (e.g., 9700 for 0.97)
    pub bidder: Pubkey,   // Public key of the bidder
    pub fulfilled: bool,  // Indicates if the bid is fulfilled
    pub purchased_stake_accounts: Vec<Pubkey>, // List of purchased stake accounts
    pub authority: Pubkey, // The authority that will manage purchased stake accounts
}
