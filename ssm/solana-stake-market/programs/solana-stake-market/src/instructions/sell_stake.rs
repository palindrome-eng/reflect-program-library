use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Epoch;
use anchor_lang::solana_program::stake::instruction::split;
use anchor_lang::solana_program::{
    stake::{instruction as stake_instruction, state::{
        StakeAuthorize,
        StakeStateV2,
    }},
    program::invoke,
    pubkey::Pubkey,
    sysvar::{self, rent::Rent}
};
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::stake::StakeAccount;
use crate::constants::{ORDERBOOK_SEED, VAULT_SEED};
use crate::errors::SsmError;
use crate::states::{Bid, OrderBook};

pub fn sell_stake<'info>(
    ctx: Context<'_, '_, 'info, 'info, SellStake<'info>>, 
    total_stake_amount: u64
) -> Result<()> {
    let seller = &ctx.accounts.seller;
    let system_program = &ctx.accounts.system_program;
    let stake_account = &ctx.accounts.stake_account;
    let clock = Clock::get();

    let stake_account_activation = stake_account
        .delegation()
        .ok_or(SsmError::StakeAccountDelegationNotFound)?
        .activation_epoch;

    // Inactive stake cannot be sold.
    require!(
        clock.unwrap().epoch > stake_account_activation,
        SsmError::StakeNotActivated
    );

    // For each bid there has to be corresponding new stake account, because of the splitting.
    // They have to be provided in tuples: 
    // [
    //  (bid, bid_vault, stake_account), 
    //  (bid, bid_vault, stake_account), 
    //  ...
    // ]
    let bids_and_new_stake_accounts = ctx.remaining_accounts;

    // Require tuples to be in the above schema.
    require!(
        bids_and_new_stake_accounts.len() % 3 == 0,
        SsmError::InvalidRemainingAccountsSchema
    );

    // Remaining stake to sell into bids.
    // Will subtract from this value after each bid.
    let mut remaining_stake = total_stake_amount;

    let groups = (bids_and_new_stake_accounts.len() / 3) as usize;

    for group in 0..groups {
        let bid_account_info = & bids_and_new_stake_accounts[group * 3];
        let mut bid_data = bid_account_info.try_borrow_mut_data()?;
        let mut bid = Bid::try_deserialize(&mut bid_data.as_ref())?;

        let bid_vault_account_info = & bids_and_new_stake_accounts[(group * 3) + 1];
        let bid_vault = SystemAccount::try_from(bid_vault_account_info)?;
        
        let new_stake_account_info = &bids_and_new_stake_accounts[(group * 3) + 2];

        // In case user provides more bids than needed, `continue` will be called till the end of the loop.
        // In case the bid got emptied in the meantime (between generating and executing the instruction), skip this bid. 
        // In the 2nd scenario, it's likely that transaction will fail anyway.
        if (remaining_stake == 0 || bid.amount == 0) {
            continue;
        }

        // Sell the max amount that the current bid can fit.
        let stake_to_sell = remaining_stake.min(bid.amount);

        msg!(
            "Selling {} SOL into bid no. {}. Remaining stake in the account: {}", 
            (stake_to_sell as f64) / 10_f64.powf(9_f64), 
            group,
            (stake_account.to_account_info().lamports() as f64) / 10_f64.powf(9_f64)
        );

        // Transfer SOL from the bid vault to the stake seller.
        let sol_to_transfer = ((stake_to_sell as f64) / 10_f64.powf(9_f64) * (bid.rate as f64)) as u64;

        let seeds = &[
            VAULT_SEED.as_bytes(),
            &bid_account_info.key().to_bytes(),
        ];

        let (_, bump) = Pubkey::find_program_address(
            seeds, 
            ctx.program_id
        );

        let signer_seeds = &[
            VAULT_SEED.as_bytes(),
            &bid_account_info.key().to_bytes(),
            &[bump]
        ];

        transfer(
            CpiContext::new_with_signer(
                system_program.to_account_info(), 
                Transfer {
                    from: bid_vault.to_account_info(),
                    to: seller.to_account_info()
                },
                &[signer_seeds]
            ), 
            sol_to_transfer,
        )?;

        // Split the seller's stake account if necessary.
        // Transfer stake from seller to the owner of the bid.
        ctx.accounts.split_and_transfer_stake(
            &bid.bidder, 
            stake_to_sell, 
            new_stake_account_info.clone(),
        )?;

        // Deduct the sold stake from remaining.
        remaining_stake -= stake_to_sell;

        // Deduct the sold stake from bid.
        bid.partial_fill(stake_to_sell);

        // Deduct the sold stake from the orderbook TVL.
        ctx.accounts.order_book.subtract_ask(stake_to_sell);

        // Serialize account changes back to the account.
        bid.try_serialize(&mut bid_data.as_mut())?;

        // In case user provided more bids than necessary, break loop
        // as soon as there is no more stake to sell.
        if remaining_stake == 0 {
            break;
        }
    }

    // In case there was not enough bids to sell entire amount, fail the instruction.
    require!(
        remaining_stake == 0, 
        SsmError::InsufficientBids
    );

    Ok(())
}

impl<'info> SellStake<'info> {
    fn split_and_transfer_stake(
        &self,
        bidder: &Pubkey,
        stake: u64,
        new_stake_account: AccountInfo<'info>
    ) -> Result<()> {
        let rent: &Sysvar<'info, Rent> = &self.rent_sysvar;

        // Mutable. If account needs splitting, we will update this variable to new_stake_account.
        let mut stake_account_to_authorize = &self.stake_account.to_account_info();

        // Split account if not selling entire stake account.
        if stake < self.stake_account.to_account_info().lamports() {

            // If we're splitting the account, we will be then authorizing the new account,
            // not entire account still owned by seller.
            stake_account_to_authorize = &new_stake_account;

            msg!(
                "Splitting the account. Stake to sell: {}. Stake in the account: {}", 
                stake,
                self.stake_account.to_account_info().lamports()
            );

            let stake_account_rent = rent.minimum_balance(StakeStateV2::size_of() as usize);

            transfer(
                CpiContext::new(
                    self.system_program.to_account_info(), 
                    Transfer {
                        from: self.seller.to_account_info(),
                        to: new_stake_account.clone()
                    }
                ), 
                stake_account_rent
            )?;

            let split_ix = split(
                &self.stake_account.key(), 
                &self.seller.key(), 
                stake,
                &new_stake_account.key()
            );

            for (index, ix) in split_ix.iter().enumerate() {
                let account_infos = &[
                    self.stake_account.to_account_info(),
                    self.seller.to_account_info(),
                    new_stake_account.to_account_info(),
                    self.stake_program.to_account_info(),
                    self.system_program.to_account_info(),
                ];

                invoke(
                    ix,
                    account_infos
                )?;

                msg!("Invoked {}/3 of split instructions.", index + 1);
            }
        }

        // Authorize bidder to stake and withdraw from the stake account being sold.
        // In case entire stake account fits into this bid, entire stake account will be authorized to the new owner.
        let stake_auth_ix = stake_instruction::authorize(
            &stake_account_to_authorize.key(),
            &self.seller.key(),
            bidder,
            StakeAuthorize::Staker,
            None,
        );

        let withdraw_auth_ix = stake_instruction::authorize(
            &stake_account_to_authorize.key(),
            &self.seller.key(),
            bidder,
            StakeAuthorize::Withdrawer,
            None,
        );

        invoke(
            &stake_auth_ix,
            &[
                self.seller.to_account_info(),
                self.stake_program.to_account_info(),
                stake_account_to_authorize.to_account_info(),
                self.clock.to_account_info()
            ],
        )?;

        msg!("Authorized stake.");

        invoke(
            &withdraw_auth_ix,
            &[
                self.seller.to_account_info(),
                self.stake_program.to_account_info(),
                stake_account_to_authorize.to_account_info(),
                self.clock.to_account_info()
            ],
        )?;

        msg!("Authorized withdrawal.");

        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(
    total_stake_amount: u64
)]
pub struct SellStake<'info> {
    #[account(
        mut
    )]
    pub seller: Signer<'info>,

    #[account(
        mut,

        // Make sure the user is authorised to withdraw from stake account.
        constraint = stake_account
            .authorized()
            .ok_or(SsmError::StakeAccountAuthorizationNotFound)?
            .withdrawer == seller.key(),

        // Not sure if this is necessary.
        // Check if staker (stake authority?) is the same person as seller.
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
        seeds = [
            ORDERBOOK_SEED.as_bytes()
        ],
        bump
    )]
    pub order_book: Account<'info, OrderBook>,

    /// CHECK: Program ID is directly checked.
    #[account(
        address = anchor_lang::solana_program::stake::program::ID
    )]
    pub stake_program: AccountInfo<'info>,

    #[account(
        address = sysvar::rent::ID
    )]
    pub rent_sysvar: Sysvar<'info, Rent>,

    #[account(
        address = sysvar::clock::ID
    )]
    pub clock: Sysvar<'info, Clock>,

    pub system_program: Program<'info, System>
}