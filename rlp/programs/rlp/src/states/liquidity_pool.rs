use anchor_lang::prelude::*;
use anchor_spl::token::{
    mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer
};
use anchor_spl::associated_token::get_associated_token_address;
use crate::constants::*;
use crate::helpers::OraclePrice;
use spl_math::precise_number::PreciseNumber;
use crate::errors::InsuranceFundError;
use crate::states::*;

#[derive(InitSpace)]
#[account]
pub struct LiquidityPool {
    pub bump: u8,
    pub index: u8,
    pub lp_token: Pubkey,
    pub cooldowns: u64,
    pub cooldown_duration: u64,
}

impl LiquidityPool {
    pub fn deposit<'info>(
        &self,
        signer: &Signer<'info>,
        asset_amount: u64,
        asset_user_account: &Account<'info, TokenAccount>,
        asset_pool: &Account<'info, TokenAccount>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        if asset_amount > 0 {
            transfer(
                CpiContext::new(
                    token_program.to_account_info(), 
                    Transfer { 
                        from: asset_user_account.to_account_info(), 
                        to: asset_pool.to_account_info(), 
                        authority: signer.to_account_info()
                    }
                ), 
                asset_amount
            )?
        }

        Ok(())
    }

    pub fn calculate_total_pool_value(
        &self,
        remaining_accounts: &[AccountInfo],
        liquidity_pool: &Account<LiquidityPool>,
        settings: &Account<Settings>,
        clock: &Clock,
    ) -> Result<PreciseNumber> {
        let mut total_pool_value = PreciseNumber::new(0)
            .ok_or(crate::errors::InsuranceFundError::MathOverflow)?;
        
        require!(
            remaining_accounts.len() == settings.assets as usize * 3,
            crate::errors::InsuranceFundError::InvalidInput
        );
        
        let mut i = 0;
        while i < remaining_accounts.len() {
            let token_account_info = &remaining_accounts[i];
            
            require!(
                token_account_info.owner == &anchor_spl::token::ID,
                crate::errors::InsuranceFundError::InvalidInput
            );
    
            let token_account = TokenAccount::try_deserialize(&mut token_account_info.try_borrow_mut_data()?.as_ref())
                .map_err(|_| crate::errors::InsuranceFundError::InvalidInput)?;
    
            require!(
                token_account.owner == liquidity_pool.key(),
                crate::errors::InsuranceFundError::InvalidInput
            );
    
            let asset_info = &remaining_accounts[i + 1];
            
            require!(
                asset_info.owner == &crate::ID,
                crate::errors::InsuranceFundError::InvalidInput
            );
            
            let asset = Asset::try_deserialize(&mut asset_info.try_borrow_mut_data()?.as_ref())
                .map_err(|_| crate::errors::InsuranceFundError::InvalidInput)?;
    
            require!(
                asset.mint == token_account.mint,
                crate::errors::InsuranceFundError::InvalidInput
            );
            
            let (expected_asset_pda, _) = Pubkey::find_program_address(
                &[
                    crate::constants::ASSET_SEED.as_bytes(),
                    &asset.mint.to_bytes()
                ],
                &crate::ID
            );

            // Verify this is the correct associated token account for the liquidity pool and asset
            let expected_pool_token_account = get_associated_token_address(
                &liquidity_pool.key(), 
                &asset.mint
            );

            require!(
                token_account_info.key() == expected_pool_token_account,
                crate::errors::InsuranceFundError::InvalidInput
            );
    
            require!(
                asset_info.key() == expected_asset_pda,
                crate::errors::InsuranceFundError::InvalidInput
            );
    
            let oracle_info = &remaining_accounts[i + 2];
    
            require!(
                oracle_info.key() == *asset.oracle.key(),
                crate::errors::InsuranceFundError::InvalidInput
            );
    
            let asset_price = asset.get_price(oracle_info, clock)
                .map_err(|_| crate::errors::InsuranceFundError::InvalidInput)?;
    
            let token_balance = token_account.amount;
            if token_balance > 0 {
                let token_value_precise = PreciseNumber::new(asset_price.mul(token_balance)?)
                    .ok_or(crate::errors::InsuranceFundError::MathOverflow)?;
                total_pool_value = total_pool_value.checked_add(&token_value_precise)
                    .ok_or(crate::errors::InsuranceFundError::MathOverflow)?;
            }
    
            i += 3;
        }
    
        Ok(total_pool_value)
    } 

    pub fn calculate_lp_tokens_on_deposit(
        &self,
        lp_token: &Account<Mint>,
        total_pool_value: PreciseNumber,
        deposit_value: PreciseNumber,
    ) -> Result<u64> {
        let lp_tokens_to_mint = if lp_token.supply == 0 {
            deposit_value
                .to_imprecise()
                .ok_or(crate::errors::InsuranceFundError::MathOverflow)?
                .try_into()
                .map_err(|_| crate::errors::InsuranceFundError::MathOverflow)?
        } else {
            let lp_supply_precise = PreciseNumber::new(lp_token.supply as u128)
                .ok_or(crate::errors::InsuranceFundError::MathOverflow)?;

            let deposit_ratio = deposit_value
                .checked_mul(&lp_supply_precise)
                .ok_or(crate::errors::InsuranceFundError::MathOverflow)?
                .checked_div(&total_pool_value)
                .ok_or(crate::errors::InsuranceFundError::MathOverflow)?;

            
            deposit_ratio
                .to_imprecise()
                .ok_or(crate::errors::InsuranceFundError::MathOverflow)?
                .try_into()
                .map_err(|_| crate::errors::InsuranceFundError::MathOverflow)?
        };

        Ok(lp_tokens_to_mint)
    }

    pub fn mint_lp_token<'info>(
        &self,
        amount: u64,
        liquidity_pool: &Account<'info, LiquidityPool>,
        lp_token: &Account<'info, Mint>,
        lockup_lp_token_vault: &Account<'info, TokenAccount>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        let signer_seeds = &[
            LIQUIDITY_POOL_SEED.as_bytes(),
            &self.index.to_le_bytes(),
            &[self.bump]
        ];

        mint_to(
            CpiContext::new_with_signer(
                token_program.to_account_info(), 
                MintTo { 
                    mint: lp_token.to_account_info(), 
                    to: lockup_lp_token_vault.to_account_info(), 
                    authority: liquidity_pool.to_account_info() 
                },
                &[signer_seeds]
            ), 
            amount
        )?;

        Ok(())
    }
}