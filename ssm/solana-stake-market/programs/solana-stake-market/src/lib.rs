// src/lib.rs
use anchor_lang::prelude::*;
pub mod instructions;
pub mod states;
pub mod errors;

pub use instructions::*;

declare_id!("sSmYaKe6tj5VKjPzHhpakpamw1PYoJFLQNyMJD3PU37");

#[program]
pub mod solana_stake_market {
    use super::*;

    pub fn initialize_order_book(ctx: Context<InitializeOrderBook>) -> Result<()> {
        initialize_order_book_handler(ctx)
    }
    pub fn place_bid(ctx: Context<PlaceBid>, rate: u64, amount: u64) -> Result<()> {
        place_bid_handler(ctx, rate, amount)
    }
    pub fn close_bid(ctx: Context<CloseBid>) -> Result<()> {
        close_bid_handler(ctx)
    }
    pub fn sell_stake<'a, 'info>(
        ctx: Context<'_, '_, 'info, 'info, SellStake<'info>>,
        total_stake_amount: u64
    ) -> Result<()> {
        sell_stake_handler(ctx, total_stake_amount)
    }
}
