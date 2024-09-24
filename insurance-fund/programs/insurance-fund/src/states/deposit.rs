use anchor_lang::prelude::*;

use crate::errors::InsuranceFundError;

#[account]
pub struct Deposit {
    pub user: Pubkey,
    pub amount: u64, // Total deposited
    pub initial_usd_value: u64, // USD value at the moment of the deposit
    pub amount_slashed: u64, // Amount lost due to slashing
    pub lockup: Pubkey, // Pointer to the lockup
    pub unlock_ts: u64, // Unlock timestamp
    pub last_slashed: Option<u64>, // Index of the last slash
}

impl Deposit {
    pub const LEN: usize = 8 + 2 * 32 + 3 * 8 + (1 + 8);

    pub fn slash(
        &mut self,
        amount: u64,
        slash_id: u64
    ) -> Result<()> {
        self
            .amount
            .checked_sub(amount)
            .ok_or(InsuranceFundError::MathOverflow)?;

        self
            .amount_slashed
            .checked_add(amount)
            .ok_or(InsuranceFundError::MathOverflow)?;

        self.last_slashed = Some(slash_id);

        Ok(())
    }
}