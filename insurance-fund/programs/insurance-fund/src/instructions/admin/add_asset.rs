use anchor_lang::prelude::*;
use crate::errors::InsuranceFundError;
use crate::states::*;
use crate::constants::*;
use anchor_spl::token::{
    Mint,
    Token
};
use pyth_solana_receiver_sdk::*;

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

    msg!("pyth: {:?}, switchboard {:?}", pyth, switchboard);
    msg!("owner {:?}", oracle.owner);
    
    let oracle: Result<Oracle> = match oracle.owner {
        switchboard => {
            msg!("matched with switchboard");
            msg!("what {:?}", oracle.owner == switchboard);
            Ok(Oracle::Switchboard(oracle.key()))
        },
        pyth => {
            msg!("matched with pyth");
            Ok(Oracle::Pyth(oracle.key()))
        }
        _ => {
            msg!("matched with pyth - dev only");
            Ok(Oracle::Pyth(oracle.key()))
        }
        // TODO: REMOVE THIS BEFORE PROD
        // _ => Err(InsuranceFundError::InvalidOracle.into())
    };

    if oracle.is_ok() {
        asset.oracle = oracle.unwrap();

        msg!("{:?}", asset.oracle);

        Ok(())
    } else {
        // TODO: Remove before prod
        panic!();
        Err(InsuranceFundError::InvalidOracle.into())
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

    #[account(

    )]
    pub system_program: Program<'info, System>,
}