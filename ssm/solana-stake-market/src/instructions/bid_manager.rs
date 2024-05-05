use anchor_lang::solana_program::system_instruction;
use anchor_lang::{prelude::*, solana_program};
use anchor_spl::token::{self, Token, Transfer};
use crate::states::{Bid, OrderBook};
use crate::errors::SsmError;

#[derive(Accounts)]
pub struct InitializeBid<'info> {
    #[account(
        init,
        payer = user,
        space = 8 + 8 + 8 + 32 + 1 + (32 * 10) + 32, // Enough space for the Bid struct
        seeds = [b"bid", user.key().as_ref(), &order_book.global_nonce.to_le_bytes()],
        bump,
    )]
    pub bid: Account<'info, Bid>,
    #[account(
        mut,
        realloc = 8 + 40 * (order_book.bids.len() + 1),
        realloc::payer = user,
        realloc::zero = false
    )]
    pub order_book: Account<'info, OrderBook>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn initialize_bid(ctx: Context<InitializeBid>, rate: u64) -> Result<()> {
    msg!("Initializing bid with rate {}", rate);
    let bid = &mut ctx.accounts.bid;
    bid.amount = 0; // No SOL yet transferred
    bid.bid_rate = rate;
    bid.bidder = ctx.accounts.user.key();
    bid.fulfilled = false;
    bid.purchased_stake_accounts = vec![];
    bid.authority = ctx.accounts.user.key();

    ctx.accounts.order_book.bids.push(bid.key()); // Add to order book
    ctx.accounts.order_book.global_nonce += 1; // Increment nonce

    msg!("Bid initialized and added to order book. Current global nonce: {}", ctx.accounts.order_book.global_nonce);

    Ok(())
}

#[derive(Accounts)]
pub struct FundBid<'info> {
    #[account(mut)]
    pub bid: Account<'info, Bid>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn fund_bid(ctx: Context<FundBid>, amount: u64) -> Result<()> {
    msg!("Funding bid with amount {}", amount);
    let transfer_ix = system_instruction::transfer(
        &ctx.accounts.user.key(),
        &ctx.accounts.bid.key(),
        amount,
    );

    // Invoke the transfer, and check for success
    let result = solana_program::program::invoke(
        &transfer_ix,
        &[
            ctx.accounts.user.to_account_info(),
            ctx.accounts.bid.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    );

    require!(result.is_ok(), SsmError::TransferFailed);

    ctx.accounts.bid.amount = amount; // Update the bid's amount after transfer

    msg!("Transfer successful. Bid amount now: {}", ctx.accounts.bid.amount);

    Ok(())
}
