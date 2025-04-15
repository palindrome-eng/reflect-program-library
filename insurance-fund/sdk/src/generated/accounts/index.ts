export * from './Admin'
export * from './Asset'
export * from './Cooldown'
export * from './DebtRecord'
export * from './Deposit'
export * from './Intent'
export * from './LiquidityPool'
export * from './Lockup'
export * from './LpLockup'
export * from './RewardBoost'
export * from './Settings'

import { Admin } from './Admin'
import { Asset } from './Asset'
import { Cooldown } from './Cooldown'
import { DebtRecord } from './DebtRecord'
import { Deposit } from './Deposit'
import { Intent } from './Intent'
import { Lockup } from './Lockup'
import { LiquidityPool } from './LiquidityPool'
import { LpLockup } from './LpLockup'
import { RewardBoost } from './RewardBoost'
import { Settings } from './Settings'

export const accountProviders = {
  Admin,
  Asset,
  Cooldown,
  DebtRecord,
  Deposit,
  Intent,
  Lockup,
  LiquidityPool,
  LpLockup,
  RewardBoost,
  Settings,
}
