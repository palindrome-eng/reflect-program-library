use anchor_lang::prelude::*;
use crate::states::*;
use crate::errors::InsuranceFundError;
use crate::constants::*;

// Rebalancing will check every existing asset account and enforce
// ratio held by SharesConfig.
pub fn rebalance_insurance_fund(
    ctx: Context<RebalanceInsuranceFund>
) -> Result<()> {
    let settings = &ctx.accounts.settings;

    let SharesConfig {
        cold_wallet_share_bps,
        hot_wallet_share_bps
    } = settings.shares_config;

    // Remaining accounts include information about all assets
    let remaining = &ctx.remaining_accounts;

    Ok(())
}

#[derive(Accounts)]
pub struct RebalanceInsuranceFund<'info> {
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
        address = settings.cold_wallet
    )]
    pub cold_wallet: UncheckedAccount<'info>,
}