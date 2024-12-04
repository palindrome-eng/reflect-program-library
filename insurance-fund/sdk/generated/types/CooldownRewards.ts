/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet'
/**
 * This type is used to derive the {@link CooldownRewards} type as well as the de/serializer.
 * However don't refer to it in your code but use the {@link CooldownRewards} type instead.
 *
 * @category userTypes
 * @category enums
 * @category generated
 * @private
 */
export type CooldownRewardsRecord = {
  Single: { fields: [beet.bignum] }
  Dual: { fields: [beet.bignum[] /* size: 2 */] }
}

/**
 * Union type respresenting the CooldownRewards data enum defined in Rust.
 *
 * NOTE: that it includes a `__kind` property which allows to narrow types in
 * switch/if statements.
 * Additionally `isCooldownRewards*` type guards are exposed below to narrow to a specific variant.
 *
 * @category userTypes
 * @category enums
 * @category generated
 */
export type CooldownRewards = beet.DataEnumKeyAsKind<CooldownRewardsRecord>

export const isCooldownRewardsSingle = (
  x: CooldownRewards
): x is CooldownRewards & { __kind: 'Single' } => x.__kind === 'Single'
export const isCooldownRewardsDual = (
  x: CooldownRewards
): x is CooldownRewards & { __kind: 'Dual' } => x.__kind === 'Dual'

/**
 * @category userTypes
 * @category generated
 */
export const cooldownRewardsBeet = beet.dataEnum<CooldownRewardsRecord>([
  [
    'Single',
    new beet.BeetArgsStruct<CooldownRewardsRecord['Single']>(
      [['fields', beet.fixedSizeTuple([beet.u64])]],
      'CooldownRewardsRecord["Single"]'
    ),
  ],
  [
    'Dual',
    new beet.BeetArgsStruct<CooldownRewardsRecord['Dual']>(
      [
        [
          'fields',
          beet.fixedSizeTuple([beet.uniformFixedSizeArray(beet.u64, 2)]),
        ],
      ],
      'CooldownRewardsRecord["Dual"]'
    ),
  ],
]) as beet.FixableBeet<CooldownRewards, CooldownRewards>