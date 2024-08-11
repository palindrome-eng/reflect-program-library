use anchor_lang::prelude::*;

use crate::{errors::SrlErrors, state::OrderBook};

#[derive(Accounts)]
pub struct ManageLock<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"order_book", OrderBook::CURRENT_VERSION.to_le_bytes().as_ref()],
        bump = order_book.bump,
    )]
    pub order_book: Account<'info, OrderBook>,
}

impl<'info> ManageLock<'info> {
    pub fn change_locked_flag(&mut self) -> Result<()> {
        
        // Change the locked flag
        self.order_book.locked = !self.order_book.locked;

        Ok(())
    }
}

pub fn handler(ctx: Context<ManageLock>) -> Result<()> {

    // Make sure the signer is the order book authority
    require_keys_eq!(ctx.accounts.signer.key(), ctx.accounts.order_book.authority, SrlErrors::UnauthorizedOrderBookAuthority);

    ctx.accounts.change_locked_flag()?;
    
    Ok(())
}