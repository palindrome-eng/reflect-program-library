/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as splToken from '@solana/spl-token'
import * as beet from '@metaplex-foundation/beet'
import * as web3 from '@solana/web3.js'
import { SwapLpArgs, swapLpArgsBeet } from '../types/SwapLpArgs'

/**
 * @category Instructions
 * @category SwapLp
 * @category generated
 */
export type SwapLpInstructionArgs = {
  args: SwapLpArgs
}
/**
 * @category Instructions
 * @category SwapLp
 * @category generated
 */
export const swapLpStruct = new beet.FixableBeetArgsStruct<
  SwapLpInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['args', swapLpArgsBeet],
  ],
  'SwapLpInstructionArgs'
)
/**
 * Accounts required by the _swapLp_ instruction
 *
 * @property [_writable_, **signer**] signer
 * @property [] admin
 * @property [] liquidityPool
 * @property [] tokenA
 * @property [] tokenAAsset
 * @property [] tokenAOracle
 * @property [] tokenB
 * @property [] tokenBAsset
 * @property [] tokenBOracle
 * @property [_writable_] tokenAPool
 * @property [_writable_] tokenBPool
 * @property [_writable_] tokenASignerAccount
 * @property [_writable_] tokenBSignerAccount
 * @property [] associatedTokenProgram
 * @category Instructions
 * @category SwapLp
 * @category generated
 */
export type SwapLpInstructionAccounts = {
  signer: web3.PublicKey
  admin: web3.PublicKey
  liquidityPool: web3.PublicKey
  tokenA: web3.PublicKey
  tokenAAsset: web3.PublicKey
  tokenAOracle: web3.PublicKey
  tokenB: web3.PublicKey
  tokenBAsset: web3.PublicKey
  tokenBOracle: web3.PublicKey
  tokenAPool: web3.PublicKey
  tokenBPool: web3.PublicKey
  tokenASignerAccount: web3.PublicKey
  tokenBSignerAccount: web3.PublicKey
  tokenProgram?: web3.PublicKey
  associatedTokenProgram: web3.PublicKey
  anchorRemainingAccounts?: web3.AccountMeta[]
}

export const swapLpInstructionDiscriminator = [
  241, 157, 54, 146, 252, 232, 95, 169,
]

/**
 * Creates a _SwapLp_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category SwapLp
 * @category generated
 */
export function createSwapLpInstruction(
  accounts: SwapLpInstructionAccounts,
  args: SwapLpInstructionArgs,
  programId = new web3.PublicKey('rhLMe6vyM1wVLJaxrWUckVmPxSia58nSWZRDtYQow6D')
) {
  const [data] = swapLpStruct.serialize({
    instructionDiscriminator: swapLpInstructionDiscriminator,
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
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.liquidityPool,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenA,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenAAsset,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenAOracle,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenB,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenBAsset,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenBOracle,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenAPool,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenBPool,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenASignerAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenBSignerAccount,
      isWritable: true,
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
