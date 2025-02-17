/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */
import * as beet from '@metaplex-foundation/beet';
import * as web3 from '@solana/web3.js';
import * as beetSolana from '@metaplex-foundation/beet-solana';
/**
 * Arguments used to create {@link Intent}
 * @category Accounts
 * @category generated
 */
export type IntentArgs = {
    amount: beet.bignum;
    lockup: web3.PublicKey;
    deposit: web3.PublicKey;
};
export declare const intentDiscriminator: number[];
/**
 * Holds the data for the {@link Intent} Account and provides de/serialization
 * functionality for that data
 *
 * @category Accounts
 * @category generated
 */
export declare class Intent implements IntentArgs {
    readonly amount: beet.bignum;
    readonly lockup: web3.PublicKey;
    readonly deposit: web3.PublicKey;
    private constructor();
    /**
     * Creates a {@link Intent} instance from the provided args.
     */
    static fromArgs(args: IntentArgs): Intent;
    /**
     * Deserializes the {@link Intent} from the data of the provided {@link web3.AccountInfo}.
     * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
     */
    static fromAccountInfo(accountInfo: web3.AccountInfo<Buffer>, offset?: number): [Intent, number];
    /**
     * Retrieves the account info from the provided address and deserializes
     * the {@link Intent} from its data.
     *
     * @throws Error if no account info is found at the address or if deserialization fails
     */
    static fromAccountAddress(connection: web3.Connection, address: web3.PublicKey, commitmentOrConfig?: web3.Commitment | web3.GetAccountInfoConfig): Promise<Intent>;
    /**
     * Provides a {@link web3.Connection.getProgramAccounts} config builder,
     * to fetch accounts matching filters that can be specified via that builder.
     *
     * @param programId - the program that owns the accounts we are filtering
     */
    static gpaBuilder(programId?: web3.PublicKey): beetSolana.GpaBuilder<{
        accountDiscriminator: any;
        lockup: any;
        amount: any;
        deposit: any;
    }>;
    /**
     * Deserializes the {@link Intent} from the provided data Buffer.
     * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
     */
    static deserialize(buf: Buffer, offset?: number): [Intent, number];
    /**
     * Serializes the {@link Intent} into a Buffer.
     * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
     */
    serialize(): [Buffer, number];
    /**
     * Returns the byteSize of a {@link Buffer} holding the serialized data of
     * {@link Intent}
     */
    static get byteSize(): number;
    /**
     * Fetches the minimum balance needed to exempt an account holding
     * {@link Intent} data from rent
     *
     * @param connection used to retrieve the rent exemption information
     */
    static getMinimumBalanceForRentExemption(connection: web3.Connection, commitment?: web3.Commitment): Promise<number>;
    /**
     * Determines if the provided {@link Buffer} has the correct byte size to
     * hold {@link Intent} data.
     */
    static hasCorrectByteSize(buf: Buffer, offset?: number): boolean;
    /**
     * Returns a readable version of {@link Intent} properties
     * and can be used to convert to JSON and/or logging
     */
    pretty(): {
        amount: number | {
            toNumber: () => number;
        };
        lockup: string;
        deposit: string;
    };
}
/**
 * @category Accounts
 * @category generated
 */
export declare const intentBeet: beet.BeetStruct<Intent, IntentArgs & {
    accountDiscriminator: number[];
}>;
