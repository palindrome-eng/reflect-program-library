export * from './Asset'
export * from './Cooldown'
export * from './LiquidityPool'
export * from './Settings'
export * from './UserPermissions'

import { Asset } from './Asset'
import { Cooldown } from './Cooldown'
import { LiquidityPool } from './LiquidityPool'
import { UserPermissions } from './UserPermissions'
import { Settings } from './Settings'

export const accountProviders = {
  Asset,
  Cooldown,
  LiquidityPool,
  UserPermissions,
  Settings,
}
