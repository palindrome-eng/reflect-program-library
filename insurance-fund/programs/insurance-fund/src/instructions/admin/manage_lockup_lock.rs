use anchor_lang::prelude::*;
use crate::errors::InsuranceFundError;
use crate::states::*;
use crate::constants::*;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct ManageLockupLockArgs {
    lockup_id: u64,
    lock: bool
}

pub fn manage_lockup_lock(
    ctx: Context<ManageLockupLock>,
    args: ManageLockupLockArgs
) -> Result<()> {
    let lockup = &mut ctx.accounts.lockup;

    match args.lock {
        true => lockup.lock(),
        false => lockup.unlock(),
    }

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: ManageLockupLockArgs
)]
pub struct ManageLockupLock<'info> {
    #[account(
        mut,
    )]
    pub signer: Signer<'info>,

    #[account(
        mut,
        constraint = admin.address == signer.key(),
        constraint = admin.has_permissions(Permissions::Superadmin) @ InsuranceFundError::InvalidSigner,
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
}