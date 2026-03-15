use std::cell::RefCell;
use mollusk_svm::Mollusk;
use mollusk_svm::result::Check;
use rlp::constants::ASSET_SEED;
use rlp_client::{
    // Instructions
    InitializeRlpBuilder,
    AddAssetBuilder,
    FreezeFunctionalityBuilder,
    UpdateActionRoleBuilder,
    CreatePermissionAccountBuilder,
    UpdateRoleHolderBuilder,
    // Types
    AccessLevel,
    Action,
    Role,
    Update,
    // Accounts
    Settings,
    UserPermissions,
    // Constants
    RLP_ID,
    SETTINGS_DISCRIMINATOR,
    USER_PERMISSIONS_DISCRIMINATOR,
};
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
pub use helpers::pda::{derive_permissions_pda, derive_settings_pda};

// Pyth program ID
const PYTH_PROGRAM_ID: Pubkey = solana_sdk::pubkey!("rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ");

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

/// Derives an asset PDA
fn derive_asset_pda(index: u8) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[ASSET_SEED.as_bytes(), &[index]],
        &RLP_ID,
    )
}

/// Creates mock Pyth PriceUpdateV2 data (134 bytes)
fn create_mock_pyth_price_data(price: i64, exponent: i32, publish_time: i64) -> Vec<u8> {
    let mut data = Vec::with_capacity(134);
    
    // Discriminator for PriceUpdateV2
    data.extend_from_slice(&[34, 241, 35, 99, 157, 126, 244, 205]);
    data.extend_from_slice(&[0u8; 32]); // write_authority
    data.push(1); data.push(0); // verification_level (Full)
    data.extend_from_slice(&[1u8; 32]); // feed_id
    data.extend_from_slice(&price.to_le_bytes());
    data.extend_from_slice(&100u64.to_le_bytes()); // conf
    data.extend_from_slice(&exponent.to_le_bytes());
    data.extend_from_slice(&publish_time.to_le_bytes());
    data.extend_from_slice(&(publish_time - 1).to_le_bytes()); // prev_publish_time
    data.extend_from_slice(&price.to_le_bytes()); // ema_price
    data.extend_from_slice(&100u64.to_le_bytes()); // ema_conf
    data.extend_from_slice(&1u64.to_le_bytes()); // posted_slot
    
    assert_eq!(data.len(), 134);
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
fn convert_instruction(client_ix: solana_instruction::Instruction) -> Instruction {
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

    let ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];

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

    // First initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];

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
            .action(Action::FreezeRestake)
            .freeze(true)
            .instruction()
    );

    let freeze_accounts = vec![
        (signer, signer_account()),
        (settings, updated_settings),
        (permissions, updated_permissions),
        (system_program::ID, system_program_account()),
    ];

    let freeze_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&freeze_ix, &freeze_accounts, &[Check::success()])
    });

    // Verify freeze
    let final_settings = get_result_account(&freeze_result, 1);
    let settings_data = Settings::from_bytes(&final_settings.data).unwrap();

    let restake_mask = 1u8 << (Action::Restake as u8);
    assert!(
        (settings_data.access_control.killswitch.frozen & restake_mask) != 0,
        "Restake should be frozen"
    );
}

#[test]
fn test_unfreeze_protocol() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];

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
            .action(Action::FreezeRestake)
            .freeze(true)
            .instruction()
    );

    let freeze_accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions.clone()),
        (system_program::ID, system_program_account()),
    ];

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
            .action(Action::FreezeRestake)
            .freeze(false)
            .instruction()
    );

    let unfreeze_accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions),
        (system_program::ID, system_program_account()),
    ];

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

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];

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
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions.clone()),
        (system_program::ID, system_program_account()),
    ];

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
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions),
        (system_program::ID, system_program_account()),
    ];

    let result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&freeze_slash_ix, &accounts, &[Check::success()])
    });

    // Verify both frozen
    let final_settings = get_result_account(&result, 1);
    let settings_data = Settings::from_bytes(&final_settings.data).unwrap();

    let withdraw_mask = 1u8 << (Action::Withdraw as u8);
    let slash_mask = 1u8 << (Action::Slash as u8);

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

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];

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
            .action(Action::Restake)
            .role(Role::PUBLIC)
            .update(Update::Add)
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions),
        (system_program::ID, system_program_account()),
    ];

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
        .find(|m| m.action == Action::Restake)
        .unwrap();

    assert!(restake_mapping.allowed_roles.contains(&Role::PUBLIC));
}

