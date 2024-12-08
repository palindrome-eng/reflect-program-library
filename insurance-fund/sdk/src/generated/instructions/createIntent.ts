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
  CreateIntentArgs,
  createIntentArgsBeet,
} from '../types/CreateIntentArgs'

/**
 * @category Instructions
 * @category CreateIntent
 * @category generated
 */
export type CreateIntentInstructionArgs = {
  args: CreateIntentArgs
}
/**
 * @category Instructions
 * @category CreateIntent
 * @category generated
 */
export const createIntentStruct = new beet.BeetArgsStruct<
  CreateIntentInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['args', createIntentArgsBeet],
  ],
  'CreateIntentInstructionArgs'
)
/**
 * Accounts required by the _createIntent_ instruction
 *
 * @property [_writable_, **signer**] user
 * @property [_writable_] settings
 * @property [_writable_] lockup
 * @property [_writable_] asset
 * @property [_writable_] assetMint
 * @property [_writable_] userAssetAta
 * @property [_writable_] lockupAssetVault
 * @property [_writable_] deposit
 * @property [_writable_] intent
 * @property [] clock
 * @category Instructions
 * @category CreateIntent
 * @category generated
 */
export type CreateIntentInstructionAccounts = {
  user: web3.PublicKey
  settings: web3.PublicKey
  lockup: web3.PublicKey
  asset: web3.PublicKey
  assetMint: web3.PublicKey
  userAssetAta: web3.PublicKey
  lockupAssetVault: web3.PublicKey
  deposit: web3.PublicKey
  intent: web3.PublicKey
  systemProgram?: web3.PublicKey
  tokenProgram?: web3.PublicKey
  clock: web3.PublicKey
  anchorRemainingAccounts?: web3.AccountMeta[]
}

export const createIntentInstructionDiscriminator = [
  216, 214, 79, 121, 23, 194, 96, 104,
]

/**
 * Creates a _CreateIntent_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category CreateIntent
 * @category generated
 */
export function createCreateIntentInstruction(
  accounts: CreateIntentInstructionAccounts,
  args: CreateIntentInstructionArgs,
  programId = new web3.PublicKey('EiMoMLXBCKpxTdBwK2mBBaGFWH1v2JdT5nAhiyJdF3pV')
) {
  const [data] = createIntentStruct.serialize({
    instructionDiscriminator: createIntentInstructionDiscriminator,
    ...args,
  })
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.user,
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
      pubkey: accounts.asset,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.assetMint,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.userAssetAta,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.lockupAssetVault,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.deposit,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.intent,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.systemProgram ?? web3.SystemProgram.programId,
      isWritable: false,
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
