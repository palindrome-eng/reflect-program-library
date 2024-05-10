// src/instructions/initialize_ob.rs

use anchor_lang::prelude::*;
use crate::states::OrderBook;

#[derive(Accounts)]
pub struct InitializeOrderBook<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 24, // discriminator + order_book_stats.
        seeds = [b"orderBook"],
        bump
    )]
    pub order_book: Account<'info, OrderBook>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_order_book_handler(ctx: Context<InitializeOrderBook>) -> Result<()> {
    let order_book = &mut ctx.accounts.order_book;
    order_book.tvl = 0; // Initialize with zero SOL tvl.
    order_book.bids = 0; // Initialize with no bids.
    order_book.global_nonce = 0; // initialize global_nonce with no bids.
    Ok(())
}
