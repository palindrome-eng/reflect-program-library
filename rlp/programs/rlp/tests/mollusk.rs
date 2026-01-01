use std::cell::RefCell;
use mollusk_svm::Mollusk;
use rlp::constants::{PERMISSIONS_SEED, SETTINGS_SEED};
use rlp_client::{InitializeRlpBuilder, RLP_ID};
use solana_sdk::{account::Account, native_loader, pubkey::Pubkey};
use solana_sdk_ids::system_program;

pub mod helpers;
pub use helpers::pda::{
    derive_permissions_pda,
    derive_settings_pda
};

// Program ID constant
fn program_id() -> Pubkey {
    Pubkey::new_from_array(RLP_ID.to_bytes())
}

// Thread-local Mollusk instance - shared across tests when run with --test-threads=1
thread_local! {
    static MOLLUSK: RefCell<Mollusk> = RefCell::new(
        Mollusk::new(&program_id(), "../../target/deploy/rlp")
    );
}

// Helper to run code with the shared Mollusk instance
fn with_mollusk<F, R>(f: F) -> R
where
    F: FnOnce(&Mollusk) -> R,
{
    MOLLUSK.with(|m| f(&m.borrow()))
}

#[test]
fn test_initialize_rlp_instruction() {
    let program_id = program_id();
    let signer = Pubkey::new_unique();
    
    let (permissions, _permissions_bump) = derive_permissions_pda(signer);
    let (settings, _settings_bump) = derive_settings_pda();

    // Build instruction using the client library
    let instruction = InitializeRlpBuilder::new()
        .signer(signer.into())
        .permissions(permissions.into())
        .settings(settings.into())
        .system_program(system_program::ID.into())
        .instruction();

    println!("Created instruction: {:?}", instruction);

    // Convert instruction for mollusk
    let ix = solana_sdk::instruction::Instruction {
        program_id,
        accounts: instruction
            .accounts
            .iter()
            .map(|a| solana_sdk::instruction::AccountMeta {
                pubkey: Pubkey::new_from_array(a.pubkey.to_bytes()),
                is_signer: a.is_signer,
                is_writable: a.is_writable,
            })
            .collect(),
        data: instruction.data,
    };

    // Set up accounts for the test
    let accounts = vec![
        (signer, Account::new(1_000_000_000, 0, &system_program::ID)),
        (permissions, Account::new(0, 0, &system_program::ID)),
        (settings, Account::new(0, 0, &system_program::ID)),
        (
            system_program::ID,
            Account {
                executable: true,
                lamports: 0,
                data: vec![],
                owner: native_loader::ID,
                rent_epoch: 0,
            },
        ),
    ];

    // Process the instruction using shared Mollusk instance
    let result = with_mollusk(|mollusk| {
        mollusk.process_instruction(&ix, &accounts)
    });

    println!("Result: {:?}", result);
}
