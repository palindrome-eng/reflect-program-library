use anchor_lang::prelude::*;
use switchboard_solana::ID as SWITCHBOARD_PROGRAM_ID;
use pyth_solana_receiver_sdk::ID as PYTH_PROGRAM_ID;
use crate::errors::InsuranceFundError;
use crate::helpers::get_price_from_pyth;
use crate::helpers::get_price_from_switchboard;
use crate::states::*;
use crate::constants::*;
use anchor_spl::token::Mint;

pub fn add_asset(
    ctx: Context<AddAsset>
) -> Result<()> {
    let asset = &mut ctx.accounts.asset;
    let asset_mint = &ctx.accounts.asset_mint;
    let oracle = &ctx.accounts.oracle;

    asset.mint = asset_mint.key();  
    asset.tvl = 0;
    asset.deposits = 0;
    asset.lockups = 0;

    if oracle.owner.eq(&PYTH_PROGRAM_ID) {
        get_price_from_pyth(oracle)?;
        asset.oracle = Oracle::Pyth(oracle.key());

        Ok(())
    } else if oracle.owner.eq(&SWITCHBOARD_PROGRAM_ID) {
        get_price_from_switchboard(oracle)?;
        asset.oracle = Oracle::Switchboard(oracle.key());
        
        Ok(())
    } else {
        Err(InsuranceFundError::InvalidOracle.into())
    }
}

#[derive(Accounts)]
pub struct AddAsset<'info> {
    #[account(
        mut
    )]
    pub signer: Signer<'info>,

    #[account(
        mut,
        constraint = admin.address == signer.key() @ InsuranceFundError::InvalidSigner,
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
        init,
        payer = signer,
        seeds = [
            ASSET_SEED.as_bytes(),
            &asset_mint.key().to_bytes()
        ],
        bump,
        space = Asset::SIZE
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