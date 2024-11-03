use anchor_lang::prelude::*;
use crate::errors::InsuranceFundError;
use crate::states::*;
use crate::constants::*;
use anchor_spl::token::Mint;

pub fn add_asset(
    ctx: Context<AddAsset>
) -> Result<()> {
    let settings = &mut ctx.accounts.settings;
    let asset = &mut ctx.accounts.asset;
    let asset_mint = &ctx.accounts.asset_mint;
    let oracle = &ctx.accounts.oracle;

    asset.mint = asset_mint.key();

    let pyth = pyth_solana_receiver_sdk::id(); 
    let switchboard = switchboard_solana::id();   
    
    let oracle: Result<Oracle> = match oracle.owner {
        switchboard => {
            Ok(Oracle::Switchboard(oracle.key()))
        },
        pyth => {
            Ok(Oracle::Pyth(oracle.key()))
        }
        _ => {
            Err(InsuranceFundError::InvalidOracle.into())
        }
    };

    match oracle {
        Ok(oracle) => {
            asset.oracle = oracle;
            Ok(())
        },
        Err(err) => Err(err)
    }
}

#[derive(Accounts)]
pub struct AddAsset<'info> {
    #[account(
        mut
    )]
    pub superadmin: Signer<'info>,

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
        payer = superadmin,
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

    /// CHECK: Admin only function, trust devs to not fuck this up.
    #[account(
        mut
    )]
    pub oracle: UncheckedAccount<'info>,

    #[account()]
    pub system_program: Program<'info, System>,
}