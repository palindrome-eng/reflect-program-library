use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;

pub fn initialize_insurance_fund(
    ctx: Context<InitializeInsuranceFund>,
    cold_wallet: Pubkey,
) -> Result<()> {
    let superadmin = &ctx.accounts.superadmin;
    let settings = &mut ctx.accounts.settings;

    settings.superadmin = *superadmin.key;
    settings.bump = ctx.bumps.settings;
    settings.lockups = 0;
    settings.cold_wallet = cold_wallet;

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeInsuranceFund<'info> {
    #[account(
        mut
    )]
    pub superadmin: Signer<'info>,

    #[account(
        init,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        payer = superadmin,
        space = Settings::SIZE
    )]
    pub settings: Account<'info, Settings>,

    #[account()]
    pub system_program: Program<'info, System>
}