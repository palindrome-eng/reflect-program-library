use litesvm::LiteSVM;
use litesvm_token::CreateMint;
use solana_sdk::{
    account::Account,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    transaction::Transaction,
};
use solana_sdk_ids::system_program;
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
use rlp::constants::ASSET_SEED;

// Pyth program ID - using solana_sdk::pubkey since pyth SDK has different Pubkey type
const PYTH_PROGRAM_ID: Pubkey = solana_sdk::pubkey!("rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ");

/// Creates mock Pyth PriceUpdateV2 data
/// 
/// PriceUpdateV2 layout (134 bytes total):
/// - discriminator: 8 bytes (Anchor account discriminator)
/// - write_authority: 32 bytes (Pubkey)
/// - verification_level: 2 bytes (enum: variant + optional u8)
/// - price_message.feed_id: 32 bytes
/// - price_message.price: 8 bytes (i64)
/// - price_message.conf: 8 bytes (u64)
/// - price_message.exponent: 4 bytes (i32)
/// - price_message.publish_time: 8 bytes (i64)
/// - price_message.prev_publish_time: 8 bytes (i64)
/// - price_message.ema_price: 8 bytes (i64)
/// - price_message.ema_conf: 8 bytes (u64)
/// - posted_slot: 8 bytes (u64)
fn create_mock_pyth_price_data(price: i64, exponent: i32, publish_time: i64) -> Vec<u8> {
    let mut data = Vec::with_capacity(134);
    
    // Discriminator for PriceUpdateV2 (from Anchor)
    // sha256("account:PriceUpdateV2")[..8]
    data.extend_from_slice(&[34, 241, 35, 99, 157, 126, 244, 205]);
    
    // write_authority (32 bytes) - zeros
    data.extend_from_slice(&[0u8; 32]);
    
    // verification_level - Full variant (variant discriminator = 1, no extra data)
    // For enum with Partial { num_signatures: u8 } and Full variants:
    // - Partial = [0, num_signatures] (2 bytes)
    // - Full = [1, 0] (padded to 2 bytes for consistent size)
    data.push(1); // Full variant
    data.push(0); // padding for alignment
    
    // price_message.feed_id (32 bytes)
    data.extend_from_slice(&[1u8; 32]);
    
    // price_message.price (i64)
    data.extend_from_slice(&price.to_le_bytes());
    
    // price_message.conf (u64)
    data.extend_from_slice(&100u64.to_le_bytes());
    
    // price_message.exponent (i32)
    data.extend_from_slice(&exponent.to_le_bytes());
    
    // price_message.publish_time (i64)
    data.extend_from_slice(&publish_time.to_le_bytes());
    
    // price_message.prev_publish_time (i64)
    data.extend_from_slice(&(publish_time - 1).to_le_bytes());
    
    // price_message.ema_price (i64)
    data.extend_from_slice(&price.to_le_bytes());
    
    // price_message.ema_conf (u64)
    data.extend_from_slice(&100u64.to_le_bytes());
    
    // posted_slot (u64)
    data.extend_from_slice(&1u64.to_le_bytes());
    
    assert_eq!(data.len(), 134, "PriceUpdateV2 data must be exactly 134 bytes");
    data
}

pub mod helpers;
pub use helpers::pda::{derive_permissions_pda, derive_settings_pda};

/// Derives an asset PDA
fn derive_asset_pda(index: u8) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[ASSET_SEED.as_bytes(), &[index]],
        &RLP_ID,
    )
}

/// Helper struct to manage test state
struct TestContext {
    svm: LiteSVM,
    admin: Keypair,
    settings: Pubkey,
    admin_permissions: Pubkey,
}

impl TestContext {
    fn new() -> Self {
        let mut svm = LiteSVM::new();
        let program_bytes = include_bytes!("../../../target/deploy/rlp.so");
        svm.add_program(RLP_ID, program_bytes).unwrap();

        let admin = Keypair::new();
        svm.airdrop(&admin.pubkey(), 100_000_000_000).unwrap();

        let (settings, _) = derive_settings_pda();
        let (admin_permissions, _) = derive_permissions_pda(admin.pubkey());

        Self {
            svm,
            admin,
            settings,
            admin_permissions,
        }
    }

