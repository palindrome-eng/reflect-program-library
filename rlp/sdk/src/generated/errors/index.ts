/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

type ErrorWithCode = Error & { code: number }
type MaybeErrorWithCode = ErrorWithCode | null | undefined

const createErrorFromCodeLookup: Map<number, () => ErrorWithCode> = new Map()
const createErrorFromNameLookup: Map<string, () => ErrorWithCode> = new Map()

/**
 * InvalidSigner: 'InvalidSigner'
 *
 * @category Errors
 * @category generated
 */
export class InvalidSignerError extends Error {
  readonly code: number = 0x1770
  readonly name: string = 'InvalidSigner'
  constructor() {
    super('InvalidSigner')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, InvalidSignerError)
    }
  }
}

createErrorFromCodeLookup.set(0x1770, () => new InvalidSignerError())
createErrorFromNameLookup.set('InvalidSigner', () => new InvalidSignerError())

/**
 * InvalidInput: 'InvalidInput'
 *
 * @category Errors
 * @category generated
 */
export class InvalidInputError extends Error {
  readonly code: number = 0x1771
  readonly name: string = 'InvalidInput'
  constructor() {
    super('InvalidInput')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, InvalidInputError)
    }
  }
}

createErrorFromCodeLookup.set(0x1771, () => new InvalidInputError())
createErrorFromNameLookup.set('InvalidInput', () => new InvalidInputError())

/**
 * AssetNotWhitelisted: 'AssetNotWhitelisted'
 *
 * @category Errors
 * @category generated
 */
export class AssetNotWhitelistedError extends Error {
  readonly code: number = 0x1772
  readonly name: string = 'AssetNotWhitelisted'
  constructor() {
    super('AssetNotWhitelisted')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, AssetNotWhitelistedError)
    }
  }
}

createErrorFromCodeLookup.set(0x1772, () => new AssetNotWhitelistedError())
createErrorFromNameLookup.set(
  'AssetNotWhitelisted',
  () => new AssetNotWhitelistedError()
)

/**
 * DepositTooLow: 'DepositTooLow'
 *
 * @category Errors
 * @category generated
 */
export class DepositTooLowError extends Error {
  readonly code: number = 0x1773
  readonly name: string = 'DepositTooLow'
  constructor() {
    super('DepositTooLow')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, DepositTooLowError)
    }
  }
}

createErrorFromCodeLookup.set(0x1773, () => new DepositTooLowError())
createErrorFromNameLookup.set('DepositTooLow', () => new DepositTooLowError())

/**
 * DepositCapOverflow: 'DepositCapOverflow'
 *
 * @category Errors
 * @category generated
 */
export class DepositCapOverflowError extends Error {
  readonly code: number = 0x1774
  readonly name: string = 'DepositCapOverflow'
  constructor() {
    super('DepositCapOverflow')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, DepositCapOverflowError)
    }
  }
}

createErrorFromCodeLookup.set(0x1774, () => new DepositCapOverflowError())
createErrorFromNameLookup.set(
  'DepositCapOverflow',
  () => new DepositCapOverflowError()
)

/**
 * NotEnoughFunds: 'NotEnoughFunds'
 *
 * @category Errors
 * @category generated
 */
export class NotEnoughFundsError extends Error {
  readonly code: number = 0x1775
  readonly name: string = 'NotEnoughFunds'
  constructor() {
    super('NotEnoughFunds')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, NotEnoughFundsError)
    }
  }
}

createErrorFromCodeLookup.set(0x1775, () => new NotEnoughFundsError())
createErrorFromNameLookup.set('NotEnoughFunds', () => new NotEnoughFundsError())

/**
 * NotEnoughReceiptTokens: 'NotEnoughReceiptTokens'
 *
 * @category Errors
 * @category generated
 */
export class NotEnoughReceiptTokensError extends Error {
  readonly code: number = 0x1776
  readonly name: string = 'NotEnoughReceiptTokens'
  constructor() {
    super('NotEnoughReceiptTokens')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, NotEnoughReceiptTokensError)
    }
  }
}

