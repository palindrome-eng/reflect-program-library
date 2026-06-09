use std::cell::RefCell;
use mollusk_svm::Mollusk;
use mollusk_svm::result::Check;
use rlp::constants::ASSET_SEED;
use rlp_client::generated::types::{
    // Types
    AccessLevel,
    Action,
    Role,
    Update
};
use rlp_client::generated::instructions::{
    InitializeRlpBuilder,
    AddAssetBuilder,
    FreezeFunctionalityBuilder,
    UpdateActionRoleBuilder,
    CreatePermissionAccountBuilder,
    UpdateRoleHolderBuilder,
};
use rlp_client::generated::accounts::{
    SETTINGS_DISCRIMINATOR,
    USER_PERMISSIONS_DISCRIMINATOR,
    Settings,
    UserPermissions
};
use rlp_client::generated::programs::RLP_ID;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    native_loader,
    pubkey::Pubkey,
};
use solana_sdk::system_program;

// SPL Token program ID
const SPL_TOKEN_ID: Pubkey = solana_sdk::pubkey!("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");

pub mod helpers;
pub use helpers::pda::{derive_event_authority, derive_permissions_pda, derive_settings_pda};

/// Returns the program-as-account entry for the RLP program ID, which is
/// required by every instruction that uses #[event_cpi] (audit-E09 fix).
/// Mollusk pre-loads the program; this is just so the instruction's account
/// list resolves.
fn rlp_program_account() -> Account {
    Account {
        executable: true,
        lamports: 0,
        data: vec![],
        owner: solana_sdk::bpf_loader_upgradeable::ID,
        rent_epoch: 0,
    }
}

/// Pair (event_authority_pda, empty_account) used by every #[event_cpi]
/// instruction. The PDA itself doesn't need any data — it's only used as a
/// CPI signer by the emit_cpi! macro.
fn event_cpi_accounts() -> Vec<(Pubkey, Account)> {
    let (event_authority, _) = derive_event_authority();
    vec![
        (event_authority, empty_account()),
        (Pubkey::new_from_array(RLP_ID.to_bytes()), rlp_program_account()),
    ]
}

// Pyth program ID
const PYTH_PROGRAM_ID: Pubkey = solana_sdk::pubkey!("rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ");

// Program ID constant
fn program_id() -> Pubkey {
    Pubkey::new_from_array(RLP_ID.to_bytes())
}

// Thread-local Mollusk instance - shared across tests when run with --test-threads=1.
// SPL Token and SPL Associated Token programs are loaded so instructions that
// CPI into them (initialize_lp, deposit, withdraw, swap, slash, etc.) work.
thread_local! {
    static MOLLUSK: RefCell<Mollusk> = RefCell::new({
        let mut m = Mollusk::new(&program_id(), "../../target/deploy/rlp");
        mollusk_svm_programs_token::token::add_program(&mut m);
        mollusk_svm_programs_token::associated_token::add_program(&mut m);
        m
    });
}

// Helper to run code with the shared Mollusk instance
fn with_mollusk<F, R>(f: F) -> R
where
    F: FnOnce(&Mollusk) -> R,
{
    MOLLUSK.with(|m| f(&m.borrow()))
}

/// Run an instruction with a specific clock unix_timestamp. Resets to 0 on
/// return so other tests that share the thread-local Mollusk see a clean state.
fn with_mollusk_clock<F, R>(unix_timestamp: i64, f: F) -> R
where
    F: FnOnce(&Mollusk) -> R,
{
    MOLLUSK.with(|cell| {
        cell.borrow_mut().sysvars.clock.unix_timestamp = unix_timestamp;
        let r = f(&cell.borrow());
        cell.borrow_mut().sysvars.clock.unix_timestamp = 0;
        r
    })
}

/// Derives an asset PDA from its mint
fn derive_asset_pda(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[ASSET_SEED.as_bytes(), &mint.to_bytes()],
        &RLP_ID,
    )
}

/// Creates mock Pyth PriceUpdateV2 data (133 bytes).
///
/// VerificationLevel::Full is a 1-byte Borsh enum tag (no payload). The
/// previous mock wrote 2 bytes for it, which shifted every subsequent field
/// by one and produced garbage feed_id, price, and publish_time when the
/// audit-3 hardening enabled strict deserialization via try_deserialize.
fn create_mock_pyth_price_data(price: i64, exponent: i32, publish_time: i64) -> Vec<u8> {
    let mut data = Vec::with_capacity(133);

    // Discriminator for PriceUpdateV2
    data.extend_from_slice(&[34, 241, 35, 99, 157, 126, 244, 205]);
    data.extend_from_slice(&[0u8; 32]); // write_authority
    data.push(1); // verification_level (Full) — 1-byte Borsh enum tag, no payload
    data.extend_from_slice(&[1u8; 32]); // feed_id
    data.extend_from_slice(&price.to_le_bytes());
    data.extend_from_slice(&100u64.to_le_bytes()); // conf
    data.extend_from_slice(&exponent.to_le_bytes());
    data.extend_from_slice(&publish_time.to_le_bytes());
    data.extend_from_slice(&(publish_time - 1).to_le_bytes()); // prev_publish_time
    data.extend_from_slice(&price.to_le_bytes()); // ema_price
    data.extend_from_slice(&100u64.to_le_bytes()); // ema_conf
    data.extend_from_slice(&1u64.to_le_bytes()); // posted_slot

    assert_eq!(data.len(), 133);
    data
}

/// Creates a mock SPL token mint account
fn create_mock_mint_account() -> Account {
    // SPL Token Mint: 82 bytes
    // - mint_authority (36 bytes: 4 option + 32 pubkey)
    // - supply (8 bytes)
    // - decimals (1 byte)
    // - is_initialized (1 byte)
    // - freeze_authority (36 bytes: 4 option + 32 pubkey)
    let mut data = vec![0u8; 82];
    
    // mint_authority: Some(pubkey) - option tag 1 + 32 zero bytes for pubkey
    data[0] = 1;
    // supply at offset 36
    // decimals at offset 44
    data[44] = 9; // 9 decimals
    // is_initialized at offset 45
    data[45] = 1;
    // freeze_authority: None - option tag 0 at offset 46
    data[46] = 0;
    
    Account {
        lamports: 1_000_000,
        data,
        owner: SPL_TOKEN_ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// Helper to convert rlp_client instruction to solana_sdk instruction
fn convert_instruction(client_ix: solana_sdk::instruction::Instruction) -> Instruction {
    Instruction {
        program_id: Pubkey::new_from_array(client_ix.program_id.to_bytes()),
        accounts: client_ix
            .accounts
            .iter()
            .map(|a| AccountMeta {
                pubkey: Pubkey::new_from_array(a.pubkey.to_bytes()),
                is_signer: a.is_signer,
                is_writable: a.is_writable,
            })
            .collect(),
        data: client_ix.data,
    }
}

/// System program account for tests
fn system_program_account() -> Account {
    Account {
        executable: true,
        lamports: 0,
        data: vec![],
        owner: native_loader::ID,
        rent_epoch: 0,
    }
}

/// New empty account for PDA initialization
fn empty_account() -> Account {
    Account::new(0, 0, &system_program::ID)
}

/// Funded signer account
fn signer_account() -> Account {
    Account::new(10_000_000_000, 0, &system_program::ID)
}

/// Get updated account from result
fn get_result_account(result: &mollusk_svm::result::InstructionResult, index: usize) -> Account {
    result.resulting_accounts[index].1.clone()
}

// ============================================================================
// INITIALIZATION TESTS
// ============================================================================

#[test]
fn test_initialize_rlp_instruction() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();

    let ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .swap_fee_bps(30)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&ix, &accounts, &[Check::success()])
    });

    // Verify settings account was created
    let settings_account = get_result_account(&result, 2);
    let settings_data = Settings::from_bytes(&settings_account.data).unwrap();

    assert_eq!(settings_data.discriminator, SETTINGS_DISCRIMINATOR);
    assert_eq!(settings_data.liquidity_pools, 0);
    assert_eq!(settings_data.assets, 0);
    assert_eq!(settings_data.access_control.access_map.action_permissions.len(), 18);
    assert_eq!(settings_data.access_control.killswitch.frozen, 0);
}

// ============================================================================
// FREEZE/UNFREEZE TESTS
// ============================================================================

#[test]
fn test_freeze_protocol() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();

    // First initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .swap_fee_bps(30)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let init_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&init_ix, &accounts, &[Check::success()])
    });

    // Get updated accounts
    let updated_settings = get_result_account(&init_result, 2);
    let updated_permissions = get_result_account(&init_result, 1);

    // Now freeze
    let freeze_ix = convert_instruction(
        FreezeFunctionalityBuilder::new()
            .admin(signer.into())
            .settings(settings.into())
            .admin_permissions(permissions.into())
            .system_program(system_program::ID.into())
            .action(Action::FreezeDeposit)
            .freeze(true)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut freeze_accounts = vec![
        (signer, signer_account()),
        (settings, updated_settings),
        (permissions, updated_permissions),
        (system_program::ID, system_program_account()),
    ];
    freeze_accounts.extend(event_cpi_accounts());

    let freeze_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&freeze_ix, &freeze_accounts, &[Check::success()])
    });

    // Verify freeze
    let final_settings = get_result_account(&freeze_result, 1);
    let settings_data = Settings::from_bytes(&final_settings.data).unwrap();

    let deposit_mask = 1u32 << (Action::Deposit as u8);
    assert!(
        (settings_data.access_control.killswitch.frozen & deposit_mask) != 0,
        "Deposit should be frozen"
    );
}

#[test]
fn test_unfreeze_protocol() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .swap_fee_bps(30)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let init_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&init_ix, &accounts, &[Check::success()])
    });

    let mut current_settings = get_result_account(&init_result, 2);
    let mut current_permissions = get_result_account(&init_result, 1);

    // Freeze
    let freeze_ix = convert_instruction(
        FreezeFunctionalityBuilder::new()
            .admin(signer.into())
            .settings(settings.into())
            .admin_permissions(permissions.into())
            .system_program(system_program::ID.into())
            .action(Action::FreezeDeposit)
            .freeze(true)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut freeze_accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions.clone()),
        (system_program::ID, system_program_account()),
    ];
    freeze_accounts.extend(event_cpi_accounts());

    let freeze_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&freeze_ix, &freeze_accounts, &[Check::success()])
    });

    current_settings = get_result_account(&freeze_result, 1);

    // Unfreeze
    let unfreeze_ix = convert_instruction(
        FreezeFunctionalityBuilder::new()
            .admin(signer.into())
            .settings(settings.into())
            .admin_permissions(permissions.into())
            .system_program(system_program::ID.into())
            .action(Action::FreezeDeposit)
            .freeze(false)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut unfreeze_accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions),
        (system_program::ID, system_program_account()),
    ];
    unfreeze_accounts.extend(event_cpi_accounts());

    let unfreeze_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&unfreeze_ix, &unfreeze_accounts, &[Check::success()])
    });

    // Verify unfreeze
    let final_settings = get_result_account(&unfreeze_result, 1);
    let settings_data = Settings::from_bytes(&final_settings.data).unwrap();
    assert_eq!(settings_data.access_control.killswitch.frozen, 0);
}

#[test]
fn test_freeze_multiple_actions() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .swap_fee_bps(30)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let init_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&init_ix, &accounts, &[Check::success()])
    });

    let mut current_settings = get_result_account(&init_result, 2);
    let current_permissions = get_result_account(&init_result, 1);

    // Freeze Withdraw
    let freeze_withdraw_ix = convert_instruction(
        FreezeFunctionalityBuilder::new()
            .admin(signer.into())
            .settings(settings.into())
            .admin_permissions(permissions.into())
            .system_program(system_program::ID.into())
            .action(Action::FreezeWithdraw)
            .freeze(true)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions.clone()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&freeze_withdraw_ix, &accounts, &[Check::success()])
    });

    current_settings = get_result_account(&result, 1);

    // Freeze Slash
    let freeze_slash_ix = convert_instruction(
        FreezeFunctionalityBuilder::new()
            .admin(signer.into())
            .settings(settings.into())
            .admin_permissions(permissions.into())
            .system_program(system_program::ID.into())
            .action(Action::FreezeSlash)
            .freeze(true)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&freeze_slash_ix, &accounts, &[Check::success()])
    });

    // Verify both frozen
    let final_settings = get_result_account(&result, 1);
    let settings_data = Settings::from_bytes(&final_settings.data).unwrap();

    let withdraw_mask = 1u32 << (Action::Withdraw as u8);
    let slash_mask = 1u32 << (Action::Slash as u8);

    assert!((settings_data.access_control.killswitch.frozen & withdraw_mask) != 0);
    assert!((settings_data.access_control.killswitch.frozen & slash_mask) != 0);
}

// ============================================================================
// ACCESS CONTROL TESTS
// ============================================================================

#[test]
fn test_set_restaking_action_to_public() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .swap_fee_bps(30)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let init_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&init_ix, &accounts, &[Check::success()])
    });

    let current_settings = get_result_account(&init_result, 2);
    let current_permissions = get_result_account(&init_result, 1);

    // Update action role
    let update_ix = convert_instruction(
        UpdateActionRoleBuilder::new()
            .admin(signer.into())
            .settings(settings.into())
            .admin_permissions(permissions.into())
            .system_program(system_program::ID.into())
            .action(Action::Deposit)
            .role(Role::PUBLIC)
            .update(Update::Add)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&update_ix, &accounts, &[Check::success()])
    });

    // Verify
    let final_settings = get_result_account(&result, 1);
    let settings_data = Settings::from_bytes(&final_settings.data).unwrap();

    let restake_mapping = settings_data
        .access_control
        .access_map
        .action_permissions
        .iter()
        .find(|m| m.action == Action::Deposit)
        .unwrap();

    assert!(restake_mapping.allowed_roles.contains(&Role::PUBLIC));
}

