/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet'
export type SwapArgs = {
  fromLockupId: beet.bignum
  toLockupId: beet.bignum
  amountIn: beet.bignum
  minAmountOut: beet.COption<beet.bignum>
}

/**
 * @category userTypes
 * @category generated
 */
export const swapArgsBeet = new beet.FixableBeetArgsStruct<SwapArgs>(
  [
    ['fromLockupId', beet.u64],
    ['toLockupId', beet.u64],
    ['amountIn', beet.u64],
    ['minAmountOut', beet.coption(beet.u64)],
  ],
  'SwapArgs'
)
