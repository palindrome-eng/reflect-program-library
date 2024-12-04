use anchor_lang::prelude::*;
use crate::helpers::{
    get_price_from_pyth,
    get_price_from_switchboard, OraclePrice
};
use crate::errors::InsuranceFundError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug)]
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
    // For stats
    pub tvl: u64,
    pub lockups: u64,
    pub deposits: u64,
}

impl Asset {
    pub const SIZE: usize = 8 + 32 + 3 * 8 + Oracle::SIZE;

    pub fn increase_tvl(
        &mut self,
        amount: u64
    ) -> Result<()> {
        self.tvl = self.tvl.checked_add(amount)
            .ok_or(InsuranceFundError::MathOverflow)?;

        Ok(())
    }

    pub fn decrease_tvl(
        &mut self,
        amount: u64
    ) -> Result<()> {
        self.tvl = self.tvl
            .checked_sub(amount)
            .ok_or(InsuranceFundError::MathOverflow)?;
        
        Ok(())
    }

    pub fn get_price(
        &self,
        account: &AccountInfo,
        clock: &Clock
    ) -> Result<OraclePrice> {
        match self.oracle {
            Oracle::Pyth(_) => get_price_from_pyth(account, clock),
            Oracle::Switchboard(_) => get_price_from_switchboard(account, clock)
        }
    }
    
    pub fn add_lockup(
        &mut self,
    ) -> Result<()> {
        self.lockups = self.lockups
            .checked_add(1)
            .ok_or(InsuranceFundError::MathOverflow)?;

        Ok(())
    }

    pub fn add_deposit(
        &mut self,
    ) -> Result<()> {
        self.deposits = self.lockups
            .checked_add(1)
            .ok_or(InsuranceFundError::MathOverflow)?;
        
        Ok(())
    }
}