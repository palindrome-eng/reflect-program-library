use anchor_lang::prelude::msg;
use litesvm::{
    LiteSVM
};
use solana_sdk::{
    pubkey::{
        Pubkey,
    },
    signature::{Keypair, Signer},
    transaction::{
        Transaction,
    },
};
use solana_sdk_ids::system_program;
use rlp_client::{InitializeRlpBuilder, RLP_ID, SETTINGS_DISCRIMINATOR, Settings};

pub mod helpers;
pub use helpers::pda::{
    derive_permissions_pda,
    derive_settings_pda
};

#[test]
fn test_initialize_rlp_instruction() {
    let mut svm = LiteSVM::new();
    let program_bytes = include_bytes!("../../../target/deploy/rlp.so");

    svm.add_program(
        RLP_ID, 
        program_bytes
    ).unwrap();

    let signer = Keypair::new();
    svm.airdrop(&signer.pubkey(), 10_000_000_000).unwrap();

    let (settings, settings_bump) = derive_settings_pda();
    let (permissions, permissions_bump) = derive_permissions_pda(signer.pubkey());

    let instruction = InitializeRlpBuilder::new()
        .settings(settings)
        .signer(signer.pubkey())
        .permissions(permissions)
        .system_program(system_program::ID)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&signer.pubkey()),
        &[&signer],
        svm.latest_blockhash(),
    );

    match svm.send_transaction(tx) {
        Ok(metadata) => {
            msg!("Transaction successful: {:?}", metadata);
        }
        Err(error) => {
            msg!("Transaction failed with logs: {:?}", error.meta.pretty_logs());
            panic!("Transaction failed: {:?}", error);
        }
    }

    let settings_account_info = svm.get_account(&settings).unwrap();
    let settings_data = Settings::from_bytes(&settings_account_info.data).unwrap();

    assert_eq!(settings_data.liquidity_pools, 0);
    assert_eq!(settings_data.assets, 0);
    assert_eq!(settings_data.discriminator, SETTINGS_DISCRIMINATOR);
    assert_eq!(settings_data.bump, settings_bump);
    assert_eq!(settings_account_info.owner, RLP_ID);
    assert_eq!(settings_data.access_control.access_map.action_permissions.len(), 18);
    assert_eq!(settings_data.access_control.access_map.mapping_count, 16);
    assert_eq!(settings_data.access_control.killswitch.frozen, 0);
}