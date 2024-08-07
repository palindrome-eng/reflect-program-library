/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as beet from '@metaplex-foundation/beet'
import * as web3 from '@solana/web3.js'

/**
 * @category Instructions
 * @category CloseBid
 * @category generated
 */
export type CloseBidInstructionArgs = {
  bidIndex: beet.bignum
}
/**
 * @category Instructions
 * @category CloseBid
 * @category generated
 */
export const closeBidStruct = new beet.BeetArgsStruct<
  CloseBidInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['bidIndex', beet.u64],
  ],
  'CloseBidInstructionArgs'
)
/**
 * Accounts required by the _closeBid_ instruction
 *
 * @property [_writable_, **signer**] user
 * @property [_writable_] bid
 * @property [_writable_] bidVault
 * @property [_writable_] orderBook
 * @category Instructions
 * @category CloseBid
 * @category generated
 */
export type CloseBidInstructionAccounts = {
  user: web3.PublicKey
  bid: web3.PublicKey
  bidVault: web3.PublicKey
  orderBook: web3.PublicKey
  systemProgram?: web3.PublicKey
  anchorRemainingAccounts?: web3.AccountMeta[]
}

export const closeBidInstructionDiscriminator = [
  169, 171, 66, 115, 220, 168, 231, 21,
]

/**
 * Creates a _CloseBid_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category CloseBid
 * @category generated
 */
export function createCloseBidInstruction(
  accounts: CloseBidInstructionAccounts,
  args: CloseBidInstructionArgs,
  programId = new web3.PublicKey('sSmYaKe6tj5VKjPzHhpakpamw1PYoJFLQNyMJD3PU37')
) {
  const [data] = closeBidStruct.serialize({
    instructionDiscriminator: closeBidInstructionDiscriminator,
    ...args,
  })
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.user,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: accounts.bid,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.bidVault,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.orderBook,
      isWritable: true,
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
