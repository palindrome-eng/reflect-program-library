use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use crate::states::*;

#[inline(never)]
pub fn load_user_token_accounts(
    signer: &Signer,
    assets: &Vec<&Asset>,
    remaining_accounts: &[AccountInfo],
) -> Result<Vec<(Pubkey, TokenAccount)>> {
    let remaining_accounts_iter = &mut remaining_accounts.iter();
    let mut user_token_accounts: Vec<(Pubkey, TokenAccount)> = Vec::with_capacity(assets.len() as usize);

    // Must be the first `assets.len()` accounts in the remaining accounts
    let (
        user_token_account_infos, 
        remaining
    ) = remaining_accounts
        .split_at(assets.len());

    for token_account_info in user_token_account_infos.iter() {
        let account_mut_data = token_account_info.try_borrow_mut_data()?;
        let token_account = TokenAccount::try_deserialize(&mut account_mut_data.as_ref())?;

        user_token_accounts.push((token_account_info.key(), token_account));
    }

    Ok(user_token_accounts)
}