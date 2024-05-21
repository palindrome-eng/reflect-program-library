//src/instructions/stake_manager.rs
use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Epoch;
use anchor_lang::solana_program::system_program;
use anchor_lang::solana_program::stake::state::Stake;
use anchor_lang::solana_program::{
    stake::{self, instruction as stake_instruction, state::{
        StakeAuthorize,
        StakeStateV2
    }},
    system_instruction,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    sysvar::{self, rent::Rent}
};
use anchor_spl::stake::StakeAccount;

use crate::errors::SsmError;
use crate::states::{Bid, OrderBook};

#[derive(Accounts)]
#[instruction(
    total_stake_amount: u64
)]
pub struct SellStake<'info> {
    #[account(
        mut,

        // Make sure the user is authorised to withdraw from stake account.
        constraint = stake_account
            .authorized()
            .ok_or(SsmError::StakeAccountAuthorizationNotFound)?
            .withdrawer == seller.key(),

        // Not sure if this is necessary.
        constraint = stake_account
        .authorized()
        .ok_or(SsmError::StakeAccountAuthorizationNotFound)?
        .staker == seller.key(),

        // Meaning stake is not deactivated.
        constraint = stake_account
            .delegation()
            .ok_or(SsmError::StakeAccountDelegationNotFound)?
            .deactivation_epoch == Epoch::MAX,

        // Check if there's enough lamports in the stake account.
        constraint = stake_account.to_account_info().lamports() > total_stake_amount,

        // Make sure lockup is not in force.
        constraint = stake_account
            .lockup()
            .ok_or(SsmError::StakeAccountLockupNotFound)?
            .is_in_force(&clock, None),

    )]
    pub stake_account: Account<'info, StakeAccount>,

    #[account(
        mut,
        seeds = [b"orderBook"],
        bump
    )]
    pub order_book: Account<'info, OrderBook>,

    #[account(
        mut
    )]
    pub seller: Signer<'info>,

    #[account(address = anchor_lang::solana_program::stake::program::ID)]
    /// CHECK: safe because pulling directly from solana_program the stake program ID. (caller can also validate)
    pub stake_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,

    #[account(address = sysvar::rent::ID)]
    pub rent_sysvar: Sysvar<'info, Rent>,

    pub clock: Sysvar<'info, Clock>,
}

pub fn sell_stake<'info>(
    ctx: Context<'_, '_, 'info, 'info, SellStake<'info>>, 
    total_stake_amount: u64
) -> Result<()> {
    let mut remaining_stake = total_stake_amount;
    let stake_account = &mut ctx.accounts.stake_account;

    let stake_account_activation = stake_account
        .delegation()
        .ok_or(SsmError::StakeAccountDelegationNotFound)?
        .activation_epoch;

    let clock = Clock::get();

    require!(
        clock.unwrap().epoch > stake_account_activation,
        SsmError::StakeNotActivated
    );

    // Assume bids are fetched and passed in sorted order as part of `remaining_accounts`
    let mut bids: Vec<Account<'info, Bid>> = ctx.remaining_accounts
        .iter()
        .map(|account_info| Account::<Bid>::try_from(account_info))
        .collect::<Result<Vec<_>>>()?;

    for bid in bids.iter_mut() {
        if remaining_stake == 0 || bid.amount == 0 {
            continue;
        }

        let stake_to_sell = remaining_stake.min(bid.amount);
        ctx.accounts.split_and_transfer_stake(bid, stake_to_sell)?;
        remaining_stake -= stake_to_sell;

        bid.amount -= stake_to_sell;
        bid.fulfilled = bid.amount == 0;
        ctx.accounts.order_book.tvl -= stake_to_sell;

        if remaining_stake == 0 {
            break;
        }
    }

    require!(remaining_stake == 0, SsmError::InsufficientBids);
    Ok(())
}

impl<'info> SellStake<'info> {
    fn split_and_transfer_stake(
        &self,
        bid: &mut Account<'info, Bid>,
        amount: u64,
    ) -> Result<()> {
        let stake_account_key = self.stake_account.key();
        let sol_amount = (amount as f64 / bid.bid_rate as f64).ceil() as u64;

        // Initialize new stake account only if splitting is necessary
        if sol_amount < self.stake_account.to_account_info().lamports() {
            let seed = format!("split{}", bid.bidder);
            let new_stake_account_key = Pubkey::create_with_seed(
                &self.seller.key(),
                &seed,
                &self.stake_program.key(),
            ).map_err(|_| SsmError::PublicKeyCreationFailed)?;

            // Create and initialize the new stake account
            let rent = &self.rent_sysvar;
            let lamports_needed = rent.minimum_balance(StakeStateV2::size_of() as usize);
            let create_acc_ix = system_instruction::create_account_with_seed(
                &self.seller.key(),
                &new_stake_account_key,
                &self.seller.key(),
                &seed,
                lamports_needed,
                StakeStateV2::size_of() as u64,
                &system_program::ID,
            );

            let init_stake_ix = stake_instruction::initialize(
                &new_stake_account_key,
                &stake::state::Authorized::auto(&self.seller.key()),
                &stake::state::Lockup::default(),
            );

            invoke_signed(
                &create_acc_ix,
                &[
                    self.seller.to_account_info(),
                    self.stake_program.to_account_info(),
                    self.system_program.to_account_info(),
                ],
                &[&[self.seller.key().as_ref(), seed.as_bytes()]],
            )?;

            invoke(
                &init_stake_ix,
                &[self.stake_program.to_account_info()],
            )?;
        }

        // Authorize the bidder and transfer stake
        let auth_ix = stake_instruction::authorize(
            &stake_account_key,
            &self.seller.key(),
            &bid.bidder,
            StakeAuthorize::Staker,
            None,
        );

        invoke(
            &auth_ix,
            &[
                self.stake_account.to_account_info(),
                self.seller.to_account_info(),
                self.stake_program.to_account_info(),
            ],
        )?;

        // Transfer SOL from the bid account to the seller
        let transfer_ix = system_instruction::transfer(
            &bid.to_account_info().key(),
            &self.seller.to_account_info().key(),
            sol_amount,
        );

        invoke(
            &transfer_ix,
            &[
                bid.to_account_info(),
                self.seller.to_account_info(),
                self.system_program.to_account_info(),
            ],
        )?;

        Ok(())
    }
}