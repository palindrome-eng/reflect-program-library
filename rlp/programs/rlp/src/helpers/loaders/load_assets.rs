use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;
use crate::errors::RlpError;

#[inline(never)]
pub fn load_assets(
    settings: &Account<Settings>,
    remaining_accounts: &[AccountInfo],
) -> Result<Vec<(Pubkey, Asset)>> {
    let remaining_accounts_iter = &mut remaining_accounts.iter();
    let mut assets: Vec<(Pubkey, Asset)> = Vec::with_capacity(settings.assets as usize);

    for asset_index in 0..settings.assets as usize {
        let (asset_address, _) = Pubkey::find_program_address(
            &[
                ASSET_SEED.as_bytes(),
                &(asset_index as u8).to_le_bytes(),
            ], 
            &crate::ID
        );

        let maybe_account = remaining_accounts_iter
                .find(|account| account.key().eq(&asset_address));

        let result = match maybe_account {
            Some(account_info) => {
                let account_mut_data = account_info.try_borrow_mut_data()?;
                let asset = Asset::try_deserialize(&mut account_mut_data.as_ref())?;

                require!(
                    asset.index as usize == asset_index,
                    RlpError::InvalidInput
                );

                Ok(asset)
            },
            None => Err(RlpError::InvalidInput)
        }?;

        assets.push((asset_address, result));
    }

    Ok(assets)
}