    fn initialize_rlp(&mut self) {
        let instruction = InitializeRlpBuilder::new()
            .settings(self.settings)
            .signer(self.admin.pubkey())
            .permissions(self.admin_permissions)
            .system_program(system_program::ID)
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&self.admin.pubkey()),
            &[&self.admin],
            self.svm.latest_blockhash(),
        );

        self.svm.send_transaction(tx).expect("initialize_rlp failed");
    }

    /// Creates a new SPL token mint using litesvm-token
    fn create_mint(&mut self, decimals: u8) -> Pubkey {
        CreateMint::new(&mut self.svm, &self.admin)
            .authority(&self.admin.pubkey())
            .decimals(decimals)
            .send()
            .expect("Failed to create mint")
    }

    /// Creates a mock Pyth oracle account with price data
    fn create_pyth_oracle(&mut self, price: i64, exponent: i32) -> Pubkey {
        let oracle_pubkey = Pubkey::new_unique();
        
        // Get current timestamp (use a reasonable mock timestamp)
        let publish_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        
        let oracle_data = create_mock_pyth_price_data(price, exponent, publish_time);
        
        self.svm.set_account(
            oracle_pubkey,
            Account {
                lamports: 1_000_000,
                data: oracle_data,
                owner: PYTH_PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
            }
        ).expect("Failed to set oracle account");
        
        oracle_pubkey
    }
}

// ============================================================================
// INITIALIZATION TESTS
// ============================================================================

#[test]
fn test_initialize_rlp_instruction() {
    let mut ctx = TestContext::new();
    ctx.initialize_rlp();

    // Verify settings account
    let settings_account = ctx.svm.get_account(&ctx.settings).unwrap();
    let settings_data = Settings::from_bytes(&settings_account.data).unwrap();

    assert_eq!(settings_data.liquidity_pools, 0);
    assert_eq!(settings_data.assets, 0);
    assert_eq!(settings_data.discriminator, SETTINGS_DISCRIMINATOR);
    assert_eq!(settings_account.owner, RLP_ID);
    assert_eq!(settings_data.access_control.access_map.action_permissions.len(), 18);
    assert_eq!(settings_data.access_control.access_map.mapping_count, 16);
    assert_eq!(settings_data.access_control.killswitch.frozen, 0);
}

// ============================================================================
// ASSET MANAGEMENT TESTS
// ============================================================================

#[test]
fn test_add_public_assets() {
    let mut ctx = TestContext::new();
    ctx.initialize_rlp();

    // Create real SPL token mints and Pyth oracles
    let mints: Vec<Pubkey> = (0..3).map(|_| ctx.create_mint(9)).collect();
    // Create mock Pyth oracles with different prices
    let oracles: Vec<Pubkey> = (0..3).map(|i| {
        // Alternate prices: $100, $50, $100 with 8 decimal exponent
        let price = if i % 2 == 0 { 100_00000000i64 } else { 50_00000000i64 };
        ctx.create_pyth_oracle(price, -8)
    }).collect();

    for i in 0..3u8 {
        let (asset, _) = derive_asset_pda(i);

        let instruction = AddAssetBuilder::new()
            .signer(ctx.admin.pubkey())
            .admin(ctx.admin_permissions)
            .settings(ctx.settings)
            .asset(asset)
            .asset_mint(mints[i as usize])
            .oracle(oracles[i as usize])
            .system_program(system_program::ID)
            .access_level(AccessLevel::Public)
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&ctx.admin.pubkey()),
            &[&ctx.admin],
            ctx.svm.latest_blockhash(),
        );

        ctx.svm.send_transaction(tx).expect(&format!("add_asset {} failed", i));

        // Verify asset account was created (exists and has correct owner)
        let asset_account = ctx.svm.get_account(&asset).unwrap();
        assert_eq!(asset_account.owner, RLP_ID, "Asset account should be owned by RLP program");
        assert!(!asset_account.data.is_empty(), "Asset account should have data");
    }

    // Verify settings was updated with correct asset count
    let settings_account = ctx.svm.get_account(&ctx.settings).unwrap();
    let settings_data = Settings::from_bytes(&settings_account.data).unwrap();
    assert_eq!(settings_data.assets, 3);
}

