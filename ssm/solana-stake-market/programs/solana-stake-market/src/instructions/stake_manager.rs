//src/instructions/stake_manager.rs
use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Epoch;
use anchor_lang::solana_program::stake::instruction::split;
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
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::stake::StakeAccount;

use crate::errors::SsmError;
use crate::states::{order_book, Bid, OrderBook};

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
        constraint = !stake_account
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

    #[account(
        address = sysvar::clock::ID
    )]
    pub clock: Sysvar<'info, Clock>,
}

pub fn sell_stake<'info>(
    ctx: Context<'_, '_, 'info, 'info, SellStake<'info>>, 
    total_stake_amount: u64
) -> Result<()> {
    let mut remaining_stake = total_stake_amount;
    let stake_account = &ctx.accounts.stake_account;

    let stake_account_activation = stake_account
        .delegation()
        .ok_or(SsmError::StakeAccountDelegationNotFound)?
        .activation_epoch;

    let clock = Clock::get();

    require!(
        clock.unwrap().epoch > stake_account_activation,
        SsmError::StakeNotActivated
    );

    // Since for each bid there has to be corresponding new stake account,
    // they have to be provided in pairs: [(bid, stake account), (bid, stake account), ...]
    let bids_and_new_stake_accounts = ctx.remaining_accounts;

    require!(
        bids_and_new_stake_accounts.len() % 2 == 0,
        SsmError::InvalidRemainingAccountsSchema
    );

    let pairs = (bids_and_new_stake_accounts.len() / 2) as usize;

    msg!("num of utilized bids: {}", pairs);

    for pair in 0..pairs {
        let bid_account_info = & bids_and_new_stake_accounts[(pair * 2)];
        let mut bid = Account::<Bid>::try_from(&bid_account_info)?;
        
        let new_stake_account_info = &bids_and_new_stake_accounts[(pair * 2) + 1];

        if remaining_stake == 0 || bid.amount == 0 {
            continue;
        }

        let stake_to_sell = remaining_stake.min(bid.amount);
        ctx.accounts.split_and_transfer_stake(
            &bid, 
            stake_to_sell, 
            new_stake_account_info.clone()
        )?;

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
        bid: &Account<'info, Bid>,
        amount: u64,
        new_stake_account: AccountInfo<'info>
    ) -> Result<()> {
        let sol_amount = (amount as f64 / bid.bid_rate as f64).ceil() as u64;

        // Transfer SOL from the bid account to the seller
        // bid.sub_lamports(sol_amount)?;
        // self.seller.add_lamports(sol_amount)?;

        msg!("bid.amount: {}", bid.amount);
        msg!("amount left to sell: {}", amount);

        // Initialize new stake account only if splitting is necessary
        if sol_amount < self.stake_account.to_account_info().lamports() {
            let rent = &self.rent_sysvar;
            let lamports_needed = rent.minimum_balance(StakeStateV2::size_of() as usize);
            
            transfer(
                CpiContext::new(
                    self.system_program.to_account_info(), 
                    Transfer {
                        from: self.seller.to_account_info(),
                        to: new_stake_account.to_account_info()
                    }
                ), 
                lamports_needed
            )?;

            msg!("transferred rent to the new account.");

            let split_ix = split(
                &self.stake_account.key(), 
                &self.seller.key(), 
                bid.amount, 
                &new_stake_account.key()
            );

            for (index, ix) in split_ix.iter().enumerate() {
                invoke(
                    ix,
                    &[
                        self.stake_account.to_account_info(),
                        self.seller.to_account_info(),
                        new_stake_account.to_account_info(),
                        self.stake_program.to_account_info(),
                        self.system_program.to_account_info(),
                    ]
                )?;

                msg!("invoked ix no. {}", index);
            }
        }

        // Authorize the bidder and transfer stake
        let stake_auth_ix = stake_instruction::authorize(
            &new_stake_account.key(),
            &self.seller.key(),
            &bid.bidder.key(),
            StakeAuthorize::Staker,
            None,
        );

        let withdraw_auth_ix = stake_instruction::authorize(
            &new_stake_account.key(),
            &self.seller.key(),
            &bid.bidder.key(),
            StakeAuthorize::Withdrawer,
            None,
        );

        invoke(
            &stake_auth_ix,
            &[
                self.seller.to_account_info(),
                self.stake_program.to_account_info(),
                new_stake_account.to_account_info(),
                self.clock.to_account_info()
            ],
        )?;

        msg!("authorized stake");

        invoke(
            &withdraw_auth_ix,
            &[
                self.seller.to_account_info(),
                self.stake_program.to_account_info(),
                new_stake_account.to_account_info(),
                self.clock.to_account_info()
            ],
        )?;

        msg!("authorized withdrawal");

        Ok(())
    }
}