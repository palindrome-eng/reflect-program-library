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
  InitializeLpTokenAccountArgs,
  initializeLpTokenAccountArgsBeet,
} from '../types/InitializeLpTokenAccountArgs'

/**
 * @category Instructions
 * @category InitializeLpTokenAccount
 * @category generated
 */
export type InitializeLpTokenAccountInstructionArgs = {
  args: InitializeLpTokenAccountArgs
}
/**
 * @category Instructions
 * @category InitializeLpTokenAccount
 * @category generated
 */
export const initializeLpTokenAccountStruct = new beet.BeetArgsStruct<
  InitializeLpTokenAccountInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['args', initializeLpTokenAccountArgsBeet],
  ],
  'InitializeLpTokenAccountInstructionArgs'
)
/**
 * Accounts required by the _initializeLpTokenAccount_ instruction
 *
 * @property [_writable_, **signer**] signer
 * @property [_writable_] admin
 * @property [_writable_] settings
 * @property [] liquidityPool
 * @property [] asset
 * @property [] mint
 * @property [_writable_] lpMintTokenAccount
 * @property [] associatedTokenProgram
 * @category Instructions
 * @category InitializeLpTokenAccount
 * @category generated
 */
export type InitializeLpTokenAccountInstructionAccounts = {
  signer: web3.PublicKey
  admin: web3.PublicKey
  settings: web3.PublicKey
  liquidityPool: web3.PublicKey
  asset: web3.PublicKey
  mint: web3.PublicKey
  lpMintTokenAccount: web3.PublicKey
  systemProgram?: web3.PublicKey
  tokenProgram?: web3.PublicKey
  associatedTokenProgram: web3.PublicKey
  anchorRemainingAccounts?: web3.AccountMeta[]
}

export const initializeLpTokenAccountInstructionDiscriminator = [
  209, 159, 81, 216, 119, 73, 6, 149,
]

/**
 * Creates a _InitializeLpTokenAccount_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category InitializeLpTokenAccount
 * @category generated
 */
export function createInitializeLpTokenAccountInstruction(
  accounts: InitializeLpTokenAccountInstructionAccounts,
  args: InitializeLpTokenAccountInstructionArgs,
  programId = new web3.PublicKey('rhLMe6vyM1wVLJaxrWUckVmPxSia58nSWZRDtYQow6D')
) {
  const [data] = initializeLpTokenAccountStruct.serialize({
    instructionDiscriminator: initializeLpTokenAccountInstructionDiscriminator,
    ...args,
  })
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.signer,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: accounts.admin,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.settings,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.liquidityPool,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.asset,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.mint,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.lpMintTokenAccount,
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
      pubkey: accounts.associatedTokenProgram,
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