#[test]
fn test_add_private_assets() {
    let mut ctx = TestContext::new();
    ctx.initialize_rlp();

    // Create a real SPL token mint and Pyth oracle
    let mint = ctx.create_mint(9);
    let oracle = ctx.create_pyth_oracle(75_00000000, -8); // $75 price
    let (asset, _) = derive_asset_pda(0);

    let instruction = AddAssetBuilder::new()
        .signer(ctx.admin.pubkey())
        .admin(ctx.admin_permissions)
        .settings(ctx.settings)
        .asset(asset)
        .asset_mint(mint)
        .oracle(oracle)
        .system_program(system_program::ID)
        .access_level(AccessLevel::Private)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&ctx.admin.pubkey()),
        &[&ctx.admin],
        ctx.svm.latest_blockhash(),
    );

    ctx.svm.send_transaction(tx).expect("add_private_asset failed");

    // Verify asset account was created (exists and has correct owner)
    let asset_account = ctx.svm.get_account(&asset).unwrap();
    assert_eq!(asset_account.owner, RLP_ID, "Asset account should be owned by RLP program");
    assert!(!asset_account.data.is_empty(), "Asset account should have data");
    
    // Verify settings was updated
    let settings_account = ctx.svm.get_account(&ctx.settings).unwrap();
    let settings_data = Settings::from_bytes(&settings_account.data).unwrap();
    assert_eq!(settings_data.assets, 1);
}

// ============================================================================
// FREEZE/UNFREEZE TESTS
// ============================================================================

#[test]
fn test_freeze_protocol() {
    let mut ctx = TestContext::new();
    ctx.initialize_rlp();

    let instruction = FreezeFunctionalityBuilder::new()
        .admin(ctx.admin.pubkey())
        .settings(ctx.settings)
        .admin_permissions(ctx.admin_permissions)
        .system_program(system_program::ID)
        .action(Action::FreezeRestake)
        .freeze(true)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&ctx.admin.pubkey()),
        &[&ctx.admin],
        ctx.svm.latest_blockhash(),
    );

    ctx.svm.send_transaction(tx).expect("freeze_functionality failed");

    // Verify killswitch was updated
    let settings_account = ctx.svm.get_account(&ctx.settings).unwrap();
    let settings_data = Settings::from_bytes(&settings_account.data).unwrap();

    // Check that Restake action is frozen (bit 0 should be set)
    let restake_mask = 1u8 << (Action::Restake as u8);
    assert!(
        (settings_data.access_control.killswitch.frozen & restake_mask) != 0,
        "Restake should be frozen"
    );
}

#[test]
fn test_unfreeze_protocol() {
    let mut ctx = TestContext::new();
    ctx.initialize_rlp();

    // First freeze
    let freeze_ix = FreezeFunctionalityBuilder::new()
        .admin(ctx.admin.pubkey())
        .settings(ctx.settings)
        .admin_permissions(ctx.admin_permissions)
        .system_program(system_program::ID)
        .action(Action::FreezeRestake)
        .freeze(true)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[freeze_ix],
        Some(&ctx.admin.pubkey()),
        &[&ctx.admin],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).expect("freeze failed");

    // Then unfreeze
    let unfreeze_ix = FreezeFunctionalityBuilder::new()
        .admin(ctx.admin.pubkey())
        .settings(ctx.settings)
        .admin_permissions(ctx.admin_permissions)
        .system_program(system_program::ID)
        .action(Action::FreezeRestake)
        .freeze(false)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[unfreeze_ix],
        Some(&ctx.admin.pubkey()),
        &[&ctx.admin],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).expect("unfreeze failed");

    // Verify killswitch was cleared
    let settings_account = ctx.svm.get_account(&ctx.settings).unwrap();
    let settings_data = Settings::from_bytes(&settings_account.data).unwrap();
    assert_eq!(settings_data.access_control.killswitch.frozen, 0);
}

// ============================================================================
// ACCESS CONTROL TESTS
// ============================================================================

