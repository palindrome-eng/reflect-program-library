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
    Asset,
    assetDiscriminator,
    Cooldown,
    cooldownDiscriminator,
    createAddAssetInstruction,
    createRequestWithdrawalInstruction,
    createSlashInstruction,
    createWithdrawInstruction,
    PROGRAM_ID,
    Settings,
    UserPermissions,
    KillSwitch,
    LiquidityPool,
    liquidityPoolDiscriminator,
    userPermissionsDiscriminator,
    createInitializeRlpInstruction,
    createInitializeLpInstruction,
    InitializeLpInstructionArgs,
    InitializeLiquidityPoolArgs,
    createDepositRewardsInstruction,
    createRestakeInstruction,
    AccessLevel
} from "../generated";
import BN from "bn.js";
import {
    Account,
    ASSOCIATED_TOKEN_PROGRAM_ID,
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
import { AccountWithPubkey } from "../types";

type InsuranceFundAccount = Asset | UserPermissions | Cooldown | KillSwitch | LiquidityPool | Settings;

export class Restaking {
    private connection: Connection;
    private settings: Settings;
    private liquidityPools: AccountWithPubkey<LiquidityPool>[];
    private assets: AccountWithPubkey<Asset>[];

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

    async load() {
        this.settings = await this.getSettingsData();
        this.liquidityPools = await this.getLiquidityPools();
        this.assets = await this.getAssets();
    }

    async getLiquidityPools() {
        const lps = await LiquidityPool
            .gpaBuilder()
            .addFilter("accountDiscriminator", liquidityPoolDiscriminator)
            .run(this.connection);

        return lps.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer<LiquidityPool>(LiquidityPool, account) }));
    }

    async getLiquidityPoolData(liquidityPoolId: number) {
        const liquidityPool = await LiquidityPool.fromAccountAddress(
            this.connection, 
            Restaking.deriveLiquidityPool(liquidityPoolId)
        );

        return liquidityPool;
    }

    async getAssets(): Promise<AccountWithPubkey<Asset>[]> {
        if (this.assets?.length > 0) {
            return this.assets;
        }

        const assets = await Asset
            .gpaBuilder()
            .addFilter("accountDiscriminator", assetDiscriminator)
            .run(this.connection);

        const result: AccountWithPubkey<Asset>[] = assets
            .map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer<Asset>(Asset, account) }))
            .sort((a: AccountWithPubkey<Asset>, b: AccountWithPubkey<Asset>) => a.account.index - b.account.index);

        this.assets = result;
        return result;
    }

    async getCooldowns(): Promise<AccountWithPubkey<Cooldown>[]> {
        const cooldowns = await Cooldown
            .gpaBuilder()
            .addFilter("accountDiscriminator", cooldownDiscriminator)
            .run(this.connection);

        return cooldowns.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer<Cooldown>(Cooldown, account) }));
    }

    async getCooldownsByUser(user: PublicKey): Promise<AccountWithPubkey<Cooldown>[]> {
        const cooldowns = await Cooldown
            .gpaBuilder()
            .addFilter("accountDiscriminator", cooldownDiscriminator)
            .addFilter("authority", user)
            .run(this.connection);

        return cooldowns.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer<Cooldown>(Cooldown, account) }));
    }

    async getUserPermissions(): Promise<AccountWithPubkey<UserPermissions>[]> {
        const permissions = await UserPermissions
            .gpaBuilder()
            .addFilter("accountDiscriminator", userPermissionsDiscriminator)
            .run(this.connection);

        return permissions.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer<UserPermissions>(UserPermissions, account) }));
    }

    async getUserPermissionsFromPublicKey(address: PublicKey): Promise<AccountWithPubkey<UserPermissions>[]> {
        const permissions = await UserPermissions
            .gpaBuilder()
            .addFilter("accountDiscriminator", userPermissionsDiscriminator)
            .addFilter("authority", address)
            .run(this.connection);

        return permissions.map(({ account, pubkey }) => ({ pubkey, account: this.accountFromBuffer<UserPermissions>(UserPermissions, account) }));
    }

    static deriveUserPermissions(
        address: PublicKey
    ) {
        const [permissions] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("permissions"),
                address.toBuffer()
            ],
            PROGRAM_ID
        );

        return permissions;
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

    static deriveLiquidityPool(liquidityPoolId: number) {
        const [liquidityPool] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("liquidity_pool"),
                new BN(liquidityPoolId).toArrayLike(Buffer, "le", 1)
            ],
            PROGRAM_ID
        );

        return liquidityPool;
    }

    async initializeRlp(signer: PublicKey) {
        return createInitializeRlpInstruction(
            {
                permissions: Restaking.deriveUserPermissions(signer),
                settings: Restaking.deriveSettings(),
                signer: signer,
                systemProgram: SystemProgram.programId
            },
            PROGRAM_ID
        );
    }

    async getSettingsData() {
        return Settings.fromAccountAddress(this.connection, Restaking.deriveSettings());
    }

    static deriveAsset(assetId: number) {
        const [asset] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("asset"),
                new BN(assetId).toArrayLike(Buffer, "le", 1)
            ],
            PROGRAM_ID
        );

        return asset;
    }

    async findAssetFromMint(mint: PublicKey) {
        const assets = await this.getAssets();
        
        const {
            pubkey
        } = assets.find(({ account }) => account.mint.equals(mint));

        return pubkey;
    }

    async createToken(
        signer: PublicKey,
        liquidityPool: PublicKey,
        withMetadata?: boolean
    ) {
        const decimals = 9;

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
                name: "Reflect Liquidity Pool",
                symbol: "RLP",
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
            liquidityPool
        );

        instructions.push(setAuthorityIx);

        return {
            instructions,
            mint: tokenKeypair
        }
    }

    async initializeLiquidityPool(
        signer: PublicKey,
        args: InitializeLiquidityPoolArgs
    ) {
        
        const {
            liquidityPools
        } = await this.getSettingsData();

        const liquidityPool = Restaking.deriveLiquidityPool(liquidityPools);

        const {
            instructions,
            mint
        } = await this.createToken(signer, liquidityPool, false);

        const ix = createInitializeLpInstruction(
            {
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                liquidityPool,
                lpTokenMint: mint.publicKey,
                permissions: Restaking.deriveUserPermissions(signer),
                settings: Restaking.deriveSettings(),
                signer,
                systemProgram: SystemProgram.programId,
                tokenProgram: TOKEN_PROGRAM_ID,
            },
            {
                args
            },
            PROGRAM_ID
        );

        return {
            instructions: [
                ...instructions,
                ix,
            ],
            signer: mint
        }
    }

    async addAsset(
        signer: PublicKey,
        assetMint: PublicKey,
        oracle: PublicKey,
        accessLevel: AccessLevel
    ) {
        const {
            account: assetData,
            pubkey: asset
        } = this.assets.find(({ account }) => account.mint === assetMint);
        const permissions = Restaking.deriveUserPermissions(signer);

        return createAddAssetInstruction(
            {
                assetMint,
                asset,
                oracle,
                signer,
                settings: Restaking.deriveSettings(),
                admin: permissions,
            },
            {
                args: {
                    accessLevel
                }
            },
            PROGRAM_ID
        );
    }

    async depositRewards(
        liquidityPoolId: number,
        amount: BN,
        mint: PublicKey,
        signer: PublicKey
    ) {
        const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);
        const permissions = Restaking.deriveUserPermissions(signer);

        const signerAssetTokenAccount = getAssociatedTokenAddressSync(
            mint,
            signer,
            false
        );

        const assetPool = getAssociatedTokenAddressSync(
            mint,
            liquidityPool,
            false
        );

        const {
            account: {
                index: assetId
            },
            pubkey: asset
        } = this.assets.find(({ account }) => account.mint === mint);

        return createDepositRewardsInstruction(
            {
                signer,
                settings: Restaking.deriveSettings(),
                asset,
                assetMint: mint,
                assetPool,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                liquidityPool,
                permissions,
                signerAssetTokenAccount,
                systemProgram: SystemProgram.programId,
                tokenProgram: TOKEN_PROGRAM_ID
            },
            {
                args: {
                    amount,
                    assetId
                }
            },
            PROGRAM_ID
        );
    }

    async slash(
        mint: PublicKey,
        amount: BN,
        signer: PublicKey,
        liquidityPoolId: number,
        destination: PublicKey,
    ) {
        const settings = Restaking.deriveSettings();
        const permissions = Restaking.deriveUserPermissions(signer);
        const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);

        const {
            account: {
                index: assetId
            },
            pubkey: asset
        } = this.assets.find(({ account }) => account.mint === mint);

        const liquidityPoolTokenAccount = getAssociatedTokenAddressSync(
            mint,
            liquidityPool,
            false
        );

        const ix = createSlashInstruction(
            {
                signer,
                settings,
                destination,
                tokenProgram: TOKEN_PROGRAM_ID,
                permissions,
                asset,
                liquidityPool,
                liquidityPoolTokenAccount,
                mint
            },
            {
                args: {
                    liquidityPoolId,
                    amount,
                    assetId
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

    async restake(signer: PublicKey, amount: BN, mint: PublicKey, liquidityPoolId: number, minLpTokens?: BN) {
        const settings = Restaking.deriveSettings();
        
        const {
            account: {
                index: assetId,
                oracle: {
                    fields: [oracleAddress]
                }
            },
            pubkey: asset
        } = this.assets.find(({ account }) => account.mint === mint);

        const {
            pubkey: liquidityPool,
            account: {
                lpToken
            }
        } = this.liquidityPools.find(({ account }) => account.index === liquidityPoolId);

        const poolAssetAccount = getAssociatedTokenAddressSync(
            mint,
            liquidityPool,
            true
        );

        const userAssetAccount = getAssociatedTokenAddressSync(
            mint,
            signer,
            true
        );

        const userLpAccount = getAssociatedTokenAddressSync(
            lpToken,
            signer,
            true
        );

        return createRestakeInstruction(
            {
                liquidityPool,
                asset,
                assetMint: mint,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                settings,
                oracle: oracleAddress,
                lpToken,
                poolAssetAccount,
                signer,
                userAssetAccount,
                userLpAccount,
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID
            },
            {
                args: {
                    amount,
                    liquidityPoolIndex: liquidityPoolId,
                    minLpTokens: minLpTokens ?? null,
                    assetId
                }
            }
        );
    }

    static deriveCooldown(deposit: number | BN) {
        const [cooldown] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("cooldown"),
                new BN(deposit).toArrayLike(Buffer, "le", 8)
            ],
            PROGRAM_ID
        );

        return cooldown;
    }

    async requestWithdrawal(
        signer: PublicKey,
        liquidityPoolId: number,
        amount: BN,
    ) {
        const {
            pubkey: liquidityPool,
            account: {
                lpToken,
                cooldowns
            }
        } = this.liquidityPools.find(({ account }) => account.index === liquidityPoolId);

        const cooldown = Restaking.deriveCooldown(cooldowns);
        const cooldownLpTokenAccount = getAssociatedTokenAddressSync(
            lpToken,
            cooldown,
            true
        );

        const signerLpTokenAccount = getAssociatedTokenAddressSync(
            lpToken,
            signer,
            true
        );

        return createRequestWithdrawalInstruction(
            {
                associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
                cooldown,
                cooldownLpTokenAccount,
                liquidityPool,
                lpTokenMint: lpToken,
                settings: Restaking.deriveSettings(),
                signer,
                signerLpTokenAccount,
                systemProgram: SystemProgram.programId,
                tokenProgram: TOKEN_PROGRAM_ID
            },
            {
                args: {
                    amount,
                    liquidityPoolId
                }
            }
        );
    }

    async withdraw(signer: PublicKey, liquidityPoolId: number, cooldownId: number | BN) {

        const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);
        const cooldown = Restaking.deriveCooldown(cooldownId);
        const {
            lpToken
        } = await this.getLiquidityPoolData(liquidityPoolId);
        
        const cooldownLpTokenAccount = getAssociatedTokenAddressSync(
            lpToken,
            cooldown,
            true
        );

        return createWithdrawInstruction(
            {
                cooldown,
                cooldownLpTokenAccount,
                liquidityPool,
                lpTokenMint: lpToken,
                settings: Restaking.deriveSettings(),
                signer: signer,
                systemProgram: SystemProgram.programId,
                tokenProgram: TOKEN_PROGRAM_ID
            },
            {
                args: {
                    cooldownId,
                    liquidityPoolId
                }
            }
        );
    }
}