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
exports.InsuranceFund = void 0;
const web3_js_1 = require("@solana/web3.js");
const generated_1 = require("../generated");
const bn_js_1 = __importDefault(require("bn.js"));
const spl_token_1 = require("@solana/spl-token");
const DEPOSITS_PER_SLASH_INSTRUCTION = 10;
class InsuranceFund {
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
    getLockupsByAsset(asset) {
        return __awaiter(this, void 0, void 0, function* () {
            const lockups = yield generated_1.Lockup
                .gpaBuilder()
                .addFilter("accountDiscriminator", generated_1.lockupDiscriminator)
                .addFilter("asset", asset)
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
    deriveAdmin(index) {
        const [admin] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from("admin"),
            (index instanceof bn_js_1.default ? index : new bn_js_1.default(index)).toArrayLike(Buffer, "le", 1)
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
    // Technically program allows for multiple admin accounts under the same signer public key.
    // Program may require small rework that will ensure there's one Admin instance per publickey, i.e via using pubkey instead of index in seeds
    getAdminFromPublicKey(address) {
        return __awaiter(this, void 0, void 0, function* () {
            const admins = yield generated_1.Admin
                .gpaBuilder()
                .addFilter("accountDiscriminator", generated_1.adminDiscriminator)
                .addFilter("address", address)
                .run(this.connection);
            return admins.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer(generated_1.Admin, account) }));
        });
    }
    deriveSettings() {
        const [settings] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from("settings"),
        ], generated_1.PROGRAM_ID);
        return settings;
    }
    initializeInsuranceFund(admin, args) {
        return __awaiter(this, void 0, void 0, function* () {
            return (0, generated_1.createInitializeInsuranceFundInstruction)({
                admin: this.deriveAdmin(0),
                settings: this.deriveSettings(),
                signer: admin,
                systemProgram: web3_js_1.SystemProgram.programId
            }, {
                args
            }, generated_1.PROGRAM_ID);
        });
    }
    getSettingsData() {
        return __awaiter(this, void 0, void 0, function* () {
            return generated_1.Settings.fromAccountAddress(this.connection, this.deriveSettings());
        });
    }
    deriveLockup(index) {
        const [lockup] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from("lockup"),
            (index instanceof bn_js_1.default ? index : new bn_js_1.default(index)).toArrayLike(Buffer, "le", 8)
        ], generated_1.PROGRAM_ID);
        return lockup;
    }
    deriveAsset(mint) {
        const [asset] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from("asset"),
            mint.toBuffer()
        ], generated_1.PROGRAM_ID);
        return asset;
    }
    deriveAssetPool(type, lockup, assetMint) {
        const [vault] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from(type),
            lockup.toBuffer(),
            assetMint.toBuffer()
        ], generated_1.PROGRAM_ID);
        return vault;
    }
    initializeLockup(signer, assetMint, rewardMint, depositCap, minDeposit, duration, governanceYield) {
        return __awaiter(this, void 0, void 0, function* () {
            const settingsData = yield this.getSettingsData();
            const adminDatas = yield this.getAdminFromPublicKey(signer);
            const admin = this.deriveAdmin(adminDatas[0].account.index);
            const asset = this.deriveAsset(assetMint);
            const lockup = this.deriveLockup(settingsData.lockups);
            const lockupAssetVault = this.deriveAssetPool("vault", lockup, assetMint);
            const assetRewardPool = this.deriveAssetPool("reward_pool", lockup, assetMint);
            return (0, generated_1.createInitializeLockupInstruction)({
                settings: this.deriveSettings(),
                lockup,
                admin,
                asset,
                assetMint,
                signer,
                rewardMint,
                lockupAssetVault,
                assetRewardPool,
                tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
                systemProgram: web3_js_1.SystemProgram.programId,
            }, {
                args: {
                    yieldMode: governanceYield ? { __kind: "Single" } : { __kind: "Dual", fields: [governanceYield] },
                    depositCap,
                    duration,
                    yieldBps: 0, // this field is useless, remove in v2
                    minDeposit
                }
            }, generated_1.PROGRAM_ID);
        });
    }
    addAsset(signer, assetMint, oracle) {
        return __awaiter(this, void 0, void 0, function* () {
            const asset = this.deriveAsset(assetMint);
            const adminDatas = yield this.getAdminFromPublicKey(signer);
            const admin = this.deriveAdmin(adminDatas[0].account.index);
            return (0, generated_1.createAddAssetInstruction)({
                assetMint,
                asset,
                admin,
                oracle,
                signer,
                settings: this.deriveSettings(),
            }, generated_1.PROGRAM_ID);
        });
    }
    deriveRewardBoost(lockup, boostId) {
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
    boostRewards(signer, lockupId, minUsdValue, boostBps) {
        return __awaiter(this, void 0, void 0, function* () {
            const adminDatas = yield this.getAdminFromPublicKey(signer);
            const admin = this.deriveAdmin(adminDatas[0].account.index);
            const lockup = this.deriveLockup(lockupId);
            const { rewardBoosts } = yield this.getLockup(lockup);
            const rewardBoost = this.deriveRewardBoost(lockup, rewardBoosts);
            return (0, generated_1.createBoostRewardsInstruction)({
                admin,
                lockup,
                settings: this.deriveSettings(),
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
            const lockup = this.deriveLockup(lockupId);
            const { rewardConfig: { main: rewardMint } } = yield this.getSettingsData();
            const callerRewardAta = (0, spl_token_1.getAssociatedTokenAddressSync)(rewardMint, signer, false);
            return (0, generated_1.createDepositRewardsInstruction)({
                caller: signer,
                settings: this.deriveSettings(),
                rewardMint,
                lockup,
                callerRewardAta,
                assetRewardPool: this.deriveAssetPool("reward_pool", lockup, rewardMint)
            }, {
                args: {
                    lockupId,
                    amount
                }
            });
        });
    }
    deriveIntent(deposit) {
        const [intent] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from("intent"),
            deposit.toBuffer()
        ], generated_1.PROGRAM_ID);
        return intent;
    }
    deriveDeposit(lockup, depositId) {
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
    createIntent(lockupId, depositId, amount) {
        return __awaiter(this, void 0, void 0, function* () {
            const lockup = this.deriveLockup(lockupId);
            const { asset: assetMint } = yield this.getLockup(lockup);
            const deposit = this.deriveDeposit(lockup, depositId);
            const { user, amount: depositAmount } = yield this.getDeposit(deposit);
            if (amount.gt(depositAmount instanceof bn_js_1.default ? depositAmount : new bn_js_1.default(depositAmount)))
                throw new Error("Cannot withdraw more funds than deposited");
            const intent = this.deriveIntent(deposit);
            const lockupAssetVault = this.deriveAssetPool("vault", lockup, assetMint);
            const userAssetAta = (0, spl_token_1.getAssociatedTokenAddressSync)(assetMint, user);
            return (0, generated_1.createCreateIntentInstruction)({
                lockup,
                assetMint,
                intent,
                settings: this.deriveSettings(),
                user,
                deposit,
                clock: web3_js_1.SYSVAR_CLOCK_PUBKEY,
                tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
                systemProgram: web3_js_1.SystemProgram.programId,
                lockupAssetVault,
                userAssetAta,
            }, {
                args: {
                    lockupId,
                    amount,
                    depositId
                }
            });
        });
    }
    getIntent(intent) {
        return __awaiter(this, void 0, void 0, function* () {
            return generated_1.Intent.fromAccountAddress(this.connection, intent);
        });
    }
    processIntent(deposit, signer) {
        return __awaiter(this, void 0, void 0, function* () {
            const intent = this.deriveIntent(deposit);
            const { lockup, } = yield this.getIntent(intent);
            const { user, index: depositId // this has to implement index
             } = yield this.getDeposit(deposit);
            const adminDatas = yield this.getAdminFromPublicKey(signer);
            const admin = adminDatas[0].pubkey;
            const { asset: assetMint } = yield this.getLockup(lockup);
            const asset = this.deriveAsset(assetMint);
            const userAssetAta = (0, spl_token_1.getAssociatedTokenAddressSync)(assetMint, user);
            const adminAssetAta = (0, spl_token_1.getAssociatedTokenAddressSync)(assetMint, signer);
            return (0, generated_1.createProcessIntentInstruction)({
                lockup,
                deposit,
                intent,
                settings: this.deriveSettings(),
                user,
                signer,
                admin,
                asset,
                assetMint,
                userAssetAta,
                adminAssetAta,
                tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
            }, {
                args: {
                    depositId
                }
            });
        });
    }
    manageFreeze(signer, freeze) {
        return __awaiter(this, void 0, void 0, function* () {
            const adminDatas = yield this.getAdminFromPublicKey(signer);
            const admin = adminDatas[0].pubkey;
            (0, generated_1.createManageFreezeInstruction)({
                admin,
                signer,
                settings: this.deriveSettings(),
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
    deriveSlash(lockup, slashId) {
        const [slash] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from("slash"),
            lockup.toBuffer(),
            (slashId instanceof bn_js_1.default ? slashId : new bn_js_1.default(slashId)).toArrayLike(Buffer, "le", 8)
        ], generated_1.PROGRAM_ID);
        return slash;
    }
    initializeSlash(signer, lockupId, amount) {
        return __awaiter(this, void 0, void 0, function* () {
            const adminDatas = this.getAdminFromPublicKey(signer);
            const admin = adminDatas[0].pubkey;
            const lockup = this.deriveLockup(lockupId);
            const { asset: assetMint, slashState: { index: slashId } } = yield this.getLockup(lockup);
            const assetLockup = this.deriveAssetPool("vault", lockup, assetMint);
            return (0, generated_1.createInitializeSlashInstruction)({
                admin,
                signer,
                lockup,
                assetMint,
                clock: web3_js_1.SYSVAR_CLOCK_PUBKEY,
                tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
                systemProgram: web3_js_1.SystemProgram.programId,
                settings: this.deriveSettings(),
                slash: this.deriveSlash(lockup, slashId),
                assetLockup
            }, {
                args: {
                    lockupId,
                    amount
                }
            });
        });
    }
    getSlash(slash) {
        return __awaiter(this, void 0, void 0, function* () {
            return generated_1.Slash.fromAccountAddress(this.connection, slash);
        });
    }
    slashPool(signer, lockupId, destination) {
        return __awaiter(this, void 0, void 0, function* () {
            const adminDatas = yield this.getAdminFromPublicKey(signer);
            const admin = adminDatas[0].pubkey;
            const lockup = this.deriveLockup(lockupId);
            const { slashState: { index: slashId }, asset: assetMint } = yield this.getLockup(lockup);
            const slash = this.deriveSlash(lockup, slashId);
            const asset = this.deriveAsset(assetMint);
            const assetLockup = this.deriveAssetPool("vault", lockup, assetMint);
            let destinationData;
            try {
                destinationData = yield (0, spl_token_1.getAccount)(this.connection, destination);
            }
            catch (err) {
                throw new Error("Make sure destination account is initialized.");
            }
            if (!destinationData.mint.equals(assetMint))
                throw new Error("Invalid destination SPL-Token account.");
            return (0, generated_1.createSlashPoolInstruction)({
                admin,
                lockup,
                slash,
                asset,
                assetMint,
                signer,
                settings: this.deriveSettings(),
                assetLockup,
                tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
                destination
            }, {
                args: {
                    lockupId,
                    slashId
                }
            });
        });
    }
    slashDepositsBatch(signer, lockupId) {
        return __awaiter(this, void 0, void 0, function* () {
            const adminDatas = yield this.getAdminFromPublicKey(signer);
            const admin = adminDatas[0].pubkey;
            const lockup = this.deriveLockup(lockupId);
            const { slashState: { index: slashId }, asset: assetMint } = yield this.getLockup(lockup);
            const slash = this.deriveSlash(lockup, slashId);
            const assetLockup = this.deriveAssetPool("vault", lockup, assetMint);
            const { targetAmount, slashedAccounts, targetAccounts } = yield this.getSlash(slash);
            const deposits = [];
            for (let i = 0; i < DEPOSITS_PER_SLASH_INSTRUCTION; i++) {
                deposits.push(this.deriveDeposit(lockup, new bn_js_1.default(slashedAccounts).addn(i)));
            }
            return (0, generated_1.createSlashDepositsInstruction)({
                settings: this.deriveSettings(),
                slash,
                signer,
                admin,
                lockup,
                assetMint,
                assetLockup,
                anchorRemainingAccounts: deposits.map((pubkey) => ({ pubkey, isWritable: true, isSigner: false }))
            }, {
                args: {
                    lockupId,
                    slashId,
                    slashAmount: targetAmount
                }
            });
        });
    }
    // Returns all remaining batches that have to be executed one by one
    slashAllRemainingDeposits(signer, lockupId) {
        return __awaiter(this, void 0, void 0, function* () {
            const adminDatas = yield this.getAdminFromPublicKey(signer);
            const admin = adminDatas[0].pubkey;
            const lockup = this.deriveLockup(lockupId);
            const { slashState: { index: slashId }, asset: assetMint } = yield this.getLockup(lockup);
            const slash = this.deriveSlash(lockup, slashId);
            const assetLockup = this.deriveAssetPool("vault", lockup, assetMint);
            const { targetAmount, slashedAccounts, targetAccounts } = yield this.getSlash(slash);
            const remainingDeposits = new bn_js_1.default(targetAccounts)
                .sub(new bn_js_1.default(slashedAccounts))
                .toNumber();
            const remainingBatches = remainingDeposits % DEPOSITS_PER_SLASH_INSTRUCTION
                ? remainingDeposits % DEPOSITS_PER_SLASH_INSTRUCTION
                : Math.floor(remainingDeposits / DEPOSITS_PER_SLASH_INSTRUCTION) + 1;
            const instructions = [];
            for (let i = 0; i < remainingBatches; i++) {
                const deposits = [];
                for (let j = 0; j < DEPOSITS_PER_SLASH_INSTRUCTION; j++) {
                    deposits.push(this.deriveDeposit(lockup, new bn_js_1.default(slashedAccounts).addn((10 * i) + j)));
                }
                const ix = (0, generated_1.createSlashDepositsInstruction)({
                    settings: this.deriveSettings(),
                    slash,
                    signer,
                    admin,
                    lockup,
                    assetMint,
                    assetLockup,
                    anchorRemainingAccounts: deposits
                        .map((pubkey) => ({ pubkey, isWritable: true, isSigner: false }))
                }, {
                    args: {
                        lockupId,
                        slashId,
                        slashAmount: targetAmount
                    }
                });
                instructions.push(ix);
            }
        });
    }
    getAsset(asset) {
        return __awaiter(this, void 0, void 0, function* () {
            return generated_1.Asset.fromAccountAddress(this.connection, asset);
        });
    }
    restake(signer, amount, lockupId) {
        return __awaiter(this, void 0, void 0, function* () {
            const settings = this.deriveSettings();
            const { coldWallet } = yield this.getSettingsData();
            const lockup = this.deriveLockup(lockupId);
            const { asset: assetMint, deposits } = yield this.getLockup(lockup);
            const asset = this.deriveAsset(assetMint);
            const { oracle: { fields: [oracleAddress] } } = yield this.getAsset(asset);
            const coldWalletVault = (0, spl_token_1.getAssociatedTokenAddressSync)(assetMint, coldWallet);
            const userAssetAta = (0, spl_token_1.getAssociatedTokenAddressSync)(assetMint, signer);
            const lockupAssetVault = this.deriveAssetPool("vault", lockup, assetMint);
            const deposit = this.deriveDeposit(lockup, deposits);
            return (0, generated_1.createRestakeInstruction)({
                lockup,
                asset,
                assetMint,
                clock: web3_js_1.SYSVAR_CLOCK_PUBKEY,
                tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
                systemProgram: web3_js_1.SystemProgram.programId,
                coldWallet,
                coldWalletVault,
                deposit,
                settings,
                user: signer,
                oracle: oracleAddress,
                userAssetAta,
                lockupAssetVault
            }, {
                args: {
                    lockupId,
                    amount
                }
            });
        });
    }
    deriveCooldown(deposit) {
        const [cooldown] = web3_js_1.PublicKey.findProgramAddressSync([
            Buffer.from("cooldown"),
            deposit.toBuffer()
        ], generated_1.PROGRAM_ID);
        return cooldown;
    }
    requestWithdrawal(signer, lockupId, depositId, amount, rewardBoostId) {
        return __awaiter(this, void 0, void 0, function* () {
            const lockup = this.deriveLockup(lockupId);
            const { asset: assetMint } = yield generated_1.Lockup.fromAccountAddress(this.connection, lockup);
            const { rewardConfig: { main: rewardAsset } } = yield this.getSettingsData();
            const asset = this.deriveAsset(assetMint);
            const deposit = this.deriveDeposit(lockup, depositId);
            const assetRewardPool = this.deriveAssetPool("reward_pool", lockup, rewardAsset);
            const lockupAssetVault = this.deriveAssetPool("vault", lockup, assetMint);
            const userAssetAta = (0, spl_token_1.getAssociatedTokenAddressSync)(assetMint, signer);
            const userRewardAta = (0, spl_token_1.getAssociatedTokenAddressSync)(rewardAsset, signer);
            return (0, generated_1.createRequestWithdrawalInstruction)({
                user: signer,
                asset,
                assetMint,
                lockup,
                deposit,
                settings: this.deriveSettings(),
                assetRewardPool,
                clock: web3_js_1.SYSVAR_CLOCK_PUBKEY,
                tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
                systemProgram: web3_js_1.SystemProgram.programId,
                cooldown: this.deriveCooldown(deposit),
                rewardMint: rewardAsset,
                lockupAssetVault,
                rewardBoost: rewardBoostId !== undefined
                    ? this.deriveRewardBoost(lockup, rewardBoostId)
                    : null,
                userAssetAta,
                userRewardAta
            }, {
                args: {
                    lockupId,
                    depositId,
                    amount,
                    rewardBoostId
                }
            });
        });
    }
    requestWithdrawalWithAutoBoostDetection(signer, depositId, lockupId, amount) {
        return __awaiter(this, void 0, void 0, function* () {
            const lockup = this.deriveLockup(lockupId);
            const deposit = this.deriveDeposit(lockup, depositId);
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
            return this.requestWithdrawal(signer, lockupId, depositId, amount, preferredRewardBoost === null || preferredRewardBoost === void 0 ? void 0 : preferredRewardBoost.index);
        });
    }
    slashColdWalletAndTransferFunds(signer, lockupId, destination) {
        return __awaiter(this, void 0, void 0, function* () {
            const { coldWallet } = yield this.getSettingsData();
            const lockup = this.deriveLockup(lockupId);
            const { slashState: { index: slashId }, asset: assetMint, } = yield this.getLockup(lockup);
            const adminDatas = yield this.getAdminFromPublicKey(signer);
            const admin = adminDatas[0].pubkey;
            const slash = this.deriveSlash(lockup, slashId);
            const asset = this.deriveAsset(assetMint);
            const source = (0, spl_token_1.getAssociatedTokenAddressSync)(assetMint, coldWallet, true);
            return (0, generated_1.createSlashColdWalletInstruction)({
                admin,
                lockup,
                signer,
                slash,
                assetMint,
                settings: this.deriveSettings(),
                destination,
                coldWallet,
                source,
                tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
            }, {
                args: {
                    lockupId,
                    slashId,
                    transferFunds: true,
                    transferSig: ""
                }
            });
        });
    }
    slashColdWalletWithTransferSignature(signer, lockupId, transferSig) {
        return __awaiter(this, void 0, void 0, function* () {
            const { coldWallet } = yield this.getSettingsData();
            const lockup = this.deriveLockup(lockupId);
            const { slashState: { index: slashId }, } = yield this.getLockup(lockup);
            const adminDatas = yield this.getAdminFromPublicKey(signer);
            const admin = adminDatas[0].pubkey;
            const slash = this.deriveSlash(lockup, slashId);
            return (0, generated_1.createSlashColdWalletInstruction)({
                admin,
                lockup,
                signer,
                slash,
                settings: this.deriveSettings(),
                coldWallet,
                tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
            }, {
                args: {
                    lockupId,
                    slashId,
                    transferFunds: false,
                    transferSig
                }
            });
        });
    }
    withdrawCooldown(signer, lockupId, depositId) {
        return __awaiter(this, void 0, void 0, function* () {
            const { rewardConfig: { main: rewardMint } } = yield this.getSettingsData();
            const lockup = this.deriveLockup(lockupId);
            const deposit = this.deriveDeposit(lockup, depositId);
            const cooldown = this.deriveCooldown(deposit);
            const cooldownDatas = yield this.getCooldownsByDeposit(depositId);
            const { account: { unlockTs } } = cooldownDatas[0];
            if (!(new bn_js_1.default(Date.now()).gte(new bn_js_1.default(unlockTs))))
                throw new Error("Funds still in cooldown.");
            const { asset: assetMint } = yield this.getLockup(lockup);
            const asset = this.deriveAsset(assetMint);
            const userAssetAta = (0, spl_token_1.getAssociatedTokenAddressSync)(assetMint, signer);
            const userRewardAta = (0, spl_token_1.getAssociatedTokenAddressSync)(rewardMint, signer);
            const lockupAssetVault = this.deriveAssetPool("vault", lockup, assetMint);
            const assetRewardPool = this.deriveAssetPool("reward_pool", lockup, rewardMint);
            return (0, generated_1.createWithdrawInstruction)({
                lockup,
                deposit,
                asset,
                assetMint,
                clock: web3_js_1.SYSVAR_CLOCK_PUBKEY,
                tokenProgram: spl_token_1.TOKEN_PROGRAM_ID,
                settings: this.deriveSettings(),
                cooldown,
                user: signer,
                userAssetAta,
                lockupAssetVault,
                assetRewardPool,
                rewardMint,
                userRewardAta
            }, {
                args: {
                    lockupId,
                    depositId
                }
            });
        });
    }
}
exports.InsuranceFund = InsuranceFund;