#[test]
fn test_set_restaking_action_to_public() {
    let mut ctx = TestContext::new();
    ctx.initialize_rlp();

    let instruction = UpdateActionRoleBuilder::new()
        .admin(ctx.admin.pubkey())
        .settings(ctx.settings)
        .admin_permissions(ctx.admin_permissions)
        .system_program(system_program::ID)
        .action(Action::Restake)
        .role(Role::PUBLIC)
        .update(Update::Add)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&ctx.admin.pubkey()),
        &[&ctx.admin],
        ctx.svm.latest_blockhash(),
    );

    ctx.svm.send_transaction(tx).expect("update_action_role failed");

    // Verify the settings were updated
    let settings_account = ctx.svm.get_account(&ctx.settings).unwrap();
    let settings_data = Settings::from_bytes(&settings_account.data).unwrap();

    // Find the Restake action mapping and check if PUBLIC role is allowed
    let restake_mapping = settings_data
        .access_control
        .access_map
        .action_permissions
        .iter()
        .find(|m| m.action == Action::Restake)
        .expect("Restake action mapping not found");

    assert!(
        restake_mapping.allowed_roles.contains(&Role::PUBLIC),
        "PUBLIC role should be in allowed_roles for Restake"
    );
}

#[test]
fn test_set_withdraw_action_to_public() {
    let mut ctx = TestContext::new();
    ctx.initialize_rlp();

    let instruction = UpdateActionRoleBuilder::new()
        .admin(ctx.admin.pubkey())
        .settings(ctx.settings)
        .admin_permissions(ctx.admin_permissions)
        .system_program(system_program::ID)
        .action(Action::Withdraw)
        .role(Role::PUBLIC)
        .update(Update::Add)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&ctx.admin.pubkey()),
        &[&ctx.admin],
        ctx.svm.latest_blockhash(),
    );

    ctx.svm.send_transaction(tx).expect("update_action_role failed");

    let settings_account = ctx.svm.get_account(&ctx.settings).unwrap();
    let settings_data = Settings::from_bytes(&settings_account.data).unwrap();

    let withdraw_mapping = settings_data
        .access_control
        .access_map
        .action_permissions
        .iter()
        .find(|m| m.action == Action::Withdraw)
        .expect("Withdraw action mapping not found");

    assert!(
        withdraw_mapping.allowed_roles.contains(&Role::PUBLIC),
        "PUBLIC role should be in allowed_roles for Withdraw"
    );
}

#[test]
fn test_update_action_role_add_and_remove() {
    let mut ctx = TestContext::new();
    ctx.initialize_rlp();

    // Add TESTEE role to SuspendDeposits action
    let add_ix = UpdateActionRoleBuilder::new()
        .admin(ctx.admin.pubkey())
        .settings(ctx.settings)
        .admin_permissions(ctx.admin_permissions)
        .system_program(system_program::ID)
        .action(Action::SuspendDeposits)
        .role(Role::TESTEE)
        .update(Update::Add)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_ix],
        Some(&ctx.admin.pubkey()),
        &[&ctx.admin],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).expect("add role failed");

    // Verify role was added
    let settings_account = ctx.svm.get_account(&ctx.settings).unwrap();
    let settings_data = Settings::from_bytes(&settings_account.data).unwrap();
    let mapping = settings_data
        .access_control
        .access_map
        .action_permissions
        .iter()
        .find(|m| m.action == Action::SuspendDeposits)
        .unwrap();
    assert!(mapping.allowed_roles.contains(&Role::TESTEE));

    // Remove TESTEE role
    let remove_ix = UpdateActionRoleBuilder::new()
        .admin(ctx.admin.pubkey())
        .settings(ctx.settings)
        .admin_permissions(ctx.admin_permissions)
        .system_program(system_program::ID)
        .action(Action::SuspendDeposits)
        .role(Role::TESTEE)
        .update(Update::Remove)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[remove_ix],
        Some(&ctx.admin.pubkey()),
        &[&ctx.admin],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).expect("remove role failed");

    // Verify role was removed
    let settings_account = ctx.svm.get_account(&ctx.settings).unwrap();
    let settings_data = Settings::from_bytes(&settings_account.data).unwrap();
    let mapping = settings_data
        .access_control
        .access_map
        .action_permissions
        .iter()
        .find(|m| m.action == Action::SuspendDeposits)
        .unwrap();
    assert!(!mapping.allowed_roles.contains(&Role::TESTEE));
}

// ============================================================================
// PERMISSION ACCOUNT TESTS
// ============================================================================