createErrorFromCodeLookup.set(0x1776, () => new NotEnoughReceiptTokensError())
createErrorFromNameLookup.set(
  'NotEnoughReceiptTokens',
  () => new NotEnoughReceiptTokensError()
)

/**
 * NotEnoughFundsToSlash: 'NotEnoughFundsToSlash'
 *
 * @category Errors
 * @category generated
 */
export class NotEnoughFundsToSlashError extends Error {
  readonly code: number = 0x1777
  readonly name: string = 'NotEnoughFundsToSlash'
  constructor() {
    super('NotEnoughFundsToSlash')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, NotEnoughFundsToSlashError)
    }
  }
}

createErrorFromCodeLookup.set(0x1777, () => new NotEnoughFundsToSlashError())
createErrorFromNameLookup.set(
  'NotEnoughFundsToSlash',
  () => new NotEnoughFundsToSlashError()
)

/**
 * DepositsLocked: 'DepositsLocked'
 *
 * @category Errors
 * @category generated
 */
export class DepositsLockedError extends Error {
  readonly code: number = 0x1778
  readonly name: string = 'DepositsLocked'
  constructor() {
    super('DepositsLocked')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, DepositsLockedError)
    }
  }
}

createErrorFromCodeLookup.set(0x1778, () => new DepositsLockedError())
createErrorFromNameLookup.set('DepositsLocked', () => new DepositsLockedError())

/**
 * DepositsOpen: 'DepositsOpen'
 *
 * @category Errors
 * @category generated
 */
export class DepositsOpenError extends Error {
  readonly code: number = 0x1779
  readonly name: string = 'DepositsOpen'
  constructor() {
    super('DepositsOpen')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, DepositsOpenError)
    }
  }
}

createErrorFromCodeLookup.set(0x1779, () => new DepositsOpenError())
createErrorFromNameLookup.set('DepositsOpen', () => new DepositsOpenError())

/**
 * DepositsNotSlashed: 'DepositsNotSlashed'
 *
 * @category Errors
 * @category generated
 */
export class DepositsNotSlashedError extends Error {
  readonly code: number = 0x177a
  readonly name: string = 'DepositsNotSlashed'
  constructor() {
    super('DepositsNotSlashed')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, DepositsNotSlashedError)
    }
  }
}

createErrorFromCodeLookup.set(0x177a, () => new DepositsNotSlashedError())
createErrorFromNameLookup.set(
  'DepositsNotSlashed',
  () => new DepositsNotSlashedError()
)

/**
 * AllDepositsSlashed: 'AllDepositsSlashed'
 *
 * @category Errors
 * @category generated
 */
export class AllDepositsSlashedError extends Error {
  readonly code: number = 0x177b
  readonly name: string = 'AllDepositsSlashed'
  constructor() {
    super('AllDepositsSlashed')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, AllDepositsSlashedError)
    }
  }
}

createErrorFromCodeLookup.set(0x177b, () => new AllDepositsSlashedError())
createErrorFromNameLookup.set(
  'AllDepositsSlashed',
  () => new AllDepositsSlashedError()
)

/**
 * SlashAmountMismatch: 'SlashAmountMismatch'
 *
 * @category Errors
 * @category generated
 */
export class SlashAmountMismatchError extends Error {
  readonly code: number = 0x177c
  readonly name: string = 'SlashAmountMismatch'
  constructor() {
    super('SlashAmountMismatch')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, SlashAmountMismatchError)
    }
  }
}

createErrorFromCodeLookup.set(0x177c, () => new SlashAmountMismatchError())
createErrorFromNameLookup.set(
  'SlashAmountMismatch',
  () => new SlashAmountMismatchError()
)

/**
 * ShareConfigOverflow: 'ShareConfigOverflow'
 *
 * @category Errors
 * @category generated
 */
export class ShareConfigOverflowError extends Error {
  readonly code: number = 0x177d
  readonly name: string = 'ShareConfigOverflow'
  constructor() {
    super('ShareConfigOverflow')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, ShareConfigOverflowError)
    }
  }
}