#[test]
fn test_set_withdraw_action_to_public() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .swap_fee_bps(30)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let init_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&init_ix, &accounts, &[Check::success()])
    });

    let current_settings = get_result_account(&init_result, 2);
    let current_permissions = get_result_account(&init_result, 1);

    // Update action role
    let update_ix = convert_instruction(
        UpdateActionRoleBuilder::new()
            .admin(signer.into())
            .settings(settings.into())
            .admin_permissions(permissions.into())
            .system_program(system_program::ID.into())
            .action(Action::Withdraw)
            .role(Role::PUBLIC)
            .update(Update::Add)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&update_ix, &accounts, &[Check::success()])
    });

    // Verify
    let final_settings = get_result_account(&result, 1);
    let settings_data = Settings::from_bytes(&final_settings.data).unwrap();

    let withdraw_mapping = settings_data
        .access_control
        .access_map
        .action_permissions
        .iter()
        .find(|m| m.action == Action::Withdraw)
        .unwrap();

    assert!(withdraw_mapping.allowed_roles.contains(&Role::PUBLIC));
}

#[test]
fn test_update_action_role_add_and_remove() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .swap_fee_bps(30)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let init_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&init_ix, &accounts, &[Check::success()])
    });

    let mut current_settings = get_result_account(&init_result, 2);
    let current_permissions = get_result_account(&init_result, 1);

    // Add TESTEE role to SuspendDeposits
    let add_ix = convert_instruction(
        UpdateActionRoleBuilder::new()
            .admin(signer.into())
            .settings(settings.into())
            .admin_permissions(permissions.into())
            .system_program(system_program::ID.into())
            .action(Action::SuspendDeposits)
            .role(Role::TESTEE)
            .update(Update::Add)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions.clone()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let add_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&add_ix, &accounts, &[Check::success()])
    });

    current_settings = get_result_account(&add_result, 1);

    // Verify role was added
    let settings_data = Settings::from_bytes(&current_settings.data).unwrap();
    let mapping = settings_data
        .access_control
        .access_map
        .action_permissions
        .iter()
        .find(|m| m.action == Action::SuspendDeposits)
        .unwrap();
    assert!(mapping.allowed_roles.contains(&Role::TESTEE));

    // Remove TESTEE role
    let remove_ix = convert_instruction(
        UpdateActionRoleBuilder::new()
            .admin(signer.into())
            .settings(settings.into())
            .admin_permissions(permissions.into())
            .system_program(system_program::ID.into())
            .action(Action::SuspendDeposits)
            .role(Role::TESTEE)
            .update(Update::Remove)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let remove_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&remove_ix, &accounts, &[Check::success()])
    });

    // Verify role was removed
    let final_settings = get_result_account(&remove_result, 1);
    let settings_data = Settings::from_bytes(&final_settings.data).unwrap();
    let mapping = settings_data
        .access_control
        .access_map
        .action_permissions
        .iter()
        .find(|m| m.action == Action::SuspendDeposits)
        .unwrap();
    assert!(!mapping.allowed_roles.contains(&Role::TESTEE));
}

#[test]
fn test_action_permissions_for_multiple_roles() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .swap_fee_bps(30)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let init_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&init_ix, &accounts, &[Check::success()])
    });

    let mut current_settings = get_result_account(&init_result, 2);
    let current_permissions = get_result_account(&init_result, 1);

    // Add multiple roles to PrivateSwap
    let roles = [Role::TESTEE, Role::FREEZE, Role::MANAGER];

    for role in roles.iter() {
        let add_ix = convert_instruction(
            UpdateActionRoleBuilder::new()
                .admin(signer.into())
                .settings(settings.into())
                .admin_permissions(permissions.into())
                .system_program(system_program::ID.into())
                .action(Action::Swap)
                .role(*role)
                .update(Update::Add)
                .event_authority(event_authority.into())
                .program(RLP_ID)
                .instruction()
        );

        let mut accounts = vec![
            (signer, signer_account()),
            (settings, current_settings),
            (permissions, current_permissions.clone()),
            (system_program::ID, system_program_account()),
        ];
        accounts.extend(event_cpi_accounts());

        let result = with_mollusk(|mollusk| {
            mollusk.process_and_validate_instruction(&add_ix, &accounts, &[Check::success()])
        });

        current_settings = get_result_account(&result, 1);
    }

    // Verify all roles
    let settings_data = Settings::from_bytes(&current_settings.data).unwrap();
    let swap_mapping = settings_data
        .access_control
        .access_map
        .action_permissions
        .iter()
        .find(|m| m.action == Action::Swap)
        .unwrap();

    for role in roles.iter() {
        assert!(swap_mapping.allowed_roles.contains(role));
    }
}

// ============================================================================
// PERMISSION ACCOUNT TESTS
// ============================================================================

#[test]
fn test_create_permission_account() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .swap_fee_bps(30)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let init_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&init_ix, &accounts, &[Check::success()])
    });

    let current_settings = get_result_account(&init_result, 2);

    // Create new permission account
    let new_admin = Pubkey::new_unique();
    let (new_admin_permissions, _) = derive_permissions_pda(new_admin);

    let create_ix = convert_instruction(
        CreatePermissionAccountBuilder::new()
            .settings(settings.into())
            .new_creds(new_admin_permissions.into())
            .caller(signer.into())
            .system_program(system_program::ID.into())
            .new_admin(new_admin.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (settings, current_settings),
        (new_admin_permissions, empty_account()),
        (signer, signer_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&create_ix, &accounts, &[Check::success()])
    });

    // Verify
    let new_permissions_account = get_result_account(&result, 1);
    let permissions_data = UserPermissions::from_bytes(&new_permissions_account.data).unwrap();

    assert_eq!(permissions_data.discriminator, USER_PERMISSIONS_DISCRIMINATOR);
    assert_eq!(permissions_data.authority, new_admin);
    assert!(permissions_data.protocol_roles.roles.is_empty());
}

#[test]
fn test_create_multiple_permission_accounts() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .swap_fee_bps(30)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let init_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&init_ix, &accounts, &[Check::success()])
    });

    let current_settings = get_result_account(&init_result, 2);

    // Create 5 permission accounts
    for _ in 0..5 {
        let new_admin = Pubkey::new_unique();
        let (new_admin_permissions, _) = derive_permissions_pda(new_admin);

        let create_ix = convert_instruction(
            CreatePermissionAccountBuilder::new()
                .settings(settings.into())
                .new_creds(new_admin_permissions.into())
                .caller(signer.into())
                .system_program(system_program::ID.into())
                .new_admin(new_admin.into())
                .event_authority(event_authority.into())
                .program(RLP_ID)
                .instruction()
        );

        let mut accounts = vec![
            (settings, current_settings.clone()),
            (new_admin_permissions, empty_account()),
            (signer, signer_account()),
            (system_program::ID, system_program_account()),
        ];
        accounts.extend(event_cpi_accounts());

        let result = with_mollusk(|mollusk| {
            mollusk.process_and_validate_instruction(&create_ix, &accounts, &[Check::success()])
        });

        // Verify
        let new_permissions_account = get_result_account(&result, 1);
        let permissions_data = UserPermissions::from_bytes(&new_permissions_account.data).unwrap();
        assert_eq!(permissions_data.authority, new_admin);
    }
}

#[test]
fn test_update_role_holder_add_role() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .swap_fee_bps(30)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let init_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&init_ix, &accounts, &[Check::success()])
    });

    let current_settings = get_result_account(&init_result, 2);
    let current_permissions = get_result_account(&init_result, 1);

    // Create target user permission account
    let target_user = Pubkey::new_unique();
    let (target_user_permissions, _) = derive_permissions_pda(target_user);

    let create_ix = convert_instruction(
        CreatePermissionAccountBuilder::new()
            .settings(settings.into())
            .new_creds(target_user_permissions.into())
            .caller(signer.into())
            .system_program(system_program::ID.into())
            .new_admin(target_user.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (settings, current_settings.clone()),
        (target_user_permissions, empty_account()),
        (signer, signer_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let create_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&create_ix, &accounts, &[Check::success()])
    });

    let target_permissions_account = get_result_account(&create_result, 1);

    // Add CRANK role
    let update_ix = convert_instruction(
        UpdateRoleHolderBuilder::new()
            .admin(signer.into())
            .settings(settings.into())
            .admin_permissions(permissions.into())
            .update_admin_permissions(target_user_permissions.into())
            .system_program(system_program::ID.into())
            .address(target_user.into())
            .role(Role::CRANK)
            .update(Update::Add)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions),
        (target_user_permissions, target_permissions_account),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&update_ix, &accounts, &[Check::success()])
    });

    // Verify
    let final_permissions = get_result_account(&result, 3);
    let permissions_data = UserPermissions::from_bytes(&final_permissions.data).unwrap();
    assert!(permissions_data.protocol_roles.roles.contains(&Role::CRANK));
}

#[test]
fn test_update_role_holder_remove_role() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .swap_fee_bps(30)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let init_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&init_ix, &accounts, &[Check::success()])
    });

    let current_settings = get_result_account(&init_result, 2);
    let current_permissions = get_result_account(&init_result, 1);

    // Create target user permission account
    let target_user = Pubkey::new_unique();
    let (target_user_permissions, _) = derive_permissions_pda(target_user);

    let create_ix = convert_instruction(
        CreatePermissionAccountBuilder::new()
            .settings(settings.into())
            .new_creds(target_user_permissions.into())
            .caller(signer.into())
            .system_program(system_program::ID.into())
            .new_admin(target_user.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (settings, current_settings.clone()),
        (target_user_permissions, empty_account()),
        (signer, signer_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let create_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&create_ix, &accounts, &[Check::success()])
    });

    let mut target_permissions_account = get_result_account(&create_result, 1);

    // Add FREEZE role
    let add_ix = convert_instruction(
        UpdateRoleHolderBuilder::new()
            .admin(signer.into())
            .settings(settings.into())
            .admin_permissions(permissions.into())
            .update_admin_permissions(target_user_permissions.into())
            .system_program(system_program::ID.into())
            .address(target_user.into())
            .role(Role::FREEZE)
            .update(Update::Add)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (settings, current_settings.clone()),
        (permissions, current_permissions.clone()),
        (target_user_permissions, target_permissions_account),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let add_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&add_ix, &accounts, &[Check::success()])
    });

    target_permissions_account = get_result_account(&add_result, 3);

    // Verify role was added
    let permissions_data = UserPermissions::from_bytes(&target_permissions_account.data).unwrap();
    assert!(permissions_data.protocol_roles.roles.contains(&Role::FREEZE));

    // Remove FREEZE role
    let remove_ix = convert_instruction(
        UpdateRoleHolderBuilder::new()
            .admin(signer.into())
            .settings(settings.into())
            .admin_permissions(permissions.into())
            .update_admin_permissions(target_user_permissions.into())
            .system_program(system_program::ID.into())
            .address(target_user.into())
            .role(Role::FREEZE)
            .update(Update::Remove)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions),
        (target_user_permissions, target_permissions_account),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&remove_ix, &accounts, &[Check::success()])
    });

    // Verify role was removed
    let final_permissions = get_result_account(&result, 3);
    let permissions_data = UserPermissions::from_bytes(&final_permissions.data).unwrap();
    assert!(!permissions_data.protocol_roles.roles.contains(&Role::FREEZE));
}

#[test]
fn test_grant_multiple_roles_to_user() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .swap_fee_bps(30)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let init_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&init_ix, &accounts, &[Check::success()])
    });

    let current_settings = get_result_account(&init_result, 2);
    let current_permissions = get_result_account(&init_result, 1);

    // Create target user permission account
    let target_user = Pubkey::new_unique();
    let (target_user_permissions, _) = derive_permissions_pda(target_user);

    let create_ix = convert_instruction(
        CreatePermissionAccountBuilder::new()
            .settings(settings.into())
            .new_creds(target_user_permissions.into())
            .caller(signer.into())
            .system_program(system_program::ID.into())
            .new_admin(target_user.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (settings, current_settings.clone()),
        (target_user_permissions, empty_account()),
        (signer, signer_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let create_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&create_ix, &accounts, &[Check::success()])
    });

    let mut target_permissions_account = get_result_account(&create_result, 1);

    // Add multiple roles
    let roles = [Role::CRANK, Role::FREEZE, Role::MANAGER];

    for role in roles.iter() {
        let add_ix = convert_instruction(
            UpdateRoleHolderBuilder::new()
                .admin(signer.into())
                .settings(settings.into())
                .admin_permissions(permissions.into())
                .update_admin_permissions(target_user_permissions.into())
                .system_program(system_program::ID.into())
                .address(target_user.into())
                .role(*role)
                .update(Update::Add)
                .event_authority(event_authority.into())
                .program(RLP_ID)
                .instruction()
        );

        let mut accounts = vec![
            (signer, signer_account()),
            (settings, current_settings.clone()),
            (permissions, current_permissions.clone()),
            (target_user_permissions, target_permissions_account),
            (system_program::ID, system_program_account()),
        ];
        accounts.extend(event_cpi_accounts());

        let result = with_mollusk(|mollusk| {
            mollusk.process_and_validate_instruction(&add_ix, &accounts, &[Check::success()])
        });

        target_permissions_account = get_result_account(&result, 3);
    }

    // Verify all roles
    let permissions_data = UserPermissions::from_bytes(&target_permissions_account.data).unwrap();

    for role in roles.iter() {
        assert!(permissions_data.protocol_roles.roles.contains(role));
    }
}

