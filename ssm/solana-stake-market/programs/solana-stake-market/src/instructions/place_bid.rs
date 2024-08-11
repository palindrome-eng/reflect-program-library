use anchor_lang::system_program::{transfer, Transfer};
use anchor_lang::prelude::*;
use crate::errors::SsmError;
use crate::constants::{
    VAULT_SEED,
    BID_SEED,
    ORDERBOOK_SEED
};
use crate::states::{Bid, OrderBook};

pub fn place_bid(
    ctx: Context<PlaceBid>,
    rate: u64,
    amount: u64
) -> Result<()> {
    let order_book = &mut ctx.accounts.order_book;
    let bid = &mut ctx.accounts.bid;
    let bid_vault = &mut ctx.accounts.bid_vault;
    let system_program = &mut ctx.accounts.system_program;
    let user = &mut ctx.accounts.user;

    require!(
        rate >= 600_000_000, 
        SsmError::BelowMinimumRate
    );

    msg!(
        "Placing bid for {} staked SOL at rate of {} liquid SOL per 1 staked SOL.", 
        (amount as f64) / 10_f64.powf(9_f64), 
        (rate as f64) / 10_f64.powf(9_f64)
    );

    // Amount / LAMPORTS_PER_SOL, otherwise double decimals.
    let deposit = ((amount as f64) / 10_f64.powf(9_f64) * (rate as f64)) as u64;

    // Deposit into bid vault.
    transfer(
        CpiContext::new(
            system_program.to_account_info(),
            Transfer {
                from: user.to_account_info(),
                to: bid_vault.to_account_info()
            } 
        ), 
        deposit
    )?;

    bid.index = order_book.global_nonce;
    bid.amount = amount;
    bid.rate = rate;
    bid.bidder = user.key();
    bid.fulfilled = false;

    order_book.add_bid(amount);

    msg!("Bid created. New TVL: {}", ctx.accounts.order_book.tvl);

    Ok(())
}

#[derive(Accounts)]
pub struct PlaceBid<'info> {
    #[account(
        mut
    )]
    pub user: Signer<'info>,

    #[account(
        init,
        payer = user,
        space = 8 + 8 * 3 + 32 + 1,
        seeds = [
            BID_SEED.as_bytes(), 
            &order_book.global_nonce.to_le_bytes()
        ],
        bump,
    )]
    pub bid: Account<'info, Bid>,

    #[account(
        mut,
        seeds = [
            VAULT_SEED.as_bytes(),
            &bid.key().to_bytes(),
        ],
        bump
    )]
    pub bid_vault: SystemAccount<'info>,

    #[account(
        mut,
        seeds = [
            ORDERBOOK_SEED.as_bytes()
        ],
        bump
    )]
    pub order_book: Account<'info, OrderBook>,

    pub system_program: Program<'info, System>,
}