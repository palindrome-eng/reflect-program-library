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
        space = 8 + 8 * 3 + 32 * 1 + 1 + 4 + 10 * 32, // Enough space for the Bid struct
        seeds = [
            "bid".as_bytes(), 
            &order_book.global_nonce.to_le_bytes()
        ],
        bump,
    )]
    pub bid: Account<'info, Bid>,

    #[account(
        mut,
        seeds = [
            "vault".as_bytes(),
            &bid.key().to_bytes(),
        ],
        bump
    )]
    pub bid_vault: SystemAccount<'info>,

    #[account(mut)]
    pub order_book: Account<'info, OrderBook>,

    pub system_program: Program<'info, System>,
}

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

    // Validation checks
    require!(rate >= 600_000_000, SsmError::BelowMinimumRate);

    msg!(
        "Placing bid to buy {} staked SOL at rate: {} liquid SOL per 1 staked SOL.", 
        (amount as f64) / 10_f64.powf(9_f64), 
        (rate as f64) / 10_f64.powf(9_f64)
    );

    // Amount / LAMPORTS_PER_SOL, otherwise double decimals.
    let to_deposit = ((amount as f64) / 10_f64.powf(9_f64) * (rate as f64)) as u64;

    // Deposit into bid vault.
    transfer(
        CpiContext::new(
            system_program.to_account_info(),
            Transfer {
                from: user.to_account_info(),
                to: bid_vault.to_account_info()
            } 
        ), 
        to_deposit
    )?;

    bid.index = order_book.global_nonce;

    bid.amount = amount;
    bid.rate = rate;
    
    bid.bidder = user.key();

    bid.fulfilled = false;
    bid.purchased_stake_accounts = vec![];

    order_book.add_bid(amount);

    msg!("Bid created. New TVL: {}", ctx.accounts.order_book.tvl);

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
        close = user
    )]
    pub bid: Account<'info, Bid>,

    #[account(
        mut,
        seeds = [
            "vault".as_bytes(),
            &bid.key().to_bytes(),
        ],
        bump
    )]
    pub bid_vault: SystemAccount<'info>,

    #[account(mut)]
    pub user: Signer<'info>,

    #[account(mut)]
    pub order_book: Account<'info, OrderBook>,

    pub system_program: Program<'info, System>,
}

// TODO: Check if there is any SOL left in bid vault & withdraw.
pub fn close_bid(ctx: Context<CloseBid>, bid_index: u64) -> Result<()> {
    let order_book = &mut ctx.accounts.order_book;
    let user = &mut ctx.accounts.user;
    let bid = &mut ctx.accounts.bid;
    let bid_vault = &mut ctx.accounts.bid_vault;
    let system_program = &mut ctx.accounts.system_program;
    let program_id = ctx.program_id;

    msg!(
        "Closing bid with remaining balance of {} SOL. Orderbook TVL before: {} SOL",
        (bid.amount as f64) / 10_f64.powf(9_f64),
        (order_book.tvl as f64) / 10_f64.powf(9_f64)
    );

    order_book.close_bid(bid.amount);

    msg!(
        "Orderbook TVL after: {} SOL",
        (order_book.tvl as f64) / 10_f64.powf(9_f64)
    );

    msg!("Transferring remaining balance from bid to bidder.");

    let seeds = &[
            "vault".as_bytes(),
            &bid.key().to_bytes(),
        ];

    let (_, bump) = Pubkey::find_program_address(
        seeds, 
        program_id
    );

    let signer_seeds = &[
        "vault".as_bytes(),
        &bid.key().to_bytes(),
        &[bump]
    ];

    transfer(
        CpiContext::new_with_signer(
            system_program.to_account_info(), 
            Transfer {
                from: bid_vault.to_account_info(),
                to: user.to_account_info()
            }, 
            &[signer_seeds]
        ), 
        // Transfer entire account.
        bid_vault.lamports()
    )?;

    Ok(())
}