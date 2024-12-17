use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;
use crate::errors::InsuranceFundError;
use anchor_spl::token::{
    Mint,
    TokenAccount,
    Token,
    transfer,
    Transfer
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SlashArgs {
    lockup_id: u64,
    amount: u64,
}

pub fn slash(
    ctx: Context<Slash>,
    args: SlashArgs
) -> Result<()> {
    let token_program = &ctx.accounts.token_program;
    let destination = &ctx.accounts.destination;
    let lockup = &ctx.accounts.lockup;
    let lockup_hot_vault = &ctx.accounts.lockup_hot_vault;

    let SlashArgs {
        amount,
        lockup_id
    } = args;

    let seeds = &[
        LOCKUP_SEED.as_bytes(),
        &lockup_id.to_le_bytes(),
        &[lockup.bump]
    ];
    
    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                authority: lockup.to_account_info(),
                from: lockup_hot_vault.to_account_info(),
                to: destination.to_account_info(),
            },
            &[seeds]
        ), 
        amount
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: SlashArgs
)]
pub struct Slash<'info> {
    #[account(
        mut,
    )]
    pub signer: Signer<'info>,

    #[account(
        mut,
        constraint = admin.address == signer.key() @ InsuranceFundError::InvalidSigner,
        constraint = admin.has_permissions(Permissions::Superadmin) @ InsuranceFundError::PermissionsTooLow,
    )]
    pub admin: Account<'info, Admin>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        constraint = !settings.frozen @ InsuranceFundError::Frozen
    )]
    pub settings: Account<'info, Settings>,

    #[account(
        mut,
        seeds = [
            LOCKUP_SEED.as_bytes(),
            &args.lockup_id.to_le_bytes()
        ],
        bump
    )]
    pub lockup: Account<'info, Lockup>,

    #[account(
        mut,
        address = lockup.asset_mint
    )]
    pub asset_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            HOT_VAULT_SEED.as_bytes(),
            lockup.key().as_ref(),
            asset_mint.key().as_ref(),
        ],
        bump,
        constraint = lockup_hot_vault.amount >= args.amount @ InsuranceFundError::NotEnoughFundsToSlash
    )]
    pub lockup_hot_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            COLD_VAULT_SEED.as_bytes(),
            lockup.key().as_ref(),
            asset_mint.key().as_ref(),
        ],
        bump
    )]
    pub lockup_cold_vault: Account<'info, TokenAccount>,

    #[account()]
    pub destination: Account<'info, TokenAccount>,

    #[account()]
    pub token_program: Program<'info, Token>,
}