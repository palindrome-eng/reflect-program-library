use crate::states::AccessControl;
use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace, Default)]
pub struct Settings {
    pub bump: u8,
    pub liquidity_pools: u8,
    pub assets: u8,
    pub frozen: bool,
    pub access_control: AccessControl,
    pub swap_fee_bps: u16, // 30 bps = 0.3%
}

impl Settings {
    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    pub fn unfreeze(&mut self) {
        self.frozen = false;
    }
}

