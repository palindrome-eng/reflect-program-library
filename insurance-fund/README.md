# Reflect Restaked Insurance Fund
This protocol offloads Reflect Protocol risk to stakers by allowing stablecoins and/or LSTs lockups in exchange for future $USDR & $R rewards. In case of Reflect Protocol generating losses, the Insurance Fund may be slashed to cover the imbalance of the Delta Neutral position.

## Settings
`Settings` account stores core settings of the program:
- Superadmin - public key of the account allowed to perform changes in the protocol settings (like freeze) and initialize new `Lockup` presets. Superadmin's signature is also required to perform `slash()` operation.
- Cold wallet - due to the nature of the protocol, majority of the insurance will be held outside of the program - it will be moved to cold wallet operated by Reflect Protocol maintainers. This field simply holds the address of this wallet as it will be used during deposits to transfer part of the deposit there.
- Lockups - counter keeping track of existing `Lockup` accounts.
- Shares config - as mentioned above, protocol allows to keep part of the insurance outside of the protocol, in the cold wallet. This config specifies ratios at which funds should be transferred to hot & cold wallets.
- Frozen - simply freeze flag, if present - all interactions with the program are rejected.

## Lockups
The main part of the protocol are `Lockup` accounts. Those accounts can only be initialized by `superadmin` and they are basically *presets* that user can select from to deposit their funds in the protocol. Lockup account specifies:
- `locked` - if present, all interactions with the lockup will be rejected. This flag is necessary for the process of slashing, which involves performing changes over multiple blocks. To keep state unchanged & guarantee fair recalculation of all deposits, Lockup account must be locked for that period.
- `index` - as mentioned above, `Settings` account keeps track of all `Lockup`s using a counter.
- `asset` - each Lockup account only allows deposits in one currency, that's where we store its mint.
- `min_deposit` - minimum acceptable deposit. Anything below will be rejected.
- `duration` - lockup duration. During this period after deposit, user is not allowed to withdraw their funds from the protocol. They may decide to hold it locked for longer.
- `yield_bps` - this is mostly cosmetical thing, it specifies APY (in basepoints) that the crank depositing rewards should try to keep up. It's not guaranteed though & is not used for any calculations in the program itself.
- `yield_mode` - this holds an `YieldMode` enum & specifies whether the `Lockup` accrues rewards in single currency, or if additional Reflect Protocol governance token rewards should be applied.
- `deposit_cap` - optional field. If present, no deposits over this amount will be accepted.
- `deposits` - counter used to keep track of deposits into this `Lockup`. This is required since every deposit is recorded separately.
- `slash_state` - structure used to keep track of slashing processes. `SlashState` keeps two fields: `index` (How many times was this lockup slashed) and `amount` (Total amount slashed since creation of the pool).
- `reward_boosts` - There are `RewardBoost` accounts created on per-Lockup basis. This simply keep tracks of the counter which is used for derivation of those `RewardBoost` PDAs.

## Deposit
`Deposit` is an account keeping track of a singular deposit in the insurance fund. As mentioned earlier, even though all tokens deposited are pooled into a vault owned by `Lockup` PDA, every deposit has its own account. This design choice was made to prevent past slashes from influencing profitability of new deposits. Also you cannot increase existing deposit. It can only be slashed & withdrew.

