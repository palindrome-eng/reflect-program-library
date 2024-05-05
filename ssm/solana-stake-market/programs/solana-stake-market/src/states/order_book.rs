// src/states/order_book.rs
use anchor_lang::prelude::*;

#[account]
pub struct OrderBook {
    pub bids: Vec<Pubkey>,  // List of Bid account public keys
    pub global_nonce: u64,
}
