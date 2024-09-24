use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;
use crate::errors::InsuranceFundError;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeInsuranceFundArgs {
    pub cold_wallet: Pubkey,
    pub hot_wallet_share_bps: u64,
    pub cold_wallet_share_bps: u64
}

pub fn initialize_insurance_fund(
    ctx: Context<InitializeInsuranceFund>,
    args: InitializeInsuranceFundArgs
) -> Result<()> {

    let InitializeInsuranceFundArgs {
        cold_wallet,
        cold_wallet_share_bps,
        hot_wallet_share_bps
    } = args;

    require!(
        cold_wallet_share_bps + hot_wallet_share_bps == 10_000,
        InsuranceFundError::ShareConfigOverflow
    );

    let superadmin = &ctx.accounts.superadmin;
    let settings = &mut ctx.accounts.settings;

    settings.superadmin = superadmin.key();
    settings.bump = ctx.bumps.settings;
    settings.lockups = 0;
    settings.cold_wallet = cold_wallet;
    settings.shares_config.cold_wallet_share_bps = cold_wallet_share_bps;
    settings.shares_config.hot_wallet_share_bps = hot_wallet_share_bps;

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
        space = Settings::SIZE,
    )]
    pub settings: Account<'info, Settings>,

    #[account()]
    pub system_program: Program<'info, System>
}