use anchor_lang::system_program::{transfer, Transfer};
use anchor_lang::prelude::*;
use crate::states::{Bid, OrderBook};

#[derive(Accounts)]
pub struct PlaceBid<'info> {
    #[account(
        mut
    )]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = 8 + 8 * 2 + 32 * 2 + 1 + 4 + 10 * 32, // Enough space for the Bid struct
        seeds = [b"bid", user.key().as_ref(), &order_book.global_nonce.to_le_bytes()],
        bump,
    )]
    pub bid: Account<'info, Bid>,

    #[account(
        mut,
        realloc = 8 + 8 + 4 + 32 * (order_book.bids.len() + 1),
        realloc::payer = user,
        realloc::zero = false
    )]
    pub order_book: Account<'info, OrderBook>,

    pub system_program: Program<'info, System>,
}

pub fn place_bid(
    ctx: Context<PlaceBid>, 
    rate: u64,
    amount: u64
) -> Result<()> {
    msg!("Placing bid with rate {} and amount {}", rate, amount);

    let bid = &mut ctx.accounts.bid;
    let system_program = &mut ctx.accounts.system_program;
    let user = &mut ctx.accounts.user;

    bid.amount = amount;
    bid.bid_rate = rate;
    
    bid.bidder = user.key();
    bid.authority = user.key();

    bid.fulfilled = false;
    bid.purchased_stake_accounts = vec![];

    ctx.accounts.order_book.bids.push(bid.key()); // Add to order book
    ctx.accounts.order_book.global_nonce += 1; // Increment nonce

    msg!("Funding bid with amount {}", amount);

    transfer(
        CpiContext::new(
            system_program.to_account_info(),
            Transfer {
                from: user.to_account_info(),
                to: bid.to_account_info()
            } 
        ), 
        amount
    )?;

    msg!("Transfer successful. Bid amount now: {}", ctx.accounts.bid.amount);

    msg!("Bid initialized and added to order book. Current global nonce: {}", ctx.accounts.order_book.global_nonce);

    Ok(())
}
