use anchor_lang::prelude::*;
use crate::states::*;

#[event]
pub struct RestakeEvent {
    pub from: Pubkey,
    pub asset: Pubkey,
    pub amount: u64
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
    pub amount: u64
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
pub struct InitializeRlpEvent {
    pub caller: Pubkey,
}

#[event]
pub struct UpdateActionRoleEvent {
    pub action: Action,
    pub role: Role,
    pub update: Update
}

#[event]
pub struct CreatePermissionAccountEvent {
    pub admin: Pubkey,
    pub new_admin: Pubkey
}

#[event]
pub struct FreezeProtocolActionEvent {
    pub action: Action,
    pub freeze: bool
}

#[event]
pub struct InitializeLiquidityPoolEvent {
    pub admin: Pubkey,
    pub liquidity_pool: Pubkey,
    pub lp_token: Pubkey,
}

#[event]
pub struct UpdateRoleHolderEvent {
    pub address: Pubkey,
    pub role: Role,
    pub update: Update
}

#[event]
pub struct UpdateDepositCapEvent {
    pub admin: Pubkey,
    pub liquidity_pool: Pubkey,
    pub new_cap: Option<u64>
}

#[event]
pub struct SlashEvent {
    pub admin: Pubkey,
    pub liquidity_pool: Pubkey,
    pub amount: u64,
    pub mint: Pubkey
}

#[event]
pub struct SwapEvent {
    pub signer: Pubkey,
    pub liquidity_pool: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub private: bool
}