/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */
import * as web3 from '@solana/web3.js';
import * as beet from '@metaplex-foundation/beet';
/**
 * This type is used to derive the {@link Oracle} type as well as the de/serializer.
 * However don't refer to it in your code but use the {@link Oracle} type instead.
 *
 * @category userTypes
 * @category enums
 * @category generated
 * @private
 */
export type OracleRecord = {
    Pyth: {
        fields: [web3.PublicKey];
    };
    Switchboard: {
        fields: [web3.PublicKey];
    };
};
/**
 * Union type respresenting the Oracle data enum defined in Rust.
 *
 * NOTE: that it includes a `__kind` property which allows to narrow types in
 * switch/if statements.
 * Additionally `isOracle*` type guards are exposed below to narrow to a specific variant.
 *
 * @category userTypes
 * @category enums
 * @category generated
 */
export type Oracle = beet.DataEnumKeyAsKind<OracleRecord>;
export declare const isOraclePyth: (x: Oracle) => x is Oracle & {
    __kind: "Pyth";
};
export declare const isOracleSwitchboard: (x: Oracle) => x is Oracle & {
    __kind: "Switchboard";
};
/**
 * @category userTypes
 * @category generated
 */
export declare const oracleBeet: beet.FixableBeet<Oracle, Oracle>;
