use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;
use pyth_solana_receiver_sdk::price_update::{
    PriceUpdateV2
};
use anchor_spl::token::{
    Mint,
    Token
};

pub fn add_asset(
    ctx: Context<AddAsset>
) -> Result<()> {
    let settings = &mut ctx.accounts.settings;

    let mint = &ctx.accounts.asset;
    let oracle = &ctx.accounts.oracle;

    settings.add_asset(
        mint.key(),
        oracle.key()
    );

    Ok(())
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
        mut
    )]
    pub asset: Account<'info, Mint>,

    #[account(
        mut
    )]
    pub oracle: Account<'info, PriceUpdateV2>,
}