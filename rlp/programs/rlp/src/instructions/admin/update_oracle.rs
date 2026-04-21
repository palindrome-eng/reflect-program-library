use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::RlpError;
use crate::events::UpdateOracleEvent;
use crate::helpers::{get_price_from_pyth, get_price_from_doppler};
use crate::states::*;
use pyth_solana_receiver_sdk::ID as PYTH_PROGRAM_ID;

pub fn update_oracle(ctx: Context<UpdateOracle>) -> Result<()> {
    let asset = &mut ctx.accounts.asset;
    let oracle_account = &ctx.accounts.oracle;
    let signer = &ctx.accounts.signer;

    let clock = Clock::get()?;

    let old_oracle = *asset.oracle.key();

    let new_oracle = if oracle_account.owner.as_ref() == PYTH_PROGRAM_ID.as_ref() {
        get_price_from_pyth(oracle_account, &clock)?;
        Oracle::Pyth(oracle_account.key())
    } else if oracle_account.owner == &DOPPLER_ORACLE_PROGRAM_ID {
        get_price_from_doppler(oracle_account)?;
        Oracle::Doppler(oracle_account.key())
    } else {
        return Err(RlpError::InvalidOracle.into());
    };

    asset.oracle = new_oracle;

    emit!(UpdateOracleEvent {
        admin: signer.key(),
        asset: asset.mint,
        old_oracle,
        new_oracle: *new_oracle.key(),
    });

    Ok(())
}

#[derive(Accounts)]
pub struct UpdateOracle<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump = admin.bump,
        constraint = admin.can_perform_protocol_action(Action::UpdateOracle, &settings.access_control) @ RlpError::PermissionsTooLow,
    )]
    pub admin: Account<'info, UserPermissions>,

    #[account(
        seeds = [SETTINGS_SEED.as_bytes()],
        bump = settings.bump,
        constraint = !settings.access_control.killswitch.is_frozen(&Action::UpdateOracle) @ RlpError::Frozen,
    )]
    pub settings: Box<Account<'info, Settings>>,

    #[account(
        mut,
        seeds = [
            ASSET_SEED.as_bytes(),
            &asset.mint.to_bytes()
        ],
        bump = asset.bump,
    )]
    pub asset: Account<'info, Asset>,

    /// CHECK: Validated in instruction body - must be owned by Pyth or Doppler program
    pub oracle: UncheckedAccount<'info>,
}
