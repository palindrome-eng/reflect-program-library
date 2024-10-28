/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as splToken from '@solana/spl-token'
import * as beet from '@metaplex-foundation/beet'
import * as web3 from '@solana/web3.js'
import { SlashPoolArgs, slashPoolArgsBeet } from '../types/SlashPoolArgs'

/**
 * @category Instructions
 * @category SlashPool
 * @category generated
 */
export type SlashPoolInstructionArgs = {
  args: SlashPoolArgs
}
/**
 * @category Instructions
 * @category SlashPool
 * @category generated
 */
export const slashPoolStruct = new beet.BeetArgsStruct<
  SlashPoolInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['args', slashPoolArgsBeet],
  ],
  'SlashPoolInstructionArgs'
)
/**
 * Accounts required by the _slashPool_ instruction
 *
 * @property [_writable_, **signer**] superadmin
 * @property [_writable_] settings
 * @property [_writable_] lockup
 * @property [_writable_] slash
 * @property [_writable_] asset
 * @property [_writable_] assetLockup
 * @property [_writable_] destination
 * @category Instructions
 * @category SlashPool
 * @category generated
 */
export type SlashPoolInstructionAccounts = {
  superadmin: web3.PublicKey
  settings: web3.PublicKey
  lockup: web3.PublicKey
  slash: web3.PublicKey
  asset: web3.PublicKey
  assetLockup: web3.PublicKey
  destination: web3.PublicKey
  tokenProgram?: web3.PublicKey
  anchorRemainingAccounts?: web3.AccountMeta[]
}

export const slashPoolInstructionDiscriminator = [
  96, 203, 37, 117, 91, 235, 38, 76,
]

/**
 * Creates a _SlashPool_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category SlashPool
 * @category generated
 */
export function createSlashPoolInstruction(
  accounts: SlashPoolInstructionAccounts,
  args: SlashPoolInstructionArgs,
  programId = new web3.PublicKey('BXopfEhtpSHLxK66tAcxY7zYEUyHL6h91NJtP2nWx54e')
) {
  const [data] = slashPoolStruct.serialize({
    instructionDiscriminator: slashPoolInstructionDiscriminator,
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
      pubkey: accounts.slash,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.asset,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.assetLockup,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.destination,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenProgram ?? splToken.TOKEN_PROGRAM_ID,
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