createErrorFromCodeLookup.set(0x177d, () => new ShareConfigOverflowError())
createErrorFromNameLookup.set(
  'ShareConfigOverflow',
  () => new ShareConfigOverflowError()
)

/**
 * Frozen: 'Frozen'
 *
 * @category Errors
 * @category generated
 */
export class FrozenError extends Error {
  readonly code: number = 0x177e
  readonly name: string = 'Frozen'
  constructor() {
    super('Frozen')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, FrozenError)
    }
  }
}

createErrorFromCodeLookup.set(0x177e, () => new FrozenError())
createErrorFromNameLookup.set('Frozen', () => new FrozenError())

/**
 * InvalidOracle: 'InvalidOracle'
 *
 * @category Errors
 * @category generated
 */
export class InvalidOracleError extends Error {
  readonly code: number = 0x177f
  readonly name: string = 'InvalidOracle'
  constructor() {
    super('InvalidOracle')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, InvalidOracleError)
    }
  }
}

createErrorFromCodeLookup.set(0x177f, () => new InvalidOracleError())
createErrorFromNameLookup.set('InvalidOracle', () => new InvalidOracleError())

/**
 * MathOverflow: 'MathOverflow'
 *
 * @category Errors
 * @category generated
 */
export class MathOverflowError extends Error {
  readonly code: number = 0x1780
  readonly name: string = 'MathOverflow'
  constructor() {
    super('MathOverflow')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, MathOverflowError)
    }
  }
}

createErrorFromCodeLookup.set(0x1780, () => new MathOverflowError())
createErrorFromNameLookup.set('MathOverflow', () => new MathOverflowError())

/**
 * LockupInForce: 'LockupInForce'
 *
 * @category Errors
 * @category generated
 */
export class LockupInForceError extends Error {
  readonly code: number = 0x1781
  readonly name: string = 'LockupInForce'
  constructor() {
    super('LockupInForce')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, LockupInForceError)
    }
  }
}

createErrorFromCodeLookup.set(0x1781, () => new LockupInForceError())
createErrorFromNameLookup.set('LockupInForce', () => new LockupInForceError())

/**
 * BoostNotApplied: 'BoostNotApplied'
 *
 * @category Errors
 * @category generated
 */
export class BoostNotAppliedError extends Error {
  readonly code: number = 0x1782
  readonly name: string = 'BoostNotApplied'
  constructor() {
    super('BoostNotApplied')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, BoostNotAppliedError)
    }
  }
}

createErrorFromCodeLookup.set(0x1782, () => new BoostNotAppliedError())
createErrorFromNameLookup.set(
  'BoostNotApplied',
  () => new BoostNotAppliedError()
)

/**
 * InvalidSigners: 'InvalidSigners'
 *
 * @category Errors
 * @category generated
 */
export class InvalidSignersError extends Error {
  readonly code: number = 0x1783
  readonly name: string = 'InvalidSigners'
  constructor() {
    super('InvalidSigners')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, InvalidSignersError)
    }
  }
}

createErrorFromCodeLookup.set(0x1783, () => new InvalidSignersError())
createErrorFromNameLookup.set('InvalidSigners', () => new InvalidSignersError())

/**
 * TransferSignatureRequired: 'TransferSignatureRequired'
 *
 * @category Errors
 * @category generated
 */
export class TransferSignatureRequiredError extends Error {
  readonly code: number = 0x1784
  readonly name: string = 'TransferSignatureRequired'
  constructor() {
    super('TransferSignatureRequired')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, TransferSignatureRequiredError)
    }
  }
}

createErrorFromCodeLookup.set(
  0x1784,
  () => new TransferSignatureRequiredError()
)
createErrorFromNameLookup.set(
  'TransferSignatureRequired',
  () => new TransferSignatureRequiredError()
)

/**
 * ColdWalletNotSlashed: 'ColdWalletNotSlashed'
 *
 * @category Errors
 * @category generated
 */
