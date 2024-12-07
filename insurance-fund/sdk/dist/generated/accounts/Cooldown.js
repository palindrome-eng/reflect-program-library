"use strict";
/**
 * This code was GENERATED using the solita package.
 * Please DO NOT EDIT THIS FILE, instead rerun solita to update it or write a wrapper to add functionality.
 *
 * See: https://github.com/metaplex-foundation/solita
 */
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || function (mod) {
    if (mod && mod.__esModule) return mod;
    var result = {};
    if (mod != null) for (var k in mod) if (k !== "default" && Object.prototype.hasOwnProperty.call(mod, k)) __createBinding(result, mod, k);
    __setModuleDefault(result, mod);
    return result;
};
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.cooldownBeet = exports.Cooldown = exports.cooldownDiscriminator = void 0;
const web3 = __importStar(require("@solana/web3.js"));
const beet = __importStar(require("@metaplex-foundation/beet"));
const beetSolana = __importStar(require("@metaplex-foundation/beet-solana"));
const CooldownRewards_1 = require("../types/CooldownRewards");
exports.cooldownDiscriminator = [50, 166, 94, 192, 234, 64, 152, 208];
/**
 * Holds the data for the {@link Cooldown} Account and provides de/serialization
 * functionality for that data
 *
 * @category Accounts
 * @category generated
 */
class Cooldown {
    constructor(user, depositId, baseAmount, unlockTs, rewards) {
        this.user = user;
        this.depositId = depositId;
        this.baseAmount = baseAmount;
        this.unlockTs = unlockTs;
        this.rewards = rewards;
    }
    /**
     * Creates a {@link Cooldown} instance from the provided args.
     */
    static fromArgs(args) {
        return new Cooldown(args.user, args.depositId, args.baseAmount, args.unlockTs, args.rewards);
    }
    /**
     * Deserializes the {@link Cooldown} from the data of the provided {@link web3.AccountInfo}.
     * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
     */
    static fromAccountInfo(accountInfo, offset = 0) {
        return Cooldown.deserialize(accountInfo.data, offset);
    }
    /**
     * Retrieves the account info from the provided address and deserializes
     * the {@link Cooldown} from its data.
     *
     * @throws Error if no account info is found at the address or if deserialization fails
     */
    static fromAccountAddress(connection, address, commitmentOrConfig) {
        return __awaiter(this, void 0, void 0, function* () {
            const accountInfo = yield connection.getAccountInfo(address, commitmentOrConfig);
            if (accountInfo == null) {
                throw new Error(`Unable to find Cooldown account at ${address}`);
            }
            return Cooldown.fromAccountInfo(accountInfo, 0)[0];
        });
    }
    /**
     * Provides a {@link web3.Connection.getProgramAccounts} config builder,
     * to fetch accounts matching filters that can be specified via that builder.
     *
     * @param programId - the program that owns the accounts we are filtering
     */
    static gpaBuilder(programId = new web3.PublicKey('EiMoMLXBCKpxTdBwK2mBBaGFWH1v2JdT5nAhiyJdF3pV')) {
        return beetSolana.GpaBuilder.fromStruct(programId, exports.cooldownBeet);
    }
    /**
     * Deserializes the {@link Cooldown} from the provided data Buffer.
     * @returns a tuple of the account data and the offset up to which the buffer was read to obtain it.
     */
    static deserialize(buf, offset = 0) {
        return exports.cooldownBeet.deserialize(buf, offset);
    }
    /**
     * Serializes the {@link Cooldown} into a Buffer.
     * @returns a tuple of the created Buffer and the offset up to which the buffer was written to store it.
     */
    serialize() {
        return exports.cooldownBeet.serialize(Object.assign({ accountDiscriminator: exports.cooldownDiscriminator }, this));
    }
    /**
     * Returns the byteSize of a {@link Buffer} holding the serialized data of
     * {@link Cooldown} for the provided args.
     *
     * @param args need to be provided since the byte size for this account
     * depends on them
     */
    static byteSize(args) {
        const instance = Cooldown.fromArgs(args);
        return exports.cooldownBeet.toFixedFromValue(Object.assign({ accountDiscriminator: exports.cooldownDiscriminator }, instance)).byteSize;
    }
    /**
     * Fetches the minimum balance needed to exempt an account holding
     * {@link Cooldown} data from rent
     *
     * @param args need to be provided since the byte size for this account
     * depends on them
     * @param connection used to retrieve the rent exemption information
     */
    static getMinimumBalanceForRentExemption(args, connection, commitment) {
        return __awaiter(this, void 0, void 0, function* () {
            return connection.getMinimumBalanceForRentExemption(Cooldown.byteSize(args), commitment);
        });
    }
    /**
     * Returns a readable version of {@link Cooldown} properties
     * and can be used to convert to JSON and/or logging
     */
    pretty() {
        return {
            user: this.user.toBase58(),
            depositId: (() => {
                const x = this.depositId;
                if (typeof x.toNumber === 'function') {
                    try {
                        return x.toNumber();
                    }
                    catch (_) {
                        return x;
                    }
                }
                return x;
            })(),
            baseAmount: (() => {
                const x = this.baseAmount;
                if (typeof x.toNumber === 'function') {
                    try {
                        return x.toNumber();
                    }
                    catch (_) {
                        return x;
                    }
                }
                return x;
            })(),
            unlockTs: (() => {
                const x = this.unlockTs;
                if (typeof x.toNumber === 'function') {
                    try {
                        return x.toNumber();
                    }
                    catch (_) {
                        return x;
                    }
                }
                return x;
            })(),
            rewards: this.rewards.__kind,
        };
    }
}
exports.Cooldown = Cooldown;
/**
 * @category Accounts
 * @category generated
 */
exports.cooldownBeet = new beet.FixableBeetStruct([
    ['accountDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['user', beetSolana.publicKey],
    ['depositId', beet.u64],
    ['baseAmount', beet.u64],
    ['unlockTs', beet.u64],
    ['rewards', CooldownRewards_1.cooldownRewardsBeet],
], Cooldown.fromArgs, 'Cooldown');