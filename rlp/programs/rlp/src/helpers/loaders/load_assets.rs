use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;
use crate::errors::RlpError;

#[inline(never)]
pub fn load_assets(
    liquidity_pool: &Account<LiquidityPool>,
    remaining_accounts: &[AccountInfo],
) -> Result<Vec<(Pubkey, Asset)>> {
    let asset_count = liquidity_pool.asset_count as usize;
    let mut assets: Vec<(Pubkey, Asset)> = Vec::with_capacity(asset_count);

    require!(
        remaining_accounts.len() >= asset_count,
        RlpError::InvalidInput
    );

    for i in 0..asset_count {
        let asset_index = liquidity_pool.assets[i];
        let account_info = &remaining_accounts[i];

        require!(
            account_info.owner == &crate::ID,
            RlpError::InvalidInput
        );

        let account_data = account_info.try_borrow_mut_data()?;
        let asset = Asset::try_deserialize(&mut account_data.as_ref())
            .map_err(|_| error!(RlpError::InvalidInput))?;

        require!(
            asset.index == asset_index,
            RlpError::InvalidInput
        );

        let (expected_address, _) = Pubkey::find_program_address(
            &[
                ASSET_SEED.as_bytes(),
                &asset.mint.to_bytes(),
            ],
            &crate::ID
        );

        require!(
            account_info.key() == expected_address,
            RlpError::InvalidInput
        );

        assets.push((account_info.key(), asset));
    }

    Ok(assets)
}