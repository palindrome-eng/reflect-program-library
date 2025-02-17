/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
import * as beetSolana from '@metaplex-foundation/beet-solana';
import { CooldownRewards } from '../types/CooldownRewards';
/**
 * Arguments used to create {@link Cooldown}
 * @category Accounts
 * @category generated
 */
export type CooldownArgs = {
    user: web3.PublicKey;
    depositId: beet.bignum;
    lockupId: beet.bignum;
    receiptAmount: beet.bignum;
    unlockTs: beet.bignum;
    rewards: CooldownRewards;
};
export declare const cooldownDiscriminator: number[];
/**
 * Holds the data for the {@link Cooldown} Account and provides de/serialization
 * functionality for that data
 *
 * @category Accounts
 * @category generated
 */
export declare class Cooldown implements CooldownArgs {
    readonly user: web3.PublicKey;
    readonly depositId: beet.bignum;
    readonly lockupId: beet.bignum;
    readonly receiptAmount: beet.bignum;
    readonly unlockTs: beet.bignum;
    readonly rewards: CooldownRewards;
    private constructor();
    /**
     * Creates a {@link Cooldown} instance from the provided args.
     */
    static fromArgs(args: CooldownArgs): Cooldown;
    /**
     * Deserializes the {@link Cooldown} from the data of the provided {@link web3.AccountInfo}.
     * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
     */
    static fromAccountInfo(accountInfo: web3.AccountInfo<Buffer>, offset?: number): [Cooldown, number];
    /**
     * Retrieves the account info from the provided address and deserializes
     * the {@link Cooldown} from its data.
     *
     * @throws Error if no account info is found at the address or if deserialization fails
     */
    static fromAccountAddress(connection: web3.Connection, address: web3.PublicKey, commitmentOrConfig?: web3.Commitment | web3.GetAccountInfoConfig): Promise<Cooldown>;
    /**
     * Provides a {@link web3.Connection.getProgramAccounts} config builder,
     * to fetch accounts matching filters that can be specified via that builder.
     *
     * @param programId - the program that owns the accounts we are filtering
     */
    static gpaBuilder(programId?: web3.PublicKey): beetSolana.GpaBuilder<CooldownArgs & {
        accountDiscriminator: number[];
    }>;
    /**
     * Deserializes the {@link Cooldown} from the provided data Buffer.
     * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
     */
    static deserialize(buf: Buffer, offset?: number): [Cooldown, number];
    /**
     * Serializes the {@link Cooldown} into a Buffer.
     * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
     */
    serialize(): [Buffer, number];
    /**
     * Returns the byteSize of a {@link Buffer} holding the serialized data of
     * {@link Cooldown} for the provided args.
     *
     * @param args need to be provided since the byte size for this account
     * depends on them
     */
    static byteSize(args: CooldownArgs): number;
    /**
     * Fetches the minimum balance needed to exempt an account holding
     * {@link Cooldown} data from rent
     *
     * @param args need to be provided since the byte size for this account
     * depends on them
     * @param connection used to retrieve the rent exemption information
     */
    static getMinimumBalanceForRentExemption(args: CooldownArgs, connection: web3.Connection, commitment?: web3.Commitment): Promise<number>;
    /**
     * Returns a readable version of {@link Cooldown} properties
     * and can be used to convert to JSON and/or logging
     */
    pretty(): {
        user: string;
        depositId: number | {
            toNumber: () => number;
        };
        lockupId: number | {
            toNumber: () => number;
        };
        receiptAmount: number | {
            toNumber: () => number;
        };
        unlockTs: number | {
            toNumber: () => number;
        };
        rewards: "Single" | "Dual";
    };
}
/**
 * @category Accounts
 * @category generated
 */
export declare const cooldownBeet: beet.FixableBeetStruct<Cooldown, CooldownArgs & {
    accountDiscriminator: number[];
}>;
