use anchor_lang::prelude::*;

#[constant]
pub const CONFIG_SEED: &str = "config";

#[constant]
pub const VAULT_SEED: &str = "vault";

#[constant]
pub const VAULT_POOL_SEED: &str = "vault_pool";

/// Virtual offset added to both deposited amount and receipt token supply in share calculations.
/// This prevents the first depositor inflation attack by ensuring there's always a meaningful
/// baseline for exchange rate calculations, without requiring any actual token operations.
#[constant]
pub const VIRTUAL_OFFSET: u64 = 1_000;