// ============================================================================
// ASSET MANAGEMENT TESTS  
// Note: Asset tests require SPL token mints and Pyth oracles which are more
// complex to set up in Mollusk. These tests demonstrate the setup pattern.
// ============================================================================

#[test]
fn test_add_public_asset() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .swap_fee_bps(30)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let init_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&init_ix, &accounts, &[Check::success()])
    });

    let current_settings = get_result_account(&init_result, 2);
    let current_permissions = get_result_account(&init_result, 1);

    // Set up mock accounts for asset creation
    let mint = Pubkey::new_unique();
    let oracle = Pubkey::new_unique();
    let (asset, _) = derive_asset_pda(&mint);

    let publish_time: i64 = 0;

    let add_asset_ix = convert_instruction(
        AddAssetBuilder::new()
            .signer(signer.into())
            .admin(permissions.into())
            .settings(settings.into())
            .asset(asset.into())
            .asset_mint(mint.into())
            .oracle(oracle.into())
            .system_program(system_program::ID.into())
            .access_level(AccessLevel::Public)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, current_permissions),
        (settings, current_settings),
        (asset, empty_account()),
        (mint, create_mock_mint_account()),
        (oracle, Account {
            lamports: 1_000_000,
            data: create_mock_pyth_price_data(100_00000000, -8, publish_time),
            owner: PYTH_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        }),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&add_asset_ix, &accounts, &[Check::success()])
    });

    // Verify settings was updated
    let final_settings = get_result_account(&result, 2);
    let settings_data = Settings::from_bytes(&final_settings.data).unwrap();
    assert_eq!(settings_data.assets, 1);

    // Verify asset was created
    let asset_account = get_result_account(&result, 3);
    assert_eq!(asset_account.owner, program_id());
    assert!(!asset_account.data.is_empty());
}

#[test]
fn test_add_private_asset() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .swap_fee_bps(30)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let init_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&init_ix, &accounts, &[Check::success()])
    });

    let current_settings = get_result_account(&init_result, 2);
    let current_permissions = get_result_account(&init_result, 1);

    // Set up mock accounts
    let mint = Pubkey::new_unique();
    let oracle = Pubkey::new_unique();
    let (asset, _) = derive_asset_pda(&mint);

    let publish_time: i64 = 0;

    let add_asset_ix = convert_instruction(
        AddAssetBuilder::new()
            .signer(signer.into())
            .admin(permissions.into())
            .settings(settings.into())
            .asset(asset.into())
            .asset_mint(mint.into())
            .oracle(oracle.into())
            .system_program(system_program::ID.into())
            .access_level(AccessLevel::Private)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, current_permissions),
        (settings, current_settings),
        (asset, empty_account()),
        (mint, create_mock_mint_account()),
        (oracle, Account {
            lamports: 1_000_000,
            data: create_mock_pyth_price_data(75_00000000, -8, publish_time),
            owner: PYTH_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        }),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&add_asset_ix, &accounts, &[Check::success()])
    });

    // Verify
    let final_settings = get_result_account(&result, 2);
    let settings_data = Settings::from_bytes(&final_settings.data).unwrap();
    assert_eq!(settings_data.assets, 1);
}

#[test]
fn test_add_multiple_assets() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .swap_fee_bps(30)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let init_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&init_ix, &accounts, &[Check::success()])
    });

    let mut current_settings = get_result_account(&init_result, 2);
    let current_permissions = get_result_account(&init_result, 1);

    let publish_time: i64 = 0;

    // Add 5 assets
    for i in 0..5u8 {
        let mint = Pubkey::new_unique();
        let oracle = Pubkey::new_unique();
        let (asset, _) = derive_asset_pda(&mint);

        let add_asset_ix = convert_instruction(
            AddAssetBuilder::new()
                .signer(signer.into())
                .admin(permissions.into())
                .settings(settings.into())
                .asset(asset.into())
                .asset_mint(mint.into())
                .oracle(oracle.into())
                .system_program(system_program::ID.into())
                .access_level(if i % 2 == 0 { AccessLevel::Public } else { AccessLevel::Private })
                .event_authority(event_authority.into())
                .program(RLP_ID)
                .instruction()
        );

        let mut accounts = vec![
            (signer, signer_account()),
            (permissions, current_permissions.clone()),
            (settings, current_settings),
            (asset, empty_account()),
            (mint, create_mock_mint_account()),
            (oracle, Account {
                lamports: 1_000_000,
                data: create_mock_pyth_price_data((i as i64 + 1) * 10_00000000, -8, publish_time),
                owner: PYTH_PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
            }),
            (system_program::ID, system_program_account()),
        ];
        accounts.extend(event_cpi_accounts());

        let result = with_mollusk(|mollusk| {
            mollusk.process_and_validate_instruction(&add_asset_ix, &accounts, &[Check::success()])
        });

        current_settings = get_result_account(&result, 2);
    }

    // Verify all assets were created
    let settings_data = Settings::from_bytes(&current_settings.data).unwrap();
    assert_eq!(settings_data.assets, 5);
}

// ============================================================================
// AUDIT-FIX NEGATIVE TESTS
// ============================================================================

/// Anchor encodes custom errors as `6000 + variant_index`. Helper to translate
/// an `RlpError` discriminant position into a `ProgramError`.
fn rlp_error_code(variant_index: u32) -> solana_sdk::program_error::ProgramError {
    solana_sdk::program_error::ProgramError::Custom(6000 + variant_index)
}

/// Helper: initialize RLP and return (current_settings, current_permissions).
fn setup_initialized_rlp(signer: Pubkey) -> (Account, Account) {
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();

    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .swap_fee_bps(30)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&init_ix, &accounts, &[Check::success()])
    });

    let current_permissions = get_result_account(&result, 1);
    let current_settings = get_result_account(&result, 2);
    (current_settings, current_permissions)
}

/// M01: `update_action_role` must reject `Role::UNSET` at the instruction
/// boundary. Otherwise UNSET could be composed with a user-side write to
/// produce a silent access-control bypass.
#[test]
fn test_m01_update_action_role_rejects_unset() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();
    let (current_settings, current_permissions) = setup_initialized_rlp(signer);

    let ix = convert_instruction(
        UpdateActionRoleBuilder::new()
            .admin(signer.into())
            .settings(settings.into())
            .admin_permissions(permissions.into())
            .system_program(system_program::ID.into())
            .action(Action::Deposit)
            .role(Role::UNSET)
            .update(Update::Add)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    // RlpError::InvalidInput is variant index 1 → custom error code 6001.
    with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(
            &ix,
            &accounts,
            &[Check::err(rlp_error_code(1))],
        )
    });
}

/// M01 companion: `update_role_holder` must reject `Role::UNSET`.
#[test]
fn test_m01_update_role_holder_rejects_unset() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();
    let (current_settings, current_permissions) = setup_initialized_rlp(signer);

    // Create a target permission account first.
    let target = Pubkey::new_unique();
    let (target_perms, _) = derive_permissions_pda(target);
    let create_ix = convert_instruction(
        CreatePermissionAccountBuilder::new()
            .caller(signer.into())
            .settings(settings.into())
            .new_creds(target_perms.into())
            .system_program(system_program::ID.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .new_admin(target.into())
            .instruction()
    );
    let mut create_accounts = vec![
        (settings, current_settings.clone()),
        (target_perms, empty_account()),
        (signer, signer_account()),
        (system_program::ID, system_program_account()),
    ];
    create_accounts.extend(event_cpi_accounts());

    let create_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&create_ix, &create_accounts, &[Check::success()])
    });

    let updated_target_perms = create_result.resulting_accounts
        .iter()
        .find(|(k, _)| *k == target_perms)
        .unwrap()
        .1
        .clone();

    // Now try to assign UNSET role to the target — should reject.
    let ix = convert_instruction(
        UpdateRoleHolderBuilder::new()
            .admin(signer.into())
            .settings(settings.into())
            .admin_permissions(permissions.into())
            .update_admin_permissions(target_perms.into())
            .system_program(system_program::ID.into())
            .address(target.into())
            .role(Role::UNSET)
            .update(Update::Add)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions),
        (target_perms, updated_target_perms),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(
            &ix,
            &accounts,
            &[Check::err(rlp_error_code(1))],
        )
    });
}

/// M03: `update_action_role` must reject `Role::PUBLIC` for any action that
/// isn't in the publicly-assignable allowlist (Deposit/Withdraw/Swap).
/// Trying to assign PUBLIC to e.g. `UpdateRole` would otherwise grant
/// privileged action to all callers.
#[test]
fn test_m03_update_action_role_rejects_public_for_privileged() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();
    let (current_settings, current_permissions) = setup_initialized_rlp(signer);

    let ix = convert_instruction(
        UpdateActionRoleBuilder::new()
            .admin(signer.into())
            .settings(settings.into())
            .admin_permissions(permissions.into())
            .system_program(system_program::ID.into())
            .action(Action::UpdateRole) // privileged, not in is_publicly_assignable
            .role(Role::PUBLIC)
            .update(Update::Add)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(
            &ix,
            &accounts,
            &[Check::err(rlp_error_code(1))],
        )
    });
}

/// M03 positive: `update_action_role` must accept `Role::PUBLIC` for an
/// action that IS in the allowlist (e.g., Deposit). Symmetric to the
/// negative test above.
#[test]
fn test_m03_update_action_role_accepts_public_for_user_action() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();
    let (current_settings, current_permissions) = setup_initialized_rlp(signer);

    let ix = convert_instruction(
        UpdateActionRoleBuilder::new()
            .admin(signer.into())
            .settings(settings.into())
            .admin_permissions(permissions.into())
            .system_program(system_program::ID.into())
            .action(Action::Deposit)
            .role(Role::PUBLIC)
            .update(Update::Add)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&ix, &accounts, &[Check::success()])
    });
}

/// Creates an SPL token mint account with custom decimals. Used to test
/// audit-M04 (asset decimals > PRECISION rejection).
fn create_mock_mint_with_decimals(decimals: u8) -> Account {
    let mut data = vec![0u8; 82];
    data[0] = 1; // mint_authority option tag = Some
    data[44] = decimals;
    data[45] = 1; // is_initialized
    data[46] = 0; // freeze_authority option tag = None
    Account {
        lamports: 1_000_000,
        data,
        owner: SPL_TOKEN_ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// M04: `add_asset` must reject mints whose `decimals` exceed `PRECISION`
/// (18). Otherwise `OraclePrice::mul` would saturate the decimal adjustment
/// to zero and overvalue the asset by 10^(decimals - 18).
#[test]
fn test_m04_add_asset_rejects_high_decimals() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();
    let (current_settings, current_permissions) = setup_initialized_rlp(signer);

    let mint = Pubkey::new_unique();
    let oracle = Pubkey::new_unique();
    let (asset, _) = derive_asset_pda(&mint);
    let publish_time: i64 = 0;

    let ix = convert_instruction(
        AddAssetBuilder::new()
            .signer(signer.into())
            .admin(permissions.into())
            .settings(settings.into())
            .asset(asset.into())
            .asset_mint(mint.into())
            .oracle(oracle.into())
            .system_program(system_program::ID.into())
            .access_level(AccessLevel::Public)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, current_permissions),
        (settings, current_settings),
        (asset, empty_account()),
        (mint, create_mock_mint_with_decimals(24)), // > PRECISION = 18
        (oracle, Account {
            lamports: 1_000_000,
            data: create_mock_pyth_price_data(100_00000000, -8, publish_time),
            owner: PYTH_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        }),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    // RlpError::InvalidInput is variant index 1.
    with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(
            &ix,
            &accounts,
            &[Check::err(rlp_error_code(1))],
        )
    });
}

// ============================================================================
// HELPERS: LP MINT + TOKEN ACCOUNT + ATA + LP/POOL SETUP
// ============================================================================

/// SPL Associated Token Program ID.
const SPL_ASSOCIATED_TOKEN_ID: Pubkey = solana_sdk::pubkey!("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

/// Derive the LP PDA for a given liquidity_pool index.
fn derive_lp_pda(index: u8) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[rlp::constants::LIQUIDITY_POOL_SEED.as_bytes(), &index.to_le_bytes()],
        &Pubkey::new_from_array(RLP_ID.to_bytes()),
    )
}

/// Derive the canonical SPL ATA for (owner, mint).
fn derive_ata(owner: &Pubkey, mint: &Pubkey) -> Pubkey {
    Pubkey::find_program_address(
        &[owner.as_ref(), &SPL_TOKEN_ID.to_bytes(), &mint.to_bytes()],
        &SPL_ASSOCIATED_TOKEN_ID,
    ).0
}

