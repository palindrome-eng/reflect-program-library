// src/lib.rs
use anchor_lang::prelude::*;
pub use instructions::bid_manager::{self, InitializeBid, FundBid};
pub use instructions::initialize_ob::{self, InitializeOrderBook};

pub mod instructions;
pub mod states;
pub mod errors;
pub mod utils;

declare_id!("sSmYaKe6tj5VKjPzHhpakpamw1PYoJFLQNyMJD3PU37");

#[program]
pub mod solana_stake_market {
    use super::*;
    use crate::instructions::bid_manager::{InitializeBid, FundBid, initialize_bid, fund_bid};
    use crate::instructions::initialize_ob::{InitializeOrderBook, handler as initialize_order_book_handler};

    pub fn initialize_bid_wrapper(
        ctx: Context<InitializeBid>, 
        rate: u64,
    ) -> Result<()> {
        initialize_bid(ctx, rate)
    }

    pub fn fund_bid_wrapper(
        ctx: Context<FundBid>, 
        amount: u64,
    ) -> Result<()> {
        fund_bid(ctx, amount)
    }

    pub fn initialize_order_book_wrapper(
        ctx: Context<InitializeOrderBook>
    ) -> Result<()> {
        initialize_order_book_handler(ctx)
    }
}

pub use instructions::*;
pub use states::*;
