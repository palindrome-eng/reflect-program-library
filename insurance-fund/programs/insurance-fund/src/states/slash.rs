use anchor_lang::prelude::*;

use crate::{errors::InsuranceFundError};

#[account]
pub struct Slash {
    pub index: u64,
    pub target_accounts: u64,
    pub slashed_accounts: u64,
    pub target_amount: u64,
    pub slashed_amount: u64,
    pub transfer_sig: Option<String>,
}

impl Slash {
    pub const LEN: usize = 8 + 5 * 8 + (1 + 4 + 64);
    // signature requires 64 bytes

    pub fn slash_account(
        &mut self,
        amount: u64
    ) -> Result<()> {
        self.slashed_amount = self.slashed_amount
            .checked_add(amount)
            .ok_or(InsuranceFundError::MathOverflow)?;

        self.slashed_accounts = self.slashed_accounts
            .checked_add(1)
            .ok_or(InsuranceFundError::MathOverflow)?;

        Ok(())
    }
}