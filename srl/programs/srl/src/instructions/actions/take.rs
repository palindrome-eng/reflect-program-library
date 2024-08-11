use anchor_lang::{
    prelude::*, 
    solana_program::{
        clock::Epoch, 
        program::invoke, 
        stake::{instruction::{authorize, split}, program::ID as STAKE_PROGRAM_ID, state::{StakeAuthorize, StakeStateV2}}
    }, 
    system_program::{transfer, Transfer}  
};

use anchor_spl::stake::StakeAccount;

use crate::{state::{Loan, LoanState, OrderBook}, errors::SrlErrors};

#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    /// CHECK: checked in the Smart Contract
    pub loan_owner: UncheckedAccount<'info>,
    #[account(mut)]
    pub stake_account: Account<'info, StakeAccount>,
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
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> Take<'info> {

    pub fn send_fees_to_orderbook(&mut self, fee_amount: u64, bump: Option<u8>) -> Result<()> {

        match self.loan.loan_state {
            LoanState::Ask => {
                transfer(
                    CpiContext::new(
                        self.system_program.to_account_info(),
                        Transfer {
                            from: self.taker.to_account_info(),
                            to: self.order_book.to_account_info()
                        }
                    ),
                    fee_amount        
                )?;
            },
            LoanState::Bid => {
                // Create the Signers Seeds
                let loan_key = self.loan.key();
                let signer_seeds = &[b"bid_vault", loan_key.as_ref(), &[bump.ok_or(SrlErrors::BumpNotFound)?]];

                transfer(
                    CpiContext::new_with_signer(
                        self.system_program.to_account_info(),
                        Transfer {
                            from: self.bid_vault.to_account_info(),
                            to: self.order_book.to_account_info()
                        },
                        &[signer_seeds]
                    ),
                    fee_amount        
                )?;
            },
            _ => return Err(SrlErrors::WrongLoanState.into())
        }

        Ok(())
    }

    pub fn send_principal_to_borrower(&mut self, principal_amount: u64, bump: Option<u8>) -> Result<()> {

        match self.loan.loan_state {
            LoanState::Ask => {
                transfer(
                    CpiContext::new(
                        self.system_program.to_account_info(),
                        Transfer {
                            from: self.taker.to_account_info(),
                            to: self.loan_owner.to_account_info()
                        }
                    ),
                    principal_amount        
                )?;
            },
            LoanState::Bid => {
                // Create the Signers Seeds
                let loan_key = self.loan.key();
                let signer_seeds = &[b"bid_vault", loan_key.as_ref(), &[bump.ok_or(SrlErrors::BumpNotFound)?]];

                transfer(
                    CpiContext::new_with_signer(
                        self.system_program.to_account_info(),
                        Transfer {
                            from: self.bid_vault.to_account_info(),
                            to: self.loan_owner.to_account_info()
                        },
                        &[signer_seeds]
                    ),
                    principal_amount        
                )?;
            },
            _ => return Err(SrlErrors::WrongLoanState.into())
        }

        Ok(())
    }
    
    pub fn split_stake_account(&mut self, new_stake_account: &AccountInfo<'info>, amount: u64) -> Result<()> {

        let stake_account_rent = self.rent.minimum_balance(StakeStateV2::size_of() as usize);

        transfer(
            CpiContext::new(
                self.system_program.to_account_info(), 
                Transfer {
                    from: self.taker.to_account_info(),
                    to: new_stake_account.clone()
                }
            ), 
            stake_account_rent
        )?;

        let split_ix = split(
            &self.stake_account.key(), 
            &self.taker.key(), 
            amount,
            &new_stake_account.key()
        );

        for (_, ix) in split_ix.iter().enumerate() {
            let account_infos = &[
                self.stake_account.to_account_info(),
                self.taker.to_account_info(),
                new_stake_account.to_account_info(),
                self.stake_program.to_account_info(),
                self.system_program.to_account_info(),
            ];

            invoke(ix, account_infos)?;
        }

        Ok(())
    }

    pub fn change_withdraw_authority (&mut self) -> Result<()> {

        let change_withdraw_auth_ix = authorize(
            &self.stake_account.key(),
            &self.taker.key(),
            &self.order_book.key(),
            StakeAuthorize::Withdrawer,
            None,
        );

        let account_infos = &[
            self.taker.to_account_info(),
            self.stake_program.to_account_info(),
            self.stake_account.to_account_info(),
            self.clock.to_account_info()
        ];

        invoke(&change_withdraw_auth_ix, account_infos)?;

        Ok(())
    }

    pub fn handle_ask_loan(&mut self) -> Result<()> {

        // Check if the Borrower is the Loan Owner
        require!(self.loan.borrower == Some(self.loan_owner.key()), SrlErrors::WrongBorrwer);

        // Check if the Stake Account is the same as the one in the Loan State
        require!(self.loan.stake_account == Some(self.stake_account.key()), SrlErrors::WrongStakeAccount);

        // Fill the Lender field of the Loan
        self.loan.lender = Some(self.taker.key());

        // Calculate the protocol fees
        let fee_amount = self.loan.loan_terms.loan_amount
            .checked_mul(self.order_book.fee_basis_points as u64)
            .ok_or(SrlErrors::NumericalOverflow)?
            .checked_div(10_000)
            .ok_or(SrlErrors::NumericalUnderflow)?;

        let principal_amount = self.loan.loan_terms.loan_amount
            .checked_sub(fee_amount)
            .ok_or(SrlErrors::NumericalUnderflow)?;

        // Send the requested lamports to the borrower
        self.send_principal_to_borrower(principal_amount, None)?;

        // Send the protocol fees to the Order Book
        self.send_fees_to_orderbook(fee_amount, None)?;

        Ok(())
    }

    pub fn handle_bid_loan(&mut self, remaining_accounts: &[AccountInfo<'info>], bump: u8) -> Result<()> {

        let stake_account = self.stake_account.clone();
        let loan = self.loan.clone();

        // Add Stake Account Checks
        require!(
            stake_account
                .authorized()
                .ok_or(SrlErrors::StakeAccountAuthorizationNotFound)?
                .withdrawer == self.taker.key(),
            SrlErrors::WrongStakeAccountWithdrawAuthority
        );

        require!(
            stake_account
                .delegation()
                .ok_or(SrlErrors::StakeAccountDelegationNotFound)?
                .deactivation_epoch == Epoch::MAX,
            SrlErrors::WrongStakeAccountDeactivationEpoch
        );
    
        require!(
            !stake_account
                .lockup()
                .ok_or(SrlErrors::StakeAccountLockupNotFound)?
                .is_in_force(&self.clock, None),
            SrlErrors::WrongStakeAccountLockupInForce
        );

        // Check if the Lender is the Loan Owner
        require!(self.loan.lender == Some(self.loan_owner.key()), SrlErrors::WrongLender);

        // Fill the Borrower field of the Loan
        self.loan.borrower = Some(self.taker.key());
    
        // Calculate the current Stake Amount and check if it is greater than the loan amount
        let stake_amount = self.stake_account.delegation().ok_or(SrlErrors::StakeAccountDelegationNotFound)?.stake;
        require_gte!(stake_amount, loan.loan_terms.loan_amount, SrlErrors::NotEnughStakeAmount);

        if loan.loan_terms.loan_amount < stake_amount {
            let new_stake_account = remaining_accounts.first().ok_or(SrlErrors::InvalidRemainingAccountsSchema)?;
            let amount_to_split = stake_amount.checked_sub(loan.loan_terms.loan_amount).ok_or(SrlErrors::NumericalUnderflow)?;

            self.split_stake_account(new_stake_account, amount_to_split)?;
        }

        self.change_withdraw_authority()?;

        // Send the protocol fees to the Order Book
        self.send_fees_to_orderbook(loan.loan_terms.loan_amount, Some(bump))?;

        // Send the principal amount to the Borrower
        self.send_principal_to_borrower(loan.loan_terms.loan_amount, Some(bump))?;

        Ok(())
    }
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, Take<'info>>) -> Result<()> {

    // Orderbook Checks
    require_eq!(ctx.accounts.order_book.locked, true, SrlErrors::OrderBookLocked);

    // Get Loan Account
    let loan = ctx.accounts.loan.clone();

    // Get Bid Vault Bump
    let bump = ctx.bumps.bid_vault;

    match loan.loan_state {
        LoanState::Ask => ctx.accounts.handle_ask_loan()?,
        LoanState::Bid => ctx.accounts.handle_bid_loan(ctx.remaining_accounts, bump)?,
        _ => return Err(SrlErrors::WrongLoanState.into())
    }

    ctx.accounts.loan.loan_terms.starting_time = Some(Clock::get()?.unix_timestamp);
    
    Ok(())
}