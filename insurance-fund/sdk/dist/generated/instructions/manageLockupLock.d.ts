/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import { ManageLockupLockArgs } from '../types/ManageLockupLockArgs';
/**
 * @category Instructions
 * @category ManageLockupLock
 * @category generated
 */
export type ManageLockupLockInstructionArgs = {
    args: ManageLockupLockArgs;
};
/**
 * @category Instructions
 * @category ManageLockupLock
 * @category generated
 */
export declare const manageLockupLockStruct: beet.BeetArgsStruct<ManageLockupLockInstructionArgs & {
    instructionDiscriminator: number[];
}>;
/**
 * Accounts required by the _manageLockupLock_ instruction
 *
 * @property [_writable_, **signer**] signer
 * @property [_writable_] admin
 * @property [_writable_] settings
 * @property [_writable_] lockup
 * @category Instructions
 * @category ManageLockupLock
 * @category generated
 */
export type ManageLockupLockInstructionAccounts = {
    signer: web3.PublicKey;
    admin: web3.PublicKey;
    settings: web3.PublicKey;
    lockup: web3.PublicKey;
    anchorRemainingAccounts?: web3.AccountMeta[];
};
export declare const manageLockupLockInstructionDiscriminator: number[];
/**
 * Creates a _ManageLockupLock_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category ManageLockupLock
 * @category generated
 */
export declare function createManageLockupLockInstruction(accounts: ManageLockupLockInstructionAccounts, args: ManageLockupLockInstructionArgs, programId?: web3.PublicKey): web3.TransactionInstruction;