#[test]
fn test_set_withdraw_action_to_public() {
    let signer = Pubkey::new_unique();
    let (settings, _) = derive_settings_pda();
    let (permissions, _) = derive_permissions_pda(signer);

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];

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
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions),
        (system_program::ID, system_program_account()),
    ];

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

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];

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
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions.clone()),
        (system_program::ID, system_program_account()),
    ];

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
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions),
        (system_program::ID, system_program_account()),
    ];

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

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];

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
                .action(Action::PrivateSwap)
                .role(*role)
                .update(Update::Add)
                .instruction()
        );

        let accounts = vec![
            (signer, signer_account()),
            (settings, current_settings),
            (permissions, current_permissions.clone()),
            (system_program::ID, system_program_account()),
        ];

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
        .find(|m| m.action == Action::PrivateSwap)
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

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];

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
            .instruction()
    );

    let accounts = vec![
        (settings, current_settings),
        (new_admin_permissions, empty_account()),
        (signer, signer_account()),
        (system_program::ID, system_program_account()),
    ];

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

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];

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
                .instruction()
        );

        let accounts = vec![
            (settings, current_settings.clone()),
            (new_admin_permissions, empty_account()),
            (signer, signer_account()),
            (system_program::ID, system_program_account()),
        ];

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

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];

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
            .instruction()
    );

    let accounts = vec![
        (settings, current_settings.clone()),
        (target_user_permissions, empty_account()),
        (signer, signer_account()),
        (system_program::ID, system_program_account()),
    ];

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
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions),
        (target_user_permissions, target_permissions_account),
        (system_program::ID, system_program_account()),
    ];

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

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];

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
            .instruction()
    );

    let accounts = vec![
        (settings, current_settings.clone()),
        (target_user_permissions, empty_account()),
        (signer, signer_account()),
        (system_program::ID, system_program_account()),
    ];

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
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (settings, current_settings.clone()),
        (permissions, current_permissions.clone()),
        (target_user_permissions, target_permissions_account),
        (system_program::ID, system_program_account()),
    ];

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
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (settings, current_settings),
        (permissions, current_permissions),
        (target_user_permissions, target_permissions_account),
        (system_program::ID, system_program_account()),
    ];

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

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];

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
            .instruction()
    );

    let accounts = vec![
        (settings, current_settings.clone()),
        (target_user_permissions, empty_account()),
        (signer, signer_account()),
        (system_program::ID, system_program_account()),
    ];

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
                .instruction()
        );

        let accounts = vec![
            (signer, signer_account()),
            (settings, current_settings.clone()),
            (permissions, current_permissions.clone()),
            (target_user_permissions, target_permissions_account),
            (system_program::ID, system_program_account()),
        ];

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

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];

    let init_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&init_ix, &accounts, &[Check::success()])
    });

    let current_settings = get_result_account(&init_result, 2);
    let current_permissions = get_result_account(&init_result, 1);

    // Set up mock accounts for asset creation
    let mint = Pubkey::new_unique();
    let oracle = Pubkey::new_unique();
    let (asset, _) = derive_asset_pda(0);

    let publish_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

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
            .instruction()
    );

    let accounts = vec![
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

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];

    let init_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&init_ix, &accounts, &[Check::success()])
    });

    let current_settings = get_result_account(&init_result, 2);
    let current_permissions = get_result_account(&init_result, 1);

    // Set up mock accounts
    let mint = Pubkey::new_unique();
    let oracle = Pubkey::new_unique();
    let (asset, _) = derive_asset_pda(0);

    let publish_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

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
            .instruction()
    );

    let accounts = vec![
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

    // Initialize
    let init_ix = convert_instruction(
        InitializeRlpBuilder::new()
            .signer(signer.into())
            .permissions(permissions.into())
            .settings(settings.into())
            .system_program(system_program::ID.into())
            .instruction()
    );

    let accounts = vec![
        (signer, signer_account()),
        (permissions, empty_account()),
        (settings, empty_account()),
        (system_program::ID, system_program_account()),
    ];

    let init_result = with_mollusk(|mollusk| {
        mollusk.process_and_validate_instruction(&init_ix, &accounts, &[Check::success()])
    });

    let mut current_settings = get_result_account(&init_result, 2);
    let current_permissions = get_result_account(&init_result, 1);

    let publish_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // Add 5 assets
    for i in 0..5u8 {
        let mint = Pubkey::new_unique();
        let oracle = Pubkey::new_unique();
        let (asset, _) = derive_asset_pda(i);

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
                .instruction()
        );

        let accounts = vec![
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

        let result = with_mollusk(|mollusk| {
            mollusk.process_and_validate_instruction(&add_asset_ix, &accounts, &[Check::success()])
        });

        current_settings = get_result_account(&result, 2);
    }

    // Verify all assets were created
    let settings_data = Settings::from_bytes(&current_settings.data).unwrap();
    assert_eq!(settings_data.assets, 5);
}
