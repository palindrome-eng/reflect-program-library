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
 * @category AddAsset
 * @category generated
 */
export declare const addAssetStruct: beet.BeetArgsStruct<{
    instructionDiscriminator: number[];
}>;
/**
 * Accounts required by the _addAsset_ instruction
 *
 * @property [_writable_, **signer**] signer
 * @property [_writable_] admin
 * @property [_writable_] settings
 * @property [_writable_] asset
 * @property [_writable_] assetMint
 * @property [_writable_] oracle
 * @category Instructions
 * @category AddAsset
 * @category generated
 */
export type AddAssetInstructionAccounts = {
    signer: web3.PublicKey;
    admin: web3.PublicKey;
    settings: web3.PublicKey;
    asset: web3.PublicKey;
    assetMint: web3.PublicKey;
    oracle: web3.PublicKey;
    systemProgram?: web3.PublicKey;
    anchorRemainingAccounts?: web3.AccountMeta[];
};
export declare const addAssetInstructionDiscriminator: number[];
/**
 * Creates a _AddAsset_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @category Instructions
 * @category AddAsset
 * @category generated
 */
export declare function createAddAssetInstruction(accounts: AddAssetInstructionAccounts, programId?: web3.PublicKey): web3.TransactionInstruction;
