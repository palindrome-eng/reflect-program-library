# Reflect Liquidity Protocol (RLP)

The Reflect Liquidity Protocol (RLP) is a DeFi protocol that provides backstop liquidity for the Reflect Protocol ecosystem. RLP enables users to provide liquidity to a multi-asset pool in exchange for LP tokens, with the liquidity being freely available to Reflect Protocol for core functionalities such as stablecoin swaps, collateral swaps, or slashing as insurance claims. In return for providing this liquidity, Reflect Protocol shares a portion of its revenue with RLP participants. The protocol implements a role-based access control system, automated market making capabilities, and risk management through slashing mechanisms.

## Core Architecture

### Settings Account
The `Settings` account serves as the central configuration hub for the entire protocol:

- **bump**: PDA bump for the settings account
- **liquidity_pools**: Counter tracking the number of active liquidity pools (default: 1 core pool, expandable)
- **assets**: Counter tracking the number of supported assets
- **frozen**: Global freeze flag that halts all protocol operations when set
- **access_control**: Complex permissions system controlling who can perform specific actions

### Access Control System
The protocol implements a role-based access control (RBAC) system with the following components:

#### Roles
- **UNSET**: Default role with no permissions
- **PUBLIC**: Allows anyone to perform the action (used for basic user operations)
- **TESTEE**: Testing role for development and validation
- **FREEZE**: Can freeze/unfreeze specific protocol actions
- **CRANK**: Can perform slashing and swap operations
- **MANAGER**: Can perform administrative tasks like adding assets and updating settings
- **SUPREMO**: Super admin with all permissions

#### Actions
The protocol defines specific actions that can be performed:
- **Core Actions**: `Restake`, `Withdraw`, `Slash`, `Swap`
- **Freeze Actions**: `FreezeRestake`, `FreezeWithdraw`, `FreezeSlash`, `FreezeSwap`
- **Administrative Actions**: `InitializeLiquidityPool`, `AddAsset`, `UpdateDepositCap`, `DepositRewards`, `Management`, `SuspendDeposits`, `FreezeProgram`, `UpdateRole`, `UpdateAction`

#### User Permissions
The `UserPermissions` account is optional and only required for permissioned actions. Public actions can be performed without this account. When a user needs to perform a permissioned instruction, they must have a `UserPermissions` account that stores:
- **authority**: The public key of the account owner
- **bump**: PDA bump for the permissions account
- **protocol_roles**: Array of roles assigned to this user at the protocol level

The permissions system uses a mapping between actions and allowed roles, enabling granular control over who can perform specific operations.

### Liquidity Pools
Liquidity pools are the core mechanism for asset management:

#### LiquidityPool Account
- **bump**: PDA bump for the liquidity pool
- **index**: Unique identifier for the pool
- **lp_token**: Mint address of the LP token representing pool shares
- **cooldowns**: Counter tracking active cooldown periods
- **cooldown_duration**: Duration of the withdrawal cooldown period

#### Pool Operations
1. **Restaking**: Users deposit assets into the pool and receive LP tokens
2. **Withdrawal**: Users burn LP tokens to withdraw their share of all pool assets
3. **Rewards**: External actors can deposit rewards directly into pool asset accounts
4. **Swapping**: Permissioned users can swap between assets within the pool

### Assets and Oracles
The `Asset` account tracks supported assets and their price feeds:

- **mint**: Token mint address
- **oracle**: Price oracle configuration (supports Pyth and Switchboard)

Assets are used for:
- Calculating USD values of deposits
- Determining swap rates
- Tracking pool composition

### Cooldown System
The protocol implements a cooldown mechanism for withdrawals to prevent frontrunning of insurance claims:

#### Cooldown Account
- **authority**: User who initiated the withdrawal
- **liquidity_pool_id**: Associated liquidity pool
- **unlock_ts**: Timestamp when withdrawal becomes available

Users must first request a withdrawal, which locks their LP tokens in a cooldown account for a specified duration. During this cooldown period, users remain subject to slashing. Liquidity becomes claimable only after the cooldown period expires.

## Core Operations

### Restaking (Depositing)
Users can deposit supported assets into liquidity pools:

1. **Permission Validation**: Validates user permissions for restaking
```rust
action_check_protocol(
    Action::Restake,
    permissions.as_deref(),
    &settings.access_control
)?;
```

2. **Input Validation**: Ensures valid deposit amount
```rust
require!(amount > 0, InsuranceFundError::InvalidInput);
```

