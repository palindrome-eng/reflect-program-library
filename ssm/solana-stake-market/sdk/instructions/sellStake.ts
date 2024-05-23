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
 * @category SellStake
 * @category generated
 */
export type SellStakeInstructionArgs = {
  totalStakeAmount: beet.bignum
}
/**
 * @category Instructions
 * @category SellStake
 * @category generated
 */
export const sellStakeStruct = new beet.BeetArgsStruct<
  SellStakeInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['totalStakeAmount', beet.u64],
  ],
  'SellStakeInstructionArgs'
)
/**
 * Accounts required by the _sellStake_ instruction
 *
 * @property [_writable_] stakeAccount
 * @property [_writable_] orderBook
 * @property [_writable_, **signer**] seller
 * @property [] stakeProgram
 * @property [] rentSysvar
 * @property [] clock
 * @category Instructions
 * @category SellStake
 * @category generated
 */
export type SellStakeInstructionAccounts = {
  stakeAccount: web3.PublicKey
  orderBook: web3.PublicKey
  seller: web3.PublicKey
  stakeProgram: web3.PublicKey
  systemProgram?: web3.PublicKey
  rentSysvar: web3.PublicKey
  clock: web3.PublicKey
  anchorRemainingAccounts?: web3.AccountMeta[]
}

export const sellStakeInstructionDiscriminator = [
  66, 94, 187, 190, 121, 224, 235, 163,
]

/**
 * Creates a _SellStake_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category SellStake
 * @category generated
 */
export function createSellStakeInstruction(
  accounts: SellStakeInstructionAccounts,
  args: SellStakeInstructionArgs,
  programId = new web3.PublicKey('sSmYaKe6tj5VKjPzHhpakpamw1PYoJFLQNyMJD3PU37')
) {
  const [data] = sellStakeStruct.serialize({
    instructionDiscriminator: sellStakeInstructionDiscriminator,
    ...args,
  })
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.stakeAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.orderBook,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.seller,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: accounts.stakeProgram,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.systemProgram ?? web3.SystemProgram.programId,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.rentSysvar,
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
