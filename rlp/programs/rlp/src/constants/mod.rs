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

#[constant]
pub const ORACLE_MAXIMUM_AGE: u64 = 2 * 60; // 2 minutes, decrease for prod.