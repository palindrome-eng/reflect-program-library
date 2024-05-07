use anchor_lang::prelude::*;
use anchor_lang::solana_program::system_program;
use anchor_lang::solana_program::{
    clock::Clock,
    stake::{self, instruction as stake_instruction, state::{StakeAuthorize, StakeStateV2}},
    system_instruction,
    program::{invoke, invoke_signed},
    pubkey::Pubkey,
    sysvar::{self, rent::Rent}
};

use crate::errors::SsmError;
use crate::states::{Bid, OrderBook};

#[derive(Accounts)]
pub struct SellStake<'info> {
    #[account(mut)]
    /// CHECK: The caller must ensure this is a valid stake account.
    pub stake_account: UncheckedAccount<'info>,

    #[account(mut)]
    pub order_book: Account<'info, OrderBook>,

    #[account(mut)]
    pub seller: Signer<'info>,

    #[account(address = anchor_lang::solana_program::stake::program::ID)]
    pub stake_program: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    #[account(address = sysvar::rent::ID)]
    pub rent_sysvar: Sysvar<'info, Rent>,
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
        if sol_amount < self.stake_account.lamports() {
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
                &[&[&self.seller.key().to_bytes(), seed.as_bytes()]],
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

    pub fn sell_stake(&self, ctx: Context<SellStake>, total_stake_amount: u64) -> Result<()> {
        let mut remaining_stake = total_stake_amount;

        // Assume bids are fetched and passed in sorted order as part of `remaining_accounts`
        let mut bids: Vec<Account<Bid>> = ctx.remaining_accounts.iter().map(Account::try_from).collect::<Result<Vec<_>>>()?;

        for bid in bids.iter_mut() {
            if remaining_stake == 0 || bid.amount == 0 {
                continue;
            }

            let stake_to_sell = remaining_stake.min(bid.amount);
            self.split_and_transfer_stake(bid, stake_to_sell)?;
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
}
