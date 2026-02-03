use solana_sdk::{
    pubkey::Pubkey,
};
use rlp::constants::{PERMISSIONS_SEED, SETTINGS_SEED};
use rlp_client::RLP_ID;

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