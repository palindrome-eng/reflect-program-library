use anchor_lang::prelude::*;
use crate::constants::*;
use anchor_spl::token::{mint_to, Mint, MintTo, Token, TokenAccount};

#[derive(InitSpace)]
#[account]
pub struct LpLockup {
    pub bump: u8,
    pub duration: u64,
    pub deposits: u64,
    // pub lp_token: u64, // not essential
    pub receipt_token: Pubkey,
    pub liquidity_pool: Pubkey,
}

impl LpLockup {
    pub fn mint_receipt_tokens<'info>(
        &self,
        receipt_token: &Account<'info, Mint>,
        receipt_token_deposit_account: &AccountInfo<'info>,
        lp_lockup: &Account<'info, LpLockup>,
        token_program: &Program<'info, Token>,
        amount: u64
    ) -> Result<()> {
        let signer_seeds = &[
            LIQUIDITY_POOL_LOCKUP_SEED.as_bytes(),
            self.liquidity_pool.as_ref(),
            &self.duration.to_le_bytes(),
            &[self.bump]
        ];

        mint_to(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                MintTo { 
                    mint: receipt_token.to_account_info(), 
                    to: receipt_token_deposit_account.to_account_info(), 
                    authority: lp_lockup.to_account_info() 
                }, 
                &[signer_seeds]
            ), 
            amount
        )?;

        Ok(())
    }
}