# RLP Program - Solana On-Chain Account Architecture

## Overview

This document describes the on-chain account structure for the RLP (Reflect Liquidity Pool) program.

**Program ID:** `rhLMe6vyM1wVLJaxrWUckVmPxSia58nSWZRDtYQow6D`

---

## Program Data Accounts (PDAs)

### Settings (Singleton)

```
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│  SETTINGS (Singleton)                                                                        │
│  ─────────────────────────────────────────────────────────────────────────────────────────  │
│  PDA Seeds: ["settings"]                                                                     │
│  Owner: RLP Program                                                                          │
│  ─────────────────────────────────────────────────────────────────────────────────────────  │
│  Fields:                                                                                     │
│    • bump: u8                                                                                │
│    • liquidity_pools: u8  ──────────────────────────┐                                        │
│    • assets: u8  ───────────────────────────────────┼──┐                                     │
│    • frozen: bool                                   │  │                                     │
│    • access_control: AccessControl                  │  │                                     │
│        ├─ actions: Vec<ActionRoles>                 │  │                                     │
│        └─ kill_switch: KillSwitch { frozen: u8 }    │  │                                     │
└─────────────────────────────────────────────────────┼──┼─────────────────────────────────────┘
                                                      │  │
                            ┌─────────────────────────┘  │
                            │ (index reference)          │ (count reference)
                            ▼                            ▼
```

### LiquidityPool & Asset

```
┌────────────────────────────────────────┐    ┌────────────────────────────────────────┐
│  LIQUIDITY POOL                        │    │  ASSET                                 │
│  ────────────────────────────────────  │    │  ────────────────────────────────────  │
│  PDA Seeds: ["liquidity_pool", index]  │    │  PDA Seeds: ["asset", mint_pubkey]     │
│  Owner: RLP Program                    │    │  Owner: RLP Program                    │
│  ────────────────────────────────────  │    │  ────────────────────────────────────  │
│  Fields:                               │    │  Fields:                               │
│    • bump: u8                          │    │    • mint: Pubkey ─────────────────────┼────┐
│    • index: u8                         │    │    • oracle: Oracle                    │    │
│    • lp_token: Pubkey ─────────────────┼─┐  │        ├─ Pyth(Pubkey) ────────────────┼────┼──┐
│    • cooldowns: u64                    │ │  │        └─ Switchboard(Pubkey) ─────────┼────┼──┤
│    • cooldown_duration: u64            │ │  │    • access_level: AccessLevel         │    │  │
│                                        │ │  │        ├─ Public                       │    │  │
│  [Acts as SIGNER for token operations] │ │  │        └─ Private                      │    │  │
└────────────────────────────────────────┘ │  └────────────────────────────────────────┘    │  │
          │                                │                                                │  │
          │ (owns)                         │ (mint_authority)                               │  │
          ▼                                ▼                                                │  │
```

### Pool Vault

```
┌────────────────────────────────────────────────────────────────────────────────┐
│  POOL VAULT (Associated Token Account)                                         │
│  ──────────────────────────────────────────────────────────────────────────────│
│  Address: get_associated_token_address(liquidity_pool_pda, asset_mint)         │
│  Owner Program: SPL Token Program                                              │
│  ──────────────────────────────────────────────────────────────────────────────│
│  Fields:                                                                       │
│    • mint: Pubkey (references Asset Token Mint)                                │
│    • owner: Pubkey (= LiquidityPool PDA)                                       │
│    • amount: u64                                                               │
│    • delegate: Option<Pubkey>                                                  │
│    • state: AccountState                                                       │
│                                                                                │
│  [One vault per asset per pool - holds deposited tokens]                       │
└────────────────────────────────────────────────────────────────────────────────┘
```

---

## Mint Accounts

