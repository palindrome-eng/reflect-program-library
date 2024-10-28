/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as splToken from '@solana/spl-token'
import * as beet from '@metaplex-foundation/beet'
import * as web3 from '@solana/web3.js'
import {
  InitializeSlashArgs,
  initializeSlashArgsBeet,
} from '../types/InitializeSlashArgs'

/**
 * @category Instructions
 * @category InitializeSlash
 * @category generated
 */
export type InitializeSlashInstructionArgs = {
  args: InitializeSlashArgs
}
/**
 * @category Instructions
 * @category InitializeSlash
 * @category generated
 */
export const initializeSlashStruct = new beet.BeetArgsStruct<
  InitializeSlashInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['args', initializeSlashArgsBeet],
  ],
  'InitializeSlashInstructionArgs'
)
/**
 * Accounts required by the _initializeSlash_ instruction
 *
 * @property [_writable_, **signer**] superadmin
 * @property [_writable_] settings
 * @property [_writable_] lockup
 * @property [_writable_] assetMint
 * @property [_writable_] assetLockup
 * @property [_writable_] slash
 * @property [] clock
 * @category Instructions
 * @category InitializeSlash
 * @category generated
 */
export type InitializeSlashInstructionAccounts = {
  superadmin: web3.PublicKey
  settings: web3.PublicKey
  lockup: web3.PublicKey
  assetMint: web3.PublicKey
  assetLockup: web3.PublicKey
  slash: web3.PublicKey
  tokenProgram?: web3.PublicKey
  clock: web3.PublicKey
  systemProgram?: web3.PublicKey
  anchorRemainingAccounts?: web3.AccountMeta[]
}

export const initializeSlashInstructionDiscriminator = [
  130, 26, 93, 222, 84, 233, 156, 7,
]

/**
 * Creates a _InitializeSlash_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category InitializeSlash
 * @category generated
 */
export function createInitializeSlashInstruction(
  accounts: InitializeSlashInstructionAccounts,
  args: InitializeSlashInstructionArgs,
  programId = new web3.PublicKey('BXopfEhtpSHLxK66tAcxY7zYEUyHL6h91NJtP2nWx54e')
) {
  const [data] = initializeSlashStruct.serialize({
    instructionDiscriminator: initializeSlashInstructionDiscriminator,
    ...args,
  })
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.superadmin,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: accounts.settings,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.lockup,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.assetMint,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.assetLockup,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.slash,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenProgram ?? splToken.TOKEN_PROGRAM_ID,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.clock,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.systemProgram ?? web3.SystemProgram.programId,
      isWritable: false,
      isSigner: false,
    },
  ]

  if (accounts.anchorRemainingAccounts != null) {
    for (const acc of accounts.anchorRemainingAccounts) {
      keys.push(acc)
    }
  }

  const ix = new web3.TransactionInstruction({
    programId,
    keys,
    data,
  })
  return ix
}
