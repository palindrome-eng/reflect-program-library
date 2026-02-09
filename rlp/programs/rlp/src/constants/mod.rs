use anchor_lang::prelude::*;

#[constant]
pub const SETTINGS_SEED: &str = "settings";

#[constant]
pub const DEPOSIT_SEED: &str = "deposit";

#[constant]
pub const SLASH_SEED: &str = "slash";

#[constant]
pub const ASSET_SEED: &str = "asset";

#[constant]
pub const REWARD_BOOST_SEED: &str = "reward_boost";

#[constant]
pub const REWARD_POOL_SEED: &str = "reward_pool";

#[constant]
pub const PRICE_PRECISION: i32 = 9;

#[constant]
pub const INTENT_SEED: &str = "intent";

#[constant]
pub const COOLDOWN_SEED: &str = "cooldown";

#[constant]
pub const COOLDOWN_VAULT_SEED: &str = "cooldown_vault";

#[constant]
pub const PERMISSIONS_SEED: &str = "permissions";

#[constant]
pub const DEPOSIT_RECEIPT_VAULT_SEED: &str = "deposit_receipt_vault";

#[constant]
pub const LIQUIDITY_POOL_SEED: &str = "liquidity_pool";

#[constant]
pub const LIQUIDITY_POOL_LOCKUP_SEED: &str = "lp_lockup";

/// Maximum oracle data age in seconds
/// Security Fix: Reduced from 3600s to 60s to prevent stale price exploitation
#[constant]
pub const ORACLE_MAXIMUM_AGE: u64 = 60; // 60 seconds

/// Dead shares to prevent LP token inflation attack
/// These shares are minted on pool initialization and never redeemable
#[constant]
pub const DEAD_SHARES: u64 = 1_000_000_000;

/// Basis points denominator
#[constant]
pub const BPS_DENOMINATOR: u64 = 10_000;

/// Maximum slash amount per transaction (as percentage of pool in BPS)
/// Security Fix: Limits centralization risk from broad slash permissions
#[constant]
pub const MAX_SLASH_BPS: u64 = 1_000; // 10% max per slash transaction
