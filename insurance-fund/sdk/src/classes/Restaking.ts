import {
    AccountInfo, ComputeBudgetInstruction, ComputeBudgetProgram,
    Connection,
    Keypair,
    PublicKey,
    SystemProgram,
    SYSVAR_CLOCK_PUBKEY,
    TransactionInstruction
} from "@solana/web3.js";
import {
    Admin,
    adminDiscriminator,
    Asset,
    assetDiscriminator,
    Cooldown,
    cooldownDiscriminator,
    createAddAssetInstruction,
    createBoostRewardsInstruction,
    createDepositRewardsInstruction,
    createInitializeInsuranceFundInstruction,
    createInitializeLockupInstruction, createInitializeLockupVaultsInstruction,
    createManageFreezeInstruction,
    createRequestWithdrawalInstruction,
    createRestakeInstruction, createSlashInstruction,
    createWithdrawInstruction,
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
} from "../generated";
import BN from "bn.js";
import {
    Account,
    AuthorityType,
    createInitializeMint2Instruction,
    createSetAuthorityInstruction,
    getAccount,
    getAssociatedTokenAddressSync,
    getMint,
    MINT_SIZE,
    TOKEN_PROGRAM_ID
} from "@solana/spl-token";
import {
    createCreateMetadataAccountV3Instruction,
    PROGRAM_ID as METAPLEX_PROGRAM_ID
} from "@metaplex-foundation/mpl-token-metadata";

type InsuranceFundAccount = Asset | Admin | Cooldown | Deposit | Intent | Lockup | RewardBoost | Settings;

export class Restaking {
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

    async getLockupsByAsset(assetMint: PublicKey) {
        const lockups = await Lockup
            .gpaBuilder()
            .addFilter("accountDiscriminator", lockupDiscriminator)
            .addFilter("assetMint", assetMint)
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

    static deriveAdmin(
        address: PublicKey
    ) {
        const [admin] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("admin"),
                address.toBuffer()
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

    async getAdminFromPublicKey(address: PublicKey) {
        const admin = Restaking.deriveAdmin(address);

        const adminData = await Admin.fromAccountAddress(this.connection, admin);
        return { pubkey: admin, account: adminData };
    }

    static deriveSettings() {
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
                admin: Restaking.deriveAdmin(admin),
                settings: Restaking.deriveSettings(),
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
        return Settings.fromAccountAddress(this.connection, Restaking.deriveSettings());
    }

    static deriveLockup(index: number | BN) {
        const [lockup] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("lockup"),
                (index instanceof BN ? index : new BN(index)).toArrayLike(Buffer, "le", 8)
            ],
            PROGRAM_ID
        );

        return lockup;
    }

