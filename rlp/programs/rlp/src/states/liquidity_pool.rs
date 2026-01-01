use anchor_lang::prelude::*;
use anchor_spl::token::{
    mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer
};
use anchor_spl::associated_token::get_associated_token_address;
use crate::constants::*;
use crate::errors::RlpError;
use crate::helpers::OraclePrice;
use spl_math::precise_number::PreciseNumber;

use crate::states::*;

#[derive(InitSpace)]
#[account]
pub struct LiquidityPool {
    pub bump: u8,
    pub index: u8,
    pub lp_token: Pubkey,
    pub cooldowns: u64,
    pub cooldown_duration: u64,
    pub deposit_cap: Option<u64>,
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
        reserves: &Vec<&TokenAccount>,
        oracle_prices: &Vec<OraclePrice>,
    ) -> Result<PreciseNumber> {
        let mut total_pool_value = PreciseNumber::new(0)
            .ok_or(crate::errors::RlpError::MathOverflow)?;

        for (
            reserve, 
            oracle_price
        ) in reserves.iter().zip(oracle_prices.iter()) {
            let token_balance = reserve.amount;

            if token_balance > 0 {
                let token_value = PreciseNumber::new(
                    oracle_price.mul(token_balance)?
                )
                    .ok_or(crate::errors::RlpError::MathOverflow)?;

                total_pool_value = total_pool_value.checked_add(&token_value)
                    .ok_or(crate::errors::RlpError::MathOverflow)?;
            }
        }
    
        Ok(total_pool_value)
    }

    pub fn calculate_lp_tokens_on_deposit(
        &self,
        lp_token: &Account<Mint>,
        total_pool_value: PreciseNumber,
        deposit_value: PreciseNumber,
    ) -> Result<u64> {
        let lp_supply_precise = PreciseNumber::new(
            lp_token
                .supply
                .checked_add(DEAD_SHARES)
                .ok_or(RlpError::MathOverflow)? as u128
        )
            .ok_or(crate::errors::RlpError::MathOverflow)?;

        let deposit_ratio = deposit_value
            .checked_mul(&lp_supply_precise)
            .ok_or(RlpError::MathOverflow)?
            .checked_div(&total_pool_value)
            .ok_or(RlpError::MathOverflow)?;

        let lp_tokens_to_mint = deposit_ratio.to_imprecise()
            .ok_or(RlpError::MathOverflow)?
            .try_into()
            .map_err(|_| RlpError::MathOverflow)?;

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