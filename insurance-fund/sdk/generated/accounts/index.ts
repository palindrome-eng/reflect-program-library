export * from './Asset'
export * from './Deposit'
export * from './Intent'
export * from './Lockup'
export * from './RewardBoost'
export * from './Settings'
export * from './Slash'

import { Asset } from './Asset'
import { Deposit } from './Deposit'
import { Intent } from './Intent'
import { Lockup } from './Lockup'
import { RewardBoost } from './RewardBoost'
import { Settings } from './Settings'
import { Slash } from './Slash'

export const accountProviders = {
  Asset,
  Deposit,
  Intent,
  Lockup,
  RewardBoost,
  Settings,
  Slash,
}
