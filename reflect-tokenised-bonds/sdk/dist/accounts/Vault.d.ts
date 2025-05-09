/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as beetSolana from '@metaplex-foundation/beet-solana';
/**
 * Arguments used to create {@link Vault}
 * @category Accounts
 * @category generated
 */
export type VaultArgs = {
    admin: web3.PublicKey;
    depositTokenMint: web3.PublicKey;
    receiptTokenMint: web3.PublicKey;
    minDeposit: beet.bignum;
    minLockup: beet.bignum;
    targetYieldRate: beet.bignum;
    depositPool: web3.PublicKey;
    rewardPool: web3.PublicKey;
    totalReceiptSupply: beet.bignum;
};
export declare const vaultDiscriminator: number[];
/**
 * Holds the data for the {@link Vault} Account and provides de/serialization
 * functionality for that data
 *
 * @category Accounts
 * @category generated
 */
export declare class Vault implements VaultArgs {
    readonly admin: web3.PublicKey;
    readonly depositTokenMint: web3.PublicKey;
    readonly receiptTokenMint: web3.PublicKey;
    readonly minDeposit: beet.bignum;
    readonly minLockup: beet.bignum;
    readonly targetYieldRate: beet.bignum;
    readonly depositPool: web3.PublicKey;
    readonly rewardPool: web3.PublicKey;
    readonly totalReceiptSupply: beet.bignum;
    private constructor();
    /**
     * Creates a {@link Vault} instance from the provided args.
     */
    static fromArgs(args: VaultArgs): Vault;
    /**
     * Deserializes the {@link Vault} from the data of the provided {@link web3.AccountInfo}.
     * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
     */
    static fromAccountInfo(accountInfo: web3.AccountInfo<Buffer>, offset?: number): [Vault, number];
    /**
     * Retrieves the account info from the provided address and deserializes
     * the {@link Vault} from its data.
     *
     * @throws Error if no account info is found at the address or if deserialization fails
     */
    static fromAccountAddress(connection: web3.Connection, address: web3.PublicKey, commitmentOrConfig?: web3.Commitment | web3.GetAccountInfoConfig): Promise<Vault>;
    /**
     * Provides a {@link web3.Connection.getProgramAccounts} config builder,
     * to fetch accounts matching filters that can be specified via that builder.
     *
     * @param programId - the program that owns the accounts we are filtering
     */
    static gpaBuilder(programId?: web3.PublicKey): beetSolana.GpaBuilder<{
        accountDiscriminator: any;
        admin: any;
        depositTokenMint: any;
        receiptTokenMint: any;
        minDeposit: any;
        minLockup: any;
        targetYieldRate: any;
        depositPool: any;
        rewardPool: any;
        totalReceiptSupply: any;
    }>;
    /**
     * Deserializes the {@link Vault} from the provided data Buffer.
     * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
     */
    static deserialize(buf: Buffer, offset?: number): [Vault, number];
    /**
     * Serializes the {@link Vault} into a Buffer.
     * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
     */
    serialize(): [Buffer, number];
    /**
     * Returns the byteSize of a {@link Buffer} holding the serialized data of
     * {@link Vault}
     */
    static get byteSize(): number;
    /**
     * Fetches the minimum balance needed to exempt an account holding
     * {@link Vault} data from rent
     *
     * @param connection used to retrieve the rent exemption information
     */
    static getMinimumBalanceForRentExemption(connection: web3.Connection, commitment?: web3.Commitment): Promise<number>;
    /**
     * Determines if the provided {@link Buffer} has the correct byte size to
     * hold {@link Vault} data.
     */
    static hasCorrectByteSize(buf: Buffer, offset?: number): boolean;
    /**
     * Returns a readable version of {@link Vault} properties
     * and can be used to convert to JSON and/or logging
     */
    pretty(): {
        admin: string;
        depositTokenMint: string;
        receiptTokenMint: string;
        minDeposit: number | {
            toNumber: () => number;
        };
        minLockup: number | {
            toNumber: () => number;
        };
        targetYieldRate: number | {
            toNumber: () => number;
        };
        depositPool: string;
        rewardPool: string;
        totalReceiptSupply: number | {
            toNumber: () => number;
        };
    };
}
/**
 * @category Accounts
 * @category generated
 */
export declare const vaultBeet: beet.BeetStruct<Vault, VaultArgs & {
    accountDiscriminator: number[];
}>;