3. **Pool Value Calculation**: Calculates total pool value before deposit
```rust
let total_pool_value_before = liquidity_pool.calculate_total_pool_value(
    &ctx.remaining_accounts,
    liquidity_pool,
    settings,
    &clock
)?;
```

4. **Asset Transfer**: Transfers user's assets to pool
```rust
liquidity_pool.deposit(
    signer,
    amount,
    &ctx.accounts.user_asset_account,
    &ctx.accounts.pool_asset_account,
    token_program
)?;
```

5. **Deposit Value Calculation**: Calculates USD value of deposit
```rust
let deposit_asset_price = asset.get_price(oracle, &clock)?;
let deposit_value = PreciseNumber::new(
    deposit_asset_price.mul(amount)?
).ok_or(InsuranceFundError::MathOverflow)?;
```

6. **LP Token Calculation**: Determines LP tokens to mint
```rust
let lp_tokens_to_mint = liquidity_pool
    .calculate_lp_tokens_on_deposit(
        lp_token,
        total_pool_value_before,
        deposit_value
    )?;
```

7. **Slippage Protection**: Validates minimum LP tokens received
```rust
require!(min_lp_tokens <= lp_tokens_to_mint, InsuranceFundError::SlippageExceeded);
```

8. **LP Token Minting**: Mints LP tokens to user
```rust
liquidity_pool.mint_lp_token(
    lp_tokens_to_mint,
    liquidity_pool,
    lp_token,
    &ctx.accounts.user_lp_account,
    token_program
)?;
```

### Withdrawal Process
Withdrawal is a two-step process:

#### Step 1: Request Withdrawal
User calls `request_withdrawal()` to initiate the withdrawal process:

1. **Permission Validation**: Validates user permissions for withdrawal
```rust
action_check_protocol(
    Action::Withdraw,
    permissions.as_deref(),
    &settings.access_control
)?;
```

2. **Cooldown Initialization**: Creates cooldown account and sets unlock timestamp
```rust
cooldown.set_inner(Cooldown {
    liquidity_pool_id,
    authority: signer.key(),
    ..Default::default()
});
cooldown.lock(liquidity_pool.cooldown_duration)?;
```

3. **LP Token Transfer**: Transfers LP tokens to cooldown account
```rust
transfer(CpiContext::new(token_program.to_account_info(), 
    Transfer {
        from: signer_lp_token_account.to_account_info(),
        to: cooldown_lp_token_account.to_account_info(),
        authority: signer.to_account_info()
    }), amount)?;
```

#### Step 2: Execute Withdrawal
After cooldown period, user calls `withdraw()` to complete withdrawal:

1. **Cooldown Validation**: Ensures cooldown period has expired
```rust
require!(
    clock.unix_timestamp as u64 >= cooldown.unlock_ts,
    InsuranceFundError::CooldownInForce
);
```

2. **Input Validation**: Validates LP token amounts
```rust
require!(
    lp_token_amount > 0 && lp_token_supply > 0,
    InsuranceFundError::InvalidInput
);
```

3. **User Share Calculation**: Calculates user's proportional share of each asset
```rust
let user_pool_share_amount = PreciseNumber::new(pool_token_account.amount as u128)
    .ok_or(InsuranceFundError::MathOverflow)?
    .checked_mul(&PreciseNumber::new(lp_token_amount as u128)
        .ok_or(InsuranceFundError::MathOverflow)?)
    .ok_or(InsuranceFundError::MathOverflow)?
    .checked_div(&PreciseNumber::new(lp_token_supply as u128)
        .ok_or(InsuranceFundError::MathOverflow)?)
    .ok_or(InsuranceFundError::MathOverflow)?
    .to_imprecise()
    .ok_or(InsuranceFundError::MathOverflow)?
    .to_u64()
    .ok_or(InsuranceFundError::MathOverflow)?;
```

4. **Asset Transfers**: Transfers user's share of each asset from pool to user
```rust
if user_pool_share_amount > 0 {
    transfer(CpiContext::new_with_signer(token_program.to_account_info(), 
        Transfer {
            from: pool_token_account_info.to_account_info(),
            to: user_token_account_info.to_account_info(),
            authority: liquidity_pool.to_account_info()
        }, &[lp_seeds]), user_pool_share_amount)?;
}
```

