use anchor_lang::system_program::{transfer, Transfer};
use anchor_lang::prelude::*;
use crate::constants::{BID_SEED, VAULT_SEED, ORDERBOOK_SEED};
use crate::errors::SsmError;
use crate::states::{Bid, OrderBook};

pub fn close_bid(
    ctx: Context<CloseBid>, 
    bid_index: u64
) -> Result<()> {
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
        VAULT_SEED.as_bytes(),
        &bid.key().to_bytes(),
    ];

    let (_, bump) = Pubkey::find_program_address(
        seeds, 
        program_id
    );

    let signer_seeds = &[
        VAULT_SEED.as_bytes(),
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
        bid_vault.lamports()
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    bid_index: u64
)]
pub struct CloseBid<'info> {
    #[account(
        mut
    )]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [
            BID_SEED.as_bytes(), 
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