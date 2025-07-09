use anchor_lang::prelude::*;

#[event]
pub struct RestakeEvent {
    pub from: Pubkey,
    pub asset: Pubkey,
    pub amount: u64,
    pub lockup_ts: u64
}

#[event]
pub struct RequestWithdrawEvent {
    pub authority: Pubkey,
    pub liquidity_pool_id: u8,
    pub amount: u64,
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
    pub authority: Pubkey,
    pub asset: Pubkey,
    pub amount: u64
}

#[event]
pub struct AddAssetEvent {
    pub admin: Pubkey,
    pub asset: Pubkey,
    pub oracle: Pubkey,
}

#[event]
pub struct InitializeRlp {
    pub caller: Pubkey,
}

#[event]
pub struct InitializeLockupEvent {
    pub admin: Pubkey,
    pub lockup: Pubkey,
    pub asset: Pubkey,
    pub duration: u64,
}

#[event]
pub struct ManageFreezeEvent {
    pub admin: Pubkey,
    pub frozen: bool
}