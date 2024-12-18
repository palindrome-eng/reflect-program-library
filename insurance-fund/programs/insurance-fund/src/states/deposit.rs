use anchor_lang::prelude::*;

use crate::errors::InsuranceFundError;

#[account]
pub struct Deposit {
    pub bump: u8,
    pub index: u64,
    pub user: Pubkey,
    pub initial_usd_value: u64, // USD value at the moment of the deposit
    pub amount_slashed: u64, // Amount lost due to slashing
    pub lockup: Pubkey, // Pointer to the lockup
    pub unlock_ts: u64, // Unlock timestamp
    pub last_slashed: Option<u64>, // Index of the last slash
    pub initial_receipt_exchange_rate_bps: u64,
}

impl Deposit {
    pub const LEN: usize = 8
        + 1
        + 8
        + 32
        + 8
        + 8
        + 32
        + 8
        + 1 + 8
        + 8;
        
    pub fn slash(
        &mut self,
        amount: u64,
        slash_id: u64
    ) -> Result<()> {
        self.amount_slashed = self
            .amount_slashed
            .checked_add(amount)
            .ok_or(InsuranceFundError::MathOverflow)?;

        self.last_slashed = Some(slash_id);

        Ok(())
    }

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
}