export class ColdWalletNotSlashedError extends Error {
  readonly code: number = 0x1785
  readonly name: string = 'ColdWalletNotSlashed'
  constructor() {
    super('ColdWalletNotSlashed')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, ColdWalletNotSlashedError)
    }
  }
}

createErrorFromCodeLookup.set(0x1785, () => new ColdWalletNotSlashedError())
createErrorFromNameLookup.set(
  'ColdWalletNotSlashed',
  () => new ColdWalletNotSlashedError()
)

/**
 * PermissionsTooLow: 'PermissionsTooLow'
 *
 * @category Errors
 * @category generated
 */
export class PermissionsTooLowError extends Error {
  readonly code: number = 0x1786
  readonly name: string = 'PermissionsTooLow'
  constructor() {
    super('PermissionsTooLow')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, PermissionsTooLowError)
    }
  }
}

createErrorFromCodeLookup.set(0x1786, () => new PermissionsTooLowError())
createErrorFromNameLookup.set(
  'PermissionsTooLow',
  () => new PermissionsTooLowError()
)

/**
 * WithdrawalThresholdOverflow: 'WithdrawalThresholdOverflow'
 *
 * @category Errors
 * @category generated
 */
export class WithdrawalThresholdOverflowError extends Error {
  readonly code: number = 0x1787
  readonly name: string = 'WithdrawalThresholdOverflow'
  constructor() {
    super('WithdrawalThresholdOverflow')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, WithdrawalThresholdOverflowError)
    }
  }
}

createErrorFromCodeLookup.set(
  0x1787,
  () => new WithdrawalThresholdOverflowError()
)
createErrorFromNameLookup.set(
  'WithdrawalThresholdOverflow',
  () => new WithdrawalThresholdOverflowError()
)

/**
 * PoolImbalance: 'PoolImbalance'
 *
 * @category Errors
 * @category generated
 */
export class PoolImbalanceError extends Error {
  readonly code: number = 0x1788
  readonly name: string = 'PoolImbalance'
  constructor() {
    super('PoolImbalance')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, PoolImbalanceError)
    }
  }
}

createErrorFromCodeLookup.set(0x1788, () => new PoolImbalanceError())
createErrorFromNameLookup.set('PoolImbalance', () => new PoolImbalanceError())

/**
 * InvalidReceiptTokenSetup: 'InvalidReceiptTokenSetup'
 *
 * @category Errors
 * @category generated
 */
export class InvalidReceiptTokenSetupError extends Error {
  readonly code: number = 0x1789
  readonly name: string = 'InvalidReceiptTokenSetup'
  constructor() {
    super('InvalidReceiptTokenSetup')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, InvalidReceiptTokenSetupError)
    }
  }
}

createErrorFromCodeLookup.set(0x1789, () => new InvalidReceiptTokenSetupError())
createErrorFromNameLookup.set(
  'InvalidReceiptTokenSetup',
  () => new InvalidReceiptTokenSetupError()
)

/**
 * InvalidReceiptTokenDecimals: 'InvalidReceiptTokenDecimals'
 *
 * @category Errors
 * @category generated
 */
export class InvalidReceiptTokenDecimalsError extends Error {
  readonly code: number = 0x178a
  readonly name: string = 'InvalidReceiptTokenDecimals'
  constructor() {
    super('InvalidReceiptTokenDecimals')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, InvalidReceiptTokenDecimalsError)
    }
  }
}

createErrorFromCodeLookup.set(
  0x178a,
  () => new InvalidReceiptTokenDecimalsError()
)
createErrorFromNameLookup.set(
  'InvalidReceiptTokenDecimals',
  () => new InvalidReceiptTokenDecimalsError()
)

/**
 * InvalidReceiptTokenMintAuthority: 'InvalidReceiptTokenMintAuthority'
 *
 * @category Errors
 * @category generated
 */
export class InvalidReceiptTokenMintAuthorityError extends Error {
  readonly code: number = 0x178b
  readonly name: string = 'InvalidReceiptTokenMintAuthority'
  constructor() {
    super('InvalidReceiptTokenMintAuthority')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, InvalidReceiptTokenMintAuthorityError)
    }
  }
}