#[test]
fn test_create_permission_account() {
    let mut ctx = TestContext::new();
    ctx.initialize_rlp();

    let new_admin = Keypair::new();
    let (new_admin_permissions, _) = derive_permissions_pda(new_admin.pubkey());

    let instruction = CreatePermissionAccountBuilder::new()
        .settings(ctx.settings)
        .new_creds(new_admin_permissions)
        .caller(ctx.admin.pubkey())
        .system_program(system_program::ID)
        .new_admin(new_admin.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&ctx.admin.pubkey()),
        &[&ctx.admin],
        ctx.svm.latest_blockhash(),
    );

    ctx.svm.send_transaction(tx).expect("create_permission_account failed");

    // Verify permission account was created
    let permissions_account = ctx.svm.get_account(&new_admin_permissions).unwrap();
    let permissions_data = UserPermissions::from_bytes(&permissions_account.data).unwrap();

    assert_eq!(permissions_data.discriminator, USER_PERMISSIONS_DISCRIMINATOR);
    assert_eq!(permissions_data.authority, new_admin.pubkey());
    assert!(permissions_data.protocol_roles.roles.is_empty());
}

#[test]
fn test_update_role_holder_add_role() {
    let mut ctx = TestContext::new();
    ctx.initialize_rlp();

    // Create permission account for target user
    let target_user = Keypair::new();
    let (target_user_permissions, _) = derive_permissions_pda(target_user.pubkey());

    let create_ix = CreatePermissionAccountBuilder::new()
        .settings(ctx.settings)
        .new_creds(target_user_permissions)
        .caller(ctx.admin.pubkey())
        .system_program(system_program::ID)
        .new_admin(target_user.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&ctx.admin.pubkey()),
        &[&ctx.admin],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).expect("create permission account failed");

    // Add CRANK role to target user
    let update_ix = UpdateRoleHolderBuilder::new()
        .admin(ctx.admin.pubkey())
        .settings(ctx.settings)
        .admin_permissions(ctx.admin_permissions)
        .update_admin_permissions(target_user_permissions)
        .system_program(system_program::ID)
        .address(target_user.pubkey())
        .role(Role::CRANK)
        .update(Update::Add)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[update_ix],
        Some(&ctx.admin.pubkey()),
        &[&ctx.admin],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).expect("update role holder failed");

    // Verify role was added
    let permissions_account = ctx.svm.get_account(&target_user_permissions).unwrap();
    let permissions_data = UserPermissions::from_bytes(&permissions_account.data).unwrap();
    
    assert_eq!(permissions_data.authority, target_user.pubkey());
    assert!(permissions_data.protocol_roles.roles.contains(&Role::CRANK));
}

#[test]
fn test_update_role_holder_remove_role() {
    let mut ctx = TestContext::new();
    ctx.initialize_rlp();

    // Create permission account for target user
    let target_user = Keypair::new();
    let (target_user_permissions, _) = derive_permissions_pda(target_user.pubkey());

    let create_ix = CreatePermissionAccountBuilder::new()
        .settings(ctx.settings)
        .new_creds(target_user_permissions)
        .caller(ctx.admin.pubkey())
        .system_program(system_program::ID)
        .new_admin(target_user.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&ctx.admin.pubkey()),
        &[&ctx.admin],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).expect("create permission account failed");

    // Add FREEZE role
    let add_ix = UpdateRoleHolderBuilder::new()
        .admin(ctx.admin.pubkey())
        .settings(ctx.settings)
        .admin_permissions(ctx.admin_permissions)
        .update_admin_permissions(target_user_permissions)
        .system_program(system_program::ID)
        .address(target_user.pubkey())
        .role(Role::FREEZE)
        .update(Update::Add)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[add_ix],
        Some(&ctx.admin.pubkey()),
        &[&ctx.admin],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).expect("add role failed");

    // Verify role was added
    let permissions_account = ctx.svm.get_account(&target_user_permissions).unwrap();
    let permissions_data = UserPermissions::from_bytes(&permissions_account.data).unwrap();
    assert!(permissions_data.protocol_roles.roles.contains(&Role::FREEZE));

    // Remove FREEZE role
    let remove_ix = UpdateRoleHolderBuilder::new()
        .admin(ctx.admin.pubkey())
        .settings(ctx.settings)
        .admin_permissions(ctx.admin_permissions)
        .update_admin_permissions(target_user_permissions)
        .system_program(system_program::ID)
        .address(target_user.pubkey())
        .role(Role::FREEZE)
        .update(Update::Remove)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[remove_ix],
        Some(&ctx.admin.pubkey()),
        &[&ctx.admin],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).expect("remove role failed");

    // Verify role was removed
    let permissions_account = ctx.svm.get_account(&target_user_permissions).unwrap();
    let permissions_data = UserPermissions::from_bytes(&permissions_account.data).unwrap();
    assert!(!permissions_data.protocol_roles.roles.contains(&Role::FREEZE));
}

