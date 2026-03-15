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
        // Find the next account owned by the RLP program that deserializes as an Asset
        let maybe_account = remaining_accounts_iter
                .find(|account| account.owner == &crate::ID);

        let (asset_address, asset) = match maybe_account {
            Some(account_info) => {
                let account_mut_data = account_info.try_borrow_mut_data()?;
                let asset = Asset::try_deserialize(&mut account_mut_data.as_ref())
                    .map_err(|_| error!(RlpError::InvalidInput))?;

                require!(
                    asset.index as usize == asset_index,
                    RlpError::InvalidInput
                );

                // Verify the account is the correct PDA derived from the asset's mint
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

                Ok((account_info.key(), asset))
            },
            None => Err(error!(RlpError::InvalidInput))
        }?;

        assets.push((asset_address, asset));
    }

    Ok(assets)
}