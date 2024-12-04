/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet'
export type RemoveAdminArgs = {
  adminId: number
}

/**
 * @category userTypes
 * @category generated
 */
export const removeAdminArgsBeet = new beet.BeetArgsStruct<RemoveAdminArgs>(
  [['adminId', beet.u8]],
  'RemoveAdminArgs'
)
