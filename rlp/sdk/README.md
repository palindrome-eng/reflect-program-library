# @reflectmoney/insurance

TypeScript SDK for the **Reflect Liquidity Pool (RLP)** Solana program — the junior tranche of Reflect's two-tranche stablecoin insurance system.

Wraps Codama-generated instruction builders and provides higher-level helpers for pool lifecycle, deposits, cooldown-gated withdrawals, NAV-based slashing, and admin operations.

---

## Install

```sh
npm install @reflectmoney/insurance @solana/kit
```

Peer expectations: a `@solana/kit` `Rpc<SolanaRpcApi>` for read paths, and a `TransactionSigner` for writes.

---

## Quick start

```ts
import { createSolanaRpc } from "@solana/kit";
import { Insurance, PdaClient } from "@reflectmoney/insurance";

const rpc = createSolanaRpc("https://api.mainnet-beta.solana.com");
const insurance = new Insurance(rpc);
await insurance.load();

// Deposit into liquidity pool 0
const ix = await insurance.deposit(
  signer,
  1_000_000_000n,  // amount in raw asset units
  USDC_MINT,
  0,               // liquidity pool id
  null,            // minLpTokens slippage guard (null = no minimum)
);
```

---

## What this SDK covers

### Pool lifecycle (admin)

| Method | Notes |
| --- | --- |
| `initializeRlp(signer, swapFeeBps)` | One-time program bootstrap. |
| `addAsset(signer, mint, oracle, accessLevel)` | Whitelist a deposit/swap asset. |
| `updateOracle(signer, assetMint, newOracle)` | Rotate an asset's Pyth feed. |
| `initializeLiquidityPool(signer, lpToken, cooldownDuration, assets, protectedVault?)` | Create a pool. `protectedVault` opts the pool into NAV-based slashing (audit-M06). |
| `initializePoolReserve(signer, liquidityPoolId, assetMint)` | Pre-create the pool's ATA for a whitelisted asset. Idempotent. |
| `forceRemoveAsset(signer, liquidityPoolId, assetMint)` | Recovery path for a pool reserve that has been permanently frozen by the mint authority (audit-M02). |
| `updateDepositCap(signer, liquidityPoolId, newCap)` | Set or clear per-pool deposit cap. |

### User flow

| Method | Notes |
| --- | --- |
| `deposit(signer, amount, mint, liquidityPoolId, minLpTokens?)` | Mints LP tokens proportional to NAV. `minLpTokens` is a slippage guard. |
| `requestWithdrawal(signer, liquidityPoolId, amount)` | Burns LP tokens and opens a per-pool cooldown account. |
| `withdraw(signer, liquidityPoolId, cooldownId)` | Settles a matured cooldown. Excess LP tokens (if pool grew during cooldown) are refunded automatically (audit-1). |
| `swap(signer, liquidityPoolId, tokenFrom, tokenTo, amountIn, minOut?)` | Oracle-priced rebalance between pool assets. |

### Slashing (NAV-based — audit-M06)

| Method | Notes |
| --- | --- |
| `slash(signer, liquidityPoolId, assetMint, proxyState, maxAmount)` | RLP (junior) covers losses of the senior proxy tranche. Pool must have been initialized with a `protectedVault`. The instruction computes `loss = max(0, principal + integratorsCommission − vaultValue)` from the `ProxyState` account and transfers up to `maxAmount` of `assetMint` into the proxy vault. |

The legacy operator-discretion slash has been removed.

### Permissions

| Method | Notes |
| --- | --- |
| `createPermissionAccount(signer, target)` | Create a user-permissions PDA. |
| `updateRoleHolder(signer, target, role, add)` | Grant/revoke a role for a user. |
| `updateActionRole(signer, action, role, add)` | Bind an action (Deposit/Withdraw/Swap/Slash/etc.) to a role. **Slash cannot be assigned to `Role.PUBLIC`.** |

### PDAs

`PdaClient` exposes pure helpers:

```ts
const [settings]         = await PdaClient.deriveSettings();
const [permissions]      = await PdaClient.deriveUserPermissions(user);
const [pool]             = await PdaClient.deriveLiquidityPool(0);
const [asset]            = await PdaClient.deriveAsset(mint);
const [cooldown]         = await PdaClient.deriveCooldown(poolId, cooldownId);
const [eventAuthority]   = await PdaClient.deriveEventAuthority();
const [proxyState]       = await PdaClient.deriveProxyState(brandedMint);
```

---

## Audit fixes in this release

The SDK is aligned with the integrated audit-fixes branch. Notable behavioral changes that callers may need to know about:

- **NAV-based slash** replaces the operator-discretion slash. Call signature is different — see above.
- **`initializeLiquidityPool` accepts `protectedVault`.** Pass `null` for a pool that is not protecting any proxy vault. Pass the proxy state PDA otherwise.
- **`withdraw` refunds excess LP tokens.** No caller change, but the on-chain math now redeems against the *current* NAV at withdraw time, capped by what the cooldown originally locked.
- **`Action.Slash` is not publicly assignable.** Attempting `updateActionRole(Action.Slash, Role.PUBLIC, …)` is rejected by the program.
- **`forceRemoveAsset` + `initializePoolReserve`** are new admin wrappers for the frozen-pool-reserve recovery path.
- **Cooldowns are now per-pool** (a separate counter on each `LiquidityPool` rather than the program-wide settings counter).

---

## Development

```sh
npm install
npm run build          # tsc → dist/
npm test               # mocha + litesvm + the in-tree rlp.so
```

The test suite uses [LiteSVM](https://github.com/LiteSVM/litesvm) and loads `../target/deploy/rlp.so`, so make sure the program is built (`anchor build` or `cargo-build-sbf`) before running.

`npm publish` runs `prepublishOnly` which cleans `dist`, rebuilds, and runs the test suite — publish will fail if any of those steps fail.
