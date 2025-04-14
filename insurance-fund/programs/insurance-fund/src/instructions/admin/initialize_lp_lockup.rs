use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;
use crate::errors::*;
use anchor_spl::token::{Mint};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeLpLockupArgs {
    pub duration_seconds: u64
}

pub fn initialize_lp_lockup(
    ctx: Context<InitializeLpLockup>,
    args: InitializeLpLockupArgs
) -> Result<()> {
    let lp_lockup = &mut ctx.accounts.lp_lockup;
    let lockup_receipt_token = &mut ctx.accounts.lockup_receipt_token;

    lp_lockup.set_inner(LpLockup { 
        duration: args.duration_seconds,
        deposits: 0,
        receipt_token: lockup_receipt_token.key()
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: InitializeLpLockupArgs
)]
pub struct InitializeLpLockup<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            ADMIN_SEED.as_bytes(),
            signer.key().as_ref(),
        ],
        bump,
        constraint = admin.has_permissions(Permissions::Freeze) @ InsuranceFundError::InvalidSigner,
    )]
    pub admin: Account<'info, Admin>,

    #[account()]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        init,
        space = 8 + LpLockup::INIT_SPACE,
        payer = signer,
        seeds = [
            LIQUIDITY_POOL_LOCKUP_SEED.as_bytes(),
            liquidity_pool.key().as_ref(),
            &args.duration_seconds.to_le_bytes()
        ],
        bump,
    )]
    pub lp_lockup: Account<'info, LpLockup>,

    #[account(
        constraint = lockup_receipt_token.supply == 0 &&
            lockup_receipt_token.mint_authority.unwrap() == liquidity_pool.key() &&
            lockup_receipt_token.freeze_authority.is_none() &&
            lockup_receipt_token.is_initialized &&
            lockup_receipt_token.decimals == 9 @ InsuranceFundError::InvalidReceiptTokenSetup
    )]
    pub lockup_receipt_token: Account<'info, Mint>,

    #[account()]
    pub system_program: Program<'info, System>
}