```
┌────────────────────────────────────────┐    ┌────────────────────────────────────────┐
│  LP TOKEN MINT                         │    │  ASSET TOKEN MINT (e.g., USDC, SOL)    │
│  ────────────────────────────────────  │    │  ────────────────────────────────────  │
│  Address: (created during init_lp)     │    │  Address: (external, e.g., USDC mint)  │
│  Owner Program: SPL Token Program      │    │  Owner Program: SPL Token Program      │
│  ────────────────────────────────────  │    │  ────────────────────────────────────  │
│  Fields:                               │    │  Fields:                               │
│    • mint_authority: LiquidityPool PDA │    │    • mint_authority: (external)        │
│    • supply: u64                       │    │    • supply: u64                       │
│    • decimals: 9 (required)            │    │    • decimals: u8                      │
│    • is_initialized: true              │    │    • is_initialized: true              │
│    • freeze_authority: None (required) │    │    • freeze_authority: Option<Pubkey>  │
│                                        │    │                                        │
│  [LP controls minting/burning]         │    │  [External token - USDC, wSOL, etc.]   │
└────────────────────────────────────────┘    └────────────────────────────────────────┘
```

---

## External Oracle Accounts

```
┌────────────────────────────────────────┐    ┌────────────────────────────────────────┐
│  PYTH PRICE ACCOUNT                    │    │  SWITCHBOARD AGGREGATOR                │
│  ────────────────────────────────────  │    │  ────────────────────────────────────  │
│  Owner: Pyth Program                   │    │  Owner: Switchboard Program            │
│  ────────────────────────────────────  │    │  ────────────────────────────────────  │
│  Type: PriceUpdateV2                   │    │  Type: AggregatorAccountData           │
│                                        │    │                                        │
│  Methods Used:                         │    │  Methods Used:                         │
│    • get_price_no_older_than(          │    │    • check_staleness(clock, max_age)   │
│        clock, 120s, feed_id)           │    │    • get_result() -> SwitchboardDec    │
│                                        │    │                                        │
│  Returns: Price { price, expo, ... }   │    │  Returns: SwitchboardDecimal           │
│                                        │    │    -> mantissa as i64                  │
└────────────────────────────────────────┘    └────────────────────────────────────────┘
          ▲                                             ▲
          │                                             │
          └──────────────────────┬──────────────────────┘
                                 │
                    (referenced by Asset.oracle)
```

---

## User Accounts

### User Wallet & Permissions

```
┌────────────────────────────────────────┐
│  USER WALLET (Signer)                  │
│  ────────────────────────────────────  │
│  Type: System Account (EOA)            │
│  Owner: System Program                 │
│  ────────────────────────────────────  │
│                                        │
│  [Signs transactions, pays rent/fees]  │
└────────────────────────────────────────┘
          │
          │ (authority for)
          │
          ├─────────────────────────────────────────────────────────┐
          │                                                         │
          ▼                                                         ▼
┌────────────────────────────────────────┐    ┌────────────────────────────────────────┐
│  USER PERMISSIONS                      │    │  USER TOKEN ACCOUNT (ATA)              │
│  ────────────────────────────────────  │    │  ────────────────────────────────────  │
│  PDA Seeds: ["permissions", authority] │    │  Address: get_associated_token_address │
│  Owner: RLP Program                    │    │             (user_wallet, asset_mint)  │
│  ────────────────────────────────────  │    │  Owner Program: SPL Token Program      │
│  Fields:                               │    │  ────────────────────────────────────  │
│    • bump: u8                          │    │  Fields:                               │
│    • authority: Pubkey (= User Wallet) │    │    • mint: Pubkey (asset mint)         │
│    • protocol_roles: LevelRoles        │    │    • owner: Pubkey (= User Wallet)     │
│        └─ Vec<Role>                    │    │    • amount: u64                       │
│            ├─ SUPREMO  (level 6)       │    │                                        │
│            ├─ MANAGER  (level 5)       │    │  [Holds user's depositable tokens]     │
│            ├─ CRANK    (level 4)       │    └────────────────────────────────────────┘
│            ├─ FREEZE   (level 3)       │
│            ├─ TESTEE   (level 2)       │
│            └─ PUBLIC   (level 1)       │    ┌────────────────────────────────────────┐
│                                        │    │  USER LP TOKEN ACCOUNT (ATA)           │
│  [Stores user's protocol permissions]  │    │  ────────────────────────────────────  │
└────────────────────────────────────────┘    │  Address: get_associated_token_address │
                                              │             (user_wallet, lp_mint)     │
                                              │  Owner Program: SPL Token Program      │
                                              │  ────────────────────────────────────  │
                                              │  Fields:                               │
                                              │    • mint: Pubkey (LP token mint)      │
                                              │    • owner: Pubkey (= User Wallet)     │
                                              │    • amount: u64                       │
                                              │                                        │
                                              │  [Holds user's LP token balance]       │
                                              └────────────────────────────────────────┘
```

