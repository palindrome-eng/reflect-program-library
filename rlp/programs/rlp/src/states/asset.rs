use anchor_lang::prelude::*;
use crate::helpers::{
    get_price_from_pyth,
    get_price_from_switchboard, OraclePrice
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Debug, InitSpace)]
pub enum Oracle {
    Pyth(Pubkey),
    Switchboard(Pubkey)
}

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone, Copy, PartialEq, Debug)]
pub enum AccessLevel {
    Public,
    Private
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
    pub access_level: AccessLevel,
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

    pub fn is_public(&self) -> bool {
        self.access_level == AccessLevel::Public
    }
}