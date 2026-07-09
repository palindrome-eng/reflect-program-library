# RLP Swap Pricing

RLP swaps are **oracle-priced with output-reserve damping**. They are not constant-product (Uniswap-style) AMM swaps. This document describes the formula the program actually implements and the design intent behind it.

## Formula

Given:

- `amount_in` — units of the input asset the caller sends
- `P_from` — oracle price of the input asset
- `P_to` — oracle price of the output asset
- `reserve_out` — pool's current balance of the output asset
- `fee_bps` — protocol swap fee (in basis points)
- `BPS = 10_000`

The program computes:

```
oracle_out          = amount_in × P_from / P_to
impact_factor       = oracle_out / (reserve_out + oracle_out)
amount_after_impact = oracle_out × (1 − impact_factor)
amount_out          = amount_after_impact × (1 − fee_bps / BPS)
```

Substituting:

```
amount_out = oracle_out × reserve_out / (reserve_out + oracle_out) × (1 − fee_bps / BPS)
```

Reference implementation: `programs/rlp/src/instructions/swap/swap.rs`.

## Design intent

The pool is an oracle-anchored basket of stable-ish assets. Pricing is anchored to the oracle quote rather than the pool's reserve ratio. The asymptotic `impact_factor` reduces the payout when a swap is large relative to the output reserve, so a trade can never claim more than the existing reserve can support:

- `impact_factor` is in `[0, 1)` for any positive `oracle_out` and `reserve_out`.
- As `oracle_out → 0` (small trade vs reserve), `impact_factor → 0` and `amount_out ≈ oracle_out × (1 − fee_bps / BPS)`.
- As `oracle_out → ∞` (large trade vs reserve), `impact_factor → 1` and `amount_out → 0`.

This is the property a linear `impact_factor = oracle_out / reserve_out` does not have — a large enough trade would otherwise produce a negative payout.

## Comparison to constant-product

A fee-less constant-product AMM uses:

```
amount_out = reserve_out × amount_in / (reserve_in + amount_in)
```

It is anchored to reserve ratios, not to an external oracle. RLP intentionally does not use this model: the protocol prices stable-asset swaps relative to oracle truth, not relative to pool composition. The constant-product expression appears in this document only to motivate the shape of the asymptotic impact factor — it is not the formula the program runs.

## Path-dependence (audit issue L13)

The per-call impact factor is not path-independent. Splitting one large swap into many smaller ones reduces the total impact and pushes the aggregate output toward the undamped oracle quote. With output reserve `R` and a desired oracle output `O`:

- One call: `amount_out ≈ O × R / (R + O)`
- `n` equal-size sub-swaps, each with oracle output `O/n`: the sum of payouts approaches `O` as `n → ∞`.

This is a known property of the per-call damping model and is not addressed in the program math itself.

### Mitigation

`swap` is access-controlled. Callers must hold a role that maps to `Action::Swap` in the protocol's `AccessControl`, and (for assets with `AccessLevel::Private`) the caller must additionally hold a permission account. The intended operator population is a small set of whitelisted keypairs with no incentive to extract value from LPs by splitting trades.

Path-dependence is therefore an accepted operational characteristic rather than an exploitable vulnerability: the attack surface described in L13 only matters under a permissionless-swap model, which RLP does not adopt. If the access-control model ever opens up to untrusted callers, the protocol should add an aggregate per-slot or per-epoch output cap to bound cumulative extraction.

## Slippage protection

Callers may pass `min_out: Option<u64>`. When `Some(n)` with `n > 0`, the program rejects swaps whose final `amount_out` falls below `n`. A `None` or `Some(0)` value disables slippage protection — callers requiring slippage protection must pass a positive `min_out`.

## Guards

The program enforces, in addition to the formula above:

- `amount_in > 0` (rejected at instruction entry).
- `amount_out > 0` (audit issue L02 fix — rejects swaps against an empty target reserve where the math saturates to zero output).
- `reserve_out ≥ amount_out` (reserve must cover the payout).
- Source and destination mints must differ; both assets must be whitelisted in the pool; oracle freshness and confidence are validated per the Pyth/Doppler readers.

Intermediate arithmetic is kept in `u128` until the final amount is bounded against the reserve, then narrowed to `u64` for the transfer (audit issue E11 fix).
