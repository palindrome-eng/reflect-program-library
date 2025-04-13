use anchor_lang::prelude::*;
use anchor_spl::token::{
    mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer
};

use crate::{errors::InsuranceFundError, helpers::OraclePrice};

#[derive(InitSpace)]
#[account]
pub struct LiquidityPool {
    pub token_a: Pubkey,
    pub token_b: Pubkey,
    pub lp_token: Pubkey,
}

impl LiquidityPool {
    pub fn deposit<'info>(
        &self,
        signer: &Signer<'info>,
        token_a_amount: u64,
        token_a_user_account: &Account<'info, TokenAccount>,
        token_a_pool: &Account<'info, TokenAccount>,
        token_b_amount: u64,
        token_b_user_account: &Account<'info, TokenAccount>,
        token_b_pool: &Account<'info, TokenAccount>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        if token_a_amount > 0 {
            transfer(
                CpiContext::new(
                    token_program.to_account_info(), 
                    Transfer { 
                        from: token_a_user_account.to_account_info(), 
                        to: token_a_pool.to_account_info(), 
                        authority: signer.to_account_info()
                    }
                ), 
                token_a_amount
            )?
        }

        if token_b_amount > 0 {
            transfer(
                CpiContext::new(
                    token_program.to_account_info(), 
                    Transfer { 
                        from: token_b_user_account.to_account_info(), 
                        to: token_b_pool.to_account_info(), 
                        authority: signer.to_account_info()
                    }
                ), 
                token_b_amount
            )?
        }

        Ok(())
    }

    pub fn compute_lp_tokens_on_deposit<'info>(
        &self,
        lp_token_supply: u64,
        token_a_liquidity: u64,
        token_b_liquidity: u64,
        token_a_deposit: u64,
        token_b_deposit: u64,
        token_a_price: OraclePrice,
        token_b_price: OraclePrice,
    ) -> Result<u64> {
        let token_a_liquidity_value = token_a_price
            .mul(token_a_liquidity)?;

        let token_b_liquidity_value = token_b_price
            .mul(token_b_liquidity)?;

        let total_lp_value = token_a_liquidity_value
            .checked_add(token_b_liquidity_value)
            .ok_or(InsuranceFundError::MathOverflow)?;

        let value_per_lp_token = total_lp_value
            .checked_div(lp_token_supply.into())
            .ok_or(InsuranceFundError::MathOverflow)?;

        let deposit_a_value = token_a_price
            .mul(token_a_deposit)?;

        let deposit_b_value = token_b_price
            .mul(token_b_deposit)?;

        let total_deposit_value = deposit_a_value
            .checked_add(deposit_b_value)
            .ok_or(InsuranceFundError::MathOverflow)?;

        let lp_token_to_mint: u64 = total_deposit_value
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
        lp_token_lockup_deposit: &Account<'info, TokenAccount>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        mint_to(
            CpiContext::new(
                token_program.to_account_info(), 
                MintTo { 
                    mint: lp_token.to_account_info(), 
                    to: lp_token_lockup_deposit.to_account_info(), 
                    authority: liquidity_pool.to_account_info() 
                }
            ), 
            amount
        )?;

        Ok(())
    }
}