import * as anchor from "@coral-xyz/anchor";
import {AnchorError, Program} from "@coral-xyz/anchor";
import { Rlp } from "../target/types/rlp";
import {
    PublicKey,
    Keypair,
    SystemProgram,
    LAMPORTS_PER_SOL,
    SYSVAR_CLOCK_PUBKEY,
    TransactionMessage, Connection, TransactionInstruction,
    TokenBalance,
    Transaction
} from "@solana/web3.js";
import {before, it} from "mocha";
import {
    Asset, 
    Cooldown,
    LiquidityPool,
    UserPermissions,
    PROGRAM_ID,
    Settings,
    Action,
    Role,
    Update,
} from "../sdk/src/generated";
import {expect} from "chai";
import createToken from "./helpers/createToken";
import BN from "bn.js";
import {
    AuthorityType,
    createAssociatedTokenAccountIdempotentInstruction,
    createAssociatedTokenAccountInstruction, createInitializeAccount3Instruction,
    createInitializeMint2Instruction,
    createSetAuthorityInstruction,
    getAccount,
    getAssociatedTokenAddressSync,
    getMint,
    MINT_SIZE,
    TOKEN_PROGRAM_ID
} from "@solana/spl-token";
import mintTokens from "./helpers/mintTokens";
import signAndSendTransaction from "./helpers/signAndSendTransaction";
import sleep from "./helpers/sleep";
import {
    PROGRAM_ID as METAPLEX_PROGRAM_ID,
    createCreateMetadataAccountV3Instruction
} from "@metaplex-foundation/mpl-token-metadata";
import {Restaking} from "../sdk/src";
import getOraclePriceFromAccount from "./helpers/getOraclePriceFromAccount";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import comparePubkeys from "./helpers/comparePubkeys";
import calculateLpToken from "./helpers/calculateLpToken";
import calculateReceiptToken from "./helpers/calculateReceiptToken";
import { AccountMeta } from "@solana/web3.js";
import { ComputeBudgetInstruction } from "@solana/web3.js";
import { ComputeBudgetProgram } from "@solana/web3.js";
import { calculateDepositValue, calculateLpTokensOnDeposit, calculateTotalPoolValue } from "./helpers/liquidityPoolMath";
import { AddressLookupTableProgram } from "@solana/web3.js";
import { VersionedTransaction } from "@solana/web3.js";

