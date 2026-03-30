use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::token::TokenAccount;
use crate::errors::RlpError;
use crate::states::*;

#[inline(never)]
pub fn load_user_token_accounts(
    signer: &Signer,
    assets: &Vec<&Asset>,
    remaining_accounts: &[AccountInfo],
) -> Result<Vec<(Pubkey, TokenAccount)>> {
    let remaining_accounts_iter = &mut remaining_accounts.iter();
    let mut user_token_accounts: Vec<(Pubkey, TokenAccount)> = Vec::with_capacity(assets.len());

    for asset in assets.iter() {
        let expected_ata = get_associated_token_address(
            &signer.key(),
            &asset.mint,
        );

        let maybe_account = remaining_accounts_iter
            .find(|account| account.key().eq(&expected_ata));

        let (key, token_account) = match maybe_account {
            Some(account_info) => {
                require!(
                    account_info.owner == &anchor_spl::token::ID,
                    RlpError::InvalidInput
                );

                let account_mut_data = account_info.try_borrow_mut_data()?;
                let token_account = TokenAccount::try_deserialize(&mut account_mut_data.as_ref())
                    .map_err(|_| error!(RlpError::InvalidInput))?;

                require!(
                    token_account.owner == signer.key(),
                    RlpError::InvalidInput
                );

                require!(
                    token_account.mint == asset.mint,
                    RlpError::InvalidInput
                );

                Ok((account_info.key(), token_account))
            },
            None => Err(error!(RlpError::InvalidInput))
        }?;

        user_token_accounts.push((key, token_account));
    }

    Ok(user_token_accounts)
}
