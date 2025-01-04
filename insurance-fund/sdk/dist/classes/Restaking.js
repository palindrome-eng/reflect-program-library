"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
Object.defineProperty(exports, "__esModule", { value: true });
exports.Restaking = void 0;
const web3_js_1 = require("@solana/web3.js");
const generated_1 = require("../generated");
const bn_js_1 = __importDefault(require("bn.js"));
const spl_token_1 = require("@solana/spl-token");
const mpl_token_metadata_1 = require("@metaplex-foundation/mpl-token-metadata");
class Restaking {
    constructor(connection) {
        this.connection = connection;
    }
    accountFromBuffer(schema, accountInfo) {
        return schema.fromAccountInfo(accountInfo)[0];
    }
    getLockups() {
        return __awaiter(this, void 0, void 0, function* () {
            const lockups = yield generated_1.Lockup
                .gpaBuilder()
                .addFilter("accountDiscriminator", generated_1.lockupDiscriminator)
                .run(this.connection);
            return lockups.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer(generated_1.Lockup, account) }));
        });
    }
    getLockupsByAsset(assetMint) {
        return __awaiter(this, void 0, void 0, function* () {
            const lockups = yield generated_1.Lockup
                .gpaBuilder()
                .addFilter("accountDiscriminator", generated_1.lockupDiscriminator)
                .addFilter("assetMint", assetMint)
                .run(this.connection);
            return lockups.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer(generated_1.Lockup, account) }));
        });
    }
    getAssets() {
        return __awaiter(this, void 0, void 0, function* () {
            const assets = yield generated_1.Asset
                .gpaBuilder()
                .addFilter("accountDiscriminator", generated_1.assetDiscriminator)
                .run(this.connection);
            return assets.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer(generated_1.Asset, account) }));
        });
    }
    getDeposits() {
        return __awaiter(this, void 0, void 0, function* () {
            const deposits = yield generated_1.Deposit
                .gpaBuilder()
                .addFilter("accountDiscriminator", generated_1.depositDiscriminator)
                .run(this.connection);
            return deposits.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer(generated_1.Deposit, account) }));
        });
    }
    getDepositsByUser(user) {
        return __awaiter(this, void 0, void 0, function* () {
            const deposits = yield generated_1.Deposit
                .gpaBuilder()
                .addFilter("accountDiscriminator", generated_1.depositDiscriminator)
                .addFilter("user", user)
                .run(this.connection);
            return deposits.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer(generated_1.Deposit, account) }));
        });
    }
    getCooldowns() {
        return __awaiter(this, void 0, void 0, function* () {
            const cooldowns = yield generated_1.Cooldown
                .gpaBuilder()
                .addFilter("accountDiscriminator", generated_1.cooldownDiscriminator)
                .run(this.connection);
            return cooldowns.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer(generated_1.Cooldown, account) }));
        });
    }
    getCooldownsByDeposit(depositId) {
        return __awaiter(this, void 0, void 0, function* () {
            const cooldowns = yield generated_1.Cooldown
                .gpaBuilder()
                .addFilter("accountDiscriminator", generated_1.cooldownDiscriminator)
                .addFilter("depositId", depositId)
                .run(this.connection);
            return cooldowns.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer(generated_1.Cooldown, account) }));
        });
    }
    getCooldownsByUser(user) {
        return __awaiter(this, void 0, void 0, function* () {
            const cooldowns = yield generated_1.Cooldown
                .gpaBuilder()
                .addFilter("accountDiscriminator", generated_1.cooldownDiscriminator)
                .addFilter("user", user)
                .run(this.connection);
            return cooldowns.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer(generated_1.Cooldown, account) }));
        });
    }
    getRewardBoostsForLockup(lockup) {
        return __awaiter(this, void 0, void 0, function* () {
            const rewardBoosts = yield generated_1.RewardBoost
                .gpaBuilder()
                .addFilter("accountDiscriminator", generated_1.rewardBoostDiscriminator)
                .addFilter("lockup", lockup)
                .run(this.connection);
            return rewardBoosts.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer(generated_1.RewardBoost, account) }));
        });
    }
    getRewardBoostsForLockupByDepositSize(lockup, depositSize) {
        return __awaiter(this, void 0, void 0, function* () {
            const rewardBoosts = yield generated_1.RewardBoost
                .gpaBuilder()
                .addFilter("accountDiscriminator", generated_1.rewardBoostDiscriminator)
                .addFilter("lockup", lockup)
                .run(this.connection);
            return rewardBoosts
                .map(({ pubkey, account }) => ({ pubkey, account: this.accountFromBuffer(generated_1.RewardBoost, account) }))
                .filter(({ account: { minUsdValue } }) => new bn_js_1.default(minUsdValue).lte(depositSize));
        });
    }
    getIntents() {
        return __awaiter(this, void 0, void 0, function* () {
            const intents = yield generated_1.Intent
                .gpaBuilder()
                .addFilter("accountDiscriminator", generated_1.intentDiscriminator)
                .run(this.connection);
            return intents.map(({ pubkey, account }) => ({ pubkey, account: this.accountFromBuffer(generated_1.Intent, account) }));
        });
    }
    static deriveAdmin(address) {
        const [admin] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from("admin"),
            address.toBuffer()
        ], generated_1.PROGRAM_ID);
        return admin;
    }
    getAdmins() {
        return __awaiter(this, void 0, void 0, function* () {
            const admins = yield generated_1.Admin
                .gpaBuilder()
                .addFilter("accountDiscriminator", generated_1.adminDiscriminator)
                .run(this.connection);
            return admins.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer(generated_1.Admin, account) }));
        });
    }
    getAdminFromPublicKey(address) {
        return __awaiter(this, void 0, void 0, function* () {
            const admin = Restaking.deriveAdmin(address);
            const adminData = yield generated_1.Admin.fromAccountAddress(this.connection, admin);
            return { pubkey: admin, account: adminData };
        });
    }
    static deriveSettings() {
        const [settings] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from("settings"),
        ], generated_1.PROGRAM_ID);
        return settings;
    }
    initializeInsuranceFund(admin, args) {
        return __awaiter(this, void 0, void 0, function* () {
            return (0, generated_1.createInitializeInsuranceFundInstruction)({
                admin: Restaking.deriveAdmin(admin),
                settings: Restaking.deriveSettings(),
                signer: admin,
                systemProgram: web3_js_1.SystemProgram.programId
            }, {
                args
            }, generated_1.PROGRAM_ID);
        });
    }
    getSettingsData() {
        return __awaiter(this, void 0, void 0, function* () {
            return generated_1.Settings.fromAccountAddress(this.connection, Restaking.deriveSettings());
        });
    }
    static deriveLockup(index) {
        const [lockup] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from("lockup"),
            (index instanceof bn_js_1.default ? index : new bn_js_1.default(index)).toArrayLike(Buffer, "le", 8)
        ], generated_1.PROGRAM_ID);
        return lockup;
    }
    static deriveAsset(mint) {
        const [asset] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from("asset"),
            mint.toBuffer()
        ], generated_1.PROGRAM_ID);
        return asset;
    }
    static deriveAssetPool(type, lockup, assetMint) {
        const [vault] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from(type),
            lockup.toBuffer(),
            assetMint.toBuffer()
        ], generated_1.PROGRAM_ID);
        return vault;
    }
    static deriveLockupColdVault(lockup, assetMint) {
        const [coldVault] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from("cold_vault"),
            lockup.toBuffer(),
            assetMint.toBuffer()
        ], generated_1.PROGRAM_ID);
        return coldVault;
    }
    getLockupColdVault(address) {
        return __awaiter(this, void 0, void 0, function* () {
            return (0, spl_token_1.getAccount)(this.connection, address, "confirmed");
        });
    }
    static deriveLockupHotVault(lockup, assetMint) {
        const [hotVault] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from("hot_vault"),
            lockup.toBuffer(),
            assetMint.toBuffer()
        ], generated_1.PROGRAM_ID);
        return hotVault;
    }
    getLockupHotVault(address) {
        return __awaiter(this, void 0, void 0, function* () {
            return (0, spl_token_1.getAccount)(this.connection, address, "confirmed");
        });
    }
    static deriveLockupCooldownVault(lockup, receiptMint) {
        const [cooldownVault] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from("cooldown_vault"),
            lockup.toBuffer(),
            receiptMint.toBuffer()
        ], generated_1.PROGRAM_ID);
        return cooldownVault;
    }
    getLockupCooldownVault(address) {
        return __awaiter(this, void 0, void 0, function* () {
            return (0, spl_token_1.getAccount)(this.connection, address, "confirmed");
        });
    }
    createToken(signer, lockup, depositToken, withMetadata) {
        return __awaiter(this, void 0, void 0, function* () {
            const { decimals } = yield (0, spl_token_1.getMint)(this.connection, depositToken, "confirmed");
            const tokenKeypair = web3_js_1.Keypair.generate();
            const instructions = [];
            const createAccountIx = web3_js_1.SystemProgram.createAccount({
                lamports: yield this.connection.getMinimumBalanceForRentExemption(spl_token_1.MINT_SIZE),
                space: spl_token_1.MINT_SIZE,
                fromPubkey: signer,
                newAccountPubkey: tokenKeypair.publicKey,
                programId: spl_token_1.TOKEN_PROGRAM_ID
            });
            instructions.push(createAccountIx);
            const createMintIx = (0, spl_token_1.createInitializeMint2Instruction)(tokenKeypair.publicKey, decimals, signer, null);
            instructions.push(createMintIx);
            if (withMetadata) {
                const metadataData = {
                    name: "Reflect | Insurance Fund Receipt",
                    symbol: "RECEIPT",
                    uri: "",
                    sellerFeeBasisPoints: 0,
                    creators: null,
                    collection: null,
                    uses: null,
                };
                const [metadata] = web3_js_1.PublicKey.findProgramAddressSync([
                    Buffer.from("metadata"),
                    mpl_token_metadata_1.PROGRAM_ID.toBuffer(),
                    tokenKeypair.publicKey.toBuffer(),
                ], mpl_token_metadata_1.PROGRAM_ID);
                const createMetadataIx = (0, mpl_token_metadata_1.createCreateMetadataAccountV3Instruction)({
                    metadata,
                    mint: tokenKeypair.publicKey,
                    mintAuthority: signer,
                    payer: signer,
                    updateAuthority: signer,
                }, {
                    createMetadataAccountArgsV3: {
                        data: metadataData,
                        isMutable: true,
                        collectionDetails: null
                    }
                });
                instructions.push(createMetadataIx);
            }
            const setAuthorityIx = (0, spl_token_1.createSetAuthorityInstruction)(tokenKeypair.publicKey, signer, spl_token_1.AuthorityType.MintTokens, lockup);
            instructions.push(setAuthorityIx);
            return {
                instructions,
                mint: tokenKeypair.publicKey
            };
        });
    }
    initializeLockup(signer, assetMint, rewardMint, depositCap, minDeposit, duration, governanceYield) {
        return __awaiter(this, void 0, void 0, function* () {
            const settingsData = yield this.getSettingsData();
            const asset = Restaking.deriveAsset(assetMint);
            const lockup = Restaking.deriveLockup(settingsData.lockups);
            const lockupAssetVault = Restaking.deriveAssetPool("vault", lockup, assetMint);
            const assetRewardPool = Restaking.deriveAssetPool("reward_pool", lockup, assetMint);
            const admin = Restaking.deriveAdmin(signer);
            const { instructions: preInstructions, mint: receiptMint } = yield this.createToken(signer, lockup, assetMint, true);
            const lockupHotVault = Restaking.deriveLockupHotVault(lockup, assetMint);
            const lockupColdVault = Restaking.deriveLockupColdVault(lockup, assetMint);
            const lockupCooldownVault = Restaking.deriveLockupCooldownVault(lockup, receiptMint);
            const initializeLockupIx = (0, generated_1.createInitializeLockupInstruction)({
                settings: Restaking.deriveSettings(),
                lockup,
                admin,
                asset,
                assetMint,
                signer,
                rewardMint,
                assetRewardPool,
                tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
                systemProgram: web3_js_1.SystemProgram.programId,
                poolShareReceipt: receiptMint,
                coldWallet: settingsData.coldWallet,
                lockupColdVault,
                lockupHotVault,
                lockupCooldownVault
            }, {
                args: {
                    yieldMode: governanceYield ? { __kind: "Single" } : { __kind: "Dual", fields: [governanceYield] },
                    depositCap,
                    duration,
                    minDeposit
                }
            }, generated_1.PROGRAM_ID);
            return [...preInstructions, initializeLockupIx];
        });
    }
    addAsset(signer, assetMint, oracle) {
        return __awaiter(this, void 0, void 0, function* () {
            const asset = Restaking.deriveAsset(assetMint);
            const { pubkey: admin } = yield this.getAdminFromPublicKey(signer);
            return (0, generated_1.createAddAssetInstruction)({
                assetMint,
                asset,
                admin,
                oracle,
                signer,
                settings: Restaking.deriveSettings(),
            }, generated_1.PROGRAM_ID);
        });
    }
    static deriveRewardBoost(lockup, boostId) {
        const [rewardBoost] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from("reward_boost"),
            lockup.toBuffer(),
            (boostId instanceof bn_js_1.default ? boostId : new bn_js_1.default(boostId)).toArrayLike(Buffer, "le", 8)
        ], generated_1.PROGRAM_ID);
        return rewardBoost;
    }
    getLockup(lockup) {
        return __awaiter(this, void 0, void 0, function* () {
            return generated_1.Lockup.fromAccountAddress(this.connection, lockup);
        });
    }
    getReceiptToDepositsExchangeRateBps(lockup) {
        return __awaiter(this, void 0, void 0, function* () {
            const { assetMint, receiptMint } = yield generated_1.Lockup.fromAccountAddress(this.connection, lockup);
            const lockupHotVault = Restaking.deriveLockupHotVault(lockup, assetMint);
            const lockupColdVault = Restaking.deriveLockupColdVault(lockup, assetMint);
            const { amount: hotAmount } = yield this.getLockupHotVault(lockupHotVault);
            const { amount: coldAmount } = yield this.getLockupColdVault(lockupColdVault);
            const { supply: receiptSupply } = yield (0, spl_token_1.getMint)(this.connection, receiptMint);
            const totalDeposit = new bn_js_1.default(hotAmount.toString()).add(new bn_js_1.default(coldAmount.toString()));
            const exchangeRateBps = totalDeposit
                .muln(10000)
                .div(new bn_js_1.default(receiptSupply.toString()));
            return exchangeRateBps;
        });
    }
    boostRewards(signer, lockupId, minUsdValue, boostBps) {
        return __awaiter(this, void 0, void 0, function* () {
            const adminDatas = yield this.getAdminFromPublicKey(signer);
            const admin = Restaking.deriveAdmin(adminDatas[0].account.index);
            const lockup = Restaking.deriveLockup(lockupId);
            const { rewardBoosts } = yield this.getLockup(lockup);
            const rewardBoost = Restaking.deriveRewardBoost(lockup, rewardBoosts);
            return (0, generated_1.createBoostRewardsInstruction)({
                admin,
                lockup,
                settings: Restaking.deriveSettings(),
                signer,
                rewardBoost,
                systemProgram: web3_js_1.SystemProgram.programId,
            }, {
                args: {
                    lockupId,
                    minUsdValue,
                    boostBps
                }
            });
        });
    }
    depositRewards(lockupId, amount, signer) {
        return __awaiter(this, void 0, void 0, function* () {
            const lockup = Restaking.deriveLockup(lockupId);
            const { rewardConfig: { main: rewardMint } } = yield this.getSettingsData();
            const { receiptMint } = yield this.getLockup(lockup);
            const lockupCooldownVault = Restaking.deriveLockupCooldownVault(lockup, receiptMint);
            const callerRewardAta = (0, spl_token_1.getAssociatedTokenAddressSync)(rewardMint, signer, false);
            return (0, generated_1.createDepositRewardsInstruction)({
                caller: signer,
                settings: Restaking.deriveSettings(),
                rewardMint,
                lockup,
                callerRewardAta,
                assetRewardPool: Restaking.deriveAssetPool("reward_pool", lockup, rewardMint),
                lockupCooldownVault,
                receiptTokenMint: receiptMint
            }, {
                args: {
                    lockupId,
                    amount
                }
            });
        });
    }
    static deriveIntent(deposit) {
        const [intent] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from("intent"),
            deposit.toBuffer()
        ], generated_1.PROGRAM_ID);
        return intent;
    }
    static deriveDeposit(lockup, depositId) {
        const [deposit] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from("deposit"),
            lockup.toBuffer(),
            (depositId instanceof bn_js_1.default ? depositId : new bn_js_1.default(depositId)).toArrayLike(Buffer, "le", 8)
        ], generated_1.PROGRAM_ID);
        return deposit;
    }
    getDeposit(deposit) {
        return __awaiter(this, void 0, void 0, function* () {
            return generated_1.Deposit.fromAccountAddress(this.connection, deposit);
        });
    }
    static deriveDepositReceiptVault(deposit, receiptToken) {
        const [depositReceiptVault] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from("deposit_receipt_vault"),
            deposit.toBuffer(),
            receiptToken.toBuffer()
        ], generated_1.PROGRAM_ID);
        return depositReceiptVault;
    }
    getDepositReceiptVault(address) {
        return __awaiter(this, void 0, void 0, function* () {
            return (0, spl_token_1.getAccount)(this.connection, address, "confirmed");
        });
    }
    createIntent(lockupId, depositId, amount) {
        return __awaiter(this, void 0, void 0, function* () {
            const settings = Restaking.deriveSettings();
            const { rewardConfig: { main: rewardMint } } = yield generated_1.Settings.fromAccountAddress(this.connection, settings);
            const lockup = Restaking.deriveLockup(lockupId);
            const { assetMint, receiptMint, } = yield this.getLockup(lockup);
            const asset = Restaking.deriveAsset(assetMint);
            const deposit = Restaking.deriveDeposit(lockup, depositId);
            const { user, } = yield this.getDeposit(deposit);
            const depositReceiptVault = Restaking.deriveDepositReceiptVault(deposit, receiptMint);
            const { amount: receiptAmount } = yield this.getDepositReceiptVault(depositReceiptVault);
            const exchangeRateBps = yield this.getReceiptToDepositsExchangeRateBps(lockup);
            const depositAmount = new bn_js_1.default(receiptAmount.toString())
                .mul(exchangeRateBps)
                .divn(10000);
            if (amount.gt(depositAmount))
                throw new Error("Cannot withdraw more funds than deposited");
            const intent = Restaking.deriveIntent(deposit);
            const lockupAssetVault = Restaking.deriveAssetPool("vault", lockup, assetMint);
            const userAssetAta = (0, spl_token_1.getAssociatedTokenAddressSync)(assetMint, user);
            const userRewardAta = (0, spl_token_1.getAssociatedTokenAddressSync)(rewardMint, user);
            const depositReceiptTokenAccount = (0, spl_token_1.getAssociatedTokenAddressSync)(receiptMint, deposit, true);
            const cooldown = Restaking.deriveCooldown(deposit);
            const lockupHotVault = Restaking.deriveLockupHotVault(lockup, assetMint);
            const lockupColdVault = Restaking.deriveLockupColdVault(lockup, assetMint);
            const assetRewardPool = Restaking.deriveAssetPool("reward_pool", lockup, assetMint);
            const lockupCooldownVault = Restaking.deriveLockupCooldownVault(lockup, assetMint);
            return (0, generated_1.createWithdrawInstruction)({
                lockup,
                assetMint,
                intent,
                settings,
                user,
                deposit,
                tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
                systemProgram: web3_js_1.SystemProgram.programId,
                userAssetAta,
                rewardMint,
                asset,
                userRewardAta,
                cooldown,
                lockupHotVault,
                lockupColdVault,
                lockupCooldownVault,
                receiptTokenMint: receiptMint,
                assetRewardPool,
                depositReceiptTokenAccount
            }, {
                args: {
                    lockupId,
                    depositId,
                }
            });
        });
    }
    getIntent(intent) {
        return __awaiter(this, void 0, void 0, function* () {
            return generated_1.Intent.fromAccountAddress(this.connection, intent);
        });
    }
    manageFreeze(signer, freeze) {
        return __awaiter(this, void 0, void 0, function* () {
            const adminDatas = yield this.getAdminFromPublicKey(signer);
            const admin = adminDatas[0].pubkey;
            (0, generated_1.createManageFreezeInstruction)({
                admin,
                signer,
                settings: Restaking.deriveSettings(),
            }, {
                args: {
                    freeze
                }
            });
        });
    }
    freeze(signer) {
        return __awaiter(this, void 0, void 0, function* () {
            return this.manageFreeze(signer, true);
        });
    }
    unfreeze(signer) {
        return __awaiter(this, void 0, void 0, function* () {
            return this.manageFreeze(signer, false);
        });
    }
    slash(amount, signer, lockupId, destination) {
        return __awaiter(this, void 0, void 0, function* () {
            const settings = Restaking.deriveSettings();
            const admin = Restaking.deriveAdmin(signer);
            const lockup = Restaking.deriveLockup(lockupId);
            const { assetMint } = yield this.getLockup(lockup);
            const lockupHotVault = Restaking.deriveLockupHotVault(lockup, assetMint);
            const lockupColdVault = Restaking.deriveLockupColdVault(lockup, assetMint);
            const ix = (0, generated_1.createSlashInstruction)({
                signer,
                admin,
                settings,
                lockup,
                assetMint,
                destination,
                tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
                lockupColdVault,
                lockupHotVault
            }, {
                args: {
                    lockupId,
                    amount
                }
            }, generated_1.PROGRAM_ID);
            return ix;
        });
    }
    getAsset(asset) {
        return __awaiter(this, void 0, void 0, function* () {
            return generated_1.Asset.fromAccountAddress(this.connection, asset);
        });
    }
    restake(signer, amount, lockupId) {
        return __awaiter(this, void 0, void 0, function* () {
            const settings = Restaking.deriveSettings();
            const { coldWallet } = yield this.getSettingsData();
            const lockup = Restaking.deriveLockup(lockupId);
            const { assetMint, deposits, receiptMint } = yield this.getLockup(lockup);
            const asset = Restaking.deriveAsset(assetMint);
            const { oracle: { fields: [oracleAddress] } } = yield this.getAsset(asset);
            const coldWalletVault = (0, spl_token_1.getAssociatedTokenAddressSync)(assetMint, coldWallet);
            const userAssetAta = (0, spl_token_1.getAssociatedTokenAddressSync)(assetMint, signer);
            const lockupAssetVault = Restaking.deriveAssetPool("vault", lockup, assetMint);
            const deposit = Restaking.deriveDeposit(lockup, deposits);
            const depositReceiptTokenAccount = (0, spl_token_1.getAssociatedTokenAddressSync)(receiptMint, deposit, true);
            const lockupHotVault = Restaking.deriveLockupHotVault(lockup, assetMint);
            const lockupColdVault = Restaking.deriveLockupColdVault(lockup, assetMint);
            return (0, generated_1.createRestakeInstruction)({
                lockup,
                asset,
                assetMint,
                tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
                systemProgram: web3_js_1.SystemProgram.programId,
                deposit,
                settings,
                user: signer,
                oracle: oracleAddress,
                userAssetAta,
                receiptTokenMint: receiptMint,
                lockupColdVault,
                lockupHotVault,
                depositReceiptTokenAccount
            }, {
                args: {
                    lockupId,
                    amount
                }
            });
        });
    }
    static deriveCooldown(deposit) {
        const [cooldown] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from("cooldown"),
            deposit.toBuffer()
        ], generated_1.PROGRAM_ID);
        return cooldown;
    }
    requestWithdrawal(signer, lockupId, depositId, mode, amount, rewardBoostId) {
        return __awaiter(this, void 0, void 0, function* () {
            const lockup = Restaking.deriveLockup(lockupId);
            const { assetMint, receiptMint } = yield generated_1.Lockup.fromAccountAddress(this.connection, lockup);
            const { rewardConfig: { main: rewardAsset } } = yield this.getSettingsData();
            const asset = Restaking.deriveAsset(assetMint);
            const deposit = Restaking.deriveDeposit(lockup, depositId);
            const assetRewardPool = Restaking.deriveAssetPool("reward_pool", lockup, rewardAsset);
            const lockupAssetVault = Restaking.deriveAssetPool("vault", lockup, assetMint);
            const userAssetAta = (0, spl_token_1.getAssociatedTokenAddressSync)(assetMint, signer);
            const userRewardAta = (0, spl_token_1.getAssociatedTokenAddressSync)(rewardAsset, signer);
            const lockupHotVault = Restaking.deriveLockupHotVault(lockup, assetMint);
            const lockupColdVault = Restaking.deriveLockupColdVault(lockup, assetMint);
            const lockupCooldownVault = Restaking.deriveLockupCooldownVault(lockup, receiptMint);
            const depositReceiptTokenAccount = (0, spl_token_1.getAssociatedTokenAddressSync)(receiptMint, deposit, true);
            return (0, generated_1.createRequestWithdrawalInstruction)({
                user: signer,
                asset,
                assetMint,
                lockup,
                deposit,
                settings: Restaking.deriveSettings(),
                assetRewardPool,
                tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
                systemProgram: web3_js_1.SystemProgram.programId,
                cooldown: Restaking.deriveCooldown(deposit),
                rewardMint: rewardAsset,
                rewardBoost: rewardBoostId !== undefined
                    ? Restaking.deriveRewardBoost(lockup, rewardBoostId)
                    : null,
                lockupCooldownVault,
                receiptTokenMint: receiptMint,
                depositReceiptTokenAccount,
                lockupColdVault,
                lockupHotVault
            }, {
                args: {
                    lockupId,
                    depositId,
                    rewardBoostId,
                    mode: {
                        __kind: mode,
                        fields: [amount]
                    }
                }
            });
        });
    }
    requestWithdrawalWithAutoBoostDetection(signer, depositId, lockupId, mode, amount) {
        return __awaiter(this, void 0, void 0, function* () {
            const lockup = Restaking.deriveLockup(lockupId);
            const deposit = Restaking.deriveDeposit(lockup, depositId);
            const { initialUsdValue } = yield this.getDeposit(deposit);
            const rewardBoosts = yield this.getRewardBoostsForLockup(lockup);
            let preferredRewardBoost;
            for (let i = 0; i < rewardBoosts.length; i++) {
                let { account } = rewardBoosts[i];
                if (new bn_js_1.default(account.minUsdValue)
                    .lte(new bn_js_1.default(initialUsdValue))
                    && (!preferredRewardBoost || new bn_js_1.default(preferredRewardBoost.boostBps)
                        .lte(new bn_js_1.default(account.boostBps)))) {
                    preferredRewardBoost = account;
                }
            }
            return this.requestWithdrawal(signer, lockupId, depositId, mode, amount, preferredRewardBoost === null || preferredRewardBoost === void 0 ? void 0 : preferredRewardBoost.index);
        });
    }
    withdrawCooldown(signer, lockupId, depositId) {
        return __awaiter(this, void 0, void 0, function* () {
            const { rewardConfig: { main: rewardMint } } = yield this.getSettingsData();
            const lockup = Restaking.deriveLockup(lockupId);
            const deposit = Restaking.deriveDeposit(lockup, depositId);
            const cooldown = Restaking.deriveCooldown(deposit);
            const cooldownDatas = yield this.getCooldownsByDeposit(depositId);
            const { account: { unlockTs } } = cooldownDatas[0];
            if (!(new bn_js_1.default(Date.now()).gte(new bn_js_1.default(unlockTs))))
                throw new Error("Funds still in cooldown.");
            const { assetMint, receiptMint } = yield this.getLockup(lockup);
            const asset = Restaking.deriveAsset(assetMint);
            const userAssetAta = (0, spl_token_1.getAssociatedTokenAddressSync)(assetMint, signer);
            const userRewardAta = (0, spl_token_1.getAssociatedTokenAddressSync)(rewardMint, signer);
            const lockupAssetVault = Restaking.deriveAssetPool("vault", lockup, assetMint);
            const assetRewardPool = Restaking.deriveAssetPool("reward_pool", lockup, rewardMint);
            const lockupCooldownVault = Restaking.deriveLockupCooldownVault(lockup, receiptMint);
            const lockupColdVault = Restaking.deriveLockupColdVault(lockup, assetMint);
            const lockupHotVault = Restaking.deriveLockupHotVault(lockup, assetMint);
            const depositReceiptTokenAccount = (0, spl_token_1.getAssociatedTokenAddressSync)(receiptMint, deposit, true);
            return (0, generated_1.createWithdrawInstruction)({
                lockup,
                deposit,
                asset,
                assetMint,
                tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
                settings: Restaking.deriveSettings(),
                cooldown,
                user: signer,
                userAssetAta,
                assetRewardPool,
                rewardMint,
                userRewardAta,
                lockupCooldownVault,
                receiptTokenMint: receiptMint,
                depositReceiptTokenAccount,
                lockupColdVault,
                lockupHotVault,
                intent: null,
                systemProgram: web3_js_1.SystemProgram.programId
            }, {
                args: {
                    lockupId,
                    depositId
                }
            });
        });
    }
}
exports.Restaking = Restaking;
