// src/instructions/initialize_ob.rs

use anchor_lang::prelude::*;
use crate::states::OrderBook;

#[derive(Accounts)]
pub struct InitializeOrderBook<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 8 + 4, // Start with empty vec.
        seeds = [b"orderBook"],
        bump
    )]
    pub order_book: Account<'info, OrderBook>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitializeOrderBook>) -> Result<()> {
    let order_book = &mut ctx.accounts.order_book;
    order_book.bids = Vec::new();  // Initialize with an empty vector of bids
    order_book.global_nonce = 0; // Init global nonce
    Ok(())
}