---

## Cooldown Accounts

```
┌────────────────────────────────────────┐
│  COOLDOWN                              │
│  ────────────────────────────────────  │
│  PDA Seeds: ["cooldown", sequence_num] │◄──── sequence_num = liquidity_pool.cooldowns
│  Owner: RLP Program                    │      (incremented each request_withdrawal)
│  ────────────────────────────────────  │
│  Fields:                               │
│    • authority: Pubkey (= User Wallet) │
│    • liquidity_pool_id: u8             │
│    • unlock_ts: u64                    │──── Clock.unix_timestamp + cooldown_duration
│                                        │
│  [Temporary account during withdrawal] │
└────────────────────────────────────────┘
          │
          │ (owns during cooldown period)
          ▼
┌────────────────────────────────────────┐
│  COOLDOWN LP TOKEN ACCOUNT (ATA)       │
│  ────────────────────────────────────  │
│  Address: get_associated_token_address │
│             (cooldown_pda, lp_mint)    │
│  Owner Program: SPL Token Program      │
│  ────────────────────────────────────  │
│  Fields:                               │
│    • mint: Pubkey (LP token mint)      │
│    • owner: Pubkey (= Cooldown PDA)    │
│    • amount: u64                       │
│                                        │
│  [Locked LP tokens until unlock_ts]    │
└────────────────────────────────────────┘
```

---

## Complete Ownership Hierarchy

```
                              ┌─────────────────────┐
                              │    SYSTEM PROGRAM   │
                              │   (11111111...)     │
                              └─────────────────────┘
                                        │
                                        │ owns
                                        ▼
                              ┌─────────────────────┐
                              │    USER WALLET      │
                              │   (EOA / Signer)    │
                              └─────────────────────┘


       ┌─────────────────────┐                    ┌─────────────────────┐
       │     RLP PROGRAM     │                    │  SPL TOKEN PROGRAM  │
       │  rhLMe6vyM1wV...    │                    │  TokenkegQf...      │
       └─────────────────────┘                    └─────────────────────┘
                 │                                          │
                 │ owns (data accounts)                     │ owns (token accounts)
                 │                                          │
    ┌────────────┼────────────┬──────────────┐              │
    │            │            │              │              │
    ▼            ▼            ▼              ▼              │
┌────────┐ ┌──────────┐ ┌─────────┐ ┌────────────┐          │
│Settings│ │Liquidity │ │  Asset  │ │   User     │          │
│        │ │  Pool    │ │         │ │Permissions │          │
└────────┘ └──────────┘ └─────────┘ └────────────┘          │
                │                                           │
                │ (PDA signs for)                           │
                ▼                                           │
    ┌───────────────────────────────────────────────────────┤
    │                                                       │
    ▼                                                       ▼
┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
│   Pool Vault     │  │ User Token ATA   │  │  User LP ATA     │
│ owner: LP PDA    │  │ owner: User      │  │ owner: User      │
└──────────────────┘  └──────────────────┘  └──────────────────┘

┌──────────────────┐  ┌──────────────────┐
│   LP Token Mint  │  │  Cooldown PDA    │──────┐
│ auth: LP PDA     │  │  (RLP owned)     │      │
└──────────────────┘  └──────────────────┘      │
                                                │ owns
                                                ▼
                                   ┌──────────────────┐
                                   │ Cooldown LP ATA  │
                                   │ owner: Cooldown  │
                                   └──────────────────┘
```

