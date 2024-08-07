/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */

import * as splToken from '@solana/spl-token'
import * as beet from '@metaplex-foundation/beet'
import * as web3 from '@solana/web3.js'

/**
 * @category Instructions
 * @category Lockup
 * @category generated
 */
export type LockupInstructionArgs = {
  receiptAmount: beet.bignum
}
/**
 * @category Instructions
 * @category Lockup
 * @category generated
 */
export const lockupStruct = new beet.BeetArgsStruct<
  LockupInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['receiptAmount', beet.u64],
  ],
  'LockupInstructionArgs'
)
/**
 * Accounts required by the _lockup_ instruction
 *
 * @property [_writable_, **signer**] user
 * @property [_writable_] userAccount
 * @property [_writable_] vault
 * @property [_writable_] lockup
 * @property [_writable_] userReceiptTokenAccount
 * @property [_writable_] lockupReceiptTokenAccount
 * @property [] clock
 * @category Instructions
 * @category Lockup
 * @category generated
 */
export type LockupInstructionAccounts = {
  user: web3.PublicKey
  userAccount: web3.PublicKey
  vault: web3.PublicKey
  lockup: web3.PublicKey
  userReceiptTokenAccount: web3.PublicKey
  lockupReceiptTokenAccount: web3.PublicKey
  tokenProgram?: web3.PublicKey
  systemProgram?: web3.PublicKey
  clock: web3.PublicKey
  anchorRemainingAccounts?: web3.AccountMeta[]
}

export const lockupInstructionDiscriminator = [
  158, 73, 228, 89, 157, 8, 70, 144,
]

/**
 * Creates a _Lockup_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category Lockup
 * @category generated
 */
export function createLockupInstruction(
  accounts: LockupInstructionAccounts,
  args: LockupInstructionArgs,
  programId = new web3.PublicKey('6ZZ1sxKGuXUBL8HSsHqHaYCg92G9VhMNTcJv1gFURCop')
) {
  const [data] = lockupStruct.serialize({
    instructionDiscriminator: lockupInstructionDiscriminator,
    ...args,
  })
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.user,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: accounts.userAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.vault,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.lockup,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.userReceiptTokenAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.lockupReceiptTokenAccount,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.tokenProgram ?? splToken.TOKEN_PROGRAM_ID,
      isWritable: false,
      isSigner: false,
    },
    {
      pubkey: accounts.systemProgram ?? web3.SystemProgram.programId,
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