// ============================================================================
// MULTIPLE FREEZE ACTION TESTS
// ============================================================================

#[test]
fn test_freeze_multiple_actions() {
    let mut ctx = TestContext::new();
    ctx.initialize_rlp();

    // Freeze Withdraw
    let freeze_withdraw = FreezeFunctionalityBuilder::new()
        .admin(ctx.admin.pubkey())
        .settings(ctx.settings)
        .admin_permissions(ctx.admin_permissions)
        .system_program(system_program::ID)
        .action(Action::FreezeWithdraw)
        .freeze(true)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[freeze_withdraw],
        Some(&ctx.admin.pubkey()),
        &[&ctx.admin],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).expect("freeze withdraw failed");

    // Freeze Slash
    let freeze_slash = FreezeFunctionalityBuilder::new()
        .admin(ctx.admin.pubkey())
        .settings(ctx.settings)
        .admin_permissions(ctx.admin_permissions)
        .system_program(system_program::ID)
        .action(Action::FreezeSlash)
        .freeze(true)
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[freeze_slash],
        Some(&ctx.admin.pubkey()),
        &[&ctx.admin],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).expect("freeze slash failed");

    // Verify both are frozen
    let settings_account = ctx.svm.get_account(&ctx.settings).unwrap();
    let settings_data = Settings::from_bytes(&settings_account.data).unwrap();

    let withdraw_mask = 1u8 << (Action::Withdraw as u8);
    let slash_mask = 1u8 << (Action::Slash as u8);

    assert!(
        (settings_data.access_control.killswitch.frozen & withdraw_mask) != 0,
        "Withdraw should be frozen"
    );
    assert!(
        (settings_data.access_control.killswitch.frozen & slash_mask) != 0,
        "Slash should be frozen"
    );
}

// ============================================================================
// EDGE CASE TESTS
// ============================================================================

#[test]
fn test_add_maximum_assets() {
    let mut ctx = TestContext::new();
    ctx.initialize_rlp();

    // Add 10 assets to test the system handles multiple assets
    for i in 0..10u8 {
        // Create a real SPL token mint and Pyth oracle for each asset
        let mint = ctx.create_mint(9);
        let oracle = ctx.create_pyth_oracle((i as i64 + 1) * 10_00000000, -8); // $10, $20, $30...
        let (asset, _) = derive_asset_pda(i);

        let instruction = AddAssetBuilder::new()
            .signer(ctx.admin.pubkey())
            .admin(ctx.admin_permissions)
            .settings(ctx.settings)
            .asset(asset)
            .asset_mint(mint)
            .oracle(oracle)
            .system_program(system_program::ID)
            .access_level(if i % 2 == 0 { AccessLevel::Public } else { AccessLevel::Private })
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&ctx.admin.pubkey()),
            &[&ctx.admin],
            ctx.svm.latest_blockhash(),
        );

        ctx.svm.send_transaction(tx).expect(&format!("add asset {} failed", i));
    }

    // Verify all assets were created
    let settings_account = ctx.svm.get_account(&ctx.settings).unwrap();
    let settings_data = Settings::from_bytes(&settings_account.data).unwrap();
    assert_eq!(settings_data.assets, 10);
}