/// SPL Token mint account with explicit authority + decimals + freeze_authority.
fn create_mint_account(mint_authority: Option<Pubkey>, decimals: u8, freeze_authority: Option<Pubkey>) -> Account {
    let mut data = vec![0u8; 82];
    match mint_authority {
        Some(pk) => {
            data[0] = 1; // Some
            data[4..36].copy_from_slice(&pk.to_bytes());
        }
        None => data[0] = 0, // None
    }
    // supply at 36..44 = 0
    data[44] = decimals;
    data[45] = 1; // is_initialized
    match freeze_authority {
        Some(pk) => {
            data[46] = 1; // Some
            data[50..82].copy_from_slice(&pk.to_bytes());
        }
        None => data[46] = 0, // None
    }
    Account {
        lamports: 1_000_000,
        data,
        owner: SPL_TOKEN_ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// SPL TokenAccount (165 bytes) with mint, owner, balance. State = Initialized.
fn create_token_account(mint: &Pubkey, owner: &Pubkey, amount: u64) -> Account {
    let mut data = vec![0u8; 165];
    data[0..32].copy_from_slice(&mint.to_bytes());
    data[32..64].copy_from_slice(&owner.to_bytes());
    data[64..72].copy_from_slice(&amount.to_le_bytes());
    // delegate: 72..108 = None (option tag 0 at offset 72)
    data[108] = 1; // state = Initialized
    // is_native: 109..121 = None
    // delegated_amount: 121..129 = 0
    // close_authority: 129..165 = None
    Account {
        lamports: 2_039_280,
        data,
        owner: SPL_TOKEN_ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// SPL TokenAccount with state = Frozen.
fn create_frozen_token_account(mint: &Pubkey, owner: &Pubkey, amount: u64) -> Account {
    let mut acc = create_token_account(mint, owner, amount);
    acc.data[108] = 2; // state = Frozen
    acc
}

/// SPL Token program loader account.
fn spl_token_program_account() -> Account {
    Account {
        executable: true,
        lamports: 0,
        data: vec![],
        owner: native_loader::ID,
        rent_epoch: 0,
    }
}

/// SPL Associated Token program loader account.
fn spl_ata_program_account() -> Account {
    Account {
        executable: true,
        lamports: 0,
        data: vec![],
        owner: native_loader::ID,
        rent_epoch: 0,
    }
}

/// Adds a Pyth-backed asset to the protocol. Returns (asset_pda, oracle, mint, updated_settings, updated_permissions).
fn add_test_asset(
    signer: Pubkey,
    settings: Pubkey,
    settings_acc: Account,
    permissions: Pubkey,
    permissions_acc: Account,
    decimals: u8,
    price: i64,
    access_level: AccessLevel,
) -> (Pubkey, Pubkey, Pubkey, Account, Account, Account) {
    let (event_authority, _) = derive_event_authority();
    let mint = Pubkey::new_unique();
    let oracle = Pubkey::new_unique();
    let (asset, _) = derive_asset_pda(&mint);

    let ix = convert_instruction(
        AddAssetBuilder::new()
            .signer(signer.into())
            .admin(permissions.into())
            .settings(settings.into())
            .asset(asset.into())
            .asset_mint(mint.into())
            .oracle(oracle.into())
            .system_program(system_program::ID.into())
            .access_level(access_level)
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, permissions_acc),
        (settings, settings_acc),
        (asset, empty_account()),
        (mint, create_mock_mint_with_decimals(decimals)),
        (oracle, Account {
            lamports: 1_000_000,
            data: create_mock_pyth_price_data(price, -8, 0),
            owner: PYTH_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        }),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(event_cpi_accounts());

    let result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&ix, &accounts, &[Check::success()])
    });

    let updated_permissions = result.resulting_accounts.iter().find(|(k, _)| *k == permissions).unwrap().1.clone();
    let updated_settings = result.resulting_accounts.iter().find(|(k, _)| *k == settings).unwrap().1.clone();
    let updated_asset = result.resulting_accounts.iter().find(|(k, _)| *k == asset).unwrap().1.clone();
    (asset, oracle, mint, updated_settings, updated_permissions, updated_asset)
}


// ============================================================================
// LP INITIALIZATION TESTS
// ============================================================================

/// Common SPL Token + ATA accounts for any instruction that CPIs into them.
fn spl_program_accounts() -> Vec<(Pubkey, Account)> {
    vec![
        mollusk_svm_programs_token::token::keyed_account(),
        mollusk_svm_programs_token::associated_token::keyed_account(),
    ]
}

/// initialize_lp happy path: pool with one whitelisted asset, LP mint with
/// mint_authority = LP PDA, freeze_authority = None.
#[test]
fn test_initialize_lp_happy_path() {
    use rlp_client::generated::instructions::InitializeLpBuilder;

    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();
    let (current_settings, current_permissions) = setup_initialized_rlp(signer);
    let (_asset, _oracle, _mint, current_settings, current_permissions, _asset_acc) =
        add_test_asset(signer, settings, current_settings, permissions, current_permissions, 6, 100_00000000, AccessLevel::Public);

    let (lp_pda, _) = derive_lp_pda(0);
    let lp_token_mint = Pubkey::new_unique();
    let dead_shares_vault = derive_ata(&lp_pda, &lp_token_mint);

    let ix = convert_instruction(
        InitializeLpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .liquidity_pool(lp_pda.into())
            .lp_token_mint(lp_token_mint.into())
            .dead_shares_vault(dead_shares_vault.into())
            .system_program(system_program::ID.into())
            .token_program(SPL_TOKEN_ID.into())
            .associated_token_program(SPL_ASSOCIATED_TOKEN_ID.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .cooldown_duration(60)
            .assets(vec![0])
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, current_permissions),
        (settings, current_settings),
        (lp_pda, empty_account()),
        (lp_token_mint, create_mint_account(Some(lp_pda), 9, None)),
        (dead_shares_vault, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(spl_program_accounts());
    accounts.extend(event_cpi_accounts());

    let result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&ix, &accounts, &[Check::success()])
    });

    let settings_account = result.resulting_accounts.iter().find(|(k, _)| *k == settings).unwrap().1.clone();
    let settings_data = Settings::from_bytes(&settings_account.data).unwrap();
    assert_eq!(settings_data.liquidity_pools, 1);
}

/// M04: initialize_lp must reject an LP mint whose freeze_authority is Some.
#[test]
fn test_m04_initialize_lp_rejects_lp_mint_with_freeze_authority() {
    use rlp_client::generated::instructions::InitializeLpBuilder;

    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();
    let (current_settings, current_permissions) = setup_initialized_rlp(signer);
    let (_asset, _oracle, _mint, current_settings, current_permissions, _asset_acc) =
        add_test_asset(signer, settings, current_settings, permissions, current_permissions, 6, 100_00000000, AccessLevel::Public);

    let (lp_pda, _) = derive_lp_pda(0);
    let lp_token_mint = Pubkey::new_unique();
    let dead_shares_vault = derive_ata(&lp_pda, &lp_token_mint);

    let ix = convert_instruction(
        InitializeLpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .liquidity_pool(lp_pda.into())
            .lp_token_mint(lp_token_mint.into())
            .dead_shares_vault(dead_shares_vault.into())
            .system_program(system_program::ID.into())
            .token_program(SPL_TOKEN_ID.into())
            .associated_token_program(SPL_ASSOCIATED_TOKEN_ID.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .cooldown_duration(60)
            .assets(vec![0])
            .instruction()
    );

    // LP mint with a freeze authority — should reject.
    let bogus_freeze = Pubkey::new_unique();
    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, current_permissions),
        (settings, current_settings),
        (lp_pda, empty_account()),
        (lp_token_mint, create_mint_account(Some(lp_pda), 9, Some(bogus_freeze))),
        (dead_shares_vault, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(spl_program_accounts());
    accounts.extend(event_cpi_accounts());

    // InvalidReceiptTokenFreezeAuthority is variant index 29.
    with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(
            &ix,
            &accounts,
            &[Check::err(rlp_error_code(29))],
        )
    });
}

/// E03: initialize_lp must reject cooldown_duration > MAX_COOLDOWN_DURATION.
#[test]
fn test_e03_initialize_lp_rejects_excessive_cooldown_duration() {
    use rlp_client::generated::instructions::InitializeLpBuilder;

    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();
    let (current_settings, current_permissions) = setup_initialized_rlp(signer);
    let (_asset, _oracle, _mint, current_settings, current_permissions, _asset_acc) =
        add_test_asset(signer, settings, current_settings, permissions, current_permissions, 6, 100_00000000, AccessLevel::Public);

    let (lp_pda, _) = derive_lp_pda(0);
    let lp_token_mint = Pubkey::new_unique();
    let dead_shares_vault = derive_ata(&lp_pda, &lp_token_mint);

    let ix = convert_instruction(
        InitializeLpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .liquidity_pool(lp_pda.into())
            .lp_token_mint(lp_token_mint.into())
            .dead_shares_vault(dead_shares_vault.into())
            .system_program(system_program::ID.into())
            .token_program(SPL_TOKEN_ID.into())
            .associated_token_program(SPL_ASSOCIATED_TOKEN_ID.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .cooldown_duration(366 * 24 * 60 * 60) // > 365 days
            .assets(vec![0])
            .instruction()
    );

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, current_permissions),
        (settings, current_settings),
        (lp_pda, empty_account()),
        (lp_token_mint, create_mint_account(Some(lp_pda), 9, None)),
        (dead_shares_vault, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(spl_program_accounts());
    accounts.extend(event_cpi_accounts());

    with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(
            &ix,
            &accounts,
            &[Check::err(rlp_error_code(1))],
        )
    });
}

// ============================================================================
// SETUP HELPERS — multi-step pool initialization
// ============================================================================

/// Bag of state returned by `setup_pool_with_one_asset`. Carries the current
/// account states so subsequent instruction calls can chain off them.
struct PoolFixture {
    signer: Pubkey,
    settings: Pubkey,
    permissions: Pubkey,
    asset: Pubkey,
    oracle: Pubkey,
    asset_mint: Pubkey,
    lp_pda: Pubkey,
    lp_token_mint: Pubkey,
    dead_shares_vault: Pubkey,
    pool_asset_account: Pubkey,
    settings_acc: Account,
    permissions_acc: Account,
    asset_acc: Account,
    asset_mint_acc: Account,
    lp_pda_acc: Account,
    lp_token_mint_acc: Account,
    dead_shares_vault_acc: Account,
    pool_asset_account_acc: Account,
}

