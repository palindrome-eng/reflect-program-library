use anchor_lang::{
    prelude::*, 
    system_program::{transfer, Transfer}
};

use crate::{
    errors::SrlErrors, 
    state::OrderBook
};

#[derive(Accounts)]
pub struct ClaimFees<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        mut,
        seeds = [b"order_book", OrderBook::CURRENT_VERSION.to_le_bytes().as_ref()],
        bump = order_book.bump,
    )]
    pub order_book: Account<'info, OrderBook>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

impl<'info> ClaimFees<'info> {
    pub fn claim_excessive_lamports_from_account(&mut self, amount: u64) -> Result<()> {

        // Create the Signer Seeds
        let current_order_book_version = OrderBook::CURRENT_VERSION.to_le_bytes();
        let singer_seeds = &[b"order_book", current_order_book_version.as_ref(), &[self.order_book.bump]];

        // Transfer the Excessive Lamports to the Signer
        transfer(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(), 
                Transfer {
                    from: self.order_book.to_account_info(),
                    to: self.signer.to_account_info(),
                },
                &[singer_seeds]
            ), 
            amount
        )?;

        Ok(())
    }
}

pub fn handler(ctx: Context<ClaimFees>) -> Result<()> {

    // Make sure the signer is the order book authority
    require_keys_eq!(ctx.accounts.signer.key(), ctx.accounts.order_book.authority, SrlErrors::UnauthorizedOrderBookAuthority);

    // Check if there are any fees to claim
    let orderbook_minimum_balance = ctx.accounts.rent.minimum_balance(OrderBook::INIT_SPACE);
    require_gt!(ctx.accounts.order_book.to_account_info().lamports(), orderbook_minimum_balance, SrlErrors::NoOrderBookFeesToClaim);

    // Claim the Excessive Lamports
    let claimable_amount = ctx.accounts.order_book.to_account_info().lamports().checked_sub(orderbook_minimum_balance).ok_or(SrlErrors::NumericalUnderflow)?;
    ctx.accounts.claim_excessive_lamports_from_account(claimable_amount)?;

    Ok(())
}