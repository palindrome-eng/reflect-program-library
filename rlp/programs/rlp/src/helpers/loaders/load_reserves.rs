use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::token::TokenAccount;
use crate::states::*;
use crate::errors::RlpError;

#[inline(never)]
pub fn load_reserves(
    liquidity_pool: &Account<LiquidityPool>,
    assets: &Vec<&Asset>,
    remaining_accounts: &[AccountInfo],
) -> Result<Vec<(Pubkey, TokenAccount)>> {
    let remaining_accounts_iter = &mut remaining_accounts.iter();
    let mut reserves: Vec<(Pubkey, TokenAccount)> = Vec::with_capacity(assets.len() as usize);

    for asset in assets.iter() {
        // Compute expected ATA address - unique per asset (assets are already verified unique)
        let reserve_key = get_associated_token_address(
            &liquidity_pool.key(),
            &asset.mint,
        );

        let maybe_account = remaining_accounts_iter
                .find(|account| account.key().eq(&reserve_key));

        let result = match maybe_account {
            Some(account_info) => {
                let account_mut_data = account_info.try_borrow_mut_data()?;
                let reserve = TokenAccount::try_deserialize(&mut account_mut_data.as_ref())?;
                    
                Ok(reserve)
            },
            None => Err(RlpError::InvalidInput)
        }?;

        reserves.push((reserve_key, result));
    }

    Ok(reserves)
}