/// Full setup: init RLP, add one Public asset, initialize_lp, initialize_pool_reserve.
/// Returns the fixture with the live account states.
fn setup_pool_with_one_asset(signer: Pubkey, asset_decimals: u8) -> PoolFixture {
    use rlp_client::generated::instructions::{InitializeLpBuilder, InitializePoolReserveBuilder};

    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();
    let (settings_acc, permissions_acc) = setup_initialized_rlp(signer);
    let (asset, oracle, asset_mint, settings_acc, permissions_acc, asset_acc) =
        add_test_asset(signer, settings, settings_acc, permissions, permissions_acc, asset_decimals, 100_00000000, AccessLevel::Public);

    // initialize_lp
    let (lp_pda, _) = derive_lp_pda(0);
    let lp_token_mint = Pubkey::new_unique();
    let dead_shares_vault = derive_ata(&lp_pda, &lp_token_mint);

    let init_lp_ix = convert_instruction(
        InitializeLpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .liquidity_pool(lp_pda.into())
            .lp_token_mint(lp_token_mint.into())
            .dead_shares_vault(dead_shares_vault.into())
            .system_program(system_program::ID.into())
            .token_program(SPL_TOKEN_ID.into())
            .associated_token_program(SPL_ASSOCIATED_TOKEN_ID.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .cooldown_duration(60)
            .assets(vec![0])
            .instruction()
    );

    let mut init_lp_accounts = vec![
        (signer, signer_account()),
        (permissions, permissions_acc),
        (settings, settings_acc),
        (lp_pda, empty_account()),
        (lp_token_mint, create_mint_account(Some(lp_pda), 9, None)),
        (dead_shares_vault, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    init_lp_accounts.extend(spl_program_accounts());
    init_lp_accounts.extend(event_cpi_accounts());

    let r = with_mollusk(|m| m.process_and_validate_instruction(&init_lp_ix, &init_lp_accounts, &[Check::success()]));

    let get = |key: Pubkey| r.resulting_accounts.iter().find(|(k, _)| *k == key).unwrap().1.clone();
    let settings_acc = get(settings);
    let permissions_acc = get(permissions);
    let lp_pda_acc = get(lp_pda);
    let lp_token_mint_acc = get(lp_token_mint);
    let dead_shares_vault_acc = get(dead_shares_vault);

    // initialize_pool_reserve
    let pool_asset_account = derive_ata(&lp_pda, &asset_mint);
    let init_reserve_ix = convert_instruction(
        InitializePoolReserveBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .liquidity_pool(lp_pda.into())
            .asset(asset.into())
            .asset_mint(asset_mint.into())
            .pool_asset_account(pool_asset_account.into())
            .system_program(system_program::ID.into())
            .token_program(SPL_TOKEN_ID.into())
            .associated_token_program(SPL_ASSOCIATED_TOKEN_ID.into())
            .liquidity_pool_id(0)
            .instruction()
    );

    let mut init_reserve_accounts = vec![
        (signer, signer_account()),
        (permissions, permissions_acc.clone()),
        (settings, settings_acc.clone()),
        (lp_pda, lp_pda_acc.clone()),
        (asset, asset_acc),
        (asset_mint, create_mock_mint_with_decimals(asset_decimals)),
        (pool_asset_account, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    init_reserve_accounts.extend(spl_program_accounts());

    let r2 = with_mollusk(|m| m.process_and_validate_instruction(&init_reserve_ix, &init_reserve_accounts, &[Check::success()]));
    let get2 = |key: Pubkey| r2.resulting_accounts.iter().find(|(k, _)| *k == key).unwrap().1.clone();

    let mut fixture_settings_acc = get2(settings);
    let mut fixture_permissions_acc = get2(permissions);

    // Open Deposit and Withdraw to PUBLIC so test users don't need a
    // dedicated permission account. Mirrors a typical production rollout
    // where the team adds PUBLIC after initial setup.
    for action in [Action::Deposit, Action::Withdraw, Action::Swap] {
        let upd_ix = convert_instruction(
            rlp_client::generated::instructions::UpdateActionRoleBuilder::new()
                .admin(signer.into())
                .settings(settings.into())
                .admin_permissions(permissions.into())
                .system_program(system_program::ID.into())
                .action(action)
                .role(Role::PUBLIC)
                .update(Update::Add)
                .event_authority(event_authority.into())
                .program(RLP_ID)
                .instruction()
        );

        let mut upd_accounts = vec![
            (signer, signer_account()),
            (settings, fixture_settings_acc.clone()),
            (permissions, fixture_permissions_acc.clone()),
            (system_program::ID, system_program_account()),
        ];
        upd_accounts.extend(event_cpi_accounts());

        let upd_r = with_mollusk(|m| m.process_and_validate_instruction(&upd_ix, &upd_accounts, &[Check::success()]));
        let g = |key: Pubkey| upd_r.resulting_accounts.iter().find(|(k, _)| *k == key).unwrap().1.clone();
        fixture_settings_acc = g(settings);
        fixture_permissions_acc = g(permissions);
    }

    PoolFixture {
        signer,
        settings,
        permissions,
        asset,
        oracle,
        asset_mint,
        lp_pda,
        lp_token_mint,
        dead_shares_vault,
        pool_asset_account,
        settings_acc: fixture_settings_acc,
        permissions_acc: fixture_permissions_acc,
        asset_acc: get2(asset),
        asset_mint_acc: get2(asset_mint),
        lp_pda_acc: get2(lp_pda),
        lp_token_mint_acc,
        dead_shares_vault_acc,
        pool_asset_account_acc: get2(pool_asset_account),
    }
}

/// initialize_pool_reserve happy path — exercised inside `setup_pool_with_one_asset`,
/// but assert independently here that the resulting pool reserve ATA is a valid
/// initialized TokenAccount with the right mint and authority.
#[test]
fn test_initialize_pool_reserve_creates_reserve_ata() {
    let signer = Pubkey::new_unique();
    let p = setup_pool_with_one_asset(signer, 6);
    // Reserve ATA exists with mint=asset_mint, owner=lp_pda, balance=0.
    let data = &p.pool_asset_account_acc.data;
    assert_eq!(data.len(), 165, "SPL TokenAccount layout = 165 bytes");
    let mint_bytes: [u8; 32] = data[0..32].try_into().unwrap();
    let owner_bytes: [u8; 32] = data[32..64].try_into().unwrap();
    let amount = u64::from_le_bytes(data[64..72].try_into().unwrap());
    assert_eq!(mint_bytes, p.asset_mint.to_bytes());
    assert_eq!(owner_bytes, p.lp_pda.to_bytes());
    assert_eq!(amount, 0);
}

// ============================================================================
// DEPOSIT TESTS
// ============================================================================

/// Helper to derive the user's lp ATA.
fn derive_user_lp_ata(user: &Pubkey, lp_mint: &Pubkey) -> Pubkey { derive_ata(user, lp_mint) }

/// SPL Token mint account with explicit current supply (for tests where we
/// need to seed the LP mint to a non-zero state, e.g., to mock a pool that
/// already has prior deposits).
fn create_mint_account_with_supply(mint_authority: Option<Pubkey>, decimals: u8, freeze_authority: Option<Pubkey>, supply: u64) -> Account {
    let mut acc = create_mint_account(mint_authority, decimals, freeze_authority);
    acc.data[36..44].copy_from_slice(&supply.to_le_bytes());
    acc
}

/// Issuing a deposit requires the user to already have a token account
/// holding the asset, plus the canonical LP ATA for the receipt token.
fn deposit_accounts(
    p: &PoolFixture,
    user: Pubkey,
    user_balance: u64,
) -> (Pubkey, Pubkey, Vec<(Pubkey, Account)>) {
    let user_asset_account = derive_ata(&user, &p.asset_mint);
    let user_lp_account = derive_user_lp_ata(&user, &p.lp_token_mint);

    let oracle_acc = Account {
        lamports: 1_000_000,
        data: create_mock_pyth_price_data(100_00000000, -8, 0),
        owner: PYTH_PROGRAM_ID,
        executable: false,
        rent_epoch: 0,
    };

    let mut accounts = vec![
        (user, signer_account()),
        (p.settings, p.settings_acc.clone()),
        (p.lp_pda, p.lp_pda_acc.clone()),
        (p.lp_token_mint, p.lp_token_mint_acc.clone()),
        (user_lp_account, create_token_account(&p.lp_token_mint, &user, 0)),
        (p.asset, p.asset_acc.clone()),
        (p.asset_mint, p.asset_mint_acc.clone()),
        (user_asset_account, create_token_account(&p.asset_mint, &user, user_balance)),
        (p.pool_asset_account, p.pool_asset_account_acc.clone()),
        (p.oracle, oracle_acc),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(spl_program_accounts());
    accounts.extend(event_cpi_accounts());

    (user_asset_account, user_lp_account, accounts)
}

/// `deposit` must reject amount = 0 (audit-23 / structural require).
#[test]
fn test_deposit_rejects_amount_zero() {
    use rlp_client::generated::instructions::DepositBuilder;

    let admin = Pubkey::new_unique();
    let p = setup_pool_with_one_asset(admin, 6);
    let user = Pubkey::new_unique();
    let (user_asset_account, user_lp_account, accounts) = deposit_accounts(&p, user, 1_000_000);
    let (event_authority, _) = derive_event_authority();

    let ix = convert_instruction(
        DepositBuilder::new()
            .signer(user.into())
            .settings(p.settings.into())
            .liquidity_pool(p.lp_pda.into())
            .lp_token(p.lp_token_mint.into())
            .user_lp_account(user_lp_account.into())
            .asset(p.asset.into())
            .asset_mint(p.asset_mint.into())
            .user_asset_account(user_asset_account.into())
            .pool_asset_account(p.pool_asset_account.into())
            .oracle(p.oracle.into())
            .token_program(SPL_TOKEN_ID.into())
            .associated_token_program(SPL_ASSOCIATED_TOKEN_ID.into())
            .system_program(system_program::ID.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .liquidity_pool_index(0)
            .amount(0)
            .min_lp_tokens(0)
            .instruction()
    );

    with_mollusk(|m| m.process_and_validate_instruction(
        &ix,
        &accounts,
        &[Check::err(rlp_error_code(1))], // InvalidInput
    ));
}

/// Per-asset (token_account, asset_pda, oracle, mint) remaining-accounts
/// quadruple that `calculate_total_pool_value` expects.
fn pool_value_remaining_accounts(p: &PoolFixture) -> Vec<solana_sdk::instruction::AccountMeta> {
    vec![
        AccountMeta { pubkey: p.pool_asset_account, is_signer: false, is_writable: false },
        AccountMeta { pubkey: p.asset, is_signer: false, is_writable: false },
        AccountMeta { pubkey: p.oracle, is_signer: false, is_writable: false },
        AccountMeta { pubkey: p.asset_mint, is_signer: false, is_writable: false },
    ]
}

/// M02: `deposit` must reject when any pool reserve is frozen.
#[test]
fn test_m02_deposit_rejects_when_pool_reserve_frozen() {
    use rlp_client::generated::instructions::DepositBuilder;

    let admin = Pubkey::new_unique();
    let p = setup_pool_with_one_asset(admin, 6);
    let user = Pubkey::new_unique();
    let (user_asset_account, user_lp_account, mut accounts) = deposit_accounts(&p, user, 1_000_000);

    // Replace the pool reserve account with a frozen variant.
    for (k, v) in accounts.iter_mut() {
        if *k == p.pool_asset_account {
            *v = create_frozen_token_account(&p.asset_mint, &p.lp_pda, 0);
        }
    }

    let (event_authority, _) = derive_event_authority();
    let mut ix = convert_instruction(
        DepositBuilder::new()
            .signer(user.into())
            .settings(p.settings.into())
            .liquidity_pool(p.lp_pda.into())
            .lp_token(p.lp_token_mint.into())
            .user_lp_account(user_lp_account.into())
            .asset(p.asset.into())
            .asset_mint(p.asset_mint.into())
            .user_asset_account(user_asset_account.into())
            .pool_asset_account(p.pool_asset_account.into())
            .oracle(p.oracle.into())
            .token_program(SPL_TOKEN_ID.into())
            .associated_token_program(SPL_ASSOCIATED_TOKEN_ID.into())
            .system_program(system_program::ID.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .liquidity_pool_index(0)
            .amount(1_000)
            .min_lp_tokens(0)
            .instruction()
    );

    // Append remaining-accounts (token, asset, oracle, mint) for the pool's single asset.
    ix.accounts.extend(pool_value_remaining_accounts(&p));

    // PoolAssetFrozen is variant index 48.
    with_mollusk(|m| m.process_and_validate_instruction(
        &ix,
        &accounts,
        &[Check::err(rlp_error_code(48))],
    ));
}

// ============================================================================
// REQUEST_WITHDRAWAL TESTS (audit-1, audit-39)
// ============================================================================

/// audit-1 (H02): `request_withdrawal` must reject `amount == 0` so cooldown
/// tickets can't be pre-created and filled via direct SPL transfer post-expiry.
#[test]
fn test_h02_request_withdrawal_rejects_amount_zero() {
    use rlp_client::generated::instructions::RequestWithdrawalBuilder;

    let admin = Pubkey::new_unique();
    let p = setup_pool_with_one_asset(admin, 6);
    let user = Pubkey::new_unique();
    let (event_authority, _) = derive_event_authority();

    let signer_lp_account = derive_ata(&user, &p.lp_token_mint);
    // cooldown PDA seeds: [COOLDOWN_SEED, pool_index(u8), pool_cooldowns(u64)]
    let cooldown_pda = Pubkey::find_program_address(
        &[
            rlp::constants::COOLDOWN_SEED.as_bytes(),
            &0u8.to_le_bytes(),  // pool index = 0 (only pool)
            &0u64.to_le_bytes(), // cooldowns = 0 (first cooldown)
        ],
        &Pubkey::new_from_array(RLP_ID.to_bytes()),
    ).0;
    let cooldown_lp_token_account = derive_ata(&cooldown_pda, &p.lp_token_mint);

    let ix = convert_instruction(
        RequestWithdrawalBuilder::new()
            .signer(user.into())
            .settings(p.settings.into())
            .liquidity_pool(p.lp_pda.into())
            .lp_token_mint(p.lp_token_mint.into())
            .signer_lp_token_account(signer_lp_account.into())
            .cooldown(cooldown_pda.into())
            .cooldown_lp_token_account(cooldown_lp_token_account.into())
            .token_program(SPL_TOKEN_ID.into())
            .associated_token_program(SPL_ASSOCIATED_TOKEN_ID.into())
            .system_program(system_program::ID.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .liquidity_pool_id(0)
            .amount(0) // <-- amount=0 must reject
            .instruction()
    );

    let mut accounts = vec![
        (user, signer_account()),
        (p.settings, p.settings_acc.clone()),
        (p.lp_pda, p.lp_pda_acc.clone()),
        (p.lp_token_mint, p.lp_token_mint_acc.clone()),
        (signer_lp_account, create_token_account(&p.lp_token_mint, &user, 0)),
        (cooldown_pda, empty_account()),
        (cooldown_lp_token_account, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(spl_program_accounts());
    accounts.extend(event_cpi_accounts());

    with_mollusk(|m| m.process_and_validate_instruction(
        &ix,
        &accounts,
        &[Check::err(rlp_error_code(1))], // InvalidInput
    ));
}

// ============================================================================
// FORCE_REMOVE_ASSET TESTS (audit-M02)
// ============================================================================

/// M02: `force_remove_asset` must reject if the target asset's pool reserve
/// ATA is NOT frozen (guards against using the instruction as a generic
/// asset-removal tool).
#[test]
fn test_m02_force_remove_asset_rejects_unfrozen() {
    use rlp_client::generated::instructions::ForceRemoveAssetBuilder;

    let admin = Pubkey::new_unique();
    let p = setup_pool_with_one_asset(admin, 6);
    let (event_authority, _) = derive_event_authority();

    let ix = convert_instruction(
        ForceRemoveAssetBuilder::new()
            .signer(admin.into())
            .permissions(p.permissions.into())
            .settings(p.settings.into())
            .liquidity_pool(p.lp_pda.into())
            .asset(p.asset.into())
            .asset_mint(p.asset_mint.into())
            .pool_token_account(p.pool_asset_account.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .liquidity_pool_id(0)
            .instruction()
    );

    let mut accounts = vec![
        (admin, signer_account()),
        (p.permissions, p.permissions_acc.clone()),
        (p.settings, p.settings_acc.clone()),
        (p.lp_pda, p.lp_pda_acc.clone()),
        (p.asset, p.asset_acc.clone()),
        (p.asset_mint, p.asset_mint_acc.clone()),
        (p.pool_asset_account, p.pool_asset_account_acc.clone()), // NOT frozen
    ];
    accounts.extend(event_cpi_accounts());

    // PoolAssetNotFrozen = variant 49.
    with_mollusk(|m| m.process_and_validate_instruction(
        &ix,
        &accounts,
        &[Check::err(rlp_error_code(49))],
    ));
}

// ============================================================================
// NAV SLASH TESTS (audit-2-37: NAV-based junior tranche coverage)
// ============================================================================

/// PROXY_PROGRAM_ID from rlp::constants — owner of every ProxyState account.
fn proxy_program_id() -> Pubkey {
    Pubkey::new_from_array(rlp::constants::PROXY_PROGRAM_ID.to_bytes())
}

/// Build a 124-byte ProxyState account matching `ProxyStateView`'s layout.
/// principal + integrators_commission = booked senior claims (USDC-denominated).
fn create_proxy_state_account(
    branded_mint: &Pubkey,
    stablecoin_mint: &Pubkey,
    principal: u64,
    integrators_commission: u64,
) -> Account {
    let mut data = vec![0u8; 124];
    data[0..32].copy_from_slice(&branded_mint.to_bytes());
    data[32..64].copy_from_slice(&stablecoin_mint.to_bytes());
    // 64..66: fee = 0
    data[66..74].copy_from_slice(&principal.to_le_bytes());
    data[74..82].copy_from_slice(&integrators_commission.to_le_bytes());
    // 82..114: authority = zeros
    // 114: bump
    // 115: frozen = 0
    // 116..124: deposit_cap = 0
    Account {
        lamports: 2_000_000,
        data,
        owner: proxy_program_id(),
        executable: false,
        rent_epoch: 0,
    }
}

/// audit-NAV (M06): `slash` must reject when the protected vault has no
/// mark-to-market loss (vault_value >= principal + commission → no gap).
#[test]
fn test_nav_slash_rejects_when_no_gap() {
    use rlp_client::generated::instructions::SlashBuilder;

    // Set up a pool whose protected_vault is a mock ProxyState.
    let admin = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(admin);
    let (event_authority, _) = derive_event_authority();
    let (settings_acc, permissions_acc) = setup_initialized_rlp(admin);

    // Add USDC+ as the protected stablecoin (and the pool's slashable asset).
    let (asset, oracle, stablecoin_mint, settings_acc, permissions_acc, asset_acc) =
        add_test_asset(admin, settings, settings_acc, permissions, permissions_acc, 6, 1_00000000, AccessLevel::Public);

    // Mock proxy state at a unique pubkey, with principal=1000, commission=0, vault holding $1000 worth.
    let branded_mint = Pubkey::new_unique();
    let proxy_state = Pubkey::new_unique();
    let principal: u64 = 1_000_000_000; // 1000 USDC raw (assuming 6 decimals)
    let proxy_state_acc = create_proxy_state_account(&branded_mint, &stablecoin_mint, principal, 0);
    let proxy_vault = derive_ata(&proxy_state, &stablecoin_mint);
    // Vault holds exactly enough USDC+ to cover principal at price=1.0 (oracle: 1_00000000 with exp=-8 → 1.0):
    // vault_balance * 1.0 = vault_value. principal = 1_000_000_000.
    // To have value = principal, vault_balance = principal = 1_000_000_000.
    let proxy_vault_acc = create_token_account(&stablecoin_mint, &proxy_state, principal);

    // Initialize LP with protected_vault = proxy_state.
    use rlp_client::generated::instructions::{InitializeLpBuilder, InitializePoolReserveBuilder};
    let (lp_pda, _) = derive_lp_pda(0);
    let lp_token_mint = Pubkey::new_unique();
    let dead_shares_vault = derive_ata(&lp_pda, &lp_token_mint);

    let init_lp_ix = convert_instruction(
        InitializeLpBuilder::new()
            .signer(admin.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .liquidity_pool(lp_pda.into())
            .lp_token_mint(lp_token_mint.into())
            .dead_shares_vault(dead_shares_vault.into())
            .system_program(system_program::ID.into())
            .token_program(SPL_TOKEN_ID.into())
            .associated_token_program(SPL_ASSOCIATED_TOKEN_ID.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .cooldown_duration(60)
            .assets(vec![0])
            .protected_vault(proxy_state.into())
            .instruction()
    );

    let mut init_lp_accounts = vec![
        (admin, signer_account()),
        (permissions, permissions_acc),
        (settings, settings_acc),
        (lp_pda, empty_account()),
        (lp_token_mint, create_mint_account(Some(lp_pda), 9, None)),
        (dead_shares_vault, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    init_lp_accounts.extend(spl_program_accounts());
    init_lp_accounts.extend(event_cpi_accounts());

    let r = with_mollusk(|m| m.process_and_validate_instruction(&init_lp_ix, &init_lp_accounts, &[Check::success()]));
    let g = |k: Pubkey| r.resulting_accounts.iter().find(|(kk, _)| *kk == k).unwrap().1.clone();
    let settings_acc = g(settings);
    let permissions_acc = g(permissions);
    let lp_pda_acc = g(lp_pda);

    // Init the pool reserve for USDC+.
    let pool_asset_account = derive_ata(&lp_pda, &stablecoin_mint);
    let init_reserve_ix = convert_instruction(
        InitializePoolReserveBuilder::new()
            .signer(admin.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .liquidity_pool(lp_pda.into())
            .asset(asset.into())
            .asset_mint(stablecoin_mint.into())
            .pool_asset_account(pool_asset_account.into())
            .system_program(system_program::ID.into())
            .token_program(SPL_TOKEN_ID.into())
            .associated_token_program(SPL_ASSOCIATED_TOKEN_ID.into())
            .liquidity_pool_id(0)
            .instruction()
    );
    let mut init_reserve_accounts = vec![
        (admin, signer_account()),
        (permissions, permissions_acc.clone()),
        (settings, settings_acc.clone()),
        (lp_pda, lp_pda_acc.clone()),
        (asset, asset_acc.clone()),
        (stablecoin_mint, create_mock_mint_with_decimals(6)),
        (pool_asset_account, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    init_reserve_accounts.extend(spl_program_accounts());
    let r2 = with_mollusk(|m| m.process_and_validate_instruction(&init_reserve_ix, &init_reserve_accounts, &[Check::success()]));
    let g2 = |k: Pubkey| r2.resulting_accounts.iter().find(|(kk, _)| *kk == k).unwrap().1.clone();
    let permissions_acc = g2(permissions);
    let settings_acc = g2(settings);
    let lp_pda_acc = g2(lp_pda);
    let asset_acc = g2(asset);
    let pool_reserve_acc = g2(pool_asset_account);

    // Fund the pool reserve so a slash would have something to transfer.
    let mut pool_reserve_funded = pool_reserve_acc.clone();
    pool_reserve_funded.data[64..72].copy_from_slice(&500_000_000u64.to_le_bytes());

    // Call slash. Vault_value = vault_balance × 1.0 = principal. Gap = 0 → expect NoNavLossToCover.
    let ix = convert_instruction(
        SlashBuilder::new()
            .signer(admin.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .liquidity_pool(lp_pda.into())
            .asset(asset.into())
            .stablecoin_mint(stablecoin_mint.into())
            .liquidity_pool_token_account(pool_asset_account.into())
            .proxy_state(proxy_state.into())
            .protected_vault_token_account(proxy_vault.into())
            .oracle(oracle.into())
            .token_program(SPL_TOKEN_ID.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .liquidity_pool_id(0)
            .amount(100_000_000)
            .instruction()
    );

    let mut accounts = vec![
        (admin, signer_account()),
        (permissions, permissions_acc),
        (settings, settings_acc),
        (lp_pda, lp_pda_acc),
        (asset, asset_acc),
        (stablecoin_mint, create_mock_mint_with_decimals(6)),
        (pool_asset_account, pool_reserve_funded),
        (proxy_state, proxy_state_acc),
        (proxy_vault, proxy_vault_acc),
        (oracle, Account {
            lamports: 1_000_000,
            data: create_mock_pyth_price_data(1_00000000, -8, 0),
            owner: PYTH_PROGRAM_ID,
            executable: false,
            rent_epoch: 0,
        }),
    ];
    accounts.extend(spl_program_accounts());
    accounts.extend(event_cpi_accounts());

    // NoNavLossToCover = variant 53.
    with_mollusk(|m| m.process_and_validate_instruction(
        &ix,
        &accounts,
        &[Check::err(rlp_error_code(53))],
    ));
}

/// audit-39 (H01): With `init_if_needed` on `cooldown_lp_token_account`, a
/// pre-existing ATA at the canonical (cooldown_pda, lp_mint) address must
/// be tolerated, not revert the whole instruction. This blocks the DoS-squat
/// attack the auditor demonstrated.
#[test]
fn test_h01_request_withdrawal_tolerates_cooldown_ata_squat() {
    use rlp_client::generated::instructions::RequestWithdrawalBuilder;

    let admin = Pubkey::new_unique();
    let p = setup_pool_with_one_asset(admin, 6);

    // User has some LP tokens — we need to seed an LP token account and the
    // mint must have non-zero supply to be realistic. Simulate by overriding
    // the lp_token_mint account and the user's LP balance.
    let user = Pubkey::new_unique();
    let (event_authority, _) = derive_event_authority();

    let signer_lp_account = derive_ata(&user, &p.lp_token_mint);
    let cooldown_pda = Pubkey::find_program_address(
        &[
            rlp::constants::COOLDOWN_SEED.as_bytes(),
            &0u8.to_le_bytes(),
            &0u64.to_le_bytes(),
        ],
        &Pubkey::new_from_array(RLP_ID.to_bytes()),
    ).0;
    let cooldown_lp_token_account = derive_ata(&cooldown_pda, &p.lp_token_mint);

    // Mint with non-zero supply so the LP token math is well-defined.
    let mut lp_mint_acc_funded = p.lp_token_mint_acc.clone();
    lp_mint_acc_funded.data[36..44].copy_from_slice(&1_000_000_000u64.to_le_bytes());

    // The squat — a pre-existing TokenAccount at the canonical address with
    // matching mint and authority. With `init` this would revert; with
    // `init_if_needed` (audit-39 fix), Anchor tolerates and reuses it.
    let squatted_cooldown_ata = create_token_account(&p.lp_token_mint, &cooldown_pda, 0);

    let ix = convert_instruction(
        RequestWithdrawalBuilder::new()
            .signer(user.into())
            .settings(p.settings.into())
            .liquidity_pool(p.lp_pda.into())
            .lp_token_mint(p.lp_token_mint.into())
            .signer_lp_token_account(signer_lp_account.into())
            .cooldown(cooldown_pda.into())
            .cooldown_lp_token_account(cooldown_lp_token_account.into())
            .token_program(SPL_TOKEN_ID.into())
            .associated_token_program(SPL_ASSOCIATED_TOKEN_ID.into())
            .system_program(system_program::ID.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .liquidity_pool_id(0)
            .amount(100_000_000)
            .instruction()
    );

    let mut accounts = vec![
        (user, signer_account()),
        (p.settings, p.settings_acc.clone()),
        (p.lp_pda, p.lp_pda_acc.clone()),
        (p.lp_token_mint, lp_mint_acc_funded),
        (signer_lp_account, create_token_account(&p.lp_token_mint, &user, 500_000_000)),
        (cooldown_pda, empty_account()),
        (cooldown_lp_token_account, squatted_cooldown_ata),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(spl_program_accounts());
    accounts.extend(event_cpi_accounts());

    with_mollusk(|m| m.process_and_validate_instruction(&ix, &accounts, &[Check::success()]));
}

/// audit-40 (L01): With `init_if_needed` on `dead_shares_vault`, a pre-existing
/// ATA at the canonical (lp_pda, lp_mint) address must be tolerated by
/// initialize_lp instead of reverting with IllegalOwner.
#[test]
fn test_l01_initialize_lp_tolerates_dead_shares_vault_squat() {
    use rlp_client::generated::instructions::InitializeLpBuilder;

    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();
    let (current_settings, current_permissions) = setup_initialized_rlp(signer);
    let (_a, _o, _m, current_settings, current_permissions, _ac) =
        add_test_asset(signer, settings, current_settings, permissions, current_permissions, 6, 100_00000000, AccessLevel::Public);

    let (lp_pda, _) = derive_lp_pda(0);
    let lp_token_mint = Pubkey::new_unique();
    let dead_shares_vault = derive_ata(&lp_pda, &lp_token_mint);

    let ix = convert_instruction(
        InitializeLpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .liquidity_pool(lp_pda.into())
            .lp_token_mint(lp_token_mint.into())
            .dead_shares_vault(dead_shares_vault.into())
            .system_program(system_program::ID.into())
            .token_program(SPL_TOKEN_ID.into())
            .associated_token_program(SPL_ASSOCIATED_TOKEN_ID.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .cooldown_duration(60)
            .assets(vec![0])
            .instruction()
    );

    // The squat — pre-existing TokenAccount at the canonical address with
    // matching mint and authority. `init` would have reverted; `init_if_needed`
    // tolerates.
    let squatted = create_token_account(&lp_token_mint, &lp_pda, 0);

    let mut accounts = vec![
        (signer, signer_account()),
        (permissions, current_permissions),
        (settings, current_settings),
        (lp_pda, empty_account()),
        (lp_token_mint, create_mint_account(Some(lp_pda), 9, None)),
        (dead_shares_vault, squatted),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(spl_program_accounts());
    accounts.extend(event_cpi_accounts());

    with_mollusk(|m| m.process_and_validate_instruction(&ix, &accounts, &[Check::success()]));
}

// ============================================================================
// DEPOSIT HAPPY PATH
// ============================================================================

/// Full deposit flow: user has `user_balance` of the asset, deposits `amount`,
/// receives newly-minted LP tokens. Returns updated accounts so a subsequent
/// withdraw test can chain off this state.
fn do_deposit(p: &PoolFixture, user: Pubkey, user_balance: u64, amount: u64) -> mollusk_svm::result::InstructionResult {
    use rlp_client::generated::instructions::DepositBuilder;

    let user_asset_account = derive_ata(&user, &p.asset_mint);
    let user_lp_account = derive_user_lp_ata(&user, &p.lp_token_mint);
    let (event_authority, _) = derive_event_authority();

    let oracle_acc = Account {
        lamports: 1_000_000,
        data: create_mock_pyth_price_data(1_00000000, -8, 0),
        owner: PYTH_PROGRAM_ID,
        executable: false,
        rent_epoch: 0,
    };

    let mut ix = convert_instruction(
        DepositBuilder::new()
            .signer(user.into())
            .settings(p.settings.into())
            .liquidity_pool(p.lp_pda.into())
            .lp_token(p.lp_token_mint.into())
            .user_lp_account(user_lp_account.into())
            .asset(p.asset.into())
            .asset_mint(p.asset_mint.into())
            .user_asset_account(user_asset_account.into())
            .pool_asset_account(p.pool_asset_account.into())
            .oracle(p.oracle.into())
            .token_program(SPL_TOKEN_ID.into())
            .associated_token_program(SPL_ASSOCIATED_TOKEN_ID.into())
            .system_program(system_program::ID.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .liquidity_pool_index(0)
            .amount(amount)
            .min_lp_tokens(0)
            .instruction()
    );
    ix.accounts.extend(pool_value_remaining_accounts(p));

    let mut accounts = vec![
        (user, signer_account()),
        (p.settings, p.settings_acc.clone()),
        (p.lp_pda, p.lp_pda_acc.clone()),
        (p.lp_token_mint, p.lp_token_mint_acc.clone()),
        (user_lp_account, create_token_account(&p.lp_token_mint, &user, 0)),
        (p.asset, p.asset_acc.clone()),
        (p.asset_mint, p.asset_mint_acc.clone()),
        (user_asset_account, create_token_account(&p.asset_mint, &user, user_balance)),
        (p.pool_asset_account, p.pool_asset_account_acc.clone()),
        (p.oracle, oracle_acc),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(spl_program_accounts());
    accounts.extend(event_cpi_accounts());

    with_mollusk(|m| m.process_and_validate_instruction(&ix, &accounts, &[Check::success()]))
}

/// Happy-path deposit: user deposits asset, receives LP tokens, pool reserve
/// holds the deposited amount.
#[test]
fn test_deposit_happy_path() {
    let admin = Pubkey::new_unique();
    let p = setup_pool_with_one_asset(admin, 6);
    let user = Pubkey::new_unique();

    let user_balance: u64 = 1_000_000_000;
    let deposit_amount: u64 = 500_000_000;

    let result = do_deposit(&p, user, user_balance, deposit_amount);

    // Pool reserve now holds deposit_amount.
    let pool_reserve_after = result.resulting_accounts.iter()
        .find(|(k, _)| *k == p.pool_asset_account).unwrap().1.clone();
    let reserve_amount = u64::from_le_bytes(pool_reserve_after.data[64..72].try_into().unwrap());
    assert_eq!(reserve_amount, deposit_amount, "pool reserve received deposit");

    // User now holds LP tokens (>0).
    let user_lp_account = derive_user_lp_ata(&user, &p.lp_token_mint);
    let user_lp_after = result.resulting_accounts.iter()
        .find(|(k, _)| *k == user_lp_account).unwrap().1.clone();
    let user_lp_balance = u64::from_le_bytes(user_lp_after.data[64..72].try_into().unwrap());
    assert!(user_lp_balance > 0, "user received LP tokens");
}

// ============================================================================
// WITHDRAW HAPPY PATH + audit-1 EXCESS REFUND
// ============================================================================

/// Helper: run request_withdrawal for `user` with `amount`. Returns updated
/// account states keyed by pubkey for the caller to thread into `withdraw`.
fn do_request_withdrawal(
    p: &PoolFixture,
    user: Pubkey,
    user_lp_balance: u64,
    amount: u64,
    deposit_result_accounts: &[(Pubkey, Account)],
) -> mollusk_svm::result::InstructionResult {
    use rlp_client::generated::instructions::RequestWithdrawalBuilder;

    let signer_lp_account = derive_ata(&user, &p.lp_token_mint);
    let cooldown_pda = Pubkey::find_program_address(
        &[
            rlp::constants::COOLDOWN_SEED.as_bytes(),
            &0u8.to_le_bytes(),
            &0u64.to_le_bytes(),
        ],
        &Pubkey::new_from_array(RLP_ID.to_bytes()),
    ).0;
    let cooldown_lp_token_account = derive_ata(&cooldown_pda, &p.lp_token_mint);
    let (event_authority, _) = derive_event_authority();

    let ix = convert_instruction(
        RequestWithdrawalBuilder::new()
            .signer(user.into())
            .settings(p.settings.into())
            .liquidity_pool(p.lp_pda.into())
            .lp_token_mint(p.lp_token_mint.into())
            .signer_lp_token_account(signer_lp_account.into())
            .cooldown(cooldown_pda.into())
            .cooldown_lp_token_account(cooldown_lp_token_account.into())
            .token_program(SPL_TOKEN_ID.into())
            .associated_token_program(SPL_ASSOCIATED_TOKEN_ID.into())
            .system_program(system_program::ID.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .liquidity_pool_id(0)
            .amount(amount)
            .instruction()
    );

    // Build accounts from the deposit result, falling back to fresh for new addresses.
    let get_or = |k: &Pubkey, default: Account| -> Account {
        deposit_result_accounts.iter().find(|(kk, _)| kk == k).map(|(_, a)| a.clone()).unwrap_or(default)
    };

    let mut accounts = vec![
        (user, signer_account()),
        (p.settings, get_or(&p.settings, p.settings_acc.clone())),
        (p.lp_pda, get_or(&p.lp_pda, p.lp_pda_acc.clone())),
        (p.lp_token_mint, get_or(&p.lp_token_mint, p.lp_token_mint_acc.clone())),
        (signer_lp_account, get_or(&signer_lp_account, create_token_account(&p.lp_token_mint, &user, user_lp_balance))),
        (cooldown_pda, empty_account()),
        (cooldown_lp_token_account, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(spl_program_accounts());
    accounts.extend(event_cpi_accounts());

    with_mollusk(|m| m.process_and_validate_instruction(&ix, &accounts, &[Check::success()]))
}

/// Happy-path full withdraw flow: deposit → request_withdrawal → advance
/// clock → withdraw → user receives back the deposited asset.
#[test]
fn test_withdraw_happy_path() {
    use rlp_client::generated::instructions::WithdrawBuilder;

    let admin = Pubkey::new_unique();
    let p = setup_pool_with_one_asset(admin, 6);
    let user = Pubkey::new_unique();

    let user_balance: u64 = 1_000_000_000;
    let deposit_amount: u64 = 500_000_000;
    let deposit_result = do_deposit(&p, user, user_balance, deposit_amount);

    // Find user's LP balance after deposit.
    let user_lp_account = derive_user_lp_ata(&user, &p.lp_token_mint);
    let user_lp_after_deposit = deposit_result.resulting_accounts.iter()
        .find(|(k, _)| *k == user_lp_account).unwrap().1.clone();
    let user_lp_balance = u64::from_le_bytes(user_lp_after_deposit.data[64..72].try_into().unwrap());
    assert!(user_lp_balance > 0);

    // Request withdrawal of all LP tokens.
    let req_result = do_request_withdrawal(
        &p,
        user,
        user_lp_balance,
        user_lp_balance,
        &deposit_result.resulting_accounts,
    );

    // Cooldown PDA was created.
    let cooldown_pda = Pubkey::find_program_address(
        &[
            rlp::constants::COOLDOWN_SEED.as_bytes(),
            &0u8.to_le_bytes(),
            &0u64.to_le_bytes(),
        ],
        &Pubkey::new_from_array(RLP_ID.to_bytes()),
    ).0;
    let cooldown_lp_ata = derive_ata(&cooldown_pda, &p.lp_token_mint);
    let user_asset_ata = derive_ata(&user, &p.asset_mint);

    // Advance clock past cooldown duration (60s set at initialize_lp).
    let unlock_ts = 1000;
    let (event_authority, _) = derive_event_authority();

    let ix = convert_instruction(
        WithdrawBuilder::new()
            .signer(user.into())
            .settings(p.settings.into())
            .liquidity_pool(p.lp_pda.into())
            .lp_token_mint(p.lp_token_mint.into())
            .cooldown_lp_token_account(cooldown_lp_ata.into())
            .signer_lp_token_account(user_lp_account.into())
            .cooldown(cooldown_pda.into())
            .token_program(SPL_TOKEN_ID.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .liquidity_pool_id(0)
            .cooldown_id(0)
            .instruction()
    );

    // Withdraw remaining_accounts: 3 per asset (asset, reserve, user_token).
    let mut ix = ix;
    ix.accounts.push(AccountMeta { pubkey: p.asset, is_signer: false, is_writable: false });
    ix.accounts.push(AccountMeta { pubkey: p.pool_asset_account, is_signer: false, is_writable: true });
    ix.accounts.push(AccountMeta { pubkey: user_asset_ata, is_signer: false, is_writable: true });

    let get = |k: &Pubkey, default: Account| -> Account {
        req_result.resulting_accounts.iter().find(|(kk, _)| kk == k).map(|(_, a)| a.clone())
            .or_else(|| deposit_result.resulting_accounts.iter().find(|(kk, _)| kk == k).map(|(_, a)| a.clone()))
            .unwrap_or(default)
    };

    let mut accounts = vec![
        (user, signer_account()),
        (p.settings, get(&p.settings, p.settings_acc.clone())),
        (p.lp_pda, get(&p.lp_pda, p.lp_pda_acc.clone())),
        (p.lp_token_mint, get(&p.lp_token_mint, p.lp_token_mint_acc.clone())),
        (cooldown_lp_ata, get(&cooldown_lp_ata, empty_account())),
        (user_lp_account, get(&user_lp_account, create_token_account(&p.lp_token_mint, &user, 0))),
        (cooldown_pda, get(&cooldown_pda, empty_account())),
        (p.asset, get(&p.asset, p.asset_acc.clone())),
        (p.pool_asset_account, get(&p.pool_asset_account, p.pool_asset_account_acc.clone())),
        (user_asset_ata, get(&user_asset_ata, create_token_account(&p.asset_mint, &user, 0))),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(spl_program_accounts());
    accounts.extend(event_cpi_accounts());

    let r = with_mollusk_clock(unlock_ts, |m| {
        m.process_and_validate_instruction(&ix, &accounts, &[Check::success()])
    });

    // User received the deposited asset back.
    let user_asset_after = r.resulting_accounts.iter().find(|(k, _)| *k == user_asset_ata).unwrap().1.clone();
    let user_asset_balance = u64::from_le_bytes(user_asset_after.data[64..72].try_into().unwrap());
    assert!(user_asset_balance > 0, "user received asset back from withdraw");
}

/// audit-1 (H02): if extra LP tokens are direct-transferred into the cooldown
/// ATA between request_withdrawal and withdraw, withdraw must burn only the
/// recorded `locked_amount` and refund the excess to the signer's LP token
/// account before closing. This blocks the cooldown bypass attack.
#[test]
fn test_h02_withdraw_refunds_excess_in_cooldown_ata() {
    use rlp_client::generated::instructions::WithdrawBuilder;

    let admin = Pubkey::new_unique();
    let p = setup_pool_with_one_asset(admin, 6);
    let user = Pubkey::new_unique();

    // Deposit so the user has LP tokens.
    let user_balance: u64 = 1_000_000_000;
    let deposit_amount: u64 = 500_000_000;
    let deposit_result = do_deposit(&p, user, user_balance, deposit_amount);

    let user_lp_account = derive_user_lp_ata(&user, &p.lp_token_mint);
    let user_lp_after_deposit = deposit_result.resulting_accounts.iter()
        .find(|(k, _)| *k == user_lp_account).unwrap().1.clone();
    let user_lp_balance = u64::from_le_bytes(user_lp_after_deposit.data[64..72].try_into().unwrap());

    // Request withdrawal of HALF the LP tokens. locked_amount = half.
    let request_amount = user_lp_balance / 2;
    let req_result = do_request_withdrawal(&p, user, user_lp_balance, request_amount, &deposit_result.resulting_accounts);

    let cooldown_pda = Pubkey::find_program_address(
        &[
            rlp::constants::COOLDOWN_SEED.as_bytes(),
            &0u8.to_le_bytes(),
            &0u64.to_le_bytes(),
        ],
        &Pubkey::new_from_array(RLP_ID.to_bytes()),
    ).0;
    let cooldown_lp_ata = derive_ata(&cooldown_pda, &p.lp_token_mint);
    let user_asset_ata = derive_ata(&user, &p.asset_mint);

    // ATTACK: Inject extra LP tokens into the cooldown ATA. With the old
    // (pre-fix) withdraw logic this would let the attacker withdraw against
    // the inflated balance. With audit-1 fix, only `locked_amount` is burned
    // and the rest is refunded.
    let mut cooldown_lp_after = req_result.resulting_accounts.iter()
        .find(|(k, _)| *k == cooldown_lp_ata).unwrap().1.clone();
    let injected: u64 = 200_000_000;
    let total_in_cooldown = request_amount + injected;
    cooldown_lp_after.data[64..72].copy_from_slice(&total_in_cooldown.to_le_bytes());

    // The user's lp account currently holds (user_lp_balance - request_amount) after request.
    // The injection has to come from somewhere — for the test it's fine to leave the
    // user's lp account untouched; the program only cares about cooldown_lp_token_account
    // and signer_lp_token_account balances.

    let (event_authority, _) = derive_event_authority();
    let ix = convert_instruction(
        WithdrawBuilder::new()
            .signer(user.into())
            .settings(p.settings.into())
            .liquidity_pool(p.lp_pda.into())
            .lp_token_mint(p.lp_token_mint.into())
            .cooldown_lp_token_account(cooldown_lp_ata.into())
            .signer_lp_token_account(user_lp_account.into())
            .cooldown(cooldown_pda.into())
            .token_program(SPL_TOKEN_ID.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .liquidity_pool_id(0)
            .cooldown_id(0)
            .instruction()
    );

    let mut ix = ix;
    ix.accounts.push(AccountMeta { pubkey: p.asset, is_signer: false, is_writable: false });
    ix.accounts.push(AccountMeta { pubkey: p.pool_asset_account, is_signer: false, is_writable: true });
    ix.accounts.push(AccountMeta { pubkey: user_asset_ata, is_signer: false, is_writable: true });

    let get = |k: &Pubkey, default: Account| -> Account {
        req_result.resulting_accounts.iter().find(|(kk, _)| kk == k).map(|(_, a)| a.clone())
            .or_else(|| deposit_result.resulting_accounts.iter().find(|(kk, _)| kk == k).map(|(_, a)| a.clone()))
            .unwrap_or(default)
    };

    let user_lp_pre_withdraw_balance = u64::from_le_bytes(
        get(&user_lp_account, create_token_account(&p.lp_token_mint, &user, 0)).data[64..72].try_into().unwrap()
    );

    let mut accounts = vec![
        (user, signer_account()),
        (p.settings, get(&p.settings, p.settings_acc.clone())),
        (p.lp_pda, get(&p.lp_pda, p.lp_pda_acc.clone())),
        (p.lp_token_mint, get(&p.lp_token_mint, p.lp_token_mint_acc.clone())),
        (cooldown_lp_ata, cooldown_lp_after), // injected balance
        (user_lp_account, get(&user_lp_account, create_token_account(&p.lp_token_mint, &user, 0))),
        (cooldown_pda, get(&cooldown_pda, empty_account())),
        (p.asset, get(&p.asset, p.asset_acc.clone())),
        (p.pool_asset_account, get(&p.pool_asset_account, p.pool_asset_account_acc.clone())),
        (user_asset_ata, get(&user_asset_ata, create_token_account(&p.asset_mint, &user, 0))),
        (system_program::ID, system_program_account()),
    ];
    accounts.extend(spl_program_accounts());
    accounts.extend(event_cpi_accounts());

    let r = with_mollusk_clock(1000, |m| {
        m.process_and_validate_instruction(&ix, &accounts, &[Check::success()])
    });

    // user_lp_account balance went UP by `injected` (refund) and otherwise stays put.
    // Cooldown ATA was closed (zero balance).
    let user_lp_after = r.resulting_accounts.iter().find(|(k, _)| *k == user_lp_account).unwrap().1.clone();
    let user_lp_post_balance = u64::from_le_bytes(user_lp_after.data[64..72].try_into().unwrap());
    assert_eq!(
        user_lp_post_balance,
        user_lp_pre_withdraw_balance + injected,
        "excess in cooldown ATA refunded to signer's LP account"
    );
}

// ============================================================================
// SWAP TESTS
// ============================================================================

/// Set up a pool with TWO whitelisted assets and both reserves initialized.
/// Returns (PoolFixture for asset 0, second_asset_pda, second_oracle, second_mint,
/// second_asset_acc, second_pool_reserve, second_pool_reserve_acc, second_mint_acc).
fn setup_pool_with_two_assets(signer: Pubkey) -> (PoolFixture, Pubkey, Pubkey, Pubkey, Account, Pubkey, Account, Account) {
    use rlp_client::generated::instructions::{InitializeLpBuilder, InitializePoolReserveBuilder};

    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);
    let (event_authority, _) = derive_event_authority();
    let (settings_acc, permissions_acc) = setup_initialized_rlp(signer);

    let (asset0, oracle0, mint0, settings_acc, permissions_acc, asset0_acc) =
        add_test_asset(signer, settings, settings_acc, permissions, permissions_acc, 6, 1_00000000, AccessLevel::Public);
    let (asset1, oracle1, mint1, settings_acc, permissions_acc, asset1_acc) =
        add_test_asset(signer, settings, settings_acc, permissions, permissions_acc, 6, 1_00000000, AccessLevel::Public);

    let (lp_pda, _) = derive_lp_pda(0);
    let lp_token_mint = Pubkey::new_unique();
    let dead_shares_vault = derive_ata(&lp_pda, &lp_token_mint);

    let init_lp_ix = convert_instruction(
        InitializeLpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .liquidity_pool(lp_pda.into())
            .lp_token_mint(lp_token_mint.into())
            .dead_shares_vault(dead_shares_vault.into())
            .system_program(system_program::ID.into())
            .token_program(SPL_TOKEN_ID.into())
            .associated_token_program(SPL_ASSOCIATED_TOKEN_ID.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .cooldown_duration(60)
            .assets(vec![0, 1])
            .instruction()
    );
    let mut init_lp_accounts = vec![
        (signer, signer_account()),
        (permissions, permissions_acc),
        (settings, settings_acc),
        (lp_pda, empty_account()),
        (lp_token_mint, create_mint_account(Some(lp_pda), 9, None)),
        (dead_shares_vault, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    init_lp_accounts.extend(spl_program_accounts());
    init_lp_accounts.extend(event_cpi_accounts());
    let r = with_mollusk(|m| m.process_and_validate_instruction(&init_lp_ix, &init_lp_accounts, &[Check::success()]));
    let g = |k: Pubkey| r.resulting_accounts.iter().find(|(kk, _)| *kk == k).unwrap().1.clone();
    let settings_acc = g(settings);
    let permissions_acc = g(permissions);
    let lp_pda_acc = g(lp_pda);
    let lp_token_mint_acc = g(lp_token_mint);
    let dead_shares_vault_acc = g(dead_shares_vault);

    // init reserve for asset0
    let pool_reserve_0 = derive_ata(&lp_pda, &mint0);
    let ix0 = convert_instruction(
        InitializePoolReserveBuilder::new()
            .signer(signer.into()).permissions(permissions.into()).settings(settings.into())
            .liquidity_pool(lp_pda.into()).asset(asset0.into()).asset_mint(mint0.into())
            .pool_asset_account(pool_reserve_0.into())
            .system_program(system_program::ID.into()).token_program(SPL_TOKEN_ID.into())
            .associated_token_program(SPL_ASSOCIATED_TOKEN_ID.into())
            .liquidity_pool_id(0).instruction()
    );
    let mut a0 = vec![
        (signer, signer_account()),
        (permissions, permissions_acc.clone()),
        (settings, settings_acc.clone()),
        (lp_pda, lp_pda_acc.clone()),
        (asset0, asset0_acc.clone()),
        (mint0, create_mock_mint_with_decimals(6)),
        (pool_reserve_0, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    a0.extend(spl_program_accounts());
    let r0 = with_mollusk(|m| m.process_and_validate_instruction(&ix0, &a0, &[Check::success()]));
    let g0 = |k: Pubkey| r0.resulting_accounts.iter().find(|(kk, _)| *kk == k).unwrap().1.clone();
    let pool_reserve_0_acc = g0(pool_reserve_0);
    let mint0_acc = g0(mint0);

    // init reserve for asset1
    let pool_reserve_1 = derive_ata(&lp_pda, &mint1);
    let ix1 = convert_instruction(
        InitializePoolReserveBuilder::new()
            .signer(signer.into()).permissions(permissions.into()).settings(settings.into())
            .liquidity_pool(lp_pda.into()).asset(asset1.into()).asset_mint(mint1.into())
            .pool_asset_account(pool_reserve_1.into())
            .system_program(system_program::ID.into()).token_program(SPL_TOKEN_ID.into())
            .associated_token_program(SPL_ASSOCIATED_TOKEN_ID.into())
            .liquidity_pool_id(0).instruction()
    );
    let mut a1 = vec![
        (signer, signer_account()),
        (permissions, permissions_acc.clone()),
        (settings, g0(settings)),
        (lp_pda, g0(lp_pda)),
        (asset1, asset1_acc.clone()),
        (mint1, create_mock_mint_with_decimals(6)),
        (pool_reserve_1, empty_account()),
        (system_program::ID, system_program_account()),
    ];
    a1.extend(spl_program_accounts());
    let r1 = with_mollusk(|m| m.process_and_validate_instruction(&ix1, &a1, &[Check::success()]));
    let g1 = |k: Pubkey| r1.resulting_accounts.iter().find(|(kk, _)| *kk == k).unwrap().1.clone();

    // Open swap to PUBLIC
    let mut settings_acc = g1(settings);
    let mut permissions_acc = g1(permissions);
    for action in [Action::Swap] {
        let upd_ix = convert_instruction(
            rlp_client::generated::instructions::UpdateActionRoleBuilder::new()
                .admin(signer.into()).settings(settings.into()).admin_permissions(permissions.into())
                .system_program(system_program::ID.into())
                .action(action).role(Role::PUBLIC).update(Update::Add)
                .event_authority(event_authority.into()).program(RLP_ID).instruction()
        );
        let mut a = vec![
            (signer, signer_account()),
            (settings, settings_acc.clone()),
            (permissions, permissions_acc.clone()),
            (system_program::ID, system_program_account()),
        ];
        a.extend(event_cpi_accounts());
        let rr = with_mollusk(|m| m.process_and_validate_instruction(&upd_ix, &a, &[Check::success()]));
        settings_acc = rr.resulting_accounts.iter().find(|(k, _)| *k == settings).unwrap().1.clone();
        permissions_acc = rr.resulting_accounts.iter().find(|(k, _)| *k == permissions).unwrap().1.clone();
    }

    let p = PoolFixture {
        signer, settings, permissions,
        asset: asset0, oracle: oracle0, asset_mint: mint0,
        lp_pda, lp_token_mint, dead_shares_vault, pool_asset_account: pool_reserve_0,
        settings_acc, permissions_acc,
        asset_acc: asset0_acc, asset_mint_acc: mint0_acc,
        lp_pda_acc: g1(lp_pda), lp_token_mint_acc, dead_shares_vault_acc,
        pool_asset_account_acc: pool_reserve_0_acc,
    };
    let pool_reserve_1_acc = g1(pool_reserve_1);
    let mint1_acc = g1(mint1);
    (p, asset1, oracle1, mint1, asset1_acc, pool_reserve_1, pool_reserve_1_acc, mint1_acc)
}

/// audit-38 (L02): `swap` must reject when the target reserve is empty (the
/// impact math saturates to amount_out=0 and old code silently consumed input).
#[test]
fn test_l02_swap_rejects_zero_output() {
    use rlp_client::generated::instructions::SwapBuilder;

    let admin = Pubkey::new_unique();
    let (p, asset1, oracle1, mint1, asset1_acc, pool_reserve_1, pool_reserve_1_acc, mint1_acc) =
        setup_pool_with_two_assets(admin);
    let user = Pubkey::new_unique();
    let (event_authority, _) = derive_event_authority();

    // User has some of token 0, wants to swap into token 1 — but token 1's
    // pool reserve is empty (we never funded it).
    let user_token_from = derive_ata(&user, &p.asset_mint);
    let user_token_to = derive_ata(&user, &mint1);

    let oracle_acc = Account {
        lamports: 1_000_000,
        data: create_mock_pyth_price_data(1_00000000, -8, 0),
        owner: PYTH_PROGRAM_ID,
        executable: false,
        rent_epoch: 0,
    };

    let ix = convert_instruction(
        SwapBuilder::new()
            .signer(user.into())
            .admin(None)
            .settings(p.settings.into())
            .liquidity_pool(p.lp_pda.into())
            .token_from(p.asset_mint.into())
            .token_from_asset(p.asset.into())
            .token_from_oracle(p.oracle.into())
            .token_to(mint1.into())
            .token_to_asset(asset1.into())
            .token_to_oracle(oracle1.into())
            .token_from_pool(p.pool_asset_account.into())
            .token_to_pool(pool_reserve_1.into())
            .token_from_signer_account(user_token_from.into())
            .token_to_signer_account(user_token_to.into())
            .token_program(SPL_TOKEN_ID.into())
            .associated_token_program(SPL_ASSOCIATED_TOKEN_ID.into())
            .event_authority(event_authority.into())
            .program(RLP_ID)
            .amount_in(1_000_000)
            .instruction()
    );

    let mut accounts = vec![
        (user, signer_account()),
        (p.settings, p.settings_acc.clone()),
        (p.lp_pda, p.lp_pda_acc.clone()),
        (p.asset_mint, p.asset_mint_acc.clone()),
        (p.asset, p.asset_acc.clone()),
        (p.oracle, oracle_acc.clone()),
        (mint1, mint1_acc),
        (asset1, asset1_acc),
        (oracle1, oracle_acc),
        (p.pool_asset_account, p.pool_asset_account_acc.clone()),
        (pool_reserve_1, pool_reserve_1_acc), // empty reserve
        (user_token_from, create_token_account(&p.asset_mint, &user, 10_000_000)),
        (user_token_to, create_token_account(&mint1, &user, 0)),
    ];
    accounts.extend(spl_program_accounts());
    accounts.extend(event_cpi_accounts());

    // NotEnoughFunds = variant 5. amount_out > 0 check returns this when
    // target reserve = 0 (oracle_out * 0 / (0 + oracle_out) = 0).
    with_mollusk(|m| m.process_and_validate_instruction(
        &ix,
        &accounts,
        &[Check::err(rlp_error_code(5))],
    ));
}