    static deriveAsset(mint: PublicKey) {
        const [asset] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("asset"),
                mint.toBuffer()
            ],
            PROGRAM_ID
        );

        return asset;
    }

    static deriveAssetPool(
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

    static deriveLockupColdVault(lockup: PublicKey, assetMint: PublicKey) {
        const [coldVault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("cold_vault"),
                lockup.toBuffer(),
                assetMint.toBuffer()
            ],
            PROGRAM_ID
        );

        return coldVault;
    }

    async getLockupColdVault(address: PublicKey) {
        return getAccount(this.connection, address, "confirmed");
    }

    static deriveLockupHotVault(lockup: PublicKey, assetMint: PublicKey) {
        const [hotVault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("hot_vault"),
                lockup.toBuffer(),
                assetMint.toBuffer()
            ],
            PROGRAM_ID
        );

        return hotVault;
    }

    async getLockupHotVault(address: PublicKey) {
        return getAccount(this.connection, address, "confirmed");
    }

    static deriveLockupCooldownVault(lockup: PublicKey, receiptMint: PublicKey) {
        const [cooldownVault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("cooldown_vault"),
                lockup.toBuffer(),
                receiptMint.toBuffer()
            ],
            PROGRAM_ID
        );

        return cooldownVault;
    }

    async getLockupCooldownVault(address: PublicKey) {
        return getAccount(this.connection, address, "confirmed");
    }

    async createToken(
        signer: PublicKey,
        lockup: PublicKey,
        depositToken: PublicKey,
        withMetadata?: boolean
    ) {
        const {
            decimals
        } = await getMint(this.connection, depositToken, "confirmed");

        const tokenKeypair = Keypair.generate();
        const instructions: TransactionInstruction[] = [];

        const createAccountIx = SystemProgram.createAccount({
            lamports: await this.connection.getMinimumBalanceForRentExemption(MINT_SIZE),
            space: MINT_SIZE,
            fromPubkey: signer,
            newAccountPubkey: tokenKeypair.publicKey,
            programId: TOKEN_PROGRAM_ID
        });

        instructions.push(createAccountIx);

        const createMintIx = createInitializeMint2Instruction(
            tokenKeypair.publicKey,
            decimals,
            signer,
            null
        );

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

            const [metadata] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("metadata"),
                    METAPLEX_PROGRAM_ID.toBuffer(),
                    tokenKeypair.publicKey.toBuffer(),
                ],
                METAPLEX_PROGRAM_ID
            );

            const createMetadataIx = createCreateMetadataAccountV3Instruction(
                {
                    metadata,
                    mint: tokenKeypair.publicKey,
                    mintAuthority: signer,
                    payer: signer,
                    updateAuthority: signer,
                },
                {
                    createMetadataAccountArgsV3: {
                        data: metadataData,
                        isMutable: true,
                        collectionDetails: null
                    }
                }
            );

            instructions.push(createMetadataIx);
        }

        const setAuthorityIx = createSetAuthorityInstruction(
            tokenKeypair.publicKey,
            signer,
            AuthorityType.MintTokens,
            lockup
        );

        instructions.push(setAuthorityIx);

        return {
            instructions,
            mint: tokenKeypair
        }
    }

    async initializeLockup(
        signer: PublicKey,
        assetMint: PublicKey,
        depositCap: BN,
        minDeposit: BN,
        duration: BN,
        governanceYield?: BN,
    ) {
        const settingsData = await this.getSettingsData();
        const asset = Restaking.deriveAsset(assetMint);
        const lockup = Restaking.deriveLockup(settingsData.lockups);
        const lockupAssetVault = Restaking.deriveAssetPool("vault", lockup, assetMint);
        const rewardMint = settingsData.rewardConfig.main;
        const assetRewardPool = Restaking.deriveAssetPool("reward_pool", lockup, rewardMint);
        const admin = Restaking.deriveAdmin(signer);

        const {
            instructions: preInstructions,
            mint: receiptMint
        } = await this.createToken(
            signer,
            lockup,
            assetMint,
            false
        );

        const lockupHotVault = Restaking.deriveLockupHotVault(lockup, assetMint);
        const lockupColdVault = Restaking.deriveLockupColdVault(lockup, assetMint);
        const lockupCooldownVault = Restaking.deriveLockupCooldownVault(lockup, receiptMint.publicKey);

        const initializeLockupIx = createInitializeLockupInstruction(
            {
                settings: Restaking.deriveSettings(),
                lockup,
                admin,
                asset,
                assetMint,
                signer,
                rewardMint: settingsData.rewardConfig.main,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                poolShareReceipt: receiptMint.publicKey,
                coldWallet: settingsData.coldWallet,
                lockupCooldownVault,
            },
            {
                args: {
                    yieldMode: governanceYield ? { __kind: "Single" } : { __kind: "Dual", fields: [governanceYield] },
                    depositCap,
                    duration,
                    minDeposit
                }
            },
            PROGRAM_ID
        );

        const initializeLockupVaultsIx = createInitializeLockupVaultsInstruction(
            {
                admin,
                signer,
                lockup,
                assetMint,
                lockupColdVault,
                lockupHotVault,
                settings: Restaking.deriveSettings(),
                systemProgram: SystemProgram.programId,
                tokenProgram: TOKEN_PROGRAM_ID,
                rewardMint,
                assetRewardPool
            },
            {
                lockupId: settingsData.lockups
            },
            PROGRAM_ID
        );

        return {
            instructions: [ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 200_000 }), ...preInstructions, initializeLockupIx, initializeLockupVaultsIx],
            signer: receiptMint
        };
    }

    async addAsset(
        signer: PublicKey,
        assetMint: PublicKey,
        oracle: PublicKey
    ) {
        const asset = Restaking.deriveAsset(assetMint);
        const {
            pubkey: admin
        } = await this.getAdminFromPublicKey(signer);

        return createAddAssetInstruction(
            {
                assetMint,
                asset,
                admin,
                oracle,
                signer,
                settings: Restaking.deriveSettings(),
            },
            PROGRAM_ID
        );
    }

    static deriveRewardBoost(
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

    async getReceiptToDepositsExchangeRateBps(
        lockup: PublicKey,
    ) {
        const {
            assetMint,
            receiptMint
        } = await Lockup.fromAccountAddress(this.connection, lockup);

        const lockupHotVault = Restaking.deriveLockupHotVault(lockup, assetMint);
        const lockupColdVault = Restaking.deriveLockupColdVault(lockup, assetMint);

        const {
            amount: hotAmount
        } = await this.getLockupHotVault(lockupHotVault);

        const {
            amount: coldAmount
        } = await this.getLockupColdVault(lockupColdVault);

        const {
            supply: receiptSupply
        } = await getMint(this.connection, receiptMint);

        const totalDeposit = new BN(hotAmount.toString()).add(new BN(coldAmount.toString()));
        const exchangeRateBps = totalDeposit
            .muln(10_000)
            .div(new BN(receiptSupply.toString()));

        return exchangeRateBps;
    }

    async boostRewards(
        signer: PublicKey,
        lockupId: BN,
        minUsdValue: BN,
        boostBps: BN
    ) {
        const adminDatas = await this.getAdminFromPublicKey(signer);
        const admin = Restaking.deriveAdmin(adminDatas[0].account.index);
        const lockup = Restaking.deriveLockup(lockupId);

        const {
            rewardBoosts
        } = await this.getLockup(lockup);

        const rewardBoost = Restaking.deriveRewardBoost(lockup, rewardBoosts);

        return createBoostRewardsInstruction(
            {
                admin,
                lockup,
                settings: Restaking.deriveSettings(),
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
        const lockup = Restaking.deriveLockup(lockupId);
        const {
            rewardConfig: {
                main: rewardMint
            }
        } = await this.getSettingsData();

        const {
            receiptMint
        } = await this.getLockup(lockup);

        const lockupCooldownVault = Restaking.deriveLockupCooldownVault(lockup, receiptMint);

        const callerRewardAta = getAssociatedTokenAddressSync(
            rewardMint,
            signer,
            false
        );

        return createDepositRewardsInstruction(
            {
                caller: signer,
                settings: Restaking.deriveSettings(),
                rewardMint,
                lockup,
                callerRewardAta,
                assetRewardPool: Restaking.deriveAssetPool("reward_pool", lockup, rewardMint),
                lockupCooldownVault,
                receiptTokenMint: receiptMint
            },
            {
                args: {
                    lockupId,
                    amount
                }
            }
        );
    }

    static deriveIntent(deposit: PublicKey) {
        const [intent] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("intent"),
                deposit.toBuffer()
            ],
            PROGRAM_ID
        );

        return intent;
    }

    static deriveDeposit(
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

    static deriveDepositReceiptVault(deposit: PublicKey, receiptToken: PublicKey) {
        const [depositReceiptVault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("deposit_receipt_vault"),
                deposit.toBuffer(),
                receiptToken.toBuffer()
            ],
            PROGRAM_ID
        );

        return depositReceiptVault;
    }

    async getDepositReceiptVault(address: PublicKey) {
        return getAccount(this.connection, address, "confirmed");
    }

    async createIntent(
        lockupId: BN | number,
        depositId: BN | number,
        amount: BN
    ) {
        const settings = Restaking.deriveSettings();
        const {
            rewardConfig: {
                main: rewardMint
            }
        } = await Settings.fromAccountAddress(
            this.connection,
            settings
        );

        const lockup = Restaking.deriveLockup(lockupId);
        const {
            assetMint,
            receiptMint,
        } = await this.getLockup(lockup);

        const asset = Restaking.deriveAsset(assetMint);

        const deposit = Restaking.deriveDeposit(lockup, depositId);
        const {
            user,
        } = await this.getDeposit(deposit);

        const depositReceiptVault = Restaking.deriveDepositReceiptVault(deposit, receiptMint);
        const {
            amount: receiptAmount
        } = await this.getDepositReceiptVault(depositReceiptVault);

        const exchangeRateBps = await this.getReceiptToDepositsExchangeRateBps(lockup);
        const depositAmount = new BN(receiptAmount.toString())
            .mul(exchangeRateBps)
            .divn(10_000);

        if (amount.gt(depositAmount))
            throw new Error("Cannot withdraw more funds than deposited");

        const intent = Restaking.deriveIntent(deposit);

        const lockupAssetVault = Restaking.deriveAssetPool("vault", lockup, assetMint);
        const userAssetAta = getAssociatedTokenAddressSync(
            assetMint,
            user
        );
        const userRewardAta = getAssociatedTokenAddressSync(
            rewardMint,
            user
        );
        const depositReceiptTokenAccount = getAssociatedTokenAddressSync(
            receiptMint,
            deposit,
            true
        );

        const cooldown = Restaking.deriveCooldown(deposit);

        const lockupHotVault = Restaking.deriveLockupHotVault(lockup, assetMint);
        const lockupColdVault = Restaking.deriveLockupColdVault(lockup, assetMint);
        const assetRewardPool = Restaking.deriveAssetPool("reward_pool", lockup, assetMint);
        const lockupCooldownVault = Restaking.deriveLockupCooldownVault(lockup, assetMint);

        return createWithdrawInstruction(
            {
                lockup,
                assetMint,
                intent,
                settings,
                user,
                deposit,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
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
            },
            {
                args: {
                    lockupId,
                    depositId,
                }
            }
        )
    }

    async getIntent(intent: PublicKey) {
        return Intent.fromAccountAddress(this.connection, intent);
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
                settings: Restaking.deriveSettings(),
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

    async slash(
        amount: BN,
        signer: PublicKey,
        lockupId: BN,
        destination: PublicKey,
    ) {
        const settings = Restaking.deriveSettings();
        const admin = Restaking.deriveAdmin(signer);
        const lockup = Restaking.deriveLockup(lockupId);

        const {
            assetMint
        } = await this.getLockup(lockup);

        const lockupHotVault = Restaking.deriveLockupHotVault(lockup, assetMint);
        const lockupColdVault = Restaking.deriveLockupColdVault(lockup, assetMint);

        const ix = createSlashInstruction(
            {
                signer,
                admin,
                settings,
                lockup,
                assetMint,
                destination,
                tokenProgram: TOKEN_PROGRAM_ID,
                lockupColdVault,
                lockupHotVault
            },
            {
                args: {
                    lockupId,
                    amount
                }
            },
            PROGRAM_ID
        );

        return ix;
    }

    async getAsset(asset: PublicKey) {
        return Asset.fromAccountAddress(
            this.connection,
            asset
        );
    }

    async restake(signer: PublicKey, amount: BN, lockupId: BN) {
        const settings = Restaking.deriveSettings();
        const {
            coldWallet
        } = await this.getSettingsData();
        const lockup = Restaking.deriveLockup(lockupId);
        const {
            assetMint,
            deposits,
            receiptMint
        } = await this.getLockup(lockup);

        const asset = Restaking.deriveAsset(assetMint);
        const { oracle: { fields: [oracleAddress] } } = await this.getAsset(asset);

        const coldWalletVault = getAssociatedTokenAddressSync(
            assetMint,
            coldWallet
        );
        const userAssetAta = getAssociatedTokenAddressSync(
            assetMint,
            signer
        );
        const lockupAssetVault = Restaking.deriveAssetPool("vault", lockup, assetMint);

        const deposit = Restaking.deriveDeposit(lockup, deposits);
        const depositReceiptTokenAccount = getAssociatedTokenAddressSync(
            receiptMint,
            deposit,
            true
        );
        const lockupHotVault = Restaking.deriveLockupHotVault(lockup, assetMint);
        const lockupColdVault = Restaking.deriveLockupColdVault(lockup, assetMint);

        return createRestakeInstruction(
            {
                lockup,
                asset,
                assetMint,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                deposit,
                settings,
                user: signer,
                oracle: oracleAddress,
                userAssetAta,
                receiptTokenMint: receiptMint,
                lockupColdVault,
                lockupHotVault,
                depositReceiptTokenAccount
            },
            {
                args: {
                    lockupId,
                    amount
                }
            }
        );
    }

    static deriveCooldown(deposit: PublicKey) {
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
        mode: "ExactIn" | "ExactOut",
        amount: BN,
        rewardBoostId?: BN | number
    ) {
        const lockup = Restaking.deriveLockup(lockupId);
        const {
            assetMint,
            receiptMint
        } = await Lockup.fromAccountAddress(this.connection, lockup);

        const {
            rewardConfig: {
                main: rewardAsset
            }
        } = await this.getSettingsData();
        const asset = Restaking.deriveAsset(assetMint);
        const deposit = Restaking.deriveDeposit(lockup, depositId);

        const assetRewardPool = Restaking.deriveAssetPool("reward_pool", lockup, rewardAsset);
        const lockupAssetVault = Restaking.deriveAssetPool("vault", lockup, assetMint);

        const userAssetAta = getAssociatedTokenAddressSync(
            assetMint,
            signer
        );

        const userRewardAta = getAssociatedTokenAddressSync(
            rewardAsset,
            signer
        );

        const lockupHotVault = Restaking.deriveLockupHotVault(lockup, assetMint);
        const lockupColdVault = Restaking.deriveLockupColdVault(lockup, assetMint);
        const lockupCooldownVault = Restaking.deriveLockupCooldownVault(lockup, receiptMint);
        const depositReceiptTokenAccount = getAssociatedTokenAddressSync(
            receiptMint,
            deposit,
            true
        );

        return createRequestWithdrawalInstruction(
            {
                user: signer,
                asset,
                assetMint,
                lockup,
                deposit,
                settings: Restaking.deriveSettings(),
                assetRewardPool,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
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
            },
            {
                args: {
                    lockupId,
                    depositId,
                    rewardBoostId,
                    mode: {
                        __kind: mode,
                        fields: [amount]
                    }
                }
            }
        );
    }

    async requestWithdrawalWithAutoBoostDetection(
        signer: PublicKey,
        depositId: BN | number,
        lockupId: BN | number,
        mode: "ExactIn" | "ExactOut",
        amount: BN
    ) {
        const lockup = Restaking.deriveLockup(lockupId);
        const deposit = Restaking.deriveDeposit(lockup, depositId);
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
            mode,
            amount,
            preferredRewardBoost?.index
        );
    }

    async withdrawCooldown(signer: PublicKey, lockupId: BN | number, depositId: BN | number) {
        const {
            rewardConfig: {
                main: rewardMint
            }
        } = await this.getSettingsData();
        const lockup = Restaking.deriveLockup(lockupId);
        const deposit = Restaking.deriveDeposit(lockup, depositId);
        const cooldown = Restaking.deriveCooldown(deposit);
        const cooldownDatas = await this.getCooldownsByDeposit(depositId);
        const {
            account: {
                unlockTs
            }
        } = cooldownDatas[0];

        if (!(new BN(Date.now()).gte(new BN(unlockTs)))) throw new Error("Funds still in cooldown.");

        const {
            assetMint,
            receiptMint
        } = await this.getLockup(lockup);

        const asset = Restaking.deriveAsset(assetMint);

        const userAssetAta = getAssociatedTokenAddressSync(
            assetMint,
            signer,
        );

        const userRewardAta = getAssociatedTokenAddressSync(
            rewardMint,
            signer,
        );

        const lockupAssetVault = Restaking.deriveAssetPool("vault", lockup, assetMint);
        const assetRewardPool = Restaking.deriveAssetPool("reward_pool", lockup, rewardMint);
        const lockupCooldownVault = Restaking.deriveLockupCooldownVault(lockup, receiptMint);
        const lockupColdVault = Restaking.deriveLockupColdVault(lockup, assetMint);
        const lockupHotVault = Restaking.deriveLockupHotVault(lockup, assetMint);
        const depositReceiptTokenAccount = getAssociatedTokenAddressSync(
            receiptMint,
            deposit,
            true
        );

        return createWithdrawInstruction(
            {
                lockup,
                deposit,
                asset,
                assetMint,
                tokenProgram: TOKEN_PROGRAM_ID,
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
                systemProgram: SystemProgram.programId
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