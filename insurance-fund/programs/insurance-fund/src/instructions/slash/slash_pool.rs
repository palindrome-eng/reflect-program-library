use anchor_lang::prelude::*;
use crate::borsh::*;
use crate::constants::*;
use crate::errors::InsuranceFundError;
use crate::states::*;
use anchor_spl::token::{
    Mint,
    Token,
    TokenAccount,
    Transfer,
    transfer
};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct SlashPoolArgs {
    pub lockup_id: u64,
    pub slash_id: u64,
}

pub fn slash_pool(
    ctx: Context<SlashPool>,
    args: SlashPoolArgs
) -> Result<()> {
    let SlashPoolArgs {
        lockup_id,
        slash_id
    } = args;

    let slash = &ctx.accounts.slash;
    let token_program = &ctx.accounts.token_program;

    let lockup = &mut ctx.accounts.lockup;
    let signer_seeds = &[
        LOCKUP_SEED.as_bytes(),
        &lockup_id.to_le_bytes(),
        &[lockup.bump]
    ];

    let destination = &ctx.accounts.destination;
    let asset_lockup = &ctx.accounts.asset_lockup;

    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(), 
            Transfer {
                authority: lockup.to_account_info(),
                to: destination.to_account_info(),
                from: asset_lockup.to_account_info()
            },
            &[signer_seeds]
        ), 
        slash.slashed_amount
    )?;

    lockup.slash_state.amount += slash.slashed_amount;
    lockup.slash_state.index += 1;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: SlashPoolArgs
)]
pub struct SlashPool<'info> {
    #[account()]
    pub superadmin: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        has_one = superadmin
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
        // All deposits have to be slashed before slashing the pool.
        constraint = slash.target_accounts == slash.slashed_accounts @ InsuranceFundError::DepositsNotSlashed
    )]
    pub slash: Account<'info, Slash>,

    #[account(
        mut,
        address = lockup.asset
    )]
    pub asset: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            VAULT_SEED.as_bytes(),
            lockup.key().as_ref()
        ],
        bump,
        constraint = destination.amount >= slash.slashed_amount @ InsuranceFundError::NotEnoughFundsToSlash
    )]
    pub asset_lockup: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = asset,
    )]
    pub destination: Account<'info, TokenAccount>,

    #[account()]
    pub token_program: Program<'info, Token>
}