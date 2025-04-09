use anchor_lang::prelude::*;
use crate::errors::InsuranceFundError;
use anchor_spl::token::{
    transfer, Token, TokenAccount, Transfer, MintTo, mint_to, Mint
};
use crate::constants::*;
use super::Settings;
use anchor_spl::token::{
    initialize_account3,
    InitializeAccount3
};
use anchor_lang::system_program::{
    create_account,
    CreateAccount
};

// Arrays are holding rates at which users are rewarded per 1 unit per lockup duration.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub enum YieldMode {
    Single, // Only offers rUSD yield.
    Dual(u64) // Offers both rUSD and $R yields. 
    // The u64 stores rate at which $R should be minted per 1 unit of deposit per lockup duration.
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy)]
pub struct SlashState {
    // How many times was this lockup slashed
    pub index: u64,
    // Total amount slashed
    pub amount: u64,
}

#[account]
pub struct Lockup {
    pub bump: u8,
    pub index: u64,
    pub asset_mint: Pubkey,
    pub receipt_mint: Pubkey,
    pub receipt_to_reward_exchange_rate_bps_accumulator: u64,
    pub min_deposit: u64,
    pub duration: u64,
    pub deposits: u64,
    pub reward_boosts: u64,
    pub yield_mode: YieldMode,
    pub slash_state: SlashState,
    pub deposit_cap: Option<u64>,
}

impl Lockup {
    pub const SIZE: usize = 8 + 1 + 8 * 6 + 2 * 32 + (1 + 8) + (1 + 8) + (2 * 8);

    pub fn slash(
        &mut self,
        amount: u64
    ) -> Result<()> {
        self.slash_state.amount = self
            .slash_state
            .amount
            .checked_add(amount)
            .ok_or(InsuranceFundError::MathOverflow)?;

        self.slash_state.index += 1;
        Ok(())
    }

    #[inline(never)]
    pub fn increase_exchange_rate_accumulator(
        &mut self,
        active_receipts_supply: u64,
        new_rewards: u64,
    ) -> Result<()> {
        
        let increase_bps = new_rewards
            .checked_mul(10_000)
            .ok_or(InsuranceFundError::MathOverflow)?
            .checked_div(active_receipts_supply)
            .ok_or(InsuranceFundError::MathOverflow)?;

        self.receipt_to_reward_exchange_rate_bps_accumulator += increase_bps;

        Ok(())
    }

    #[inline(never)]
    pub fn deposit<'info>(
        &self,
        amount: u64,
        user: &Signer<'info>,
        user_asset_ata: &Account<'info, TokenAccount>,
        settings: &Account<'info, Settings>,
        lockup_hot_vault: &Account<'info, TokenAccount>,
        lockup_cold_vault: &Account<'info, TokenAccount>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        let hot_wallet_deposit = settings
            .calculate_hot_wallet_deposit(amount)?;

        // Transfer to hot wallet
        transfer(
            CpiContext::new(
                token_program.to_account_info(), 
                Transfer {
                    from: user_asset_ata.to_account_info(),
                    to: lockup_hot_vault.to_account_info(),
                    authority: user.to_account_info()
                }
            ), 
            hot_wallet_deposit
        )?;

        transfer(
            CpiContext::new(
                token_program.to_account_info(),
                Transfer {
                    from: user_asset_ata.to_account_info(),
                    authority: user.to_account_info(),
                    to: lockup_cold_vault.to_account_info()
                }
            ),
            amount - hot_wallet_deposit
        )?;

