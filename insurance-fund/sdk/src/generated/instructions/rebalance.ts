/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as splToken from '@solana/spl-token'
import * as beet from '@metaplex-foundation/beet'
import * as web3 from '@solana/web3.js'
import { RebalanceArgs, rebalanceArgsBeet } from '../types/RebalanceArgs'

/**
 * @category Instructions
 * @category Rebalance
 * @category generated
 */
export type RebalanceInstructionArgs = {
  args: RebalanceArgs
}
/**
 * @category Instructions
 * @category Rebalance
 * @category generated
 */
export const rebalanceStruct = new beet.BeetArgsStruct<
  RebalanceInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['args', rebalanceArgsBeet],
  ],
  'RebalanceInstructionArgs'
)
/**
 * Accounts required by the _rebalance_ instruction
 *
 * @property [_writable_, **signer**] signer
 * @property [_writable_] admin
 * @property [_writable_] settings
 * @property [_writable_] lockup
 * @property [_writable_] assetMint
 * @property [_writable_] lockupHotVault
 * @property [] coldWallet
 * @property [_writable_] lockupColdVault
 * @category Instructions
 * @category Rebalance
 * @category generated
 */
export type RebalanceInstructionAccounts = {
  signer: web3.PublicKey
  admin: web3.PublicKey
  settings: web3.PublicKey
  lockup: web3.PublicKey
  assetMint: web3.PublicKey
  lockupHotVault: web3.PublicKey
  coldWallet: web3.PublicKey
  lockupColdVault: web3.PublicKey
  tokenProgram?: web3.PublicKey
  anchorRemainingAccounts?: web3.AccountMeta[]
}

export const rebalanceInstructionDiscriminator = [
  108, 158, 77, 9, 210, 52, 88, 62,
]

/**
 * Creates a _Rebalance_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category Rebalance
 * @category generated
 */
export function createRebalanceInstruction(
  accounts: RebalanceInstructionAccounts,
  args: RebalanceInstructionArgs,
  programId = new web3.PublicKey('rhLMe6vyM1wVLJaxrWUckVmPxSia58nSWZRDtYQow6D')
) {
  const [data] = rebalanceStruct.serialize({
    instructionDiscriminator: rebalanceInstructionDiscriminator,
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
      pubkey: accounts.lockupHotVault,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.coldWallet,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.lockupColdVault,
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