createErrorFromCodeLookup.set(
  0x178b,
  () => new InvalidReceiptTokenMintAuthorityError()
)
createErrorFromNameLookup.set(
  'InvalidReceiptTokenMintAuthority',
  () => new InvalidReceiptTokenMintAuthorityError()
)

/**
 * InvalidReceiptTokenSupply: 'InvalidReceiptTokenSupply'
 *
 * @category Errors
 * @category generated
 */
export class InvalidReceiptTokenSupplyError extends Error {
  readonly code: number = 0x178c
  readonly name: string = 'InvalidReceiptTokenSupply'
  constructor() {
    super('InvalidReceiptTokenSupply')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, InvalidReceiptTokenSupplyError)
    }
  }
}

createErrorFromCodeLookup.set(
  0x178c,
  () => new InvalidReceiptTokenSupplyError()
)
createErrorFromNameLookup.set(
  'InvalidReceiptTokenSupply',
  () => new InvalidReceiptTokenSupplyError()
)

/**
 * InvalidReceiptTokenFreezeAuthority: 'InvalidReceiptTokenFreezeAuthority'
 *
 * @category Errors
 * @category generated
 */
export class InvalidReceiptTokenFreezeAuthorityError extends Error {
  readonly code: number = 0x178d
  readonly name: string = 'InvalidReceiptTokenFreezeAuthority'
  constructor() {
    super('InvalidReceiptTokenFreezeAuthority')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, InvalidReceiptTokenFreezeAuthorityError)
    }
  }
}

createErrorFromCodeLookup.set(
  0x178d,
  () => new InvalidReceiptTokenFreezeAuthorityError()
)
createErrorFromNameLookup.set(
  'InvalidReceiptTokenFreezeAuthority',
  () => new InvalidReceiptTokenFreezeAuthorityError()
)

/**
 * MinimumSuperadminsRequired: 'MinimumSuperadminsRequired'
 *
 * @category Errors
 * @category generated
 */
export class MinimumSuperadminsRequiredError extends Error {
  readonly code: number = 0x178e
  readonly name: string = 'MinimumSuperadminsRequired'
  constructor() {
    super('MinimumSuperadminsRequired')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, MinimumSuperadminsRequiredError)
    }
  }
}

createErrorFromCodeLookup.set(
  0x178e,
  () => new MinimumSuperadminsRequiredError()
)
createErrorFromNameLookup.set(
  'MinimumSuperadminsRequired',
  () => new MinimumSuperadminsRequiredError()
)

/**
 * IntentValueTooLow: 'IntentValueTooLow'
 *
 * @category Errors
 * @category generated
 */
export class IntentValueTooLowError extends Error {
  readonly code: number = 0x178f
  readonly name: string = 'IntentValueTooLow'
  constructor() {
    super('IntentValueTooLow')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, IntentValueTooLowError)
    }
  }
}

createErrorFromCodeLookup.set(0x178f, () => new IntentValueTooLowError())
createErrorFromNameLookup.set(
  'IntentValueTooLow',
  () => new IntentValueTooLowError()
)

/**
 * WithdrawalNeedsIntent: 'WithdrawalNeedsIntent'
 *
 * @category Errors
 * @category generated
 */
export class WithdrawalNeedsIntentError extends Error {
  readonly code: number = 0x1790
  readonly name: string = 'WithdrawalNeedsIntent'
  constructor() {
    super('WithdrawalNeedsIntent')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, WithdrawalNeedsIntentError)
    }
  }
}

createErrorFromCodeLookup.set(0x1790, () => new WithdrawalNeedsIntentError())
createErrorFromNameLookup.set(
  'WithdrawalNeedsIntent',
  () => new WithdrawalNeedsIntentError()
)

/**
 * PriceError: 'PriceError'
 *
 * @category Errors
 * @category generated
 */
