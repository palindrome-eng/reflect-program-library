use anchor_lang::prelude::*;
use switchboard_solana::ID as SWITCHBOARD_PROGRAM_ID;
use pyth_solana_receiver_sdk::ID as PYTH_PROGRAM_ID;
use crate::errors::InsuranceFundError;
use crate::events::AddAssetEvent;
use crate::helpers::get_price_from_pyth;
use crate::helpers::get_price_from_switchboard;
use crate::states::*;
use crate::constants::*;
use anchor_spl::token::Mint;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct AddAssetArgs {
    pub access_level: AccessLevel,
}

pub fn add_asset(
    ctx: Context<AddAsset>,
    args: AddAssetArgs
) -> Result<()> {
    let settings = &mut ctx.accounts.settings;
    let asset = &mut ctx.accounts.asset;
    let asset_mint = &ctx.accounts.asset_mint;
    let oracle = &ctx.accounts.oracle;
    let signer = &ctx.accounts.signer;

    let clock = Clock::get()?;

    let oracle = if oracle.owner.eq(&PYTH_PROGRAM_ID) {
        get_price_from_pyth(oracle, &clock)?;
        Oracle::Pyth(oracle.key())
    } else if oracle.owner.eq(&SWITCHBOARD_PROGRAM_ID) {
        get_price_from_switchboard(oracle, &clock)?;
        Oracle::Switchboard(oracle.key())
    } else {
        return Err(InsuranceFundError::InvalidOracle.into());
    };

    asset.set_inner(Asset { 
        mint: asset_mint.key(), 
        oracle, 
        access_level: args.access_level 
    });

    settings.assets = settings
        .assets
        .checked_add(1)
        .ok_or(InsuranceFundError::MathOverflow)?;

    emit!(AddAssetEvent {
        admin: signer.key(),
        asset: asset_mint.key(),
        oracle: *oracle.key()
    });

    Ok(())
}

#[derive(Accounts)]
pub struct AddAsset<'info> {
    #[account(
        mut
    )]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump,
        constraint = admin.can_perform_protocol_action(Action::AddAsset, &settings.access_control) @ InsuranceFundError::PermissionsTooLow,
    )]
    pub admin: Account<'info, UserPermissions>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump
    )]
    pub settings: Account<'info, Settings>,
    
    #[account(
        init,
        payer = signer,
        seeds = [
            ASSET_SEED.as_bytes(),
            &asset_mint.key().to_bytes()
        ],
        bump,
        space = 8 + Asset::INIT_SPACE
    )]
    pub asset: Account<'info, Asset>,

    #[account(
        mut
    )]
    pub asset_mint: Account<'info, Mint>,

    /// CHECK: We're checking owner of this account later
    #[account(
        mut
    )]
    pub oracle: UncheckedAccount<'info>,

    #[account()]
    pub system_program: Program<'info, System>,
}