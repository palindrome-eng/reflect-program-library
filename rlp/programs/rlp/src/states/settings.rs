use crate::states::AccessControl;
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, Default)]
pub struct Settings {
    pub bump: u8,
    pub liquidity_pools: u8,
    pub assets: u8,
    pub access_control: AccessControl,
    pub swap_fee_bps: u16, // 30 bps = 0.3%
}