5. **LP Token Burning**: Burns user's LP tokens from cooldown account
```rust
burn(CpiContext::new_with_signer(token_program.to_account_info(), 
    Burn {
        authority: cooldown.to_account_info(),
        from: cooldown_lp_token_account.to_account_info(),
        mint: lp_token_mint.to_account_info()
    }, &[&[COOLDOWN_SEED.as_bytes(), &cooldown_id.to_le_bytes(), &[ctx.bumps.cooldown]]]), 
    lp_token_amount)?;
```

### Permissioned Swap Function
The protocol includes a swap mechanism:

#### Swap Operation
- **Permission Required**: Only users with specific role can perform swaps
- **Oracle Integration**: Uses price feeds to calculate swap rates
- **Fee-less**: No trading fees are charged on swaps
- **Oracle Price Execution**: Swaps are always executed at the oracle price
- **Slippage Protection**: Optional minimum output amount protection
- **Pool Balance Checks**: Ensures sufficient liquidity for swaps

#### Swap Process
1. **Input Validation**: Validates swap parameters and prevents self-swapping
```rust
require!(amount_in > 0, InsuranceFundError::InvalidInput);
require!(token_from.key() != token_to.key(), InsuranceFundError::InvalidInput);
```

2. **Oracle Price Fetching**: Retrieves current prices from configured oracles
```rust
let token_from_price = token_from_asset.get_price(token_from_oracle, clock)?;
let token_to_price = token_to_asset.get_price(token_to_oracle, clock)?;
```

3. **Output Amount Calculation**: Calculates swap output using oracle price ratio
```rust
let amount_out: u64 = token_from_price
    .mul(amount_in)?
    .div(token_to_price.mul(1)?)
    .try_into()
    .map_err(|_| InsuranceFundError::MathOverflow)?;
```

4. **Pool Balance Validation**: Ensures sufficient liquidity for the swap
```rust
require!(token_to_pool.amount >= amount_out, InsuranceFundError::NotEnoughFunds);
```

5. **Slippage Protection**: Optional minimum output amount validation
```rust
if let Some(min_amount) = min_out {
    require!(amount_out >= min_amount, InsuranceFundError::SlippageExceeded);
}
```

6. **Token Transfers**: Executes atomic token transfers
```rust
// Transfer input tokens from user to pool
transfer(CpiContext::new(token_program.to_account_info(), 
    Transfer { 
        from: token_from_signer_account.to_account_info(), 
        to: token_from_pool.to_account_info(), 
        authority: signer.to_account_info() 
    }), amount_in)?;

// Transfer output tokens from pool to user
transfer(CpiContext::new_with_signer(token_program.to_account_info(), 
    Transfer { 
        from: token_to_pool.to_account_info(), 
        to: token_to_signer_account.to_account_info(), 
        authority: liquidity_pool.to_account_info()
    }, &[lp_seeds]), amount_out)?;
```

### Slashing Mechanism
The protocol includes a slashing mechanism for risk management:

#### Slash Operation
- **Permission Required**: Only users with specific role can slash
- **Target**: Specific asset within a liquidity pool
- **Process**: Transfers specified amount from pool to destination account
- **Use Case**: Insurance claims management, emergency fund transfers, protocol fees

#### Slashing Impact
When slashing occurs, it reduces the total pool value, making all existing LP tokens worth slightly less in USD terms. However, this does not affect future deposits because:

- **LP Token Calculation**: New deposits are calculated based on the ratio of deposit USD value to total pool USD value
- **Fair Distribution**: All LP token holders share the loss proportionally
- **Future Deposits**: New users receive LP tokens based on current pool value, ensuring fair pricing regardless of past slashing events
- **USD Value Basis**: The protocol always operates on USD value ratios, so slashing affects all participants equally

## Permissions System Design

### Hierarchical Access Control
The permissions system operates at multiple levels:

1. **Protocol Level**: Global permissions that apply across all pools
2. **Action Level**: Specific permissions for each action type
3. **User Level**: Individual user role assignments

### Permission Inheritance
- **SUPREMO** role has access to all actions
- Lower-level roles inherit permissions based on action mappings
- Public actions can be performed by anyone
- Frozen actions are blocked for all users

### Dynamic Permission Management
The system allows for:
- Adding/removing roles from specific actions
- Updating user role assignments
- Freezing/unfreezing specific actions
- Granular control over protocol operations

### Security Features
- **Kill Switch**: Can freeze specific actions or entire protocol
- **Role Validation**: All actions validate user permissions
- **Action Mapping**: Clear mapping between actions and allowed roles
- **Audit Trail**: All permission changes are logged

## Liquidity Pool Mechanics

### Pool Value Calculation
The protocol uses precise mathematical calculations with the following formulas:

