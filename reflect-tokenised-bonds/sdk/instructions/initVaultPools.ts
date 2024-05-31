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
 * @category InitVaultPools
 * @category generated
 */
export type InitVaultPoolsInstructionArgs = {
  vaultSeed: beet.bignum
}
/**
 * @category Instructions
 * @category InitVaultPools
 * @category generated
 */
export const initVaultPoolsStruct = new beet.BeetArgsStruct<
  InitVaultPoolsInstructionArgs & {
    instructionDiscriminator: number[] /* size: 8 */
  }
>(
  [
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['vaultSeed', beet.u64],
  ],
  'InitVaultPoolsInstructionArgs'
)
/**
 * Accounts required by the _initVaultPools_ instruction
 *
 * @property [_writable_] vault
 * @property [_writable_, **signer**] admin
 * @property [_writable_] depositPool
 * @property [_writable_] rewardPool
 * @property [_writable_] depositTokenMint
 * @property [_writable_] receiptTokenMint
 * @category Instructions
 * @category InitVaultPools
 * @category generated
 */
export type InitVaultPoolsInstructionAccounts = {
  vault: web3.PublicKey
  admin: web3.PublicKey
  depositPool: web3.PublicKey
  rewardPool: web3.PublicKey
  depositTokenMint: web3.PublicKey
  receiptTokenMint: web3.PublicKey
  systemProgram?: web3.PublicKey
  tokenProgram?: web3.PublicKey
  rent?: web3.PublicKey
  anchorRemainingAccounts?: web3.AccountMeta[]
}

export const initVaultPoolsInstructionDiscriminator = [
  194, 23, 179, 243, 55, 77, 141, 166,
]

/**
 * Creates a _InitVaultPools_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category InitVaultPools
 * @category generated
 */
export function createInitVaultPoolsInstruction(
  accounts: InitVaultPoolsInstructionAccounts,
  args: InitVaultPoolsInstructionArgs,
  programId = new web3.PublicKey('6ZZ1sxKGuXUBL8HSsHqHaYCg92G9VhMNTcJv1gFURCop')
) {
  const [data] = initVaultPoolsStruct.serialize({
    instructionDiscriminator: initVaultPoolsInstructionDiscriminator,
    ...args,
  })
  const keys: web3.AccountMeta[] = [
    {
      pubkey: accounts.vault,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.admin,
      isWritable: true,
      isSigner: true,
    },
    {
      pubkey: accounts.depositPool,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.rewardPool,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.depositTokenMint,
      isWritable: true,
      isSigner: false,
    },
    {
      pubkey: accounts.receiptTokenMint,
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
      pubkey: accounts.rent ?? web3.SYSVAR_RENT_PUBKEY,
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