#[test]
fn test_create_multiple_permission_accounts() {
    let mut ctx = TestContext::new();
    ctx.initialize_rlp();

    for _ in 0..5 {
        let new_user = Keypair::new();
        let (new_user_permissions, _) = derive_permissions_pda(new_user.pubkey());

        let instruction = CreatePermissionAccountBuilder::new()
            .settings(ctx.settings)
            .new_creds(new_user_permissions)
            .caller(ctx.admin.pubkey())
            .system_program(system_program::ID)
            .new_admin(new_user.pubkey())
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[instruction],
            Some(&ctx.admin.pubkey()),
            &[&ctx.admin],
            ctx.svm.latest_blockhash(),
        );

        ctx.svm.send_transaction(tx).expect("create permission account failed");

        // Verify
        let permissions_account = ctx.svm.get_account(&new_user_permissions).unwrap();
        let permissions_data = UserPermissions::from_bytes(&permissions_account.data).unwrap();
        assert_eq!(permissions_data.authority, new_user.pubkey());
    }
}

#[test]
fn test_grant_multiple_roles_to_user() {
    let mut ctx = TestContext::new();
    ctx.initialize_rlp();

    let target_user = Keypair::new();
    let (target_user_permissions, _) = derive_permissions_pda(target_user.pubkey());

    // Create permission account
    let create_ix = CreatePermissionAccountBuilder::new()
        .settings(ctx.settings)
        .new_creds(target_user_permissions)
        .caller(ctx.admin.pubkey())
        .system_program(system_program::ID)
        .new_admin(target_user.pubkey())
        .instruction();

    let tx = Transaction::new_signed_with_payer(
        &[create_ix],
        Some(&ctx.admin.pubkey()),
        &[&ctx.admin],
        ctx.svm.latest_blockhash(),
    );
    ctx.svm.send_transaction(tx).expect("create permission account failed");

    // Add multiple roles
    let roles = [Role::CRANK, Role::FREEZE, Role::MANAGER];
    
    for role in roles.iter() {
        let add_ix = UpdateRoleHolderBuilder::new()
            .admin(ctx.admin.pubkey())
            .settings(ctx.settings)
            .admin_permissions(ctx.admin_permissions)
            .update_admin_permissions(target_user_permissions)
            .system_program(system_program::ID)
            .address(target_user.pubkey())
            .role(*role)
            .update(Update::Add)
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[add_ix],
            Some(&ctx.admin.pubkey()),
            &[&ctx.admin],
            ctx.svm.latest_blockhash(),
        );
        ctx.svm.send_transaction(tx).expect(&format!("add role {:?} failed", role));
    }

    // Verify all roles were added
    let permissions_account = ctx.svm.get_account(&target_user_permissions).unwrap();
    let permissions_data = UserPermissions::from_bytes(&permissions_account.data).unwrap();
    
    for role in roles.iter() {
        assert!(
            permissions_data.protocol_roles.roles.contains(role),
            "User should have role {:?}",
            role
        );
    }
}

#[test]
fn test_action_permissions_for_multiple_roles() {
    let mut ctx = TestContext::new();
    ctx.initialize_rlp();

    // Add multiple roles to PrivateSwap action (fresh action without pre-assigned roles)
    // Avoid PUBLIC since it's commonly pre-assigned
    let roles = [Role::TESTEE, Role::FREEZE, Role::MANAGER];
    
    for role in roles.iter() {
        let add_ix = UpdateActionRoleBuilder::new()
            .admin(ctx.admin.pubkey())
            .settings(ctx.settings)
            .admin_permissions(ctx.admin_permissions)
            .system_program(system_program::ID)
            .action(Action::PrivateSwap)
            .role(*role)
            .update(Update::Add)
            .instruction();

        let tx = Transaction::new_signed_with_payer(
            &[add_ix],
            Some(&ctx.admin.pubkey()),
            &[&ctx.admin],
            ctx.svm.latest_blockhash(),
        );
        ctx.svm.send_transaction(tx).expect(&format!("add role {:?} to action failed", role));
    }

    // Verify all roles are allowed for PrivateSwap
    let settings_account = ctx.svm.get_account(&ctx.settings).unwrap();
    let settings_data = Settings::from_bytes(&settings_account.data).unwrap();
    
    let swap_mapping = settings_data
        .access_control
        .access_map
        .action_permissions
        .iter()
        .find(|m| m.action == Action::PrivateSwap)
        .unwrap();

    for role in roles.iter() {
        assert!(
            swap_mapping.allowed_roles.contains(role),
            "PrivateSwap should allow role {:?}",
            role
        );
    }
}