export class PriceErrorError extends Error {
  readonly code: number = 0x1791
  readonly name: string = 'PriceError'
  constructor() {
    super('PriceError')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, PriceErrorError)
    }
  }
}

createErrorFromCodeLookup.set(0x1791, () => new PriceErrorError())
createErrorFromNameLookup.set('PriceError', () => new PriceErrorError())

/**
 * CooldownInForce: 'CooldownInForce'
 *
 * @category Errors
 * @category generated
 */
export class CooldownInForceError extends Error {
  readonly code: number = 0x1792
  readonly name: string = 'CooldownInForce'
  constructor() {
    super('CooldownInForce')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, CooldownInForceError)
    }
  }
}

createErrorFromCodeLookup.set(0x1792, () => new CooldownInForceError())
createErrorFromNameLookup.set(
  'CooldownInForce',
  () => new CooldownInForceError()
)

/**
 * SlippageExceeded: 'SlippageExceeded'
 *
 * @category Errors
 * @category generated
 */
export class SlippageExceededError extends Error {
  readonly code: number = 0x1793
  readonly name: string = 'SlippageExceeded'
  constructor() {
    super('SlippageExceeded')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, SlippageExceededError)
    }
  }
}

createErrorFromCodeLookup.set(0x1793, () => new SlippageExceededError())
createErrorFromNameLookup.set(
  'SlippageExceeded',
  () => new SlippageExceededError()
)

/**
 * InvalidTokenOrder: 'InvalidTokenOrder'
 *
 * @category Errors
 * @category generated
 */
export class InvalidTokenOrderError extends Error {
  readonly code: number = 0x1794
  readonly name: string = 'InvalidTokenOrder'
  constructor() {
    super('InvalidTokenOrder')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, InvalidTokenOrderError)
    }
  }
}

createErrorFromCodeLookup.set(0x1794, () => new InvalidTokenOrderError())
createErrorFromNameLookup.set(
  'InvalidTokenOrder',
  () => new InvalidTokenOrderError()
)

/**
 * ActionFrozen: 'ActionFrozen'
 *
 * @category Errors
 * @category generated
 */
export class ActionFrozenError extends Error {
  readonly code: number = 0x1795
  readonly name: string = 'ActionFrozen'
  constructor() {
    super('ActionFrozen')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, ActionFrozenError)
    }
  }
}

createErrorFromCodeLookup.set(0x1795, () => new ActionFrozenError())
createErrorFromNameLookup.set('ActionFrozen', () => new ActionFrozenError())

/**
 * ActionNotFound: 'ActionNotFound'
 *
 * @category Errors
 * @category generated
 */
export class ActionNotFoundError extends Error {
  readonly code: number = 0x1796
  readonly name: string = 'ActionNotFound'
  constructor() {
    super('ActionNotFound')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, ActionNotFoundError)
    }
  }
}

createErrorFromCodeLookup.set(0x1796, () => new ActionNotFoundError())
createErrorFromNameLookup.set('ActionNotFound', () => new ActionNotFoundError())

/**
 * NoEntriesLeft: 'NoEntriesLeft'
 *
 * @category Errors
 * @category generated
 */
export class NoEntriesLeftError extends Error {
  readonly code: number = 0x1797
  readonly name: string = 'NoEntriesLeft'
  constructor() {
    super('NoEntriesLeft')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, NoEntriesLeftError)
    }
  }
}

createErrorFromCodeLookup.set(0x1797, () => new NoEntriesLeftError())
createErrorFromNameLookup.set('NoEntriesLeft', () => new NoEntriesLeftError())

/**
 * RoleNotUnderAction: 'RoleNotUnderAction'
 *
 * @category Errors
 * @category generated
 */
export class RoleNotUnderActionError extends Error {
  readonly code: number = 0x1798
  readonly name: string = 'RoleNotUnderAction'
  constructor() {
    super('RoleNotUnderAction')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, RoleNotUnderActionError)
    }
  }
}

createErrorFromCodeLookup.set(0x1798, () => new RoleNotUnderActionError())
createErrorFromNameLookup.set(
  'RoleNotUnderAction',
  () => new RoleNotUnderActionError()
)

