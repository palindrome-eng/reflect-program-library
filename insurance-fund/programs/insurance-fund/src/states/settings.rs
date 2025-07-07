use anchor_lang::prelude::*;
use crate::errors::InsuranceFundError;

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, PartialEq, InitSpace)]
pub struct SharesConfig {
    pub hot_wallet_share_bps: u64,
    pub cold_wallet_share_bps: u64
}

impl SharesConfig {
    pub const SIZE: usize = 2 * 8;
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy, PartialEq, InitSpace)]
pub struct RewardConfig {
    pub main: Pubkey,
}

impl RewardConfig {
    pub const SIZE: usize = 32;
}

#[account]
#[derive(InitSpace)]
pub struct Settings {
    pub bump: u8,
    pub lockups: u64,
    pub liquidity_pools: u64,
    pub cooldown_duration: u64,
    pub reward_config: RewardConfig,
    pub frozen: bool,
}

impl Settings {
    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    pub fn unfreeze(&mut self) {
        self.frozen = false;
    }
}