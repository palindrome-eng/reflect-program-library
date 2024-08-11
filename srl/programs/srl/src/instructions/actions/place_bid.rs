use anchor_lang::{
    prelude::*, 
    system_program::{transfer, Transfer}
};

use crate::{
    state::{Loan, LoanTerms, LoanState, OrderBook},
    errors::SrlErrors
};

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct PlaceBidArgs {
    pub loan_id: u64,
    pub loan_amount: u64,
    pub loan_to_value: u16,
    pub loan_duration: u64,
}

#[derive(Accounts)]
#[instruction(args: PlaceBidArgs)]
pub struct PlaceBid<'info> {
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
        init,
        payer = lender,
        space = Loan::INIT_SPACE,
        seeds = [b"loan", args.loan_id.to_le_bytes().as_ref()],
        bump
    )]
    pub loan: Account<'info, Loan>,

    pub system_program: Program<'info, System>,
}

impl<'info> PlaceBid<'info> {
    pub fn initialize_loan(&mut self, args: PlaceBidArgs, bump: u8) -> Result<()> {

        let loan_terms = LoanTerms {
            loan_amount: args.loan_amount,
            loan_to_value: args.loan_to_value,
            loan_duration: args.loan_duration,
            starting_time: None
        };

        let loan_state = LoanState::Bid;

        self.loan.set_inner(
            Loan {
                loan_state,
                order_book: self.order_book.key(),
                loan_terms,
                borrower: None,
                lender: Some(self.lender.key()),
                stake_account: None,    
                id: args.loan_id,
                bump,
            }
        );

        Ok(())
    }

    pub fn transfer_lamports_into_bid_vault(&mut self, lamports: u64) -> Result<()> {
        
        transfer(
            CpiContext::new(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.lender.to_account_info(),
                    to: self.bid_vault.to_account_info(),
                }
            ),
            lamports
        )?;

        Ok(())
    }
}

pub fn handler(ctx: Context<PlaceBid>, args: PlaceBidArgs) -> Result<()> {

    // Orderbook Checks
    require_eq!(ctx.accounts.order_book.locked, true, SrlErrors::OrderBookLocked);
    
    // Get the Bump
    let bump = ctx.bumps.loan;

    // Initialize the Loan State
    ctx.accounts.initialize_loan(args.clone(), bump)?;

    // Transfer the Bid amount into the Bid Vault
    ctx.accounts.transfer_lamports_into_bid_vault(args.clone().loan_amount)?;

    // Add the TVL data inside of the Order Book
    ctx.accounts.order_book.tvl = ctx.accounts.order_book.tvl.checked_add(args.loan_amount).ok_or(SrlErrors::NumericalOverflow)?;    

    Ok(())
}