// src/states/order_book.rs
use anchor_lang::prelude::*;

#[account]
pub struct OrderBook {
    pub tvl: u64, // total amount of actively deposited solana in bids.
    pub bids: u64,  // amount of active bids in the program.
    pub global_nonce: u64, // historical bid counter used for bid pda seed generation.
}

impl OrderBook {
    pub fn add_bid(
        &mut self,
        amount: u64
    ) -> () {
        self.tvl += amount;
        self.global_nonce += 1;
        self.bids += 1;
    }

    pub fn subtract_ask(
        &mut self,
        amount: u64
    ) -> () {
        self.tvl -= amount;
    }

    pub fn close_bid(
        &mut self, 
        remaining_funds: u64
    ) -> () {
        self.bids -= 1;
        self.subtract_ask(remaining_funds);
    }
}