---

## PDA Derivation Reference

| Account | Seeds | Bump |
|---------|-------|------|
| Settings | `["settings"]` | stored |
| LiquidityPool | `["liquidity_pool", pool_index: u8]` | stored |
| Asset | `["asset", mint: Pubkey]` | implicit |
| UserPermissions | `["permissions", authority: Pubkey]` | stored |
| Cooldown | `["cooldown", sequence: u64]` | implicit |

---

## Token Flow Diagrams

### RESTAKE (Deposit)

```
┌──────────────┐                              ┌──────────────┐
│ User Token   │ ─────── transfer ──────────► │ Pool Vault   │
│    ATA       │        (user signs)          │    ATA       │
└──────────────┘                              └──────────────┘

┌──────────────┐                              ┌──────────────┐
│ LP Token     │ ─────── mint_to ───────────► │ User LP      │
│    Mint      │      (LP PDA signs)          │    ATA       │
└──────────────┘                              └──────────────┘
```

### REQUEST WITHDRAWAL

```
┌──────────────┐                              ┌──────────────┐
│ User LP      │ ─────── transfer ──────────► │ Cooldown LP  │
│    ATA       │        (user signs)          │    ATA       │
└──────────────┘                              └──────────────┘
```

### WITHDRAW

```
┌──────────────┐                              ┌──────────────┐
│ Cooldown LP  │ ─────── burn ──────────────► │ LP Token     │
│    ATA       │     (cooldown signs)         │    Mint      │
└──────────────┘                              └──────────────┘

┌──────────────┐                              ┌──────────────┐
│ Pool Vault   │ ─────── transfer ──────────► │ User Token   │
│    ATA       │      (LP PDA signs)          │    ATA       │
└──────────────┘                              └──────────────┘
      (for each asset, proportional amount)
```

### SWAP

```
┌──────────────┐                              ┌──────────────┐
│ User Token   │ ─────── transfer ──────────► │ Pool Vault   │
│ ATA (FROM)   │        (user signs)          │ ATA (FROM)   │
└──────────────┘                              └──────────────┘

┌──────────────┐                              ┌──────────────┐
│ Pool Vault   │ ─────── transfer ──────────► │ User Token   │
│ ATA (TO)     │      (LP PDA signs)          │ ATA (TO)     │
└──────────────┘                              └──────────────┘
```

### SLASH

```
┌──────────────┐                              ┌──────────────┐
│ Pool Vault   │ ─────── transfer ──────────► │ Destination  │
│    ATA       │      (LP PDA signs)          │   Account    │
└──────────────┘                              └──────────────┘
```

---

## Account Summary Table

| Account Type | Owner Program | PDA? | Purpose |
|--------------|---------------|------|---------|
| **Settings** | RLP Program | Yes `["settings"]` | Global protocol state |
| **LiquidityPool** | RLP Program | Yes `["liquidity_pool", idx]` | Pool state, signs token ops |
| **Asset** | RLP Program | Yes `["asset", mint]` | Asset metadata + oracle ref |
| **UserPermissions** | RLP Program | Yes `["permissions", user]` | User role storage |
| **Cooldown** | RLP Program | Yes `["cooldown", seq]` | Withdrawal lockup state |
| **Pool Vault ATA** | SPL Token | No (ATA) | Holds pool's asset tokens |
| **User Token ATA** | SPL Token | No (ATA) | Holds user's asset tokens |
| **User LP ATA** | SPL Token | No (ATA) | Holds user's LP tokens |
| **Cooldown LP ATA** | SPL Token | No (ATA) | Locked LP during cooldown |
| **LP Token Mint** | SPL Token | No | LP token supply control |
