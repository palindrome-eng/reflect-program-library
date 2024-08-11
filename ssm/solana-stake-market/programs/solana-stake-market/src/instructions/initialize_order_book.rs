use anchor_lang::prelude::*;
use crate::{constants::ORDERBOOK_SEED, states::OrderBook};

#[derive(Accounts)]
pub struct InitializeOrderBook<'info> {
    #[account(
        mut
    )]
    pub user: Signer<'info>,
    
    #[account(
        init,
        payer = user,
        space = 8 + 3 * 8,
        seeds = [
            ORDERBOOK_SEED.as_bytes()
        ],
        bump
    )]
    pub order_book: Account<'info, OrderBook>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_order_book(ctx: Context<InitializeOrderBook>) -> Result<()> {
    let order_book = &mut ctx.accounts.order_book;

    order_book.tvl = 0; // Initialize with zero SOL tvl.
    order_book.bids = 0;  // Initialize with no bids.
    order_book.global_nonce = 0; // initialize global_nonce with no bids.

    Ok(())
}
