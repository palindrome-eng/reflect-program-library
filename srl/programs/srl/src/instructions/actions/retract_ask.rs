use anchor_lang::{
    prelude::*, 
    solana_program::{
        program::invoke_signed, 
        stake::{
            instruction::authorize, 
            program::ID as STAKE_PROGRAM_ID, 
            state::StakeAuthorize
        }
    }, 
};

use anchor_spl::stake::StakeAccount;

use crate::{
    errors::SrlErrors, state::{
        Loan, LoanState, OrderBook
    } 
};

#[derive(Accounts)]
pub struct RetractAsk<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,
    #[account(mut)]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(
        seeds = [b"order_book", OrderBook::CURRENT_VERSION.to_le_bytes().as_ref()],
        bump = order_book.bump,
    )]
    pub order_book: Account<'info, OrderBook>,
    #[account(
        mut,
        close = borrower,
        constraint = loan.borrower.ok_or(SrlErrors::BorrowerNotFound)? == borrower.key(),
        constraint =loan.stake_account.ok_or(SrlErrors::StakeAccountNotFound)? == stake_account.key(),
        has_one = order_book,
        seeds = [b"loan", loan.id.to_le_bytes().as_ref()],
        bump = loan.bump
    )]
    pub loan: Account<'info, Loan>,

    pub system_program: Program<'info, System>,
    #[account(address = STAKE_PROGRAM_ID)]
    /// CHECK: checked by address constraint
    pub stake_program: UncheckedAccount<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> RetractAsk<'info> {

    pub fn give_back_withdraw_authority (&mut self) -> Result<()> {

        // Create the Signers Seeds
        let current_version = OrderBook::CURRENT_VERSION.to_le_bytes();
        let signer_seeds = &[b"order_book", current_version.as_ref(), &[self.order_book.bump]];

        // Change Withdraw Authority Instruction
        let change_withdraw_auth_ix = authorize(
            &self.stake_account.key(),
            &self.order_book.key(),
            &self.borrower.key(),
            StakeAuthorize::Withdrawer,
            None,
        );

        let account_infos = &[
            self.order_book.to_account_info(),
            self.stake_program.to_account_info(),
            self.stake_account.to_account_info(),
            self.clock.to_account_info()
        ];

        // Invoke the Instruction and Sign it with Signer Seeds 
        invoke_signed(&change_withdraw_auth_ix, account_infos, &[signer_seeds])?;

        Ok(())
    }
}

pub fn handler<'info>(ctx: Context<RetractAsk>) -> Result<()> {

    // Orderbook Checks - Should we let them withdraw their order no matter what?
    // require_eq!(ctx.accounts.order_book.locked, true, SrlErrors::OrderBookLocked);

    // Check if we're in the right Loan State
    match ctx.accounts.loan.loan_state {
        LoanState::Ask => {},
        _ => return Err(SrlErrors::WrongLoanState.into())
    }

    // Give Back the Withdraw Authority of the Staking Account to the Borrower
    ctx.accounts.give_back_withdraw_authority()?;

    // Subtract the TVL data inside of the Order Book
    let stake_amount = ctx.accounts.stake_account.delegation().ok_or(SrlErrors::StakeAccountDelegationNotFound)?.stake;
    ctx.accounts.order_book.tvl = ctx.accounts.order_book.tvl.checked_sub(stake_amount).ok_or(SrlErrors::NumericalUnderflow)?;
    
    Ok(())
}