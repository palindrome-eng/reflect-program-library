use anchor_lang::prelude::*;
use crate::errors::InsuranceFundError;
use crate::constants::*;
use anchor_lang::system_program::{
    create_account,
    CreateAccount
};
use anchor_spl::token::{
    initialize_account3,
    InitializeAccount3,
    Token,
    Mint,
};

#[account]
#[derive(InitSpace)]
pub struct Deposit {
    pub bump: u8,
    pub index: u64,
    pub user: Pubkey,
    pub initial_usd_value: u64, // USD value at the moment of the deposit
    pub lockup: Pubkey, // Pointer to the lockup
    pub unlock_ts: u64, // Unlock timestamp
    pub initial_receipt_exchange_rate_bps: u64,
}

impl Deposit {
    #[inline(never)]
    pub fn compute_accrued_rewards(
        &self, 
        current_receipt_exchange_rate_bps: u64,
        owned_receipts: u64
    ) -> Result<u64> {
        let exchange_rate_diff_bps = current_receipt_exchange_rate_bps
            .checked_sub(self.initial_receipt_exchange_rate_bps)
            .ok_or(InsuranceFundError::MathOverflow)?;

        let result = owned_receipts
            .checked_mul(exchange_rate_diff_bps)
            .ok_or(InsuranceFundError::MathOverflow)?
            .checked_div(10_000) // basepoints
            .ok_or(InsuranceFundError::MathOverflow)?;

        Ok(result)
    }

    #[inline(never)]
    pub fn create_deposit_receipt_token_account<'info>(
        &self,
        user: &Signer<'info>,
        deposit_receipt_token_account: &AccountInfo<'info>,
        bump: u8,
        deposit: &Account<'info, Deposit>,
        receipt_token_mint: &Account<'info, Mint>,
        token_program: &Program<'info, Token>,
        system_program: &Program<'info, System>,
    ) -> Result<()> {
        let rent = Rent::get()?;
        let lamports = rent.minimum_balance(165);

        let deposit_key = deposit.key();
        let receipt_key = receipt_token_mint.key();

        let signer_seeds = &[
            DEPOSIT_RECEIPT_VAULT_SEED.as_bytes(),
            deposit_key.as_ref(),
            receipt_key.as_ref(),
            &[bump]
        ];

        create_account(
            CpiContext::new_with_signer(
                system_program.to_account_info(), 
                CreateAccount {
                    from: user.to_account_info(),
                    to: deposit_receipt_token_account.to_account_info()
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
                    account: deposit_receipt_token_account.to_account_info(),
                    mint: receipt_token_mint.to_account_info(),
                    authority: deposit.to_account_info()
                },
                &[signer_seeds]
            )
        )?;

        Ok(())
    }
}