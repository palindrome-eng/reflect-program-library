use anchor_lang::prelude::*;

use crate::{errors::InsuranceFundError, program::InsuranceFund};

#[account]
pub struct Slash {
    pub index: u64,
    pub target_accounts: u64,
    pub slashed_accounts: u64,
    pub target_amount: u64,
    pub slashed_amount: u64
}

impl Slash {
    pub const LEN: usize = 8 + 5 * 8;

    pub fn slash_account(
        &self,
        amount: u64
    ) -> Result<()> {
        self.slashed_amount
            .checked_add(amount)
            .ok_or(InsuranceFundError::MathOverflow)?;

        self.slashed_accounts
            .checked_add(1)
            .ok_or(InsuranceFundError::MathOverflow)?;

        Ok(())
    }
}