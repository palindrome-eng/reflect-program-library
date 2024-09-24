use anchor_lang::prelude::*;

#[event]
pub struct RestakeEvent {
    pub from: Pubkey,
    pub asset: Pubkey,
    pub amount: u64,
    pub lockup_ts: u64
}

#[event]
pub struct WithdrawEvent {
    pub from: Pubkey,
    pub asset: Pubkey,
    pub base_amount: u64,
    pub reward_amount: u64
}

#[event]
pub struct DepositRewardEvent {
    pub from: Pubkey,
    pub asset: Pubkey,
    pub amount: u64
}

#[event]
pub struct InitializeSlashEvent {
    pub id: u64,
    pub lockup: Pubkey,
    pub asset: Pubkey,
    pub amount: u64,
    pub slot: u64
}

#[event]
pub struct ProcessSlashEvent {
    pub progress_accounts: u64,
    pub target_accounts: u64,
    pub progress_amount: u64,
    pub target_amount: u64,
}

#[event]
pub struct FinalizeSlash {
    pub id: u64,
    pub amount: u64,
}