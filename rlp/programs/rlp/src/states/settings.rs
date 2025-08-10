use anchor_lang::prelude::*;
use crate::states::AccessControl;

#[account]
#[derive(InitSpace, Default)]
pub struct Settings {
    pub bump: u8,
    pub liquidity_pools: u8,
    pub assets: u8,
    pub access_control: AccessControl,
}

impl Settings {}