use anchor_lang::prelude::*;
pub mod instructions;
pub mod states;
pub mod errors;
pub mod utils;
pub mod constants;
pub use instructions::*;

declare_id!("sSmYaKe6tj5VKjPzHhpakpamw1PYoJFLQNyMJD3PU37");

#[program]
pub mod solana_stake_market {
    use super::*;

    pub fn place_bid(
        ctx: Context<PlaceBid>,
        rate: u64,
        amount: u64
    ) -> Result<()> {
        instructions::place_bid(ctx, rate, amount)
    }

    pub fn close_bid(
        ctx: Context<CloseBid>,
        bid_index: u64,
    ) -> Result<()> {
        instructions::close_bid(ctx, bid_index)
    }

    pub fn initialize_order_book_wrapper(
        ctx: Context<InitializeOrderBook>
    ) -> Result<()> {
        instructions::initialize_order_book(ctx)
    }

    pub fn sell_stake<'a, 'b, 'c: 'info, 'info>(
        ctx: Context<'a, 'b, 'c, 'info, SellStake<'info>>,
        total_stake_amount: u64
    ) -> Result<()> {
        instructions::sell_stake(ctx, total_stake_amount)
    }
}
