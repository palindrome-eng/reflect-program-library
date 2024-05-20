//src/instructions/bid_manager.rs
use anchor_lang::system_program::{transfer, Transfer};
use anchor_lang::prelude::*;
use crate::errors::SsmError;
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
        seeds = [
            "bid".as_bytes(), 
            &order_book.global_nonce.to_le_bytes()
        ],
        bump,
    )]
    pub bid: Account<'info, Bid>,

    #[account(mut)]
    pub order_book: Account<'info, OrderBook>,

    pub system_program: Program<'info, System>,
}

pub fn place_bid(
    ctx: Context<PlaceBid>, 
    rate: u64,
    amount: u64
) -> Result<()> {
    msg!("Placing bid with rate {} and amount {}", rate, amount);
     // Validation checks
     require!(rate >= 600_000_000, SsmError::BelowMinimumRate);
     require!(amount >= rate, SsmError::UnfundedBid);

    let bid = &mut ctx.accounts.bid;
    let system_program = &mut ctx.accounts.system_program;
    let user = &mut ctx.accounts.user;

    bid.amount = amount;
    bid.bid_rate = rate;
    
    bid.bidder = user.key();
    bid.authority = user.key();

    bid.fulfilled = false;
    bid.purchased_stake_accounts = vec![];

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

    ctx.accounts.order_book.bids += 1;
    ctx.accounts.order_book.tvl += ctx.accounts.bid.amount;
    ctx.accounts.order_book.global_nonce += 1;

    msg!("bid created with amount: {} and rate {} | new tvl: {}", ctx.accounts.bid.amount, ctx.accounts.bid.bid_rate, ctx.accounts.order_book.tvl);
    Ok(())
}

#[derive(Accounts)]
#[instruction(
    bid_index: u64
)]
pub struct CloseBid<'info> {
    #[account(
        mut,
        seeds = [
            "bid".as_bytes(), 
            &bid_index.to_le_bytes()
        ],
        bump,
        constraint = bid.bidder == user.key() @ SsmError::Unauthorized,
        constraint = bid.purchased_stake_accounts.is_empty() @ SsmError::Uncloseable,
        close = user
    )]
    pub bid: Account<'info, Bid>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub order_book: Account<'info, OrderBook>,
}

pub fn close_bid(ctx: Context<CloseBid>, bid_index: u64) -> Result<()> {
    msg!("Closing bid with amount: {}, rate: {}", ctx.accounts.bid.amount, ctx.accounts.bid.bid_rate);
    msg!("Order book stats before closing: total_bids: {}, tvl: {}", ctx.accounts.order_book.bids, ctx.accounts.order_book.tvl);

    ctx.accounts.order_book.tvl -= ctx.accounts.bid.amount;
    ctx.accounts.order_book.bids -= 1; // remove active bid count from order_book.
    msg!("bid closed | stats: total_bids: {}, tvl: {}", ctx.accounts.order_book.bids, ctx.accounts.order_book.tvl);
    Ok(())
}