        Ok(())
    }

    #[inline(never)]
    pub fn deposit_hot_wallet<'info>(
        &self,
        amount: u64,
        user: &Signer<'info>,
        user_asset_ata: &Account<'info, TokenAccount>,
        lockup_hot_vault: &Account<'info, TokenAccount>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        transfer(
            CpiContext::new(
                token_program.to_account_info(), 
                Transfer {
                    from: user_asset_ata.to_account_info(),
                    to: lockup_hot_vault.to_account_info(),
                    authority: user.to_account_info()
                }
            ), 
            amount
        )?;

        Ok(())
    }

    #[inline(never)]
    pub fn mint_receipts<'info>(
        &self,
        amount: u64,
        lockup: &Account<'info, Lockup>,
        receipt_token_mint: &Account<'info, Mint>,
        deposit_receipt_token_account: &AccountInfo<'info>,
        token_program: &Program<'info, Token>
    ) -> Result<()> {
        let seeds = &[
            LOCKUP_SEED.as_bytes(),
            &self.index.to_le_bytes(),
            &[self.bump]
        ];

        mint_to(
            CpiContext::new_with_signer(
                token_program.to_account_info(), 
                MintTo {
                    to: deposit_receipt_token_account.to_account_info(),
                    mint: receipt_token_mint.to_account_info(),
                    authority: lockup.to_account_info()
                }, 
                &[seeds]
            ), 
            amount
        )?;

        Ok(())
    }

    #[inline(never)]
    pub fn initialize_hot_vault<'info>(
        &self,
        payer: &Signer<'info>,
        lockup: &Account<'info, Lockup>,
        asset_mint: &AccountInfo<'info>,
        vault: &AccountInfo<'info>,
        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
        bump: u8,
        lamports: u64
    ) -> Result<()> {
        let lockup_key = lockup.key();
        let asset_mint_key = asset_mint.key();

        let signer_seeds = &[
            HOT_VAULT_SEED.as_bytes(),
            lockup_key.as_ref(),
            asset_mint_key.as_ref(),
            &[bump]
        ];

        create_account(
            CpiContext::new_with_signer(
                system_program.to_account_info(), 
                CreateAccount {
                    from: payer.to_account_info(),
                    to: vault.to_account_info()
                }, 
                &[signer_seeds]
            ), 
            lamports, 
            165, 
            &token_program.key()
        )?;

        initialize_account3(
            CpiContext::new_with_signer(
                token_program.to_account_info(), 
                InitializeAccount3 {
                    mint: asset_mint.to_account_info(),
                    account: vault.to_account_info(),
                    authority: lockup.to_account_info()
                },
                 &[signer_seeds]
            )
        )?;

        Ok(())
    }

    #[inline(never)]
    pub fn initialize_cold_vault<'info>(
        &self,
        payer: &Signer<'info>,
        lockup: &Account<'info, Lockup>,
        asset_mint: &AccountInfo<'info>,
        vault: &AccountInfo<'info>,
        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
        bump: u8,
        lamports: u64
    ) -> Result<()> {
        let lockup_key = lockup.key();
        let asset_mint_key = asset_mint.key();

        let signer_seeds = &[
            COLD_VAULT_SEED.as_bytes(),
            lockup_key.as_ref(),
            asset_mint_key.as_ref(),
            &[bump]
        ];

        create_account(
            CpiContext::new_with_signer(
                system_program.to_account_info(), 
                CreateAccount {
                    from: payer.to_account_info(),
                    to: vault.to_account_info()
                }, 
                &[signer_seeds]
            ), 
            lamports, 
            165, 
            &token_program.key()
        )?;

        initialize_account3(
            CpiContext::new_with_signer(
                token_program.to_account_info(), 
                InitializeAccount3 {
                    mint: asset_mint.to_account_info(),
                    account: vault.to_account_info(),
                    authority: lockup.to_account_info()
                },
                 &[signer_seeds]
            )
        )?;

        Ok(())
    }

    pub fn withdraw_hot_vault<'info>(
        &self,
        amount: u64,
        lockup_hot_vault: &Account<'info, TokenAccount>,
        lockup: &Account<'info, Lockup>,
        recipient: &Account<'info, TokenAccount>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        let seeds = &[
            LOCKUP_SEED.as_bytes(),
            &self.index.to_le_bytes(),
            &[self.bump]
        ];

        transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(), 
                Transfer { 
                    from: lockup_hot_vault.to_account_info(), 
                    to: recipient.to_account_info(), 
                    authority: lockup.to_account_info()
                }, 
                &[seeds]
            ), 
            amount
        )?;

        Ok(())
    }
    
}