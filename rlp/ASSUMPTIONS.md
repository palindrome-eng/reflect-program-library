# RLP design assumptions

This document enumerates the design assumptions the RLP protocol relies on. Each entry corresponds to an item raised during the audit by AdevarLabs (`reflect-money-proxy-rlp-20260515`) and labeled `Assumption` — items the auditor flagged for explicit acknowledgement rather than a code change.

If any of these assumptions stops holding (for example, MANAGER is delegated to a hot wallet, or governance decides to support reactivation of deactivated assets), the corresponding behavior must be revisited.

## Pool reserves are derived from live SPL balances

**Issue #4** — Unsolicited transfers to pool ATAs are socialized to LP holders.

Pool reserves are read from the live SPL token account balance, so any direct transfer into a pool ATA becomes claimable by existing LP holders via pro-rata withdrawals.

We accept that accidental or unsolicited transfers into pool ATAs cannot be distinguished from legitimate liquidity on-chain. The protocol does not provide an operator-controlled recovery path; surplus value flows to LPs through normal redemption.

## Killswitch covers user-facing actions

**Issue #7** — Admin-action freeze coverage.

The freeze mechanism is intended primarily for user-facing protocol actions (`Deposit`, `Withdraw`, `Swap`, `Slash`). It is not a circuit breaker for every administrative action. Some admin actions check `killswitch.is_frozen(...)`, but the freeze instruction only maps to the core runtime actions.

We accept this as a governance/design limitation. If a full admin freeze is later required, the killswitch action map must be extended.

## Deployment ordering is controlled

**Issue #8** — Initialization ordering.

`initialize_rlp` grants the initial `SUPREMO` role to the first successful caller. We assume the team controls deployment and that `initialize_rlp` is invoked by the intended deployer immediately after program deployment.

This is deployment hygiene rather than an on-chain enforced invariant — the protocol cannot distinguish the intended deployer before initialization. Standard controlled deployment procedures are sufficient.

## MANAGER and other privileged roles are held by trusted operators

**Issue #9** — Governance-level trust in manager roles.

The access-control model grants broad authority to privileged roles. In particular, `MANAGER` can update action-role mappings and assign or remove non-`SUPREMO` roles, materially changing who can perform sensitive operations.

We assume `MANAGER` and equivalent roles are held only by governance-level trusted actors (secure multisig or equivalent controlled operational process). Under this assumption, the broad reconfiguration power is an intentional administrative trust model.

This assumption must be revisited if any privileged role is delegated to an operational hot wallet or individual operator key.

## AccessLevel::Private restricts direct swaps only

**Issue #11** — Private asset access level applies only to direct swaps.

`AccessLevel::Private` restricts direct swap operations involving the asset. It does **not** prevent LP holders from receiving the asset during normal pro-rata withdrawals from a basket pool.

We accept that distributing private assets via `deposit`/`withdraw` is expected behavior for a basket liquidity pool and is not an access-control bypass.

## Single-sided deposits price via oracle, withdrawals pay pro-rata

**Issue #15** — Single-sided deposits enable oracle-NAV basket conversion.

Users deposit one asset and receive LP tokens priced by that asset's oracle value, while withdrawals redeem a pro-rata share of all pool assets. This is economically fair only if oracle prices are current and all assets are similarly liquid.

If a volatile or illiquid asset is temporarily overvalued relative to its true market price, a user could deposit it, receive inflated LP, and withdraw a share of more liquid pool assets at a profit.

We accept this as a known design property of basket pools. The Pyth confidence-interval check (audit issue #33) and the oracle freshness check bound — but do not eliminate — this risk. Operationally, mitigations like asset caps, target weights, or fees on imbalanced flow can be layered on later if a specific asset's risk profile warrants it.
