use anchor_lang::prelude::*;
use crate::helpers::{
    get_price_from_pyth,
    get_price_from_switchboard, OraclePrice
};
use crate::errors::InsuranceFundError;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug, InitSpace)]
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
#[derive(InitSpace)]
pub struct Asset {
    pub mint: Pubkey,
    pub oracle: Oracle,
}

impl Asset {
    #[inline(never)]
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
}