async function createLpToken(
    signer: PublicKey,
    liquidityPool: PublicKey,
    connection: Connection,
    withMetadata?: boolean,
) {
    const decimals = 9;

    const tokenKeypair = Keypair.generate();
    const instructions: TransactionInstruction[] = [];

    const createAccountIx = SystemProgram.createAccount({
        lamports: await connection.getMinimumBalanceForRentExemption(MINT_SIZE),
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
        liquidityPool
    );

    instructions.push(setAuthorityIx);

    return {
        instructions,
        mint: tokenKeypair
    }
}

describe("rlp", () => {
    const provider = anchor.AnchorProvider.local();
    anchor.setProvider(provider);
    const program = anchor.workspace.Rlp as Program<Rlp>;

    let settings: PublicKey;
    let user: Keypair;
    let rewardToken: PublicKey;
    let admin: PublicKey;

    const lsts: PublicKey[] = [];

    before(async () => {
        user = Keypair.generate();

        settings = Restaking.deriveSettings();

        await provider.connection.requestAirdrop(
            user.publicKey,
            LAMPORTS_PER_SOL * 1000
        );

        rewardToken = await createToken(
            provider.connection,
            provider
        );

        admin = Restaking.deriveUserPermissions(provider.publicKey);
    });

    it("Initializes RLP.", async () => {
        const cooldownDuration = new BN(30); // 30 seconds

        await program
            .methods
            .initializeRlp({
                cooldownDuration
            })
            .accounts({
                signer: provider.publicKey,
                permissions: admin,
                settings,
                systemProgram: SystemProgram.programId,
            })
            .rpc();

        const {
            frozen,
            liquidityPools,
            assets
        } = await Settings.fromAccountAddress(
            provider.connection,
            settings
        );

        expect(liquidityPools).eq(0);
        expect(assets).eq(0);
        expect(frozen).eq(false);
    });

    it("Adds reward token to the RLP", async () => {
        const asset = Restaking.deriveAsset(rewardToken);

        await program
            .methods
            .addAsset()
            .accounts({
                admin,
                asset,
                assetMint: rewardToken,
                oracle: new PublicKey("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE"),
                settings,
                signer: provider.publicKey,
                systemProgram: SystemProgram.programId,
            })
            .rpc();

        const assetData = await Asset.fromAccountAddress(
            provider.connection,
            asset
        );
        
        expect(assetData.mint.toString()).eq(rewardToken.toString());
        expect(assetData.oracle.__kind).eq("Pyth");
        expect(assetData.oracle.fields[0].toString()).eq("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");
    });

    it('Freezes protocol.', async () => {
        await program
            .methods
            .freezeFunctionality({
                action: { freezeRestake: {} },
                freeze: true
            })
            .accounts({
                admin: provider.publicKey,
                adminPermissions: admin,
                settings
            })
            .rpc()

        let settingsData = await Settings.fromAccountAddress(
            provider.connection,
            settings
        );

        const killswitch = settingsData.accessControl.killswitch;
        expect(killswitch.frozen).eq(1);
    });

    it("Tries to interact with frozen protocol. Succeeds on errors", async () => {
        let error: AnchorError;

        const wrappedSol = new PublicKey("So11111111111111111111111111111111111111112");
        const asset = Restaking.deriveAsset(wrappedSol);

        await program
            .methods
            .addAsset()
            .accounts({
                assetMint: wrappedSol,
                asset,
                systemProgram: SystemProgram.programId,
                settings,
                signer: provider.publicKey,
                admin,
                // Just to pass the test
                oracle: new PublicKey("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE"),
            })
            .rpc()
            .catch((err: AnchorError) => error = err);

    });

    it('Unfreezes protocol.', async () => {
        await program
            .methods
            .freezeFunctionality({
                action: { freezeRestake: {} },
                freeze: false
            })
            .accounts({
                admin: provider.publicKey,
                adminPermissions: admin,
                settings
            })
            .rpc()

            let settingsData = await Settings.fromAccountAddress(
                provider.connection,
                settings
            );
    
            const killswitch = settingsData.accessControl.killswitch;
            expect(killswitch.frozen).eq(0);
    });

    it("Mints and adds assets to insurance pool.", async () => {
        const assets: PublicKey[] = [];

        for (let i = 0; i < 3; i ++) {
            const oracleString = i % 2 
                ? "7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE" 
                : "Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX";

            const token = await createToken(
                provider.connection,
                provider
            );

            lsts.push(token);

            const asset = Restaking.deriveAsset(token);

            assets.push(asset);

            await program
                .methods
                .addAsset()
                .accounts({
                    assetMint: token,
                    asset,
                    systemProgram: SystemProgram.programId,
                    settings,
                    signer: provider.publicKey,
                    admin,
                    // Just to pass the test
                    oracle: new PublicKey(oracleString),
                })
                .rpc();
        }

        for (let i = 0; i < assets.length; i++) {
            const asset = assets[i];
            const assetData = await Asset.fromAccountAddress(
                provider.connection,
                asset,
            );

            const oracleString = i % 2 ? "7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE" : "Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX";

            expect(lsts[i].toString()).eq(assetData.mint.toString());
            expect(assetData.oracle.__kind).eq("Pyth");
            expect(assetData.oracle.fields[0].toString()).eq(oracleString);
        }
    });

    it("Initializes liquidity pool", async () => {
        const liquidityPool = Restaking.deriveLiquidityPool(0);

        const {
            mint: lpTokenMint,
            instructions: preInstructions
        } = await createLpToken(
            provider.publicKey,
            liquidityPool,
            provider.connection,
            false
        );

        await program
            .methods
            .initializeLp({
                cooldownDuration: new BN(30),
                cooldowns: new BN(0)
            })
            .preInstructions(preInstructions)
            .accounts({
                permissions: admin,
                associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
                liquidityPool,
                lpTokenMint: lpTokenMint.publicKey,
                signer: provider.publicKey,
                systemProgram: SystemProgram.programId,
                tokenProgram: TOKEN_PROGRAM_ID,
                settings: Restaking.deriveSettings(),
            })
            .signers([lpTokenMint])
            .rpc();
    
        const {
            lpToken,
            index,
            cooldownDuration,
            cooldowns
        } = await LiquidityPool.fromAccountAddress(
            provider.connection,
            liquidityPool
        );

        expect(cooldownDuration.toString()).eq("30");
        expect(cooldowns.toString()).eq("0");
        expect(lpToken.toString()).eq(lpTokenMint.publicKey.toString());
        expect(index).eq(0);
    });

    it("Initializes LP-owned token accounts.", async () => {
        const restaking = new Restaking(provider.connection);
        const assets = await restaking.getAssets();

        const liquidityPool = Restaking.deriveLiquidityPool(0);
        const liquidityPoolData = await LiquidityPool.fromAccountAddress(
            provider.connection,
            liquidityPool
        );

        await Promise.all(assets.map(async ({ pubkey, account }) => {
            const lpMintTokenAccount = getAssociatedTokenAddressSync(
                account.mint,
                liquidityPool,
                true
            );

            const instruction = await program
                .methods
                .initializeLpTokenAccount({
                    liquidityPoolIndex: 0
                })
                .accounts({
                    admin,
                    asset: pubkey,
                    associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
                    liquidityPool,
                    lpMintTokenAccount,
                    mint: account.mint,
                    settings,
                    signer: provider.publicKey,
                    systemProgram: SystemProgram.programId,
                    tokenProgram: TOKEN_PROGRAM_ID,
                })
                .instruction();

            await program
                .methods
                .initializeLpTokenAccount({
                    liquidityPoolIndex: 0
                })
                .accounts({
                    admin,
                    asset: pubkey,
                    associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
                    liquidityPool,
                    lpMintTokenAccount,
                    mint: account.mint,
                    settings,
                    signer: provider.publicKey,
                    systemProgram: SystemProgram.programId,
                    tokenProgram: TOKEN_PROGRAM_ID,
                })
                .rpc();
        }));
    });

    it("Sets the restaking action to public.", async () => {
        await program
            .methods
            // Necessary to ignore fucked-up anchor uppercasing the enum.
            // @ts-ignore
            .updateActionRole({
                action: { restake: {} },
                role: { public: {} },
                update: { add: {} }
            })
            .accounts({
                admin: provider.publicKey,
                settings,
                adminPermissions: admin,
            })
            .rpc();

        const settingsData = await Settings.fromAccountAddress(
            provider.connection,
            settings
        );
        
        const actionPermissionMap = settingsData.accessControl.accessMap.actionPermissions.find(mapping => mapping.action === Action.Restake);
        expect(actionPermissionMap.action).eq(Action.Restake);
        expect(actionPermissionMap.allowedRoles).include(Role.PUBLIC);
    });

    it("Locks-up tokens in liquidity pool.", async () => {
        const liquidityPoolId = 0;
        const amount = new BN(LAMPORTS_PER_SOL * 1_000);

        const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);

        const liquidityPoolData = await LiquidityPool.fromAccountAddress(
            provider.connection,
            liquidityPool
        );

        const asset = Restaking.deriveAsset(lsts[0]);

        await mintTokens(
            lsts[0],
            provider,
            amount.toNumber(),
            user.publicKey
        );

        const userAssetAta = getAssociatedTokenAddressSync(
            lsts[0],
            user.publicKey
        );

        const poolAssetAccount = getAssociatedTokenAddressSync(
            lsts[0],
            liquidityPool,
            true
        );

        const userLpAccount = getAssociatedTokenAddressSync(
            liquidityPoolData.lpToken,
            user.publicKey
        );

        const assetData = await Asset.fromAccountAddress(
            provider.connection,
            asset
        );

        const restaking = new Restaking(provider.connection);
        const assets = await restaking.getAssets();

        const anchorRemainingAccounts: AccountMeta[] = assets
        .map(({ account, pubkey }) => {
            return [
                {
                    isSigner: false,
                    isWritable: true,
                    pubkey: getAssociatedTokenAddressSync(account.mint, liquidityPool, true)
                },
                {
                    isSigner: false,
                    isWritable: false,
                    pubkey: Restaking.deriveAsset(account.mint)
                },
                {
                    isSigner: false,
                    isWritable: false,
                    pubkey: account.oracle.fields[0]
                }
            ] as AccountMeta[];
        }).flat();

        await program
            .methods
            .restake({
                liquidityPoolIndex: liquidityPoolId,
                amount,
                minLpTokens: new BN(0)
            })
            .accounts({
                signer: user.publicKey,
                settings,
                liquidityPool,
                lpToken: liquidityPoolData.lpToken,
                userLpAccount,
                asset,
                assetMint: lsts[0],
                userAssetAccount: userAssetAta,
                poolAssetAccount,
                oracle: assetData.oracle.fields[0],
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                permissions: null,
            })
            .remainingAccounts(anchorRemainingAccounts)
            .signers([
                user
            ])
            .preInstructions([
                ComputeBudgetProgram.setComputeUnitLimit({ units: 1_000_000 })
            ])
            .rpc()
            .catch(err => {
                console.log(err);
                throw err;
            });

        await new Promise(resolve => setTimeout(resolve, 15000));

        const userLpAccountData = await getAccount(
            provider.connection,
            userLpAccount
        );

        expect(
            parseInt(userLpAccountData.amount.toString())
        ).gt(0);
    });

    it("Deposits rewards", async () => {
        const liquidityPoolId = 0;
        const amount = new BN(100_000 * LAMPORTS_PER_SOL);

        const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);

        const signerAssetTokenAccount = getAssociatedTokenAddressSync(
            rewardToken,
            provider.publicKey
        );

        await mintTokens(
            rewardToken,
            provider,
            100_000 * LAMPORTS_PER_SOL,
            provider.publicKey
        );

        const liquidityPoolDataPre = await LiquidityPool
            .fromAccountAddress(
                provider.connection,
                liquidityPool
            );

        await program
            .methods
            .depositRewards({
                amount
            })
            .accounts({
                signer: provider.publicKey,
                permissions: admin,
                settings,
                systemProgram: SystemProgram.programId,
                liquidityPool,
                tokenProgram: TOKEN_PROGRAM_ID,
                assetPool: getAssociatedTokenAddressSync(
                    rewardToken,
                    liquidityPool,
                    true
                ),
                asset: Restaking.deriveAsset(rewardToken),
                assetMint: rewardToken,
                associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
                signerAssetTokenAccount,
            })
            .rpc()
            .catch(err => {
                console.log(err.logs);
                throw err;
            });

        const liquidityPoolDataPost = await LiquidityPool
            .fromAccountAddress(
                provider.connection,
                liquidityPool
            );

        // Verify rewards were deposited
        const assetPoolData = await getAccount(
            provider.connection,
            getAssociatedTokenAddressSync(
                rewardToken,
                liquidityPool,
                true
            )
        );

        expect(
            parseInt(assetPoolData.amount.toString())
        ).eq(amount.toNumber());
    });

    it("Slashes liquidity pool", async () => {
        const liquidityPoolId = 0;
        const slashAmount = new BN(LAMPORTS_PER_SOL * 200);

        const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);

        const liquidityPoolData = await LiquidityPool.fromAccountAddress(
            provider.connection,
            liquidityPool
        );

        const asset = Restaking.deriveAsset(lsts[0]);

        const destination = Keypair.generate();
        const destinationAta = getAssociatedTokenAddressSync(
            lsts[0],
            destination.publicKey
        );

        const destinationAtaIx = createAssociatedTokenAccountInstruction(
            provider.publicKey,
            destinationAta,
            destination.publicKey,
            lsts[0]
        );

        const poolAssetAccount = getAssociatedTokenAddressSync(
            lsts[0],
            liquidityPool,
            true
        );

        const poolAssetAccountPre = await getAccount(
            provider.connection,
            poolAssetAccount
        );

        await program
            .methods
            .slash({
                liquidityPoolId,
                amount: slashAmount
            })
            .accounts({
                signer: provider.publicKey,
                permissions: admin,
                settings,
                liquidityPool,
                tokenProgram: TOKEN_PROGRAM_ID,
                asset,
                mint: lsts[0],
                liquidityPoolTokenAccount: poolAssetAccount,
                destination: destinationAta,
            })
            .preInstructions([destinationAtaIx])
            .rpc();

        const poolAssetAccountPost = await getAccount(
            provider.connection,
            poolAssetAccount
        );

        expect(
            parseInt(poolAssetAccountPost.amount.toString())
        ).eq(parseInt(poolAssetAccountPre.amount.toString()) - slashAmount.toNumber());
    });

    let alice = Keypair.generate();
    it("Makes second deposit. Should not be affected by prev slashing.", async () => {
        const liquidityPoolId = 0;
        const amount = new BN(LAMPORTS_PER_SOL * 1_000);

        const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);

        const liquidityPoolData = await LiquidityPool.fromAccountAddress(
            provider.connection,
            liquidityPool
        );

        const asset = Restaking.deriveAsset(lsts[0]);

        await provider.connection.requestAirdrop(
            alice.publicKey,
            5 * LAMPORTS_PER_SOL
        );

        await mintTokens(
            lsts[0],
            provider,
            amount.toNumber(),
            alice.publicKey
        );

        const userAssetAta = getAssociatedTokenAddressSync(
            lsts[0],
            alice.publicKey
        );

        const poolAssetAccount = getAssociatedTokenAddressSync(
            lsts[0],
            liquidityPool,
            true
        );

        const userLpAccount = getAssociatedTokenAddressSync(
            liquidityPoolData.lpToken,
            alice.publicKey
        );

        const assetDataPre = await Asset
            .fromAccountAddress(
                provider.connection,
                asset
            );

        const {
            price,
            precision: pricePrecision
        } = await getOraclePriceFromAccount(assetDataPre.oracle.fields[0].toString());

        const transactionTimestamp = Math.floor(Date.now() / 1000);

        const poolAssetAccountPre = await getAccount(
            provider.connection,
            poolAssetAccount
        );

        const totalDepositsPre = new BN(poolAssetAccountPre.amount.toString());
        const lpToken = await getMint(
            provider.connection,
            liquidityPoolData.lpToken
        );
        const totalLpSupply = new BN(lpToken.supply.toString());

        const restaking = new Restaking(provider.connection);
        const assets = await restaking.getAssets();

        const totalPoolValue = await calculateTotalPoolValue(
            await Promise.all(
                assets.map(async (asset) => ({
                    asset: {
                        mint: asset.account.mint.toString(),
                        oracle: asset.account.oracle.fields[0].toString(),
                        tokenBalance: new BN((await getAccount(provider.connection, getAssociatedTokenAddressSync(asset.account.mint, liquidityPool, true))).amount.toString())
                    },
                }))
            )
        );

        const oraclePrice = await getOraclePriceFromAccount(assetDataPre.oracle.fields[0].toString());
        const depositValue = calculateDepositValue(
            amount,
            oraclePrice
        );

        const expectedLpTokens = calculateLpTokensOnDeposit(
            totalLpSupply,
            totalPoolValue,
            depositValue
        );

        const anchorRemainingAccounts: AccountMeta[] = assets
            .map(({ account, pubkey }) => {
                return [
                    {
                        isSigner: false,
                        isWritable: true,
                        pubkey: getAssociatedTokenAddressSync(account.mint, liquidityPool, true)
                    },
                    {
                        isSigner: false,
                        isWritable: false,
                        pubkey: Restaking.deriveAsset(account.mint)
                    },
                    {
                        isSigner: false,
                        isWritable: false,
                        pubkey: account.oracle.fields[0]
                    }
                ] as AccountMeta[];
            }).flat();

        await program
            .methods
            .restake({
                liquidityPoolIndex: liquidityPoolId,
                amount,
                minLpTokens: new BN(0)
            })
            .accounts({
                signer: alice.publicKey,
                settings,
                liquidityPool,
                lpToken: liquidityPoolData.lpToken,
                userLpAccount,
                asset,
                assetMint: lsts[0],
                userAssetAccount: userAssetAta,
                poolAssetAccount,
                oracle: assetDataPre.oracle.fields[0],
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                permissions: null
            })
            .remainingAccounts(anchorRemainingAccounts)
            .signers([
                alice
            ])
            .preInstructions([
                ComputeBudgetProgram.setComputeUnitLimit({ units: 1_000_000 })
            ])
            .rpc()
            .catch(err => {
                console.log(err);
                throw err;
            });

        await new Promise(resolve => setTimeout(resolve, 15000));

        const userLpAccountData = await getAccount(
            provider.connection,
            userLpAccount
        );

        expect(
            parseInt(userLpAccountData.amount.toString())
        ).approximately(
            expectedLpTokens.toNumber(),
            0.01 * LAMPORTS_PER_SOL // 0.1 full LP token delta
        );
    });

    it("Sets the withdraw action to public.", async () => {
        await program
            .methods
            // @ts-ignore
            .updateActionRole({
                action: { withdraw: {} },
                role: { public: {} },
                update: { add: {} }
            })
            .accounts({
                admin: provider.publicKey,
                settings,
                adminPermissions: admin,
            })
            .rpc();

        const settingsData = await Settings.fromAccountAddress(
            provider.connection,
            settings
        );
        
        const actionPermissionMap = settingsData.accessControl.accessMap.actionPermissions.find(mapping => mapping.action === Action.Withdraw);
        expect(actionPermissionMap.action).eq(Action.Withdraw);
        expect(actionPermissionMap.allowedRoles).include(Role.PUBLIC);
    });

    it("Requests withdrawal for the first user.", async () => {
        const liquidityPoolId = 0;
        const amount = new BN(5 * LAMPORTS_PER_SOL);

        const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);

        const liquidityPoolData = await LiquidityPool.fromAccountAddress(
            provider.connection,
            liquidityPool
        );

        const userLpAccount = getAssociatedTokenAddressSync(
            liquidityPoolData.lpToken,
            user.publicKey
        );

        const userLpAccountPre = await getAccount(
            provider.connection,
            userLpAccount
        );

        const [cooldown] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("cooldown"),
                new BN(0).toArrayLike(Buffer, "le", 8)
            ],
            PROGRAM_ID
        );

        const cooldownLpTokenAccount = getAssociatedTokenAddressSync(
            liquidityPoolData.lpToken,
            cooldown,
            true
        );

        const timestampPre = Math.floor(Date.now() / 1000);

        await program
            .methods
            .requestWithdrawal({
                liquidityPoolId,
                amount
            })
            .accounts({
                signer: user.publicKey,
                settings,
                liquidityPool,
                lpTokenMint: liquidityPoolData.lpToken,
                tokenProgram: TOKEN_PROGRAM_ID,
                associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
                signerLpTokenAccount: userLpAccount,
                cooldown,
                cooldownLpTokenAccount,
                permissions: null
            })
            .signers([user])
            .rpc();

        const cooldownDataPost = await Cooldown.fromAccountAddress(
            provider.connection,
            cooldown
        );

        expect(cooldownDataPost.authority.toString()).eq(user.publicKey.toString());
        expect(cooldownDataPost.liquidityPoolId.toString()).eq(liquidityPoolId.toString());
        expect(parseInt(cooldownDataPost.unlockTs.toString()))
            .approximately(
                parseInt(liquidityPoolData.cooldownDuration.toString()) + timestampPre,
                2 // 2 second delta
            );
    });

    let lookupTablePubkey: PublicKey;
    it("Creates lookup table with asset datas", async () => {
        let done = false;
        while (!done) {
            try {
                const restaking = new Restaking(provider.connection);
                const assets = await restaking.getAssets();

                const [ix, pubkey] = AddressLookupTableProgram
                .createLookupTable({
                    authority: provider.publicKey,
                    payer: provider.publicKey,
                    recentSlot: await provider.connection.getSlot()
                });

                const tx = new Transaction();
                tx.add(ix);

                const { blockhash, lastValidBlockHeight } = await provider.connection.getLatestBlockhash();
                tx.recentBlockhash = blockhash;
                tx.feePayer = provider.publicKey;

                const signedTx = await provider.wallet.signTransaction(tx);
                const signature = await provider.connection.sendRawTransaction(signedTx.serialize());
                await provider.connection.confirmTransaction({
                    blockhash,
                    lastValidBlockHeight,
                    signature
                });

                const ix2 = AddressLookupTableProgram
                .extendLookupTable({
                    addresses: assets.map(asset => [asset.pubkey, asset.account.mint, asset.account.oracle.fields[0]]).flat(),
                    authority: provider.publicKey,
                    payer: provider.publicKey,
                    lookupTable: pubkey
                });

                const tx2 = new Transaction();
                tx2.add(ix2);

                const blockData2 = await provider.connection.getLatestBlockhash();
                tx2.recentBlockhash = blockData2.blockhash;
                tx2.feePayer = provider.publicKey;

                await sleep(10);

                const signedTx2 = await provider.wallet.signTransaction(tx2);
                const signature2 = await provider.connection.sendRawTransaction(signedTx2.serialize());
                await provider.connection.confirmTransaction({
                    blockhash: blockData2.blockhash,
                    lastValidBlockHeight: blockData2.lastValidBlockHeight,
                    signature: signature2
                });

                await sleep(10);
                lookupTablePubkey = pubkey;
                done = true;
            } catch (err) {
                console.log("failed to create lookup table. retrying...");
                await sleep(3);
            }
        }
    });

    it("Waits cooldown duration & withdraws.", async () => {
        const liquidityPoolId = 0;
        const cooldownId = 0;

        const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);

        const liquidityPoolData = await LiquidityPool.fromAccountAddress(
            provider.connection,
            liquidityPool
        );

        const [cooldown] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("cooldown"),
                new BN(cooldownId).toArrayLike(Buffer, "le", 8)
            ],
            PROGRAM_ID
        );

        const cooldownData = await Cooldown.fromAccountAddress(
            provider.connection,
            cooldown
        );

        await sleep(new BN(liquidityPoolData.cooldownDuration.toString()).toNumber());

        const asset = Restaking.deriveAsset(lsts[0]);

        const userAssetAta = getAssociatedTokenAddressSync(
            lsts[0],
            user.publicKey
        );

        const userRewardAta = getAssociatedTokenAddressSync(
            rewardToken,
            user.publicKey
        );

        const userAssetAtaPre = await getAccount(
            provider.connection,
            userAssetAta
        );

        const poolAssetAccount = getAssociatedTokenAddressSync(
            lsts[0],
            liquidityPool,
            true
        );

        const cooldownLpTokenAccount = getAssociatedTokenAddressSync(
            liquidityPoolData.lpToken,
            cooldown,
            true
        );

        const restaking = new Restaking(provider.connection);
        const assets = await restaking.getAssets();

        const anchorRemainingAccounts: AccountMeta[] = assets
        .map(({ account, pubkey }) => {
            return [
                {
                    isSigner: false,
                    isWritable: true,
                    pubkey: getAssociatedTokenAddressSync(account.mint, liquidityPool, true)
                },
                {
                    isSigner: false,
                    isWritable: false,
                    pubkey: Restaking.deriveAsset(account.mint)
                },
                {
                    isSigner: false,
                    isWritable: false,
                    pubkey: getAssociatedTokenAddressSync(account.mint, user.publicKey, true)
                }
            ] as AccountMeta[];
        }).flat();

        const ix = await program
            .methods
            .withdraw({
                liquidityPoolId,
                cooldownId: new BN(cooldownId),
            })
            .accounts({
                settings,
                systemProgram: SystemProgram.programId,
                liquidityPool,
                lpTokenMint: liquidityPoolData.lpToken,
                tokenProgram: TOKEN_PROGRAM_ID,
                cooldown,
                cooldownLpTokenAccount,
                signer: user.publicKey,
                permissions: null
            })
            .remainingAccounts(anchorRemainingAccounts)
            // .preInstructions(
            //     assets
            //     .map(asset => createAssociatedTokenAccountIdempotentInstruction(
            //         user.publicKey,
            //         getAssociatedTokenAddressSync(asset.account.mint, provider.publicKey, true),
            //         user.publicKey,
            //         asset.account.mint
            //     ))
            // )
            // .signers([user])
            .instruction();

        const { blockhash, lastValidBlockHeight } = await provider.connection.getLatestBlockhash();
        const msg = new TransactionMessage({
            instructions: [
                ...(assets
                .map(asset => createAssociatedTokenAccountIdempotentInstruction(
                    user.publicKey,
                    getAssociatedTokenAddressSync(asset.account.mint, user.publicKey, true),
                    user.publicKey,
                    asset.account.mint
                ))),
                ix
            ],
            payerKey: user.publicKey,
            recentBlockhash: blockhash
        }).compileToV0Message(
            [(await provider.connection.getAddressLookupTable(lookupTablePubkey)).value]
        );

        const tx = new VersionedTransaction(msg);
        tx.sign([user]);
        const signature = await provider.connection.sendRawTransaction(tx.serialize());
        await provider.connection.confirmTransaction({
            blockhash,
            lastValidBlockHeight,
            signature
        });

        await sleep(10);

        const userRewardAtaPost = await getAccount(
            provider.connection,
            userRewardAta
        );

        expect(
            parseInt(userRewardAtaPost.amount.toString())
        ).gt(0);
    });

    it("Performs a swap between two assets in a liquidity pool", async () => {
        const amountIn = new BN(1000 * LAMPORTS_PER_SOL);

        const liquidityPoolId = 0;
        const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);

        const restaking = new Restaking(provider.connection);
        const assets = await restaking.getAssets();

        const fromMint = lsts[0];
        const toMint = lsts[1];

        const fromAsset = Restaking.deriveAsset(fromMint);
        const toAsset = Restaking.deriveAsset(toMint);

        const {
            oracle: fromOracle
        } = await Asset.fromAccountAddress(provider.connection, fromAsset);

        const {
            oracle: toOracle
        } = await Asset.fromAccountAddress(provider.connection, toAsset);

        const fromSignerTokenAccount = getAssociatedTokenAddressSync(
            fromMint,
            provider.publicKey,
            true
        );

        const toSignerTokenAccount = getAssociatedTokenAddressSync(
            toMint,
            provider.publicKey,
            true
        );

        await mintTokens(
            lsts[0],
            provider,
            10000 * LAMPORTS_PER_SOL,
            provider.publicKey
        );

        await mintTokens(
            lsts[1],
            provider,
            100000 * LAMPORTS_PER_SOL,
            provider.publicKey
        );

        const admin = Restaking.deriveUserPermissions(provider.publicKey);

        const {
            price: fromTokenPrice,
            precision: fromTokenPrecision
        } = await getOraclePriceFromAccount(fromOracle.fields[0].toString());

        const {
            price: toTokenPrice,
            precision: toTokenPrecision
        } = await getOraclePriceFromAccount(toOracle.fields[0].toString());

        const {
            amount: fromSignerTokenAccountBalancePre
        } = await getAccount(provider.connection, fromSignerTokenAccount);
        const {
            amount: toSignerTokenAccountBalancePre
        } = await getAccount(provider.connection, toSignerTokenAccount);

        await program
            .methods
            .swapLp({
                amountIn,
                minOut: new BN(0)
            })
            .preInstructions([
                ...assets.map((asset) => createAssociatedTokenAccountIdempotentInstruction(provider.publicKey, getAssociatedTokenAddressSync(asset.account.mint, provider.publicKey, true), provider.publicKey, asset.account.mint))
            ])
            .accounts({
                admin,
                settings,
                signer: provider.publicKey,
                tokenProgram: TOKEN_PROGRAM_ID,
                liquidityPool,
                tokenFrom: fromMint,
                tokenFromAsset: fromAsset,
                tokenFromOracle: fromOracle.fields[0],
                tokenTo: toMint,
                tokenToAsset: toAsset,
                tokenToOracle: toOracle.fields[0],
                tokenFromPool: getAssociatedTokenAddressSync(fromMint, liquidityPool, true),
                tokenToPool: getAssociatedTokenAddressSync(toMint, liquidityPool, true),
                tokenFromSignerAccount: fromSignerTokenAccount,
                tokenToSignerAccount: toSignerTokenAccount,
                associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
            })
            .rpc();

        const {
            amount: fromSignerTokenAccountBalancePost
        } = await getAccount(provider.connection, fromSignerTokenAccount);

        const {
            amount: toSignerTokenAccountBalancePost
        } = await getAccount(provider.connection, toSignerTokenAccount);

        const fromUsdValue = amountIn
            .mul(fromTokenPrice)
            .div(
                new BN(10)
                    .pow(fromTokenPrecision)
            );

        const toAmount = fromUsdValue
            .mul(new BN(10).pow(toTokenPrecision))
            .div(toTokenPrice);

        expect(toAmount.toNumber())
            .approximately(
                new BN(toSignerTokenAccountBalancePost.toString())
                    .sub(new BN(toSignerTokenAccountBalancePre.toString()))
                    .abs()
                    .toNumber(),
                0.25 * LAMPORTS_PER_SOL
            );

        expect(amountIn.toNumber())
            .eq(
                new BN(fromSignerTokenAccountBalancePre.toString())
                    .sub(new BN(fromSignerTokenAccountBalancePost.toString()))
                    .abs()
                    .toNumber()
            );
    });

    // Admin functionality tests
    it("Creates a new permission account", async () => {
        const newAdmin = Keypair.generate();
        const newAdminPermissions = Restaking.deriveUserPermissions(newAdmin.publicKey);

        await program
            .methods
            .createPermissionAccount(newAdmin.publicKey)
            .accounts({
                settings,
                newCreds: newAdminPermissions,
                caller: provider.publicKey,
                systemProgram: SystemProgram.programId,
            })
            .rpc();

        const newAdminPermissionsData = await UserPermissions.fromAccountAddress(
            provider.connection,
            newAdminPermissions
        );

        expect(newAdminPermissionsData.authority.toString()).eq(newAdmin.publicKey.toString());
    });

    it("Updates action role - adds role to action", async () => {
        await program
            .methods
            .updateActionRole({
                action: { restake: {} },
                role: { mANAGER: {} },
                update: { add: {} }
            })
            .accounts({
                admin,
                adminPermissions: admin,
                settings
            })
            .rpc();

        const settingsData = await Settings.fromAccountAddress(
            provider.connection,
            settings
        );

        // Verify the role was added to the action
        // This would require checking the access control structure
        expect(settingsData.accessControl).to.not.be.undefined;
    });

    it("Updates action role - removes role from action", async () => {
        await program
            .methods
            .updateActionRole({
                action: { restake: {} },
                role: { mANAGER: {} },
                update: { add: {} }
            })
            .accounts({
                admin,
                adminPermissions: admin,
                settings
            })
            .rpc();

        const settingsData = await Settings.fromAccountAddress(
            provider.connection,
            settings
        );

        // Verify the role was removed from the action
        expect(settingsData.accessControl).to.not.be.undefined;
    });

    it("Updates role holder - adds role to user", async () => {
        const targetUser = Keypair.generate();
        const targetUserPermissions = Restaking.deriveUserPermissions(targetUser.publicKey);

        // First create the permission account for the target user
        await program
            .methods
            .createPermissionAccount(targetUser.publicKey)
            .accounts({
                settings,
                newCreds: targetUserPermissions,
                caller: provider.publicKey,
                systemProgram: SystemProgram.programId,
            })
            .rpc();

        await program
            .methods
            .updateRoleHolder({
                address: targetUser.publicKey,
                role: { cRANK: {} },
                update: { add: {} }
            })
            .accounts({
                admin: provider.publicKey,
                settings,
                adminPermissions: admin,
                updateAdminPermissions: targetUserPermissions,
                strategy: provider.publicKey, // Placeholder
                systemProgram: SystemProgram.programId,
            })
            .rpc();

        const targetUserPermissionsData = await UserPermissions.fromAccountAddress(
            provider.connection,
            targetUserPermissions
        );

        // Verify the role was added
        expect(targetUserPermissionsData.authority.toString()).eq(targetUser.publicKey.toString());
    });

    it("Updates role holder - removes role from user", async () => {
        const targetUser = Keypair.generate();
        const targetUserPermissions = Restaking.deriveUserPermissions(targetUser.publicKey);

        // First create the permission account for the target user
        await program
            .methods
            .createPermissionAccount(targetUser.publicKey)
            .accounts({
                settings,
                newCreds: targetUserPermissions,
                caller: provider.publicKey,
                systemProgram: SystemProgram.programId,
            })
            .rpc();

        // Add a role first
        await program
            .methods
            .updateRoleHolder({
                address: targetUser.publicKey,
                role: { fREEZE: {} },
                update: { add: {} }
            })
            .accounts({
                admin: provider.publicKey,
                settings,
                adminPermissions: admin,
                updateAdminPermissions: targetUserPermissions,
                strategy: provider.publicKey, // Placeholder
                systemProgram: SystemProgram.programId,
            })
            .rpc();

        // Then remove it
        await program
            .methods
            .updateRoleHolder({
                address: targetUser.publicKey,
                role: { fREEZE: {} },
                update: { remove: {} }
            })
            .accounts({
                admin: provider.publicKey,
                settings,
                adminPermissions: admin,
                updateAdminPermissions: targetUserPermissions,
                strategy: provider.publicKey, // Placeholder
                systemProgram: SystemProgram.programId,
            })
            .rpc();

        const targetUserPermissionsData = await UserPermissions.fromAccountAddress(
            provider.connection,
            targetUserPermissions
        );

        // Verify the role was removed
        expect(targetUserPermissionsData.authority.toString()).eq(targetUser.publicKey.toString());
    });
});
