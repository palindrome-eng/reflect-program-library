use anchor_lang::prelude::*;
use crate::constants::*;
use crate::states::*;
use anchor_spl::token::{
    Mint,
    TokenAccount
};
use crate::events::*;
use crate::errors::InsuranceFundError;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct SlashDepositsArgs {
    lockup_id: u64,
    slash_id: u64,
    // Total amount to be slashed from the pool. We will calculate % of the entire pool and update
    // all deposits respectively.
    slash_amount: u64,
}

pub fn slash_deposits(
    ctx: Context<SlashDeposits>,
    args: SlashDepositsArgs
) -> Result<()> {
    let SlashDepositsArgs {
        slash_amount,
        lockup_id,
        slash_id
    } = args;

    let settings = &ctx.accounts.settings;
    let lockup = &ctx.accounts.lockup;
    let asset_lockup = &ctx.accounts.asset_lockup;
    let cold_wallet_vault = &ctx.accounts.cold_wallet_vault;
    let slash = &ctx.accounts.slash;

    let deposits = ctx.remaining_accounts;

    // Number of already slashed accounts.
    let starting_index = slash.slashed_accounts;

    let total_deposit = asset_lockup.amount + cold_wallet_vault.amount;
    let slashed_share_bps = 10_000 * slash_amount / total_deposit;

    msg!("total_deposit: {:?}", total_deposit);
    msg!("to slash: {:?}", slash_amount);
    msg!("slashed_share_bps: {:?}", slashed_share_bps);

    require!(
        slash_amount == slash.target_amount,
        InsuranceFundError::SlashAmountMismatch
    );

    let SharesConfig {
        cold_wallet_share_bps,
        hot_wallet_share_bps
    } = settings.shares_config;

    for (index, deposit_account_info) in deposits.iter().enumerate() {
        ctx.accounts.validate_deposit(
            deposit_account_info,
            starting_index + (index as u64),
            ctx.program_id
        )?;

        let deposit_data = deposit_account_info.try_borrow_mut_data()?;
        let mut deposit = Deposit::try_deserialize(&mut deposit_data.as_ref())?;

        msg!("deposit: {:?}", deposit.amount);

        // We only slash 30% (hot_wallet_share_bps)
        // 70% of the slash will be taken from the cold wallet
        // to make sure that the ratio is preserved.
        let slashed = deposit.amount * hot_wallet_share_bps / 10_000 * slashed_share_bps / 10_000;
        msg!("slashed: {:?}", slashed);

        deposit.slash(slashed, slash_id)?;
        {
            let slash = &mut ctx.accounts.slash;
            slash.slash_account(slashed)?;
        }
    }

    let slash = &ctx.accounts.slash;
    emit!(ProcessSlashEvent {
        progress_accounts: slash.slashed_accounts,
        target_accounts: slash.target_accounts,
        progress_amount: slash.slashed_amount,
        target_amount: slash.target_amount
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: SlashDepositsArgs
)]
pub struct SlashDeposits<'info> {
    #[account(
        mut
    )]
    pub superadmin: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        has_one = superadmin,
        constraint = !settings.frozen @ InsuranceFundError::Frozen
    )]
    pub settings: Account<'info, Settings>,

    #[account(
        mut,
        seeds = [
            LOCKUP_SEED.as_bytes(),
            &args.lockup_id.to_le_bytes()
        ],
        bump,
    )]
    pub lockup: Account<'info, Lockup>,

    #[account(
        mut,
        seeds = [
            SLASH_SEED.as_bytes(),
            lockup.key().as_ref(),
            &args.slash_id.to_le_bytes()
        ],
        bump,
        constraint = slash.index == args.slash_id @ InsuranceFundError::InvalidInput,
        constraint = slash.index == lockup.slash_state.index @ InsuranceFundError::InvalidInput,
        constraint = slash.slashed_accounts <= lockup.deposits @ InsuranceFundError::AllDepositsSlashed
    )]
    pub slash: Account<'info, Slash>,

    #[account(
        mut,
        address = lockup.asset
    )]
    pub asset_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            VAULT_SEED.as_bytes(),
            lockup.key().as_ref(),
            asset_mint.key().as_ref(),
        ],
        bump,
        token::mint = asset_mint,
        token::authority = lockup,
    )]
    pub asset_lockup: Account<'info, TokenAccount>,

    /// CHECK: Directly checking address against field in settings
    #[account(
        mut,
        address = settings.cold_wallet
    )]
    pub cold_wallet: UncheckedAccount<'info>,

    #[account(
        mut,
        associated_token::mint = asset_mint,
        associated_token::authority = cold_wallet
    )]
    pub cold_wallet_vault: Account<'info, TokenAccount>,
}

impl SlashDeposits<'_> {
    pub fn validate_deposit(
        &self,
        deposit: &AccountInfo,
        index: u64,
        program_id: &Pubkey
    ) -> Result<()> {
        let (rederived_pda, _) = Pubkey::find_program_address(
            &[
                DEPOSIT_SEED.as_bytes(),
                self.lockup.key().as_ref(),
                &index.to_le_bytes()
            ], 
            program_id
        );

        require!(
            deposit.key() == rederived_pda,
            InsuranceFundError::InvalidInput
        );

        Ok(())
    }
}