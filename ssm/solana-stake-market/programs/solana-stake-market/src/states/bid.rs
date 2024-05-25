// src/states/bid.rs
use anchor_lang::prelude::*;

#[account]
pub struct Bid {
    pub index: u64, // 8
    pub amount: u64,      // Amount of SOL offered, 8
    pub rate: u64,    // Rate, scaled by 1e4 for precision (e.g., 9700 for 0.97), 8
    pub bidder: Pubkey,   // Public key of the bidder, 32
    pub fulfilled: bool,  // Indicates if the bid is fulfilled, 1
    pub purchased_stake_accounts: Vec<Pubkey>, // List of purchased stake accounts, 4 + x*size
}

impl Bid {
    pub fn partial_fill(&mut self, amount: u64) -> () {
        self.amount -= amount;

        if self.amount == 0 {
            self.fulfilled = true;
        }
    }
}
