use anchor_lang::prelude::*;
use crate::constants::*;
use crate::states::*;

pub fn unlock_deposits(
    ctx: Context<UnlockDeposits>,
) -> Result<()> {
    let settings = &mut ctx.accounts.settings;
    settings.deposits_locked = false;

    Ok(())
}

#[derive(Accounts)]
pub struct UnlockDeposits<'info> {
    #[account(
        mut,
        address = settings.superadmin
    )]
    pub superadmin: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        has_one = superadmin,
    )]
    pub settings: Account<'info, Settings>,
}