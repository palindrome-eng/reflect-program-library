use anchor_lang::{
    prelude::*, 
    system_program::{transfer, Transfer}
};

use crate::{
    state::{Loan, LoanState, OrderBook},
    errors::SrlErrors
};

#[derive(Accounts)]
pub struct RetractBid<'info> {
    #[account(mut)]
    pub lender: Signer<'info>,
    #[account(
        mut,
        seeds = [b"bid_vault", loan.key().as_ref()],
        bump
    )]
    pub bid_vault: SystemAccount<'info>,

    #[account(
        seeds = [b"order_book", OrderBook::CURRENT_VERSION.to_le_bytes().as_ref()],
        bump = order_book.bump,
    )]
    pub order_book: Account<'info, OrderBook>,
    #[account(
        mut,
        constraint = loan.lender.ok_or(SrlErrors::BorrowerNotFound)? == lender.key(),
        has_one = order_book,
        seeds = [b"loan", loan.id.to_le_bytes().as_ref()],
        bump
    )]
    pub loan: Account<'info, Loan>,

    pub system_program: Program<'info, System>,
}

impl<'info> RetractBid<'info> {
    pub fn transfer_back_lamports_to_lender(&mut self, bump: u8) -> Result<()> {

        // Create the Signers Seeds
        let loan_key = self.loan.key();
        let signer_seeds = &[b"bid_vault", loan_key.as_ref(), &[bump]];

        // Transfer Back all the Lamports in the bid_vault to the Lender        
        transfer(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.bid_vault.to_account_info(),
                    to: self.lender.to_account_info(),
                },
                &[signer_seeds]
            ),
            self.bid_vault.lamports()
        )?;

        Ok(())
    }
}

pub fn handler(ctx: Context<RetractBid>) -> Result<()> {

    // Orderbook Checks - Should we let them withdraw their order no matter what?
    // require_eq!(ctx.accounts.order_book.locked, true, SrlErrors::OrderBookLocked);

    // Check if we're in the right Loan State
    match ctx.accounts.loan.loan_state {
        LoanState::Bid => {},
        _ => return Err(SrlErrors::WrongLoanState.into())
    }
    
    // Get the Bump
    let bump = ctx.bumps.bid_vault;

    // Transfer the Bid amount into the Bid Vault
    ctx.accounts.transfer_back_lamports_to_lender(bump)?;

    // Add the TVL data inside of the Order Book
    ctx.accounts.order_book.tvl = ctx.accounts.order_book.tvl.checked_sub(ctx.accounts.bid_vault.lamports()).ok_or(SrlErrors::NumericalOverflow)?;    

    Ok(())
}