![Untitled Diagram drawio (3)](https://github.com/user-attachments/assets/bd1628f7-299d-4993-93d7-0809cf20e2c1)


`Deposit` holds the following set of fields:
- `user` - pointer to the account that deposited funds into the protocol.
- `amount` - total deposited. During slashing, we'll subtract from this.
- `initial_usd_value` - USD value of the deposit calculated at the moment of the deposit. This is important since we only care about the initial USD value when finding appicable `RewardBoost`, even if the deposit value decreases and/or is slashed.
- `amount_slashed` - amount lost due to slashing, we increase this during slashing. `amount` + `amount_slashed` = initial deposit
- `lockup` - pointer to the `Lockup` account that specify rules of this deposit. To be fair this could be `u64`.
- `unlock_ts` - timestamp when this particular deposit opens for withdrawals. It's simply `duration` from `Lockup` + timestamp of the deposit.
- `last_slashed` - as mentioned previously, `Lockup` keeps track (by counter) of all slashes performed on particular lockup. This field corresponds to one of the past slashes. This exists to prevent slashing the same deposit twice during one slashing operation.

## Reward Boosts
`RewardBoost` is an account created on per-lockup basis to create on-demand incentives for deposits. Take into account that reward boost only applies to $R governance token rewards, and not USDR. USDR rewards are deposited by crank, and cannot be simply *boosted*. This account is very simple and holds 3 fields:
- `min_usd_value` - minimum **initial** USD value of the deposit that user can apply this reward boost to.
- `boost_bps` - reward boost in basepoints.
- `lockup` - `u64` index of the lockup that this boost applies to.

## Assets & Oracles
`Asset` is an account used for tracking assets that are accepted as deposits into the protocol. This account is also necessary to store pointer to oracle that we're gonna use for calculating `initial_usd_value` of the `Deposit`, as mentioned above. It also keeps some basic stats about the Asset like `tvl` (total value locked), `lockups` (number of lockups accepting this asset as deposit) and `deposits` (total number of deposits in this currency accross all lockups). New assets can be added to the protocol using `add_asset` instruction (superadmin-only). This instruction requires to provide an Oracle account corresponding to this asset. For now, only two providers are accepted - Pyth and Switchboard.

## Intents
Due to the nature of the protocol, it is possible that `amount` of a `deposit` will be larger than actual balance of the `Lockup` vault - this is due to division of the deposit into hot and cold wallet parts. In case of a withdrawal that is larger than the `Lockup` hot balance, the withdrawal must be processed manually, using `Intent` account. `Intent`s are simple accounts holding `amount` (requested withdrawal) and two pointers - to the lockup and deposit accounts. Intents are processed by the `superadmin` using `process_intent` instruction, involving simple token transfer and closing the `Intent` account.

## Slash & Slashing
In case of Reflect Protocol generating losses, the Insurance Fund is a subject for slashing. The only account having permission for slashing is `superadmin`. Due to its nature, slashing is not instantenous, but a complex process involving temporary lock of the `Lockup` operations and off-chain cranking. 

Slashing is a per-lockup process. Slashing of `Deposit`s under one `Lockup` does not affect operations of another `Lockup`. This also allows for differentiation of slashing rates between different lockups, therefore incentivising longer lockup periods. This is a subject to change.

Slashing starts with `initialize_slash` instruction invoked by Insurance Fund `superadmin`. This instruction is responsible for creating `Slash` account, which is used for tracking different stages of slashing. It will also freeze the `Lockup` account to ensure all calculations are ran in a fair manner, for all deposits equally.

The next step of slashing is `slash_deposits` instruction. This instructions should be bundled together and invoked in an ordered manner by an off-chain crank. This instruction is responsible for iterative modification of all deposits of a certain lockup, where deposits are provided using `remaining_accounts`. To prevent slashing the same account twice, what could happen in case of off-chain crank misconfiguration, specific `deposit` accounts holds a `last_slashed` field. Since all slashing processes are indexed, the same `deposit` account cannot be slashed `if deposit.last_slashed == <current slash index>`.

To ensure that insurance fund always keeps the correct ratio between cold and hot wallets, slashing process also involves slashing of the cold wallet. Due to the nature of cold wallet, it might be a case that superadmin
might not want to actually cosign the smart contract transaction using cold wallet.
For this reason, this instruction might be just used to update fields (and invoked by
superadmin hot wallet). 

For transparency reasons, it is advised to provide signature of the transfer instruction that will be included in logs (if actual transfer happened in different transaction).

![Untitled Diagram drawio (5)](https://github.com/user-attachments/assets/d742a2af-c00c-4959-9923-9d313ba9f283)


