/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
/**
 * @category Instructions
 * @category InitializeLockupVaults
 * @category generated
 */
export type InitializeLockupVaultsInstructionArgs = {
    lockupId: beet.bignum;
};
/**
 * @category Instructions
 * @category InitializeLockupVaults
 * @category generated
 */
export declare const initializeLockupVaultsStruct: beet.BeetArgsStruct<InitializeLockupVaultsInstructionArgs & {
    instructionDiscriminator: number[];
}>;
/**
 * Accounts required by the _initializeLockupVaults_ instruction
 *
 * @property [_writable_, **signer**] signer
 * @property [_writable_] admin
 * @property [_writable_] settings
 * @property [_writable_] lockup
 * @property [_writable_] assetMint
 * @property [_writable_] lockupColdVault
 * @property [_writable_] lockupHotVault
 * @property [] rewardMint
 * @property [_writable_] assetRewardPool
 * @category Instructions
 * @category InitializeLockupVaults
 * @category generated
 */
export type InitializeLockupVaultsInstructionAccounts = {
    signer: web3.PublicKey;
    admin: web3.PublicKey;
    settings: web3.PublicKey;
    lockup: web3.PublicKey;
    assetMint: web3.PublicKey;
    lockupColdVault: web3.PublicKey;
    lockupHotVault: web3.PublicKey;
    rewardMint: web3.PublicKey;
    assetRewardPool: web3.PublicKey;
    tokenProgram?: web3.PublicKey;
    systemProgram?: web3.PublicKey;
    anchorRemainingAccounts?: web3.AccountMeta[];
};
export declare const initializeLockupVaultsInstructionDiscriminator: number[];
/**
 * Creates a _InitializeLockupVaults_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category InitializeLockupVaults
 * @category generated
 */
export declare function createInitializeLockupVaultsInstruction(accounts: InitializeLockupVaultsInstructionAccounts, args: InitializeLockupVaultsInstructionArgs, programId?: web3.PublicKey): web3.TransactionInstruction;
