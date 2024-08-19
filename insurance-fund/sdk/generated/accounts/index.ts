export * from './Deposit'
export * from './Lockup'
export * from './Settings'
export * from './Slash'

import { Deposit } from './Deposit'
import { Lockup } from './Lockup'
import { Settings } from './Settings'
import { Slash } from './Slash'

export const accountProviders = { Deposit, Lockup, Settings, Slash }
