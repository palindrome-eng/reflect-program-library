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
pub const ORACLE_MAXIMUM_AGE: u64 = 2 * 60;

#[constant]
pub const BPS_PRECISION: u128 = 10_000;

#[constant]
pub const PRECISION: u32 = 18;

#[constant]
pub const MAX_SLASH_BPS: u64 = 1_000;

#[constant]
pub const BPS_DENOMINATOR: u64 = 10_000;

#[constant]
pub const DEAD_SHARES: u64 = 1_000_000;

pub const DOPPLER_ORACLE_PROGRAM_ID: Pubkey = Pubkey::new_from_array([
    0x05, 0xbe, 0xb9, 0xd8, 0x8c, 0xb5, 0xc1, 0xa2,
    0x1e, 0x48, 0xe9, 0x94, 0x3b, 0x25, 0x84, 0xd6,
    0xe9, 0x30, 0x52, 0x66, 0x2a, 0x83, 0x99, 0x72,
    0x3f, 0xcd, 0xac, 0x29, 0x36, 0xe1, 0x3b, 0x93,
]);

#[constant]
pub const DOPPLER_MAX_STALENESS: u64 = 200;
