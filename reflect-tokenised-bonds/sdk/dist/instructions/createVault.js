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
exports.createVaultInstructionDiscriminator = exports.createVaultStruct = void 0;
exports.createCreateVaultInstruction = createCreateVaultInstruction;
const beet = __importStar(require("@metaplex-foundation/beet"));
const web3 = __importStar(require("@solana/web3.js"));
/**
 * @category Instructions
 * @category CreateVault
 * @category generated
 */
exports.createVaultStruct = new beet.BeetArgsStruct([
    ['instructionDiscriminator', beet.uniformFixedSizeArray(beet.u8, 8)],
    ['minDeposit', beet.u64],
    ['minLockup', beet.i64],
    ['targetYieldRate', beet.u64],
    ['vaultSeed', beet.u64],
], 'CreateVaultInstructionArgs');
exports.createVaultInstructionDiscriminator = [
    29, 237, 247, 208, 193, 82, 54, 135,
];
/**
 * Creates a _CreateVault_ instruction.
 *
 * @param accounts that will be accessed while the instruction is processed
 * @param args to provide as instruction data to the program
 *
 * @category Instructions
 * @category CreateVault
 * @category generated
 */
function createCreateVaultInstruction(accounts, args, programId = new web3.PublicKey('6ZZ1sxKGuXUBL8HSsHqHaYCg92G9VhMNTcJv1gFURCop')) {
    var _a, _b;
    const [data] = exports.createVaultStruct.serialize(Object.assign({ instructionDiscriminator: exports.createVaultInstructionDiscriminator }, args));
    const keys = [
        {
            pubkey: accounts.admin,
            isWritable: true,
            isSigner: true,
        },
        {
            pubkey: accounts.rtbProtocol,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: accounts.vault,
            isWritable: true,
            isSigner: false,
        },
        {
            pubkey: (_a = accounts.systemProgram) !== null && _a !== void 0 ? _a : web3.SystemProgram.programId,
            isWritable: false,
            isSigner: false,
        },
        {
            pubkey: (_b = accounts.rent) !== null && _b !== void 0 ? _b : web3.SYSVAR_RENT_PUBKEY,
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
