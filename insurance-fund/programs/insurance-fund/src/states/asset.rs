use anchor_lang::prelude::*;

use crate::errors::InsuranceFundError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq)]
pub enum Oracle {
    Pyth(Pubkey),
    Switchboard(Pubkey)
}

impl Oracle {
    pub const SIZE: usize = 1 + 32;

    pub fn key(&self) -> &Pubkey {
        match self {
            Oracle::Pyth(key) | Oracle::Switchboard(key) => key,
        }
    }
}

#[account]
pub struct Asset {
    pub mint: Pubkey,
    pub oracle: Oracle,
    pub tvl: u64
}

impl Asset {
    pub const SIZE: usize = 8 + 32 + 8 + Oracle::SIZE;

    pub fn increase_tvl(
        &mut self,
        amount: u64
    ) -> Result<()> {
        self.tvl.checked_add(amount)
            .ok_or(InsuranceFundError::MathOverflow)?;

        Ok(())
    }

    pub fn decrease_tvl(
        &mut self,
        amount: u64
    ) -> Result<()> {
        self.tvl.checked_sub(amount)
            .ok_or(InsuranceFundError::MathOverflow)?;
        
        Ok(())
    }
}