/**
 * ActionHasAssignedRole: 'ActionHasAssignedRole'
 *
 * @category Errors
 * @category generated
 */
export class ActionHasAssignedRoleError extends Error {
  readonly code: number = 0x1799
  readonly name: string = 'ActionHasAssignedRole'
  constructor() {
    super('ActionHasAssignedRole')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, ActionHasAssignedRoleError)
    }
  }
}

createErrorFromCodeLookup.set(0x1799, () => new ActionHasAssignedRoleError())
createErrorFromNameLookup.set(
  'ActionHasAssignedRole',
  () => new ActionHasAssignedRoleError()
)

/**
 * InvalidState: 'InvalidState'
 *
 * @category Errors
 * @category generated
 */
export class InvalidStateError extends Error {
  readonly code: number = 0x179a
  readonly name: string = 'InvalidState'
  constructor() {
    super('InvalidState')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, InvalidStateError)
    }
  }
}

createErrorFromCodeLookup.set(0x179a, () => new InvalidStateError())
createErrorFromNameLookup.set('InvalidState', () => new InvalidStateError())

/**
 * IncorrectAdmin: 'IncorrectAdmin'
 *
 * @category Errors
 * @category generated
 */
export class IncorrectAdminError extends Error {
  readonly code: number = 0x179b
  readonly name: string = 'IncorrectAdmin'
  constructor() {
    super('IncorrectAdmin')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, IncorrectAdminError)
    }
  }
}

createErrorFromCodeLookup.set(0x179b, () => new IncorrectAdminError())
createErrorFromNameLookup.set('IncorrectAdmin', () => new IncorrectAdminError())

/**
 * SameAdmin: 'SameAdmin'
 *
 * @category Errors
 * @category generated
 */
export class SameAdminError extends Error {
  readonly code: number = 0x179c
  readonly name: string = 'SameAdmin'
  constructor() {
    super('SameAdmin')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, SameAdminError)
    }
  }
}

createErrorFromCodeLookup.set(0x179c, () => new SameAdminError())
createErrorFromNameLookup.set('SameAdmin', () => new SameAdminError())

/**
 * AlreadyFrozen: 'AlreadyFrozen'
 *
 * @category Errors
 * @category generated
 */
export class AlreadyFrozenError extends Error {
  readonly code: number = 0x179d
  readonly name: string = 'AlreadyFrozen'
  constructor() {
    super('AlreadyFrozen')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, AlreadyFrozenError)
    }
  }
}

createErrorFromCodeLookup.set(0x179d, () => new AlreadyFrozenError())
createErrorFromNameLookup.set('AlreadyFrozen', () => new AlreadyFrozenError())

/**
 * AlreadyUnfrozen: 'AlreadyUnfrozen'
 *
 * @category Errors
 * @category generated
 */
export class AlreadyUnfrozenError extends Error {
  readonly code: number = 0x179e
  readonly name: string = 'AlreadyUnfrozen'
  constructor() {
    super('AlreadyUnfrozen')
    if (typeof Error.captureStackTrace === 'function') {
      Error.captureStackTrace(this, AlreadyUnfrozenError)
    }
  }
}

createErrorFromCodeLookup.set(0x179e, () => new AlreadyUnfrozenError())
createErrorFromNameLookup.set(
  'AlreadyUnfrozen',
  () => new AlreadyUnfrozenError()
)

/**
 * Attempts to resolve a custom program error from the provided error code.
 * @category Errors
 * @category generated
 */
export function errorFromCode(code: number): MaybeErrorWithCode {
  const createError = createErrorFromCodeLookup.get(code)
  return createError != null ? createError() : null
}

/**
 * Attempts to resolve a custom program error from the provided error name, i.e. 'Unauthorized'.
 * @category Errors
 * @category generated
 */
export function errorFromName(name: string): MaybeErrorWithCode {
  const createError = createErrorFromNameLookup.get(name)
  return createError != null ? createError() : null
}
