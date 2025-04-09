use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token};
use crate::states::*;
use crate::constants::*;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct RepayArgs {
    pub debt_record_id: u64
}

pub fn repay(
    ctx: Context<Repay>,
    args: RepayArgs
) -> Result<()> {
    let signer = &ctx.accounts.signer;
    let reflect_token_account = &ctx.accounts.reflect_token_account;
    let lockup = &ctx.accounts.lockup;
    let debt_record = &ctx.accounts.debt_record;
    let token_program = &ctx.accounts.token_program;
    let lockup_hot_vault = &ctx.accounts.lockup_hot_vault;
    
    lockup.deposit_hot_wallet(
        debt_record.amount, 
        signer, 
        reflect_token_account, 
        lockup_hot_vault, 
        token_program
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: RepayArgs
)]
pub struct Repay<'info> {
    #[account()]
    pub signer: Signer<'info>,

    #[account(
        mut,
        close = signer,
        seeds = [
            DEBT_RECORD_SEED.as_bytes(),
            &args.debt_record_id.to_le_bytes()
        ],
        bump
    )]
    pub debt_record: Account<'info, DebtRecord>,

    #[account(
        seeds = [
            LOCKUP_SEED.as_bytes(),
            &debt_record.lockup.to_le_bytes()
        ],
        bump
    )]
    pub lockup: Account<'info, Lockup>,

    #[account(
        constraint = asset.mint == asset_mint.key()
    )]
    pub asset: Account<'info, Asset>,

    #[account(
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
        bump
    )]
    pub lockup_hot_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = asset_mint,
        token::authority = signer
    )]
    pub reflect_token_account: Account<'info, TokenAccount>,

    #[account()]
    pub token_program: Program<'info, Token>,
}