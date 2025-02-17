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
exports.manageFreezeInstructionDiscriminator = exports.manageFreezeStruct = void 0;
exports.createManageFreezeInstruction = createManageFreezeInstruction;
const beet = __importStar(require("@metaplex-foundation/beet"));
const web3 = __importStar(require("@solana/web3.js"));
const ManageFreezeArgs_1 = require("../types/ManageFreezeArgs");
/**
 * @category Instructions
 * @category ManageFreeze
 * @category generated
 */
exports.manageFreezeStruct = new beet.BeetArgsStruct([
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['args', ManageFreezeArgs_1.manageFreezeArgsBeet],
], 'ManageFreezeInstructionArgs');
exports.manageFreezeInstructionDiscriminator = [
    163, 151, 84, 239, 6, 113, 250, 208,
];
/**
 * Creates a _ManageFreeze_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category ManageFreeze
 * @category generated
 */
function createManageFreezeInstruction(accounts, args, programId = new web3.PublicKey('2MN1Dbnu7zM9Yj4ougn6ZCNNKevrSvi9AR56iawzkye8')) {
    const [data] = exports.manageFreezeStruct.serialize(Object.assign({ instructionDiscriminator: exports.manageFreezeInstructionDiscriminator }, args));
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
            pubkey: accounts.settings,
            isWritable: true,
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