#### Total Pool Value
```
Total Pool Value = Σ(Asset Balance × Asset Price)
```
Where each asset's USD value is calculated as:
```
Asset USD Value = Token Balance × Oracle Price
```

#### LP Token Minting (First Deposit)
For the first deposit to an empty pool:
```
LP Tokens Minted = Deposit USD Value
```

#### LP Token Minting (Subsequent Deposits)
For subsequent deposits:
```
LP Tokens Minted = (Deposit USD Value × Current LP Supply) / Total Pool Value
```

#### User Share Calculation (Withdrawal)
When withdrawing, user receives their proportional share of each asset:
```
User Asset Share = (Pool Asset Balance × User LP Tokens) / Total LP Supply
```

#### Swap Rate Calculation
Swaps are executed at oracle prices:
```
Amount Out = (Amount In × Token From Price) / Token To Price
```

### Asset Management
- Each pool can hold multiple assets
- Assets are tracked with their USD values
- Oracle prices ensure accurate valuations
- Pool composition can change through swaps and deposits

### Risk Management
- **Cooldown Periods**: Prevent rapid withdrawals
- **Slashing Mechanism**: Insurance claims management
- **Permission Controls**: Limit sensitive operations to authorized users

## Technical Implementation

### Program Structure

The RLP program follows a modular architecture organized by functionality and access levels:

#### Module Organization
```
src/
├── lib.rs                 # Program entry point and instruction definitions
├── states/                # Account structures and data models
│   ├── settings.rs        # Global protocol settings
│   ├── liquidity_pool.rs  # Liquidity pool state and operations
│   ├── asset.rs          # Asset definitions and oracle integration
│   ├── cooldown.rs       # Withdrawal cooldown mechanism
│   ├── permissions.rs    # User permission management
│   ├── access.rs         # RBAC system implementation
│   ├── action.rs         # Action definitions and validation
│   ├── killswitch.rs     # Granular action freezing
│   └── update.rs         # Update tracking
├── instructions/          # Business logic organized by access level
│   ├── admin/            # Administrative operations
│   │   ├── initialize_rlp.rs
│   │   ├── initialize_lp.rs
│   │   ├── add_asset.rs
│   │   ├── freeze_functionality.rs
│   │   ├── update_deposit_cap.rs
│   │   ├── action_update.rs
│   │   ├── role_holder_update.rs
│   │   ├── create_permission_account.rs
│   │   └── rlp_admin_context.rs
│   ├── user/             # End-user operations
│   │   ├── restake.rs
│   │   ├── request_withdraw.rs
│   │   └── withdraw.rs
│   ├── crank/            # Automated operations
│   │   └── deposit_rewards.rs
│   ├── swap/             # Asset swapping
│   │   └── swap.rs
│   └── slash/            # Risk management
│       └── slash.rs
├── helpers/              # Utility functions and calculations
│   ├── action_check_protocol.rs
│   ├── calculate_receipts_on_mint.rs
│   ├── get_price_from_pyth.rs
│   ├── get_price_from_switchboard.rs
│   └── calculate_total_deposits.rs
├── events/               # On-chain event definitions
├── constants/            # Protocol configuration values
└── errors/               # Custom error types
```

#### Instruction Categories

**Admin Instructions** (`instructions/admin/`)
- **Initialization**: `initialize_rlp`, `initialize_lp`, `initialize_lp_token_account`
- **Asset Management**: `add_asset`, `update_deposit_cap`
- **Permission Management**: `create_permission_account`, `action_update`, `role_holder_update`
- **Security Controls**: `freeze_functionality`

**User Instructions** (`instructions/user/`)
- **Liquidity Operations**: `restake` (deposit), `request_withdraw`, `withdraw`
- **Access Level**: Requires appropriate permissions or public access

**Crank Instructions** (`instructions/crank/`)
- **Reward Distribution**: `deposit_rewards` - allows external actors to deposit rewards

**Swap Instructions** (`instructions/swap/`)
- **Asset Trading**: `swap_lp` - permissioned asset swapping within pools

**Slash Instructions** (`instructions/slash/`)
- **Risk Management**: `slash` - emergency fund extraction for insurance claims

#### State Management

**Core State Accounts**
- `Settings`: Global protocol configuration and access control
- `LiquidityPool`: Individual pool state and parameters
- `Asset`: Supported assets and oracle configurations
- `UserPermissions`: User role assignments and permissions
- `Cooldown`: Withdrawal request state and timing

