# Reflect Insurance Fund
Exchange Reflect Protocol risk for future rUSD and $R rewards.

## Basics
Reflect Insurance Fund lets users restake liquid stake derivatives in exchange for future rUSD and $R yield. Users can restake LSTs with one of the presets offered by protocol maintainers. Presets vary in lockup duration, offered yield and slashing risk.

## Restaking
LSTs can be restaked in Reflect Insurance Fund by invoking `restake()` instruction. Only limited, but mutable set of assets is accepted for restaking. 70% of the deposits are kept outside of the protocol, in a multisig kept by Reflect Protocol Maintainers. This helps minimize the protocol risk. Only 30% of the Insurance Fund is kept in a protocol-owned hot wallet and available for instantenous withdrawal.

## Slashing
Reflect Insurance Fund is a program designed to minimize risk of the Reflect Protocol. In case of Reflect Protocol generating losses, the Insurance Fund is a subject for slashing. The only account having permission for slashing is `superadmin`. Due to its nature, slashing is not instantenous, but a complex process involving temporary lock of the Insurance Fund operations and off-chain cranking. 

Slashing is a per-lockup process. Slashing of accounts under one lockup does not affect operations of another lockup. This also allows for differentiation of slashing rates between different lockups, therefore incentivising longer lockup periods. This is a subject to change.

Slashing starts with `initialize_slash` instruction invoked by Insurance Fund `superadmin`. This instruction is responsible for creating `Slash` account, which is used for tracking different stages of slashing.

The next step of slashing is `slash_deposits` instruction. This instructions should be bundled together and invoked in an ordered manner by an off-chain crank. This instruction is responsible for iterative modification of all deposits of a certain lockup. `Slash` account holds fields like `target_accounts` and `slashed_accounts`. Slashing process cannot be finalized unless values of both fields are equal (meaning all target accounts have been slashed). To prevent slashing the same account twice, what could happen in case of off-chain crank misconfiguration, specific `deposit` accounts holds a `last_slashed` field. Since all slashing processes are indexed, the same `deposit` account cannot be slashed `if deposit.last_slashed == <current slash index>`.

# TODO
- oracles on deposit, select reward boost
- withdrawal intents shouldn't be created via additional instruction, but conditionally depending on the `amount` parameter passed to the `withdraw()` instruction