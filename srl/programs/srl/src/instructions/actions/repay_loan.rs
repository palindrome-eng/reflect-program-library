use anchor_lang::{
    prelude::*, 
    solana_program::{
        program::invoke_signed, 
        stake::{instruction::{authorize, withdraw}, program::ID as STAKE_PROGRAM_ID, state::{StakeAuthorize, StakeStateV2}}
    }, 
    system_program::{transfer, Transfer}  
};

use anchor_spl::stake::StakeAccount;

use crate::{
    errors::SrlErrors, state::{ Loan, LoanState, OrderBook}
};

#[derive(Accounts)]
pub struct RepayLoan<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,
    #[account(mut)]
    /// CHECK: checked by address constraint
    pub lender: UncheckedAccount<'info>,
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
        constraint = loan.lender.ok_or(SrlErrors::LenderNotFound)? == lender.key(),
        constraint = loan.stake_account.ok_or(SrlErrors::StakeAccountNotFound)? == stake_account.key(),
        has_one = order_book,
        seeds = [b"loan", loan.id.to_le_bytes().as_ref()],
        bump = loan.bump,
    )]
    pub loan: Account<'info, Loan>,

    pub system_program: Program<'info, System>,
    #[account(address = STAKE_PROGRAM_ID)]
    /// CHECK: checked by address constraint
    pub stake_program: UncheckedAccount<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub stake_history: Sysvar<'info, StakeHistory>,
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> RepayLoan<'info> {
    pub fn repay_principal_to_lender(&mut self, principal_amount: u64) -> Result<()> {

        transfer(
            CpiContext::new(
                self.system_program.to_account_info(),
                Transfer {
                    from: self.borrower.to_account_info(),
                    to: self.lender.to_account_info()
                }
            ),
            principal_amount
        )?;

        Ok(())
    }

    pub fn withdraw_principal_to_lender(&mut self, principal_amount: u64) -> Result<()> {
        
        // Create the seeds for the signer
        let current_version = OrderBook::CURRENT_VERSION.to_le_bytes();
        let signer_seeds = &[b"order_book", current_version.as_ref(), &[self.order_book.bump]];

        // Withdraw Instruction
        let withdraw_ix = withdraw(
            &self.stake_account.key(),
            &self.order_book.key(),
            &self.lender.key(),
            principal_amount,
            None,
        );

        let account_infos = &[
            self.stake_account.to_account_info(),
            self.lender.to_account_info(),
            self.clock.to_account_info(),
            self.stake_history.to_account_info(),
            self.order_book.to_account_info(),
            self.stake_program.to_account_info(),
        ];

        // Invoke the Withdraw Instruction
        invoke_signed(&withdraw_ix, account_infos, &[signer_seeds])?;

        Ok(())
    }

    pub fn change_withdraw_authority (&mut self) -> Result<()> {

        // Create the seeds for the signer
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

        // Invoke the Change Withdraw Authority Instruction
        invoke_signed(&change_withdraw_auth_ix, account_infos, &[signer_seeds])?;

        Ok(())
    }

    pub fn repay_loan(&mut self) -> Result<()> {

        let loan = self.loan.clone();

        // Amount to Repay = Amount Borrowed + Staking Yield
        let amount_borrowed = loan.loan_terms.loan_amount;

        // Initial Stake Amount always > Amount Borrowed / LTV
        let starting_stake_amount =  loan.loan_terms.loan_amount
            .checked_div(loan.loan_terms.loan_to_value as u64)
            .ok_or(SrlErrors::NumericalUnderflow)?
            .checked_mul(10_000)
            .ok_or(SrlErrors::NumericalOverflow)?;

        let time_passed_in_seconds = Clock::get()?.unix_timestamp
            .checked_sub(loan.loan_terms.starting_time.ok_or(SrlErrors::StartingTimeNotFound)?)
            .ok_or(SrlErrors::NumericalUnderflow)?;

        // 7.5% in basis points
        let apy_basis_points: u16 = 750; 

        // Seconds in a year
        let seconds_in_year: i64 = 31_557_600; 

        // Staking Yield
        let staking_yield = starting_stake_amount
            .checked_mul(time_passed_in_seconds as u64)
            .ok_or(SrlErrors::NumericalOverflow)?
            .checked_mul(apy_basis_points as u64)
            .ok_or(SrlErrors::NumericalOverflow)?
            .checked_div(10_000)
            .ok_or(SrlErrors::NumericalUnderflow)?
            .checked_div(seconds_in_year as u64)
            .ok_or(SrlErrors::NumericalUnderflow)?;
        
        let amount_to_repay = amount_borrowed.checked_add(staking_yield).ok_or(SrlErrors::NumericalUnderflow)?;

        // Check if the StakingAccount is Active or Not
        // - If it's Active: Normal Transfer
        // - If it's Inactive: Withdraw the Stake and Transfer
        let stake_account_info = self.stake_account.to_account_info();
        let stake_account_data = stake_account_info.try_borrow_data()?;

        match StakeStateV2::deserialize(&mut &stake_account_data[..])? {
            StakeStateV2::Stake(_, _, _) => self.repay_principal_to_lender(amount_to_repay)?,
            StakeStateV2::Initialized(_) => self.withdraw_principal_to_lender(amount_to_repay)?,
            _ => return Err(SrlErrors::StakeAccountNotFound.into())
        }
        
        Ok(())
    }

}

pub fn handler<'info>(ctx: Context<RepayLoan>) -> Result<()> {

    // Orderbook Checks - Should we let them repay their order no matter what?
    // require_eq!(ctx.accounts.order_book.locked, true, SrlErrors::OrderBookLocked);

    // Check if we're in the right Loan State
    match ctx.accounts.loan.loan_state {
        LoanState::Taken => {},
        _ => return Err(SrlErrors::WrongLoanState.into())
    } 

    ctx.accounts.repay_loan()?;

    ctx.accounts.change_withdraw_authority()?;
    
    Ok(())
}