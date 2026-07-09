use solana_sdk::{
    pubkey::Pubkey,
};
use rlp::constants::{PERMISSIONS_SEED, SETTINGS_SEED};
use rlp_client::generated::programs::RLP_ID;

/// Anchor's standard event_authority PDA seed. Used by every instruction
/// annotated with #[event_cpi] (audit-E09 fix).
pub const EVENT_AUTHORITY_SEED: &[u8] = b"__event_authority";

pub fn derive_event_authority() -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[EVENT_AUTHORITY_SEED],
        &RLP_ID,
    )
}

pub fn derive_settings_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            SETTINGS_SEED.as_bytes(),
        ], 
        &RLP_ID
    )
}

pub fn derive_permissions_pda(user: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PERMISSIONS_SEED.as_bytes(),
            &user.to_bytes(),
        ], 
        &RLP_ID
    )
}