use anchor_lang::prelude::*;

use crate::errors::InsuranceFundError;

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, PartialEq)]
pub struct Asset {
    pub mint: Pubkey,
    pub oracle: Pubkey,
    pub tvl: u64
}

impl Asset {
    pub const SIZE: usize = 2 * 32 + 8;
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, PartialEq)]
pub struct SharesConfig {
    pub hot_wallet_share_bps: u64,
    pub cold_wallet_share_bps: u64
}

impl SharesConfig {
    pub const SIZE: usize = 2 * 8;
}

#[account]
pub struct Settings {
    pub bump: u8,
    pub superadmin: Pubkey,
    pub cold_wallet: Pubkey,
    pub lockups: u64,
    pub assets: Vec<Asset>,
    pub deposits_locked: bool,
    pub shares_config: SharesConfig
}

impl Settings {
    pub const SIZE: usize = 8 + 1 + 2 * 32 + 8 + 4 + 1 + SharesConfig::SIZE;

    pub fn add_asset(
        &mut self,
        mint: Pubkey,
        oracle: Pubkey
    ) {
        self.assets.push(Asset {
            mint,
            oracle,
            tvl: 0
        });
    }

    pub fn increase_tvl(
        &mut self,
        asset_mint: &Pubkey,
        amount: u64
    ) -> Result<()> {
        let asset = self
            .assets
            .iter_mut()
            .find(|asset| asset.mint.eq(asset_mint))
            .ok_or(InsuranceFundError::AssetNotWhitelisted)?;

        asset.tvl += amount;

        Ok(())
    }

    pub fn decrease_tvl(
        &mut self,
        asset_mint: &Pubkey,
        amount: u64
    ) -> Result<()> {
        let asset = self
            .assets
            .iter_mut()
            .find(|asset| asset.mint.eq(asset_mint))
            .ok_or(InsuranceFundError::AssetNotWhitelisted)?;

        asset.tvl -= amount;

        Ok(())
    }
}