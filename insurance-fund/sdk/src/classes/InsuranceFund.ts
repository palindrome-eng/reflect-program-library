import {
    AccountInfo,
    Connection,
    PublicKey,
    SystemProgram,
    SYSVAR_CLOCK_PUBKEY,
    TransactionInstruction
} from "@solana/web3.js";
import {
    Admin,
    adminDiscriminator,
    Asset,
    assetDiscriminator, Cooldown, cooldownDiscriminator,
    createAddAssetInstruction,
    createBoostRewardsInstruction,
    createCreateIntentInstruction,
    createDepositRewardsInstruction,
    createInitializeInsuranceFundInstruction,
    createInitializeLockupInstruction,
    createInitializeSlashInstruction,
    createManageFreezeInstruction,
    createProcessIntentInstruction, createRequestWithdrawalInstruction,
    createRestakeInstruction, createSlashColdWalletInstruction,
    createSlashDepositsInstruction,
    createSlashPoolInstruction, createWithdrawInstruction,
    Deposit,
    depositDiscriminator,
    InitializeInsuranceFundArgs,
    Intent,
    intentDiscriminator,
    Lockup,
    lockupDiscriminator,
    PROGRAM_ID,
    RewardBoost,
    rewardBoostDiscriminator,
    Settings,
    Slash
} from "../generated";
import BN from "bn.js";
import {Account, getAccount, getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID} from "@solana/spl-token";

type InsuranceFundAccount = Asset | Admin | Cooldown | Deposit | Intent | Lockup | RewardBoost | Settings | Slash;

const DEPOSITS_PER_SLASH_INSTRUCTION = 10;

export class InsuranceFund {
    private connection: Connection;

    constructor(
        connection: Connection,
    ) {
        this.connection = connection;
    }

    accountFromBuffer<T extends InsuranceFundAccount>(
        schema: { fromAccountInfo: (accountInfo: AccountInfo<Buffer>) => [T, number] },
        accountInfo: AccountInfo<Buffer>
    ): T {
        return schema.fromAccountInfo(accountInfo)[0];
    }

    async getLockups() {
        const lockups = await Lockup
            .gpaBuilder()
            .addFilter("accountDiscriminator", lockupDiscriminator)
            .run(this.connection);

        return lockups.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer<Lockup>(Lockup, account) }));
    }

    async getLockupsByAsset(asset: PublicKey) {
        const lockups = await Lockup
            .gpaBuilder()
            .addFilter("accountDiscriminator", lockupDiscriminator)
            .addFilter("asset", asset)
            .run(this.connection);

        return lockups.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer<Lockup>(Lockup, account) }));
    }

    async getAssets() {
        const assets = await Asset
            .gpaBuilder()
            .addFilter("accountDiscriminator", assetDiscriminator)
            .run(this.connection);

        return assets.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer<Asset>(Asset, account) }));
    }

    async getDeposits() {
        const deposits = await Deposit
            .gpaBuilder()
            .addFilter("accountDiscriminator", depositDiscriminator)
            .run(this.connection);

        return deposits.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer<Deposit>(Deposit, account) }));
    }

    async getDepositsByUser(user: PublicKey) {
        const deposits = await Deposit
            .gpaBuilder()
            .addFilter("accountDiscriminator", depositDiscriminator)
            .addFilter("user", user)
            .run(this.connection);

        return deposits.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer<Deposit>(Deposit, account) }));
    }

    async getCooldowns() {
        const cooldowns = await Cooldown
            .gpaBuilder()
            .addFilter("accountDiscriminator", cooldownDiscriminator)
            .run(this.connection);

        return cooldowns.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer<Cooldown>(Cooldown, account) }));
    }

    async getCooldownsByDeposit(depositId: BN | number) {
        const cooldowns = await Cooldown
            .gpaBuilder()
            .addFilter("accountDiscriminator", cooldownDiscriminator)
            .addFilter("depositId", depositId)
            .run(this.connection);

        return cooldowns.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer<Cooldown>(Cooldown, account) }));
    }

    async getCooldownsByUser(user: PublicKey) {
        const cooldowns = await Cooldown
            .gpaBuilder()
            .addFilter("accountDiscriminator", cooldownDiscriminator)
            .addFilter("user", user)
            .run(this.connection);

        return cooldowns.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer<Cooldown>(Cooldown, account) }));
    }

    async getRewardBoostsForLockup(lockup: PublicKey) {
        const rewardBoosts = await RewardBoost
            .gpaBuilder()
            .addFilter("accountDiscriminator", rewardBoostDiscriminator)
            .addFilter("lockup", lockup)
            .run(this.connection);

        return rewardBoosts.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer<RewardBoost>(RewardBoost, account) }));
    }

    async getRewardBoostsForLockupByDepositSize(lockup: PublicKey, depositSize: BN) {
        const rewardBoosts = await RewardBoost
            .gpaBuilder()
            .addFilter("accountDiscriminator", rewardBoostDiscriminator)
            .addFilter("lockup", lockup)
            .run(this.connection);

        return rewardBoosts
            .map(({ pubkey, account }) => ({ pubkey, account: this.accountFromBuffer<RewardBoost>(RewardBoost, account) }))
            .filter(({ account: { minUsdValue } }) => new BN(minUsdValue).lte(depositSize));
    }

    async getIntents() {
        const intents = await Intent
            .gpaBuilder()
            .addFilter("accountDiscriminator", intentDiscriminator)
            .run(this.connection);

        return intents.map(({ pubkey, account }) => ({ pubkey, account: this.accountFromBuffer<Intent>(Intent, account) }));
    }

    deriveAdmin(
        index: number | BN
    ) {
        const [admin] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("admin"),
                (index instanceof BN ? index : new BN(index)).toArrayLike(Buffer, "le", 1)
            ],
            PROGRAM_ID
        );

        return admin;
    }

    async getAdmins() {
        const admins = await Admin
            .gpaBuilder()
            .addFilter("accountDiscriminator", adminDiscriminator)
            .run(this.connection);

        return admins.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer<Admin>(Admin, account) }))
    }

    // Technically program allows for multiple admin accounts under the same signer public key.
    // Program may require small rework that will ensure there's one Admin instance per publickey, i.e via using pubkey instead of index in seeds
    async getAdminFromPublicKey(address: PublicKey) {
        const admins = await Admin
            .gpaBuilder()
            .addFilter("accountDiscriminator", adminDiscriminator)
            .addFilter("address", address)
            .run(this.connection);

        return admins.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer<Admin>(Admin, account) }))
    }

    deriveSettings() {
        const [settings] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("settings"),
            ],
            PROGRAM_ID
        );

        return settings;
    }

    async initializeInsuranceFund(admin: PublicKey, args: InitializeInsuranceFundArgs) {
        return createInitializeInsuranceFundInstruction(
            {
                admin: this.deriveAdmin(0),
                settings: this.deriveSettings(),
                signer: admin,
                systemProgram: SystemProgram.programId
            },
            {
                args
            },
            PROGRAM_ID
        );
    }

    async getSettingsData() {
        return Settings.fromAccountAddress(this.connection, this.deriveSettings());
    }

    deriveLockup(index: number | BN) {
        const [lockup] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("lockup"),
                (index instanceof BN ? index : new BN(index)).toArrayLike(Buffer, "le", 8)
            ],
            PROGRAM_ID
        );

        return lockup;
    }

    deriveAsset(mint: PublicKey) {
        const [asset] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("asset"),
                mint.toBuffer()
            ],
            PROGRAM_ID
        );

        return asset;
    }

    deriveAssetPool(
        type: "vault" | "reward_pool",
        lockup: PublicKey,
        assetMint: PublicKey
    ) {
        const [vault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from(type),
                lockup.toBuffer(),
                assetMint.toBuffer()
            ],
            PROGRAM_ID
        );

        return vault;
    }

    async initializeLockup(
        signer: PublicKey,
        assetMint: PublicKey,
        rewardMint: PublicKey,
        depositCap: BN,
        minDeposit: BN,
        duration: BN,
        governanceYield?: BN,
    ) {
        const settingsData = await this.getSettingsData();
        const adminDatas = await this.getAdminFromPublicKey(signer);
        const admin = this.deriveAdmin(adminDatas[0].account.index);
        const asset = this.deriveAsset(assetMint);
        const lockup = this.deriveLockup(settingsData.lockups);
        const lockupAssetVault = this.deriveAssetPool("vault", lockup, assetMint)
        const assetRewardPool = this.deriveAssetPool("reward_pool", lockup, assetMint);

        return createInitializeLockupInstruction(
            {
                settings: this.deriveSettings(),
                lockup,
                admin,
                asset,
                assetMint,
                signer,
                rewardMint,
                lockupAssetVault,
                assetRewardPool,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
            },
            {
                args: {
                    yieldMode: governanceYield ? { __kind: "Single" } : { __kind: "Dual", fields: [governanceYield] },
                    depositCap,
                    duration,
                    yieldBps: 0, // this field is useless, remove in v2
                    minDeposit
                }
            },
            PROGRAM_ID
        );
    }

    async addAsset(
        signer: PublicKey,
        assetMint: PublicKey,
        oracle: PublicKey
    ) {
        const asset = this.deriveAsset(assetMint);
        const adminDatas = await this.getAdminFromPublicKey(signer);
        const admin = this.deriveAdmin(adminDatas[0].account.index);

        return createAddAssetInstruction(
            {
                assetMint,
                asset,
                admin,
                oracle,
                signer,
                settings: this.deriveSettings(),
            },
            PROGRAM_ID
        );
    }

    deriveRewardBoost(
        lockup: PublicKey,
        boostId: number | BN
    ) {
        const [rewardBoost] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("reward_boost"),
                lockup.toBuffer(),
                (boostId instanceof BN ? boostId : new BN(boostId)).toArrayLike(Buffer, "le", 8)
            ],
            PROGRAM_ID
        );

        return rewardBoost;
    }

    async getLockup(
        lockup: PublicKey,
    ) {
        return Lockup.fromAccountAddress(this.connection, lockup);
    }

    async boostRewards(
        signer: PublicKey,
        lockupId: BN,
        minUsdValue: BN,
        boostBps: BN
    ) {
        const adminDatas = await this.getAdminFromPublicKey(signer);
        const admin = this.deriveAdmin(adminDatas[0].account.index);
        const lockup = this.deriveLockup(lockupId);

        const {
            rewardBoosts
        } = await this.getLockup(lockup);

        const rewardBoost = this.deriveRewardBoost(lockup, rewardBoosts);

        return createBoostRewardsInstruction(
            {
                admin,
                lockup,
                settings: this.deriveSettings(),
                signer,
                rewardBoost,
                systemProgram: SystemProgram.programId,
            },
            {
                args: {
                    lockupId,
                    minUsdValue,
                    boostBps
                }
            }
        );
    }

    async depositRewards(
        lockupId: BN,
        amount: BN,
        signer: PublicKey
    ) {
        const lockup = this.deriveLockup(lockupId);
        const {
            rewardConfig: {
                main: rewardMint
            }
        } = await this.getSettingsData();

        const callerRewardAta = getAssociatedTokenAddressSync(
            rewardMint,
            signer,
            false
        );

        return createDepositRewardsInstruction(
            {
                caller: signer,
                settings: this.deriveSettings(),
                rewardMint,
                lockup,
                callerRewardAta,
                assetRewardPool: this.deriveAssetPool("reward_pool", lockup, rewardMint)
            },
            {
                args: {
                    lockupId,
                    amount
                }
            }
        );
    }

    deriveIntent(deposit: PublicKey) {
        const [intent] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("intent"),
                deposit.toBuffer()
            ],
            PROGRAM_ID
        );

        return intent;
    }

    deriveDeposit(
        lockup: PublicKey,
        depositId: BN | number
    ) {
        const [deposit] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("deposit"),
                lockup.toBuffer(),
                (depositId instanceof BN ? depositId : new BN(depositId)).toArrayLike(Buffer, "le", 8)
            ],
            PROGRAM_ID
        );

        return deposit;
    }

    async getDeposit(deposit: PublicKey) {
        return Deposit.fromAccountAddress(this.connection, deposit);
    }

    async createIntent(
        lockupId: BN | number,
        depositId: BN | number,
        amount: BN
    ) {
        const lockup = this.deriveLockup(lockupId);
        const {
            asset: assetMint
        } = await this.getLockup(lockup);

        const deposit = this.deriveDeposit(lockup, depositId);
        const {
            user,
            amount: depositAmount
        } = await this.getDeposit(deposit);

        if (amount.gt(depositAmount instanceof BN ? depositAmount : new BN(depositAmount)))
            throw new Error("Cannot withdraw more funds than deposited");

        const intent= this.deriveIntent(deposit);

        const lockupAssetVault = this.deriveAssetPool("vault", lockup, assetMint);
        const userAssetAta = getAssociatedTokenAddressSync(
            assetMint,
            user
        );

        return createCreateIntentInstruction(
            {
                lockup,
                assetMint,
                intent,
                settings: this.deriveSettings(),
                user,
                deposit,
                clock: SYSVAR_CLOCK_PUBKEY,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                lockupAssetVault,
                userAssetAta,
            },
            {
                args: {
                    lockupId,
                    amount,
                    depositId
                }
            }
        )
    }

    async getIntent(intent: PublicKey) {
        return Intent.fromAccountAddress(this.connection, intent);
    }

    async processIntent(
        deposit: PublicKey,
        signer: PublicKey
    ) {
        const intent = this.deriveIntent(deposit);
        const {
            lockup,
        } = await this.getIntent(intent);

        const {
            user,
            index: depositId // this has to implement index
        } = await this.getDeposit(deposit);

        const adminDatas = await this.getAdminFromPublicKey(signer);
        const admin = adminDatas[0].pubkey;

        const {
            asset: assetMint
        } = await this.getLockup(lockup);

        const asset = this.deriveAsset(assetMint);

        const userAssetAta = getAssociatedTokenAddressSync(assetMint, user);
        const adminAssetAta = getAssociatedTokenAddressSync(assetMint, signer);

        return createProcessIntentInstruction(
            {
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
                tokenProgram: TOKEN_PROGRAM_ID,
            },
            {
                args: {
                    depositId
                }
            }
        );
    }

    private async manageFreeze(
        signer: PublicKey,
        freeze: boolean
    ) {

        const adminDatas = await this.getAdminFromPublicKey(signer);
        const admin = adminDatas[0].pubkey;

        createManageFreezeInstruction(
            {
                admin,
                signer,
                settings: this.deriveSettings(),
            },
            {
                args: {
                    freeze
                }
            }
        )
    }

    async freeze(signer: PublicKey) {
        return this.manageFreeze(signer, true);
    }

    async unfreeze(signer: PublicKey) {
        return this.manageFreeze(signer, false);
    }

    deriveSlash(lockup: PublicKey, slashId: number | BN) {
        const [slash] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("slash"),
                lockup.toBuffer(),
                (slashId instanceof BN ? slashId : new BN(slashId)).toArrayLike(Buffer, "le", 8)
            ],
            PROGRAM_ID
        );

        return slash;
    }

    async initializeSlash(
        signer: PublicKey,
        lockupId: BN | number,
        amount: BN
    ) {
        const adminDatas = this.getAdminFromPublicKey(signer);
        const admin = adminDatas[0].pubkey;

        const lockup = this.deriveLockup(lockupId);
        const {
            asset: assetMint,
            slashState: {
                index: slashId
            }
        } = await this.getLockup(lockup);

        const assetLockup = this.deriveAssetPool("vault", lockup, assetMint);

        return createInitializeSlashInstruction(
            {
                admin,
                signer,
                lockup,
                assetMint,
                clock: SYSVAR_CLOCK_PUBKEY,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                settings: this.deriveSettings(),
                slash: this.deriveSlash(lockup, slashId),
                assetLockup
            },
            {
                args: {
                    lockupId,
                    amount
                }
            }
        );
    }

    async getSlash(slash: PublicKey) {
        return Slash.fromAccountAddress(this.connection, slash);
    }

    async slashPool(signer: PublicKey, lockupId: BN | number, destination: PublicKey) {

        const adminDatas = await this.getAdminFromPublicKey(signer);
        const admin = adminDatas[0].pubkey;

        const lockup = this.deriveLockup(lockupId);
        const { slashState: { index: slashId }, asset: assetMint } = await this.getLockup(lockup);
        const slash = this.deriveSlash(lockup, slashId);

        const asset = this.deriveAsset(assetMint);
        const assetLockup = this.deriveAssetPool("vault", lockup, assetMint);

        let destinationData: Account;
        try {
            destinationData = await getAccount(this.connection, destination);
        } catch (err) {
            throw new Error("Make sure destination account is initialized.");
        }

        if (!destinationData.mint.equals(assetMint)) throw new Error("Invalid destination SPL-Token account.");

        return createSlashPoolInstruction(
            {
                admin,
                lockup,
                slash,
                asset,
                assetMint,
                signer,
                settings: this.deriveSettings(),
                assetLockup,
                tokenProgram: TOKEN_PROGRAM_ID,
                destination
            },
            {
                args: {
                    lockupId,
                    slashId
                }
            }
        );
    }

    async slashDepositsBatch(signer: PublicKey, lockupId: BN | number) {
        const adminDatas = await this.getAdminFromPublicKey(signer);
        const admin = adminDatas[0].pubkey;

        const lockup = this.deriveLockup(lockupId);
        const { slashState: { index: slashId }, asset: assetMint } = await this.getLockup(lockup);
        const slash = this.deriveSlash(lockup, slashId);
        const assetLockup = this.deriveAssetPool("vault", lockup, assetMint);

        const {
            targetAmount,
            slashedAccounts,
            targetAccounts
        } = await this.getSlash(slash);

        const deposits: PublicKey[] = [];
        for (let i = 0; i < DEPOSITS_PER_SLASH_INSTRUCTION; i++) {
            deposits.push(this.deriveDeposit(lockup, new BN(slashedAccounts).addn(i)));
        }

        return createSlashDepositsInstruction(
            {
                settings: this.deriveSettings(),
                slash,
                signer,
                admin,
                lockup,
                assetMint,
                assetLockup,
                anchorRemainingAccounts: deposits.map((pubkey) => ({ pubkey, isWritable: true, isSigner: false }))
            },
            {
                args: {
                    lockupId,
                    slashId,
                    slashAmount: targetAmount
                }
            }
        );
    }

    // Returns all remaining batches that have to be executed one by one
    async slashAllRemainingDeposits(
        signer: PublicKey,
        lockupId: BN | number
    ) {
        const adminDatas = await this.getAdminFromPublicKey(signer);
        const admin = adminDatas[0].pubkey;
        const lockup = this.deriveLockup(lockupId);
        const { slashState: { index: slashId }, asset: assetMint } = await this.getLockup(lockup);
        const slash = this.deriveSlash(lockup, slashId);
        const assetLockup = this.deriveAssetPool("vault", lockup, assetMint);

        const {
            targetAmount,
            slashedAccounts,
            targetAccounts
        } = await this.getSlash(slash);

        const remainingDeposits = new BN(targetAccounts)
            .sub(new BN(slashedAccounts))
            .toNumber();

        const remainingBatches = remainingDeposits % DEPOSITS_PER_SLASH_INSTRUCTION
            ? remainingDeposits % DEPOSITS_PER_SLASH_INSTRUCTION
            : Math.floor(remainingDeposits / DEPOSITS_PER_SLASH_INSTRUCTION) + 1;

        const instructions: TransactionInstruction[] = [];
        for (let i = 0; i < remainingBatches; i++) {

            const deposits: PublicKey[] = [];
            for (let j = 0; j < DEPOSITS_PER_SLASH_INSTRUCTION; j++) {
                deposits.push(
                    this.deriveDeposit(
                        lockup,
                        new BN(slashedAccounts).addn((10 * i) + j)
                    )
                );
            }

            const ix = createSlashDepositsInstruction(
                {
                    settings: this.deriveSettings(),
                    slash,
                    signer,
                    admin,
                    lockup,
                    assetMint,
                    assetLockup,
                    anchorRemainingAccounts: deposits
                        .map((pubkey) => ({ pubkey, isWritable: true, isSigner: false }))
                },
                {
                    args: {
                        lockupId,
                        slashId,
                        slashAmount: targetAmount
                    }
                }
            );

            instructions.push(ix);
        }
    }

    async getAsset(asset: PublicKey) {
        return Asset.fromAccountAddress(
            this.connection,
            asset
        );
    }

    async restake(signer: PublicKey, amount: BN, lockupId: BN) {
        const settings = this.deriveSettings();
        const {
            coldWallet
        } = await this.getSettingsData();
        const lockup = this.deriveLockup(lockupId);
        const {
            asset: assetMint,
            deposits
        } = await this.getLockup(lockup);

        const asset = this.deriveAsset(assetMint);
        const { oracle: { fields: [oracleAddress] } } = await this.getAsset(asset);

        const coldWalletVault = getAssociatedTokenAddressSync(
            assetMint,
            coldWallet
        );
        const userAssetAta = getAssociatedTokenAddressSync(
            assetMint,
            signer
        );
        const lockupAssetVault = this.deriveAssetPool("vault", lockup, assetMint);

        const deposit = this.deriveDeposit(lockup, deposits);

        return createRestakeInstruction(
            {
                lockup,
                asset,
                assetMint,
                clock: SYSVAR_CLOCK_PUBKEY,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                coldWallet,
                coldWalletVault,
                deposit,
                settings,
                user: signer,
                oracle: oracleAddress,
                userAssetAta,
                lockupAssetVault
            },
            {
                args: {
                    lockupId,
                    amount
                }
            }
        );
    }

    deriveCooldown(deposit: PublicKey) {
        const [cooldown] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("cooldown"),
                deposit.toBuffer()
            ],
            PROGRAM_ID
        );

        return cooldown;
    }

    async requestWithdrawal(
        signer: PublicKey,
        lockupId: BN | number,
        depositId: BN | number,
        amount: BN,
        rewardBoostId?: BN | number
    ) {
        const lockup = this.deriveLockup(lockupId);
        const {
            asset: assetMint
        } = await Lockup.fromAccountAddress(this.connection, lockup);

        const {
            rewardConfig: {
                main: rewardAsset
            }
        } = await this.getSettingsData();
        const asset = this.deriveAsset(assetMint);
        const deposit = this.deriveDeposit(lockup, depositId);

        const assetRewardPool = this.deriveAssetPool("reward_pool", lockup, rewardAsset);
        const lockupAssetVault = this.deriveAssetPool("vault", lockup, assetMint);

        const userAssetAta = getAssociatedTokenAddressSync(
            assetMint,
            signer
        );

        const userRewardAta = getAssociatedTokenAddressSync(
            rewardAsset,
            signer
        );

        return createRequestWithdrawalInstruction(
            {
                user: signer,
                asset,
                assetMint,
                lockup,
                deposit,
                settings: this.deriveSettings(),
                assetRewardPool,
                clock: SYSVAR_CLOCK_PUBKEY,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                cooldown: this.deriveCooldown(deposit),
                rewardMint: rewardAsset,
                lockupAssetVault,
                rewardBoost: rewardBoostId !== undefined
                    ? this.deriveRewardBoost(lockup, rewardBoostId)
                    : null,
                userAssetAta,
                userRewardAta
            },
            {
                args: {
                    lockupId,
                    depositId,
                    amount,
                    rewardBoostId
                }
            }
        );
    }

    async requestWithdrawalWithAutoBoostDetection(
        signer: PublicKey,
        depositId: BN | number,
        lockupId: BN | number,
        amount: BN
    ) {
        const lockup = this.deriveLockup(lockupId);
        const deposit = this.deriveDeposit(lockup, depositId);
        const {
            initialUsdValue
        } = await this.getDeposit(deposit);

        const rewardBoosts = await this.getRewardBoostsForLockup(lockup);
        let preferredRewardBoost: RewardBoost;

        for (let i = 0; i < rewardBoosts.length; i++) {
            let { account } = rewardBoosts[i];

            if (
                new BN(account.minUsdValue)
                    .lte(new BN(initialUsdValue))
                && (
                    !preferredRewardBoost || new BN(preferredRewardBoost.boostBps)
                        .lte(new BN(account.boostBps))
                )
            ) {
                preferredRewardBoost = account;
            }
        }

        return this.requestWithdrawal(
            signer,
            lockupId,
            depositId,
            amount,
            preferredRewardBoost?.index
        );
    }

    async slashColdWalletAndTransferFunds(
        signer: PublicKey,
        lockupId: BN | number,
        destination: PublicKey,
    ) {
        const { coldWallet } = await this.getSettingsData();
        const lockup = this.deriveLockup(lockupId);
        const {
            slashState: { index: slashId },
            asset: assetMint,
        } = await this.getLockup(lockup);

        const adminDatas = await this.getAdminFromPublicKey(signer);
        const admin = adminDatas[0].pubkey;

        const slash = this.deriveSlash(lockup, slashId);
        const asset = this.deriveAsset(assetMint);

        const source = getAssociatedTokenAddressSync(
            assetMint,
            coldWallet,
            true
        );

        return createSlashColdWalletInstruction(
            {
                admin,
                lockup,
                signer,
                slash,
                assetMint,
                settings: this.deriveSettings(),
                destination,
                coldWallet,
                source,
                tokenProgram: TOKEN_PROGRAM_ID,
            },
            {
                args: {
                    lockupId,
                    slashId,
                    transferFunds: true,
                    transferSig: ""
                }
            }
        );
    }

    async slashColdWalletWithTransferSignature(
        signer: PublicKey,
        lockupId: BN | number,
        transferSig: string
    ) {
        const { coldWallet } = await this.getSettingsData();
        const lockup = this.deriveLockup(lockupId);
        const {
            slashState: { index: slashId },
        } = await this.getLockup(lockup);

        const adminDatas = await this.getAdminFromPublicKey(signer);
        const admin = adminDatas[0].pubkey;

        const slash = this.deriveSlash(lockup, slashId);

        return createSlashColdWalletInstruction(
            {
                admin,
                lockup,
                signer,
                slash,
                settings: this.deriveSettings(),
                coldWallet,
                tokenProgram: TOKEN_PROGRAM_ID,
            },
            {
                args: {
                    lockupId,
                    slashId,
                    transferFunds: false,
                    transferSig
                }
            }
        );
    }

    async withdrawCooldown(signer: PublicKey, lockupId: BN | number, depositId: BN | number) {
        const {
            rewardConfig: {
                main: rewardMint
            }
        } = await this.getSettingsData();
        const lockup = this.deriveLockup(lockupId);
        const deposit = this.deriveDeposit(lockup, depositId);
        const cooldown = this.deriveCooldown(deposit);
        const cooldownDatas = await this.getCooldownsByDeposit(depositId);
        const {
            account: {
                unlockTs
            }
        } = cooldownDatas[0];

        if (!(new BN(Date.now()).gte(new BN(unlockTs)))) throw new Error("Funds still in cooldown.");

        const {
            asset: assetMint
        } = await this.getLockup(lockup);

        const asset = this.deriveAsset(assetMint);

        const userAssetAta = getAssociatedTokenAddressSync(
            assetMint,
            signer,
        );

        const userRewardAta = getAssociatedTokenAddressSync(
            rewardMint,
            signer,
        );

        const lockupAssetVault = this.deriveAssetPool("vault", lockup, assetMint);
        const assetRewardPool = this.deriveAssetPool("reward_pool", lockup, rewardMint);

        return createWithdrawInstruction(
            {
                lockup,
                deposit,
                asset,
                assetMint,
                clock: SYSVAR_CLOCK_PUBKEY,
                tokenProgram: TOKEN_PROGRAM_ID,
                settings: this.deriveSettings(),
                cooldown,
                user: signer,
                userAssetAta,
                lockupAssetVault,
                assetRewardPool,
                rewardMint,
                userRewardAta
            },
            {
                args: {
                    lockupId,
                    depositId
                }
            }
        );
    }
}