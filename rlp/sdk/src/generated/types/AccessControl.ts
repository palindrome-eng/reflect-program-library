/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet'
import { AccessMap, accessMapBeet } from './AccessMap'
import { KillSwitch, killSwitchBeet } from './KillSwitch'
export type AccessControl = {
  accessMap: AccessMap
  killswitch: KillSwitch
}

/**
 * @category userTypes
 * @category generated
 */
export const accessControlBeet = new beet.BeetArgsStruct<AccessControl>(
  [
    ['accessMap', accessMapBeet],
    ['killswitch', killSwitchBeet],
  ],
  'AccessControl'
)
