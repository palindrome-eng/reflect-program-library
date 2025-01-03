export * from './Admin'
export * from './Asset'
export * from './Cooldown'
export * from './Deposit'
export * from './Intent'
export * from './Lockup'
export * from './RewardBoost'
export * from './Settings'

import { Admin } from './Admin'
import { Asset } from './Asset'
import { Cooldown } from './Cooldown'
import { Deposit } from './Deposit'
import { Intent } from './Intent'
import { Lockup } from './Lockup'
import { RewardBoost } from './RewardBoost'
import { Settings } from './Settings'

export const accountProviders = {
  Admin,
  Asset,
  Cooldown,
  Deposit,
  Intent,
  Lockup,
  RewardBoost,
  Settings,
}
