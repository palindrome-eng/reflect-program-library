import { address } from "@solana/kit";

export const SETTINGS_SEED = "settings";
export const PERMISSIONS_SEED = "permissions";
export const LIQUIDITY_POOL_SEED = "liquidity_pool";
export const ASSET_SEED = "asset";
export const COOLDOWN_SEED = "cooldown";

/** Anchor `#[event_cpi]` standard PDA seed (audit-E09). */
export const EVENT_AUTHORITY_SEED = "__event_authority";

/**
 * Reflect proxy program PDA seed for ProxyState accounts. Used when deriving
 * the senior tranche's ProxyState address from a branded mint.
 * (audit-M06 / NAV slash dependency.)
 */
export const PROXY_STATE_SEED = "proxy";

/** Reflect proxy program ID (mainnet/dev). */
export const PROXY_PROGRAM_ADDRESS =
  "pRoxYU64BSjv8HbhENna8a7LVCrkzzNrnvbYuTwas8C" as const;

/** Maximum cooldown_duration allowed at initialize_lp (365 days, audit-E03). */
export const MAX_COOLDOWN_DURATION_SECONDS = 365 * 24 * 60 * 60;

/**
 * Pyth confidence-interval cap used by the program's oracle reader
 * (audit-3 / audit-33). conf × this ≤ price (i.e., conf within ~2% of price).
 */
export const MAX_ORACLE_CONFIDENCE_RATIO = 50;

/** Mainnet staging deployment of this program — real tokens/oracles, isolated state. Pass to PdaClient overrides. */
export const STAGING_PROGRAM_ID = address("GSEYK2FtDLywAoxn2mioXCft9FWH6Zn4VmKUArGyTVj8");
