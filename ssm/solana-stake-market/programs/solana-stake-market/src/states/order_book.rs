// src/states/order_book.rs
use anchor_lang::prelude::*;

#[account]
pub struct OrderBook {
    pub tvl: u64, // total amount of actively deposited solana in bids.
    pub bids: u64,  // amount of active bids in the program.
    pub global_nonce: u64, // historical bid counter used for bid pda seed generation.
    pub total_trades: u64,
}
