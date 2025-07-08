use anchor_lang::prelude::*;
use anchor_spl::token::{
    mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer
};
use crate::{constants::LIQUIDITY_POOL_SEED, errors::InsuranceFundError, helpers::OraclePrice};

#[derive(InitSpace)]
#[account]
pub struct LiquidityPool {
    pub bump: u8,
    pub index: u64,
    pub lp_token: Pubkey
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

    /// DEPRECATED: This method only considers a single asset. Use the new deposit_asset instruction
    /// that calculates LP tokens based on the total USD value of the entire liquidity pool.
    pub fn compute_lp_tokens_on_deposit<'info>(
        &self,
        lp_token_supply: u64,
        asset_liquidity: u64,
        asset_deposit: u64,
        asset_price: OraclePrice,
    ) -> Result<u64> {
        let deposit_value = asset_price
            .mul(asset_deposit)?;

        if lp_token_supply == 0 {
            let result: u64 = deposit_value
                .try_into()
                .map_err(|_| InsuranceFundError::MathOverflow)?;

            return Ok(result);
        }

        let asset_liquidity_value = asset_price
            .mul(asset_liquidity)?;

        let value_per_lp_token = asset_liquidity_value
            .checked_div(lp_token_supply.into())
            .ok_or(InsuranceFundError::MathOverflow)?;

        let lp_token_to_mint: u64 = deposit_value
            .checked_div(value_per_lp_token)
            .ok_or(InsuranceFundError::MathOverflow)?
            .try_into()
            .map_err(|_| InsuranceFundError::MathOverflow)?;

        Ok(lp_token_to_mint)
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