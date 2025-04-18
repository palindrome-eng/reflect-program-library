use core::{convert::Into, result::Result::Ok};

use anchor_lang::prelude::*;
use anchor_spl::token::{burn, mint_to, transfer, Burn, Mint, MintTo, Token, TokenAccount, Transfer};
use crate::{constants::VAULT_SEED, errors::ReflectError};

#[account]
#[derive(InitSpace)]
pub struct Vault {
    pub bump: u8,
    pub index: u64,
    pub creator: Pubkey,
    pub deposit_token_mint: Pubkey,
    pub receipt_token_mint: Pubkey,
}

impl Vault {
    pub fn deposit<'info>(
        &self,
        amount: u64,
        signer: &Signer<'info>,
        signer_token_account: &Account<'info, TokenAccount>,
        pool: &Account<'info, TokenAccount>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {

        transfer(
            CpiContext::new(
                token_program.to_account_info(), 
                Transfer { 
                    from: signer_token_account.to_account_info(), 
                    to: pool.to_account_info(), 
                    authority: signer.to_account_info()
                }
            ), 
            amount
        )?;

        Ok(())
    }

    pub fn mint_receipt_tokens<'info>(
        &self,
        amount: u64,
        vault: &Account<'info, Vault>,
        recipient: &Account<'info, TokenAccount>,
        receipt_mint: &Account<'info, Mint>,
        token_program: &Program<'info, Token>
    ) -> Result<()> {

        let signer_seeds = &[
            VAULT_SEED.as_bytes(),
            &self.index.to_be_bytes(),
            &[self.bump]
        ];

        mint_to(
            CpiContext::new_with_signer(
                token_program.to_account_info(), 
                MintTo { 
                    mint: receipt_mint.to_account_info(), 
                    to: recipient.to_account_info(), 
                    authority: vault.to_account_info()
                },
                &[signer_seeds]
            ), 
            amount
        )?;

        Ok(())
    }

    pub fn withdraw<'info>(
        &self,
        amount: u64,
        vault: &Account<'info, Vault>,
        recipient: &Account<'info, TokenAccount>,
        pool: &Account<'info, TokenAccount>,
        token_program: &Program<'info, Token>
    ) -> Result<()> {

        let signer_seeds = &[
            VAULT_SEED.as_bytes(),
            &self.index.to_be_bytes(),
            &[self.bump]
        ];

        transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(), 
                Transfer { 
                    from: pool.to_account_info(), 
                    to: recipient.to_account_info(), 
                    authority: vault.to_account_info()
                },
                &[signer_seeds]
            ), 
            amount
        )?;

        Ok(())
    }

    pub fn burn_receipt_tokens<'info>(
        &self,
        amount: u64,
        signer: &Signer<'info>,
        signer_token_account: &Account<'info, TokenAccount>,
        receipt_mint: &Account<'info, Mint>,
        token_program: &Program<'info, Token>
    ) -> Result<()> {
        burn(
            CpiContext::new(
                token_program.to_account_info(), 
                Burn { 
                    mint: receipt_mint.to_account_info(),
                    authority: signer.to_account_info(),
                    from: signer_token_account.to_account_info()
                }
            ), 
            amount
        )?;

        Ok(())
    }

    pub fn compute_receipt_token<'info>(
        &self,
        deposit: u64,
        deposited: u64,
        receipt_token_supply: u64,
    ) -> Result<u64> {

        if receipt_token_supply == 0 { return Ok(deposit); };

        deposit
            .checked_mul(receipt_token_supply)
            .ok_or(ReflectError::MathOverflow)?
            .checked_div(deposited)
            .ok_or(ReflectError::ZeroDivision.into())
    }

    pub fn compute_base_token(
        &self,
        receipt: u64,
        deposited: u64,
        receipt_token_supply: u64,
    ) -> Result<u64> {
        receipt
            .checked_mul(deposited)
            .ok_or(ReflectError::MathOverflow)?
            .checked_div(receipt_token_supply)
            .ok_or(ReflectError::ZeroDivision.into())
    }
}
