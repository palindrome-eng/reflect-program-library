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
Object.defineProperty(exports, "__esModule", { value: true });
exports.processIntentInstructionDiscriminator = exports.processIntentStruct = void 0;
exports.createProcessIntentInstruction = createProcessIntentInstruction;
const splToken = __importStar(require("@solana/spl-token"));
const beet = __importStar(require("@metaplex-foundation/beet"));
const web3 = __importStar(require("@solana/web3.js"));
const ProcessIntentArgs_1 = require("../types/ProcessIntentArgs");
/**
 * @category Instructions
 * @category ProcessIntent
 * @category generated
 */
exports.processIntentStruct = new beet.BeetArgsStruct([
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['args', ProcessIntentArgs_1.processIntentArgsBeet],
], 'ProcessIntentInstructionArgs');
exports.processIntentInstructionDiscriminator = [
    205, 26, 207, 143, 70, 246, 24, 206,
];
/**
 * Creates a _ProcessIntent_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category ProcessIntent
 * @category generated
 */
function createProcessIntentInstruction(accounts, args, programId = new web3.PublicKey('EiMoMLXBCKpxTdBwK2mBBaGFWH1v2JdT5nAhiyJdF3pV')) {
    var _a;
    const [data] = exports.processIntentStruct.serialize(Object.assign({ instructionDiscriminator: exports.processIntentInstructionDiscriminator }, args));
    const keys = [
        {
            pubkey: accounts.signer,
            isWritable: true,
            isSigner: true,
        },
        {
            pubkey: accounts.admin,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: accounts.user,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: accounts.settings,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: accounts.deposit,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: accounts.intent,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: accounts.lockup,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: accounts.asset,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: accounts.assetMint,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: accounts.adminAssetAta,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: accounts.userAssetAta,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: (_a = accounts.tokenProgram) !== null && _a !== void 0 ? _a : splToken.TOKEN_PROGRAM_ID,
            isWritable: false,
            isSigner: false,
        },
    ];
    if (accounts.anchorRemainingAccounts != null) {
        for (const acc of accounts.anchorRemainingAccounts) {
            keys.push(acc);
        }
    }
    const ix = new web3.TransactionInstruction({
        programId,
        keys,
        data,
    });
    return ix;
}