**State Relationships**
```
Settings (Global)
├── AccessControl (Permissions + Killswitch)
├── Assets (Counter)
└── LiquidityPools (Counter)

LiquidityPool (Per Pool)
├── LP Token Mint
├── Asset Accounts (Per Asset)
└── Cooldown Accounts (Per User)

UserPermissions (Per User)
└── Protocol Roles
```

#### Helper Functions

**Security & Permissions**
- `action_check_protocol`: Validates user permissions for actions
- `calculate_receipts_on_mint`: LP token calculation logic

**Oracle Integration**
- `get_price_from_pyth`: Pyth oracle price fetching
- `get_price_from_switchboard`: Switchboard oracle price fetching

**Mathematical Operations**
- `calculate_total_deposits`: Pool value calculations
- Precise number operations for overflow protection

### Key Features
- **PDA Management**: All accounts use Program Derived Addresses
- **Oracle Integration**: Support for multiple price feed providers
- **Mathematical Precision**: Uses precise number calculations
- **Error Handling**: Comprehensive error types and validation
- **Event Logging**: Detailed on-chain event tracking

### Security Considerations

#### Permission Validation
All operations validate user permissions through the `action_check_protocol` function:

```rust
pub fn action_check_protocol(
    action: Action,
    creds: Option<&UserPermissions>,
    protocol_access_control: &AccessControl,
) -> Result<()> {
    // Check if action is frozen
    if action.is_core() {
        protocol_access_control.action_unsuspended(&action)?;
    }
    
    // Check if action is public
    if protocol_access_control.is_public_action(action) {
        return Ok(());
    }
    
    // Validate user permissions
    if let Some(creds) = creds {
        if creds.can_perform_protocol_action(action, protocol_access_control) {
            return Ok(());
        }
    }
    
    Err(InsuranceFundError::IncorrectAdmin.into())
}
```

#### Killswitch Implementation
The protocol implements a granular killswitch system using bitwise operations:

```rust
pub struct KillSwitch {
    pub frozen: u8, // Bitmap of frozen actions
}

impl KillSwitch {
    pub fn is_frozen(&self, action: &Action) -> bool {
        let mask = 1u8 << (*action as u8);
        (self.frozen & mask) != 0
    }
    
    pub fn freeze(&mut self, action: &Action) {
        let mask = 1u8 << (*action as u8);
        self.frozen |= mask;
    }
    
    pub fn unfreeze(&mut self, action: &Action) {
        let mask = 1u8 << (*action as u8);
        self.frozen &= !mask;
    }
}
```

#### Role-Based Access Control
The protocol uses a hierarchical RBAC system with action-role mappings:

```rust
pub struct ActionMapping {
    pub action: Action,
    pub allowed_roles: [Role; MAX_ROLES],
    pub role_count: u8,
}

pub struct AccessMap {
    pub action_permissions: [ActionMapping; MAX_ACTION_MAPPINGS],
    pub mapping_count: u8,
}
```

#### Instruction-Level Security
Each instruction validates permissions through Anchor constraints:

```rust
#[account(
    seeds = [PERMISSIONS_SEED.as_bytes(), signer.key().as_ref()],
    bump,
    constraint = permissions.can_perform_protocol_action(
        Action::Slash, 
        &settings.access_control
    ) @ InsuranceFundError::PermissionsTooLow,
    constraint = !settings.access_control.killswitch.is_frozen(&Action::Slash) 
        @ InsuranceFundError::Frozen,
)]
pub permissions: Account<'info, UserPermissions>,
```

#### Default Permission Configuration
The protocol initializes with secure default permissions:

```rust
pub fn new_defaults() -> Result<Self> {
    let mut access_control: AccessControl = Self::default();
    
    // Manager actions
    access_control.add_role_to_action(Action::UpdateDepositCap, Role::MANAGER)?;
    access_control.add_role_to_action(Action::AddAsset, Role::MANAGER)?;
    
    // Crank actions
    access_control.add_role_to_action(Action::Slash, Role::CRANK)?;
    access_control.add_role_to_action(Action::Swap, Role::CRANK)?;
    
    // User actions
    access_control.add_role_to_action(Action::Restake, Role::TESTEE)?;
    access_control.add_role_to_action(Action::Withdraw, Role::TESTEE)?;
    
    // Freeze actions
    access_control.add_role_to_action(Action::FreezeRestake, Role::FREEZE)?;
    access_control.add_role_to_action(Action::FreezeWithdraw, Role::FREEZE)?;
    
    Ok(access_control)
}
```