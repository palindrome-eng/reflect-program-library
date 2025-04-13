use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;
use crate::errors::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeLpLockupArgs {
    pub duration_seconds: u64
}

pub fn initialize_lp_lockup(
    ctx: Context<InitializeLpLockup>,
    args: InitializeLpLockupArgs
) -> Result<()> {
    let lp_lockup = &mut ctx.accounts.lp_lockup;

    lp_lockup.set_inner(LpLockup { 
        duration: args.duration_seconds,
        deposits: 0
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
        space = LpLockup::INIT_SPACE,
        payer = signer,
        seeds = [
            LIQUIDITY_POOL_LOCKUP_SEED.as_bytes(),
            liquidity_pool.key().as_ref(),
            &args.duration_seconds.to_le_bytes()
        ],
        bump,
    )]
    pub lp_lockup: Account<'info, LpLockup>,

    #[account()]
    pub system_program: Program<'info, System>
}