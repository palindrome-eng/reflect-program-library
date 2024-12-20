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
 * Arguments used to create {@link Deposit}
 * @category Accounts
 * @category generated
 */
export type DepositArgs = {
    index: beet.bignum;
    user: web3.PublicKey;
    amount: beet.bignum;
    initialUsdValue: beet.bignum;
    amountSlashed: beet.bignum;
    lockup: web3.PublicKey;
    unlockTs: beet.bignum;
    lastSlashed: beet.COption<beet.bignum>;
};
export declare const depositDiscriminator: number[];
/**
 * Holds the data for the {@link Deposit} Account and provides de/serialization
 * functionality for that data
 *
 * @category Accounts
 * @category generated
 */
export declare class Deposit implements DepositArgs {
    readonly index: beet.bignum;
    readonly user: web3.PublicKey;
    readonly amount: beet.bignum;
    readonly initialUsdValue: beet.bignum;
    readonly amountSlashed: beet.bignum;
    readonly lockup: web3.PublicKey;
    readonly unlockTs: beet.bignum;
    readonly lastSlashed: beet.COption<beet.bignum>;
    private constructor();
    /**
     * Creates a {@link Deposit} instance from the provided args.
     */
    static fromArgs(args: DepositArgs): Deposit;
    /**
     * Deserializes the {@link Deposit} from the data of the provided {@link web3.AccountInfo}.
     * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
     */
    static fromAccountInfo(accountInfo: web3.AccountInfo<Buffer>, offset?: number): [Deposit, number];
    /**
     * Retrieves the account info from the provided address and deserializes
     * the {@link Deposit} from its data.
     *
     * @throws Error if no account info is found at the address or if deserialization fails
     */
    static fromAccountAddress(connection: web3.Connection, address: web3.PublicKey, commitmentOrConfig?: web3.Commitment | web3.GetAccountInfoConfig): Promise<Deposit>;
    /**
     * Provides a {@link web3.Connection.getProgramAccounts} config builder,
     * to fetch accounts matching filters that can be specified via that builder.
     *
     * @param programId - the program that owns the accounts we are filtering
     */
    static gpaBuilder(programId?: web3.PublicKey): beetSolana.GpaBuilder<DepositArgs & {
        accountDiscriminator: number[];
    }>;
    /**
     * Deserializes the {@link Deposit} from the provided data Buffer.
     * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
     */
    static deserialize(buf: Buffer, offset?: number): [Deposit, number];
    /**
     * Serializes the {@link Deposit} into a Buffer.
     * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
     */
    serialize(): [Buffer, number];
    /**
     * Returns the byteSize of a {@link Buffer} holding the serialized data of
     * {@link Deposit} for the provided args.
     *
     * @param args need to be provided since the byte size for this account
     * depends on them
     */
    static byteSize(args: DepositArgs): number;
    /**
     * Fetches the minimum balance needed to exempt an account holding
     * {@link Deposit} data from rent
     *
     * @param args need to be provided since the byte size for this account
     * depends on them
     * @param connection used to retrieve the rent exemption information
     */
    static getMinimumBalanceForRentExemption(args: DepositArgs, connection: web3.Connection, commitment?: web3.Commitment): Promise<number>;
    /**
     * Returns a readable version of {@link Deposit} properties
     * and can be used to convert to JSON and/or logging
     */
    pretty(): {
        index: number | {
            toNumber: () => number;
        };
        user: string;
        amount: number | {
            toNumber: () => number;
        };
        initialUsdValue: number | {
            toNumber: () => number;
        };
        amountSlashed: number | {
            toNumber: () => number;
        };
        lockup: string;
        unlockTs: number | {
            toNumber: () => number;
        };
        lastSlashed: beet.bignum;
    };
}
/**
 * @category Accounts
 * @category generated
 */
export declare const depositBeet: beet.FixableBeetStruct<Deposit, DepositArgs & {
    accountDiscriminator: number[];
}>;
