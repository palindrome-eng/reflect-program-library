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

use crate::{
    errors::SrlErrors, 
    state::{Loan, LoanState, LoanTerms, OrderBook}
};

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct PlaceAskArgs {
    pub loan_id: u64,
    pub loan_amount: u64,
    pub loan_to_value: u16,
    pub loan_duration: u64,
}

#[derive(Accounts)]
#[instruction(args: PlaceAskArgs)]
pub struct PlaceAsk<'info> {
    #[account(mut)]
    pub borrower: Signer<'info>,
    #[account(
        mut,

        // Make sure the user is authorised to withdraw from stake account.
        constraint = stake_account
            .authorized()
            .ok_or(SrlErrors::StakeAccountAuthorizationNotFound)?
            .withdrawer == borrower.key(),

        // Meaning stake is not deactivated.
        constraint = stake_account
            .delegation()
            .ok_or(SrlErrors::StakeAccountDelegationNotFound)?
            .deactivation_epoch == Epoch::MAX,

        // Make sure lockup is not in force.
        constraint = !stake_account
            .lockup()
            .ok_or(SrlErrors::StakeAccountLockupNotFound)?
            .is_in_force(&clock, None),
    )]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(
        seeds = [b"order_book", OrderBook::CURRENT_VERSION.to_le_bytes().as_ref()],
        bump = order_book.bump,
    )]
    pub order_book: Account<'info, OrderBook>,
    #[account(
        init,
        payer = borrower,
        space = Loan::INIT_SPACE,
        seeds = [b"loan", args.loan_id.to_le_bytes().as_ref()],
        bump
    )]
    pub loan: Account<'info, Loan>,

    pub system_program: Program<'info, System>,
    #[account(address = STAKE_PROGRAM_ID)]
    /// CHECK: checked by address constraint
    pub stake_program: UncheckedAccount<'info>,
    pub rent: Sysvar<'info, Rent>,
    pub clock: Sysvar<'info, Clock>,
}

impl<'info> PlaceAsk<'info> {
    pub fn initialize_loan(&mut self, args: PlaceAskArgs, bump: u8) -> Result<()> {

        // Loan Amount Calculation to do on the Frontend
        // let loan_amount = args.loan_amount
        //     .checked_mul(args.loan_to_value as u64)
        //     .ok_or(SrlErrors::NumericalOverflow)?
        //     .checked_div(10_000)
        //     .ok_or(SrlErrors::NumericalUnderflow)?;

        let loan_terms = LoanTerms {
            loan_amount: args.loan_amount,
            loan_to_value: args.loan_to_value,
            loan_duration: args.loan_duration,
            starting_time: None
        };

        let loan_state = LoanState::Ask;

        self.loan.set_inner(
            Loan {
                loan_state,
                order_book: self.order_book.key(),
                loan_terms,
                borrower: Some(self.borrower.key()),
                lender: None,
                stake_account: Some(self.stake_account.key()),    
                id: args.loan_id,
                bump,
            }
        );

        Ok(())
    }

    pub fn split_stake_account(&mut self, new_stake_account: &AccountInfo<'info>, amount: u64) -> Result<()> {

        let stake_account_rent = self.rent.minimum_balance(StakeStateV2::size_of() as usize);

        transfer(
            CpiContext::new(
                self.system_program.to_account_info(), 
                Transfer {
                    from: self.borrower.to_account_info(),
                    to: new_stake_account.clone()
                }
            ), 
            stake_account_rent
        )?;

        let split_ix = split(
            &self.stake_account.key(), 
            &self.borrower.key(), 
            amount,
            &new_stake_account.key()
        );

        for (_, ix) in split_ix.iter().enumerate() {
            let account_infos = &[
                self.stake_account.to_account_info(),
                self.borrower.to_account_info(),
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
            &self.borrower.key(),
            &self.order_book.key(),
            StakeAuthorize::Withdrawer,
            None,
        );

        let account_infos = &[
            self.borrower.to_account_info(),
            self.stake_program.to_account_info(),
            self.stake_account.to_account_info(),
            self.clock.to_account_info()
        ];

        invoke(&change_withdraw_auth_ix, account_infos)?;

        Ok(())
    }
}

pub fn handler<'info>(ctx: Context<'_, '_, '_, 'info, PlaceAsk<'info>>, args: PlaceAskArgs) -> Result<()> {

    // Orderbook Checks
    require_eq!(ctx.accounts.order_book.locked, true, SrlErrors::OrderBookLocked);

    // Get the bump
    let bump = ctx.bumps.loan;

    // Initialize the loan
    ctx.accounts.initialize_loan(args.clone(), bump)?;

    // If the staked amount is greater than the amount required, split the stake account
    let stake_amount = ctx.accounts.stake_account.delegation().ok_or(SrlErrors::StakeAccountDelegationNotFound)?.stake;
    if stake_amount > args.loan_amount {
        let new_stake_account = ctx.remaining_accounts.first().ok_or(SrlErrors::InvalidRemainingAccountsSchema)?;
        ctx.accounts.split_stake_account(new_stake_account, stake_amount.checked_sub(args.loan_amount).ok_or(SrlErrors::NumericalUnderflow)?)?;
    }

    // Give the Withdraw Authority of the Staking Account to the Order Book
    ctx.accounts.change_withdraw_authority()?;

    // Add the TVL data inside of the Order Book
    ctx.accounts.order_book.tvl = ctx.accounts.order_book.tvl.checked_add(stake_amount).ok_or(SrlErrors::NumericalOverflow)?;
    
    Ok(())
}