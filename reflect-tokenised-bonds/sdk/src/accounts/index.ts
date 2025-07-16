export * from './Admin'
export * from './Config'
export * from './Vault'

import { Admin } from './Admin'
import { Config } from './Config'
import { Vault } from './Vault'

export const accountProviders = { Admin, Config, Vault }
