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
    pub superadmins: u8,
    pub cold_wallet: Pubkey,
    pub lockups: u64,
    pub cooldown_duration: u64,
    pub shares_config: SharesConfig,
    pub reward_config: RewardConfig,
    pub frozen: bool,
    pub debt_records: u64,
}

impl Settings {
    pub fn freeze(&mut self) {
        self.frozen = true;
    }

    pub fn unfreeze(&mut self) {
        self.frozen = false;
    }

    pub fn calculate_cold_wallet_deposit(
        &self,
        amount: u64
    ) -> Result<u64> {
        let result = amount
            .checked_mul(self.shares_config.cold_wallet_share_bps)
            .ok_or(InsuranceFundError::MathOverflow)?
            .checked_div(10_000)
            .ok_or(InsuranceFundError::MathOverflow)?;

        Ok(result)
    }

    pub fn calculate_hot_wallet_deposit(
        &self,
        amount: u64
    ) -> Result<u64> {
        let result = amount
            .checked_mul(self.shares_config.hot_wallet_share_bps)
            .ok_or(InsuranceFundError::MathOverflow)?
            .checked_div(10_000)
            .ok_or(InsuranceFundError::MathOverflow)?;

        Ok(result)
    }
}