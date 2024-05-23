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
    let system_program = &ctx.accounts.system_program;
    let rent = &mut ctx.accounts.rent_sysvar;
    let seller = &mut ctx.accounts.seller;

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
    // they have to be provided in groups: [(bid, bid_vault, stake_account), (bid, bid_vault, stake_account), ...]
    let bids_and_new_stake_accounts = ctx.remaining_accounts;

    require!(
        bids_and_new_stake_accounts.len() % 3 == 0,
        SsmError::InvalidRemainingAccountsSchema
    );

    let groups = (bids_and_new_stake_accounts.len() / 3) as usize;

    for pair in 0..groups {
        let bid_account_info = & bids_and_new_stake_accounts[(pair * 3)];
        let mut bid = Account::<Bid>::try_from(&bid_account_info)?;

        let bid_vault_account_info = & bids_and_new_stake_accounts[(pair * 3) + 1];
        let mut bid_vault = SystemAccount::try_from(bid_vault_account_info)?;
        
        let new_stake_account_info = &bids_and_new_stake_accounts[(pair * 3) + 2];
        msg!("new_stake_account: {}", new_stake_account_info.key());

        if remaining_stake == 0 || bid.amount == 0 {
            continue;
        }

        let stake_to_sell = remaining_stake.min(bid.amount);
        ctx.accounts.split_and_transfer_stake(
            &bid, 
            &bid_vault,
            stake_to_sell, 
            new_stake_account_info.clone(),
            ctx.program_id
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
        bid_vault: &SystemAccount<'info>,
        stake: u64,
        new_stake_account: AccountInfo<'info>,
        program_id: &Pubkey
    ) -> Result<()> {
        let rent: &Sysvar<'info, Rent> = &self.rent_sysvar;
        let sol_to_transfer = ((stake as f64) / 10_f64.powf(9_f64) * (bid.bid_rate as f64)) as u64;

        let mut stake_account_to_authorize = &self.stake_account.to_account_info();

        // Split account if not selling entire stake account.
        if stake < self.stake_account.to_account_info().lamports() {

            // If we're splitting the account, we will be then authorizing the new account,
            // not entire account still owned by user.
            stake_account_to_authorize = &new_stake_account;

            msg!("Splitting the account. Stake to sell: {}. Stake in the account: {}", stake,self.stake_account.to_account_info().lamports());

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

            msg!("Made the new stake account rent exempt.");

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

                msg!("Invoked {}/3 instructions (split).", index + 1);
            }

            // Authorize bidder to stake and withdraw from the new stake account.
            let stake_auth_ix = stake_instruction::authorize(
                &stake_account_to_authorize.key(),
                &self.seller.key(),
                &bid.bidder.key(),
                StakeAuthorize::Staker,
                None,
            );

            let withdraw_auth_ix = stake_instruction::authorize(
                &stake_account_to_authorize.key(),
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
        }

        // Transfer SOL from the bid account to the seller
        let seeds = &[
            "vault".as_bytes(),
            &bid.key().to_bytes(),
        ];

        let (_, bump) = Pubkey::find_program_address(
            seeds, 
            program_id
        );

        let signer_seeds = &[
            "vault".as_bytes(),
            &bid.key().to_bytes(),
            &[bump]
        ];

        transfer(
            CpiContext::new_with_signer(
                self.system_program.to_account_info(), 
                Transfer {
                    from: bid_vault.to_account_info(),
                    to: self.seller.to_account_info()
                },
                &[signer_seeds]
            ), 
            sol_to_transfer,
        )?;

        Ok(())
    }
}