import * as anchor from "@coral-xyz/anchor";
import {AnchorError, Program} from "@coral-xyz/anchor";
import { InsuranceFund } from "../target/types/insurance_fund";
import {
    PublicKey,
    Keypair,
    SystemProgram,
    LAMPORTS_PER_SOL,
    SYSVAR_CLOCK_PUBKEY,
    TransactionMessage, Connection, TransactionInstruction
} from "@solana/web3.js";
import {before} from "mocha";
import {
    Admin,
    Asset, Cooldown,
    createDepositRewardsInstruction,
    Deposit,
    Intent,
    Lockup, Permissions,
    PROGRAM_ID,
    Settings,
    Slash
} from "../sdk/src/generated";
import {expect} from "chai";
import createToken from "./helpers/createToken";
import BN from "bn.js";
import {
    AuthorityType,
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
import getOraclePrice from "./helpers/getOraclePrice";
import sleep from "./helpers/sleep";
import {
    PROGRAM_ID as METAPLEX_PROGRAM_ID,
    createCreateMetadataAccountV3Instruction
} from "@metaplex-foundation/mpl-token-metadata";

async function createReceiptToken(
    signer: PublicKey,
    lockup: PublicKey,
    connection: Connection,
    depositToken: PublicKey,
    withMetadata?: boolean,
) {
    const {
        decimals
    } = await getMint(connection, depositToken, "confirmed");

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
        lockup
    );

    instructions.push(setAuthorityIx);

    return {
        instructions,
        mint: tokenKeypair
    }
}

describe("insurance-fund", () => {
    const provider = anchor.AnchorProvider.local();
    anchor.setProvider(provider);
    const program = anchor.workspace.InsuranceFund as Program<InsuranceFund>;

    let coldWallet: PublicKey;
    let settings: PublicKey;
    let user: Keypair;
    let rewardToken: PublicKey;
    let price: BN;
    let pricePrecision: BN;
    let admin: PublicKey;
    let receiptTokenMint: PublicKey;

    const lsts: PublicKey[] = [];

    before(async () => {
        coldWallet = Keypair.generate().publicKey;
        user  = Keypair.generate();

        [settings] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("settings")
            ],
            program.programId
        );

        await provider.connection.requestAirdrop(
            user.publicKey,
            LAMPORTS_PER_SOL * 1000
        );

        rewardToken = await createToken(
            provider.connection,
            provider
        );

        [admin] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("admin"),
                provider.publicKey.toBuffer()
            ],
            PROGRAM_ID
        );
    });

    it("Initializes insurance fund.", async () => {

        await program
            .methods
            .initializeInsuranceFund({
                coldWallet,
                coldWalletShareBps: new BN(0.7 * 10_000),
                hotWalletShareBps: new BN(0.3 * 10_000),
                rewardMint: rewardToken,
                cooldownDuration: new BN(30) // 30 seconds
            })
            .accounts({
                signer: provider.publicKey,
                admin,
                settings,
                systemProgram: SystemProgram.programId,
            })
            .rpc();

        const {
            coldWallet: setColdWallet,
            lockups,
            frozen,
            sharesConfig,
            rewardConfig,
            cooldownDuration
        } = await Settings.fromAccountAddress(
            provider.connection,
            settings
        );

        expect(setColdWallet.toString()).eq(coldWallet.toString());
        expect(lockups.toString()).eq("0");
        expect(cooldownDuration.toString()).eq(`${30}`);
        expect(sharesConfig.coldWalletShareBps.toString()).eq(`${7_000}`);
        expect(sharesConfig.hotWalletShareBps.toString()).eq(`${3_000}`);
        expect(frozen).eq(false);
        expect(rewardConfig.main.toString()).eq(rewardToken.toString());
    });

    it('Freezes protocol.', async () => {
        await program
            .methods
            .manageFreeze({
                freeze: true
            })
            .accounts({
                admin,
                signer: provider.publicKey,
                settings
            })
            .rpc()

        let settingsData = await Settings.fromAccountAddress(
            provider.connection,
            settings
        );

        expect(settingsData.frozen).eq(true);
    });

    it("Tries to interact with frozen protocol. Succeeds on errors", async () => {
        let error: AnchorError;

        const wrappedSol = new PublicKey("So11111111111111111111111111111111111111112");
        const [asset] = PublicKey.findProgramAddressSync(
            [Buffer.from("asset"), wrappedSol.toBuffer()],
            PROGRAM_ID
        );

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

        expect(error.error.errorCode.code).eq("Frozen");
        expect(error.error.origin).eq("settings");
    });

    it("Unfreezes protocol", async () => {
        await program
            .methods
            .manageFreeze({
                freeze: false
            })
            .accounts({
                admin,
                signer: provider.publicKey,
                settings
            })
            .rpc();

        let settingsData = await Settings.fromAccountAddress(
            provider.connection,
            settings
        );

        expect(settingsData.frozen).eq(false);
    });

    it("Rotates admins", async () => {
        for (let i = 0; i < 3; i++) {
            const keypair = Keypair.generate();

            const [newAdmin] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("admin"),
                    keypair.publicKey.toBuffer()
                ],
                PROGRAM_ID
            );

            await program
                .methods
                .addAdmin({
                    address: keypair.publicKey,
                    permissions: { superadmin: undefined }
                })
                .accounts({
                    signer: provider.publicKey,
                    existingAdmin: admin,
                    newAdmin,
                    settings,
                    systemProgram: SystemProgram.programId
                })
                .rpc();

            const {
                permissions,
                address
            } = await Admin.fromAccountAddress(
                provider.connection,
                newAdmin
            );

            expect(address.toString()).eq(keypair.publicKey.toString());
            expect(permissions).eq(Permissions.Superadmin);

            await program
                .methods
                .removeAdmin({
                    address: keypair.publicKey
                })
                .accounts({
                    // self-destruct
                    signer: keypair.publicKey,
                    admin: newAdmin,
                    adminToRemove: newAdmin,
                    settings,
                    systemProgram: SystemProgram.programId
                })
                .signers([keypair])
                .rpc();
        }
    });

    it("Mints and adds assets to insurance pool.", async () => {
        const assets: PublicKey[] = [];

        for (let i = 0; i < 3; i ++) {
            const token = await createToken(
                provider.connection,
                provider
            );

            lsts.push(token);

            const [asset] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("asset"),
                    token.toBuffer()
                ],
                PROGRAM_ID
            );

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
                    oracle: new PublicKey("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE"),
                })
                .rpc();
        }

        for (let i = 0; i < assets.length; i++) {
            const asset = assets[i];
            const assetData = await Asset.fromAccountAddress(
                provider.connection,
                asset,
            );

            expect(lsts[i].toString()).eq(assetData.mint.toString());
            expect(assetData.tvl.toString()).eq("0");
            expect(assetData.oracle.__kind).eq("Pyth");
            expect(assetData.oracle.fields[0].toString()).eq("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE");
            expect(assetData.deposits.toString()).eq("0");
            expect(assetData.lockups.toString()).eq("0");
        }
    });

    it("Initializes lockups", async () => {
        for (let [j, mint] of lsts.entries()) {
            const MONTH = 30 * 24 * 60 * 60;

            for (let i = 0; i < 4; i ++) {
                // the first one is just 10s for testing purposes
                // the rest is 6, 9, 12 months
                const duration = new BN(i == 0 ? 10 : ((i + 1) * 3 * MONTH));

                const lockupId = j * 4 + i;
                const [lockup] = PublicKey.findProgramAddressSync(
                    [
                        Buffer.from("lockup"),
                        new BN(lockupId).toArrayLike(Buffer, "le", 8)
                    ],
                    program.programId
                );

                const [asset] = PublicKey.findProgramAddressSync(
                    [
                        Buffer.from("asset"),
                        mint.toBuffer()
                    ],
                    program.programId
                );

                const [assetRewardPool] = PublicKey.findProgramAddressSync(
                    [
                        Buffer.from("reward_pool"),
                        lockup.toBuffer(),
                        rewardToken.toBuffer(),
                    ],
                    program.programId
                );

                const [assetLockup] = PublicKey.findProgramAddressSync(
                    [
                        Buffer.from("vault"),
                        lockup.toBuffer(),
                        mint.toBuffer(),
                    ],
                    program.programId
                );

                const [lockupHotVault] = PublicKey
                    .findProgramAddressSync(
                        [
                            Buffer.from("hot_vault"),
                            lockup.toBuffer(),
                            mint.toBuffer()
                        ],
                        PROGRAM_ID
                    );

                const [lockupColdVault] = PublicKey
                    .findProgramAddressSync(
                        [
                            Buffer.from("cold_vault"),
                            lockup.toBuffer(),
                            mint.toBuffer()
                        ],
                        PROGRAM_ID
                    );

                const {
                    mint: receiptMint,
                    instructions: preInstructions
                } = await createReceiptToken(
                    provider.publicKey,
                    lockup,
                    provider.connection,
                    mint,
                    false
                );

                if (j == 0 && i == 0) {
                    receiptTokenMint = receiptMint.publicKey
                };

                const [lockupCooldownVault] = PublicKey
                    .findProgramAddressSync(
                        [
                            Buffer.from("cooldown_vault"),
                            lockup.toBuffer(),
                            receiptMint.publicKey.toBuffer()
                        ],
                        PROGRAM_ID
                    );

                await program
                    .methods
                    .initializeLockup({
                        minDeposit: new BN(1), // 1 token for now
                        duration,
                        yieldMode: {
                            single: {} // No $R rewards
                        },
                        depositCap: new BN("100000000000000"),
                    })
                    .accounts({
                        admin,
                        signer: provider.publicKey,
                        settings,
                        lockup,
                        assetMint: mint,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        systemProgram: SystemProgram.programId,
                        asset,
                        assetRewardPool,
                        rewardMint: rewardToken,
                        lockupHotVault,
                        lockupColdVault,
                        lockupCooldownVault,
                        poolShareReceipt: receiptMint.publicKey,
                        coldWallet,
                    })
                    .preInstructions(preInstructions)
                    .signers([receiptMint])
                    .rpc();

                const {
                    assetMint,
                    duration: setDuration,
                    minDeposit,
                    yieldMode,
                    index,
                    deposits,
                    depositCap,
                    rewardBoosts,
                    slashState,
                } = await Lockup.fromAccountAddress(
                    provider.connection,
                    lockup
                );

                expect(assetMint.toString()).eq(mint.toString());
                expect(setDuration.toString()).eq(duration.toString());
                expect(minDeposit.toString()).eq("1");
                expect(yieldMode.__kind).eq("Single");
                expect(index.toString()).eq(lockupId.toString());
                expect(deposits.toString()).eq("0");
                expect(depositCap.toString()).eq(`100000000000000`);
                expect(rewardBoosts.toString()).eq("0");
                expect(slashState.index.toString()).eq("0");
                expect(slashState.amount.toString()).eq("0");
            }
        }
    });

    it("Locks-up tokens.", async () => {
        const lockupId = new BN(0);

        const [lockup] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("lockup"),
                lockupId.toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        const lockupData = await Lockup.fromAccountAddress(
            provider.connection,
            lockup
        );

        const {
            rewardConfig: {
                main: rewardMint
            }
        } = await Settings.fromAccountAddress(
            provider.connection,
            settings
        );

        const [assetLockup] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                lockup.toBuffer(),
                lockupData.assetMint.toBuffer(),
            ],
            program.programId
        );

        const [asset] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("asset"),
                lockupData.assetMint.toBuffer()
            ],
            program.programId
        );

        const amount = new BN(LAMPORTS_PER_SOL * 1_000);
        await mintTokens(
            lockupData.assetMint,
            provider,
            amount.toNumber(),
            user.publicKey
        );

        const [deposit] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("deposit"),
                lockup.toBuffer(),
                new BN(lockupData.deposits).toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        const coldWalletVault = getAssociatedTokenAddressSync(
            lockupData.assetMint,
            coldWallet,
            true
        );

        const userAssetAta = getAssociatedTokenAddressSync(
            lockupData.assetMint,
            user.publicKey
        );

        const coldWalletAtaIx = createAssociatedTokenAccountInstruction(
            user.publicKey,
            coldWalletVault,
            coldWallet,
            lockupData.assetMint
        );

        const {
            price: assetPrice,
            precision
        } = await getOraclePrice();

        price = assetPrice;
        pricePrecision = precision;

        const [lockupHotVault] = PublicKey
            .findProgramAddressSync(
                [
                    Buffer.from("hot_vault"),
                    lockup.toBuffer(),
                    lockupData.assetMint.toBuffer()
                ],
                PROGRAM_ID
            );

        const [lockupColdVault] = PublicKey
            .findProgramAddressSync(
                [
                    Buffer.from("cold_vault"),
                    lockup.toBuffer(),
                    lockupData.assetMint.toBuffer()
                ],
                PROGRAM_ID
            );

        const [depositReceiptTokenAccount] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("deposit_receipt_vault"),
                deposit.toBuffer(),
                lockupData.receiptMint.toBuffer(),
            ],
            PROGRAM_ID
        );

        // const initializeDepositReceiptTokenAccountInstructions = [
        //     SystemProgram.createAccount({
        //         lamports: await provider.connection.getMinimumBalanceForRentExemption(MINT_SIZE),
        //         space: 165,
        //         fromPubkey: user.publicKey,
        //         newAccountPubkey: depositReceiptTokenAccount,
        //         programId: TOKEN_PROGRAM_ID
        //     }),
        //     createInitializeAccount3Instruction(
        //         depositReceiptTokenAccount,
        //         lockupData.receiptMint,
        //         deposit,
        //         TOKEN_PROGRAM_ID
        //     )
        // ];

        const [assetRewardPool] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("reward_pool"),
                lockup.toBuffer(),
                rewardMint.toBuffer(),
            ],
            program.programId
        );


        const transactionTimestamp = Math.floor(Date.now() / 1000);
        await program
            .methods
            .restake({
                lockupId,
                amount
            })
            .accounts({
                user: user.publicKey,
                settings,
                lockup,
                deposit,
                assetMint: lockupData.assetMint,
                userAssetAta,
                oracle: new PublicKey("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE"),
                tokenProgram: TOKEN_PROGRAM_ID,
                asset,
                systemProgram: SystemProgram.programId,
                receiptTokenMint: lockupData.receiptMint,
                lockupHotVault,
                lockupColdVault,
                depositReceiptTokenAccount,
            })
            .preInstructions([
                coldWalletAtaIx,
                // ...initializeDepositReceiptTokenAccountInstructions
            ])
            .signers([
                user
            ])
            .rpc()
            .catch(err => {
                console.log(err);
                throw err;
            });

        await new Promise(resolve => setTimeout(resolve, 15000));

        const settingsData = await Settings.fromAccountAddress(
            provider.connection,
            settings
        );

        const assetData = await Asset.fromAccountAddress(
            provider.connection,
            asset
        );

        expect(assetData.tvl.toString()).eq(amount.toString());

        const lockupHotVaultData = await getAccount(
            provider.connection,
            lockupHotVault
        );

        const lockupColdVaultData = await getAccount(
            provider.connection,
            lockupColdVault
        );

        expect(
            new BN(lockupColdVaultData.amount.toString()).toNumber()
        ).approximately(
            amount.mul(
                new BN(settingsData.sharesConfig.coldWalletShareBps)
            ).divn(10_000).toNumber(),
            1 * LAMPORTS_PER_SOL
        );

        expect(
            new BN(lockupHotVaultData.amount.toString()).toNumber()
        ).approximately(
            amount.mul(
                new BN(settingsData.sharesConfig.hotWalletShareBps)
            ).divn(10_000).toNumber(),
            1 * LAMPORTS_PER_SOL
        );

        const {
            user: userPointer,
            lockup: lockupPointer,
            initialUsdValue,
            lastSlashed,
            amountSlashed,
            unlockTs,
            index,
            initialReceiptExchangeRateBps
        } = await Deposit.fromAccountAddress(
            provider.connection,
            deposit
        );

        const usdValue = amount
            .div(new BN(LAMPORTS_PER_SOL))
            .mul(price)
            .div(
                new BN(Math.pow(10, pricePrecision.toNumber())),
            )
            .toNumber();

        expect(
            parseInt(initialUsdValue.toString())
        ).approximately(
            usdValue,
            0.5 * (amount.toNumber() / LAMPORTS_PER_SOL) // 0.5 usd delta per deposited token
        );

        expect(index.toString()).eq("0");
        expect(initialReceiptExchangeRateBps.toString()).eq("0");
        expect(userPointer.toString()).eq(user.publicKey.toString());
        expect(lockupPointer.toString()).eq(lockup.toString());
        expect(lastSlashed).eq(null);
        expect(amountSlashed.toString()).eq("0");
        expect(parseInt(unlockTs.toString()))
            .approximately(
                transactionTimestamp + parseInt(lockupData.duration.toString()),
                10 // 10 seconds delta
            );

        const depositReceiptTokenAccountData = await getAccount(
            provider.connection,
            depositReceiptTokenAccount
        );

        expect(
            parseInt(depositReceiptTokenAccountData.amount.toString())
        ).eq(amount.toNumber());
    });

    it("Deposits rewards", async () => {
        const lockupId = new BN(0);
        const amount = new BN(100_000 * LAMPORTS_PER_SOL);

        const [lockup] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("lockup"),
                lockupId.toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        const [assetRewardPool] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("reward_pool"),
                lockup.toBuffer(),
                rewardToken.toBuffer(),
            ],
            program.programId
        );

        const callerRewardAta = getAssociatedTokenAddressSync(
            rewardToken,
            provider.publicKey
        );

        await mintTokens(
            rewardToken,
            provider,
            100_000 * LAMPORTS_PER_SOL,
            provider.publicKey
        );

        const [lockupCooldownVault] = PublicKey
            .findProgramAddressSync(
                [
                    Buffer.from("cooldown_vault"),
                    lockup.toBuffer(),
                    receiptTokenMint.toBuffer()
                ],
                PROGRAM_ID
            );

        const lockupCooldownVaultData = await getAccount(
            provider.connection,
            lockupCooldownVault
        );

        const lockupDataPre = await Lockup
            .fromAccountAddress(
                provider.connection,
                lockup
            );

        const assetRewardPoolPre = await getAccount(
            provider.connection,
            assetRewardPool
        );

        const receipt = await getMint(
            provider.connection,
            receiptTokenMint
        );

        await program
            .methods
            .depositRewards({
                lockupId,
                amount
            })
            .accounts({
                caller: provider.publicKey,
                callerRewardAta,
                lockup,
                tokenProgram: TOKEN_PROGRAM_ID,
                rewardMint: rewardToken,
                assetRewardPool,
                settings,
                lockupCooldownVault,
                receiptTokenMint,
            })
            .rpc()
            .catch(err => {
                console.log(err.logs);
                throw err;
            });

        const assetRewardPoolPost = await getAccount(
            provider.connection,
            assetRewardPool
        );

        const lockupDataPost = await Lockup
            .fromAccountAddress(
                provider.connection,
                lockup
            );

        expect(
            parseInt(assetRewardPoolPost.amount.toString())
        ).eq(parseInt(assetRewardPoolPre.amount.toString()) + amount.toNumber());

        const totalSupply = new BN(receipt.supply.toString());
        const inCooldown = new BN(lockupCooldownVaultData.amount.toString());
        const activeSupply = totalSupply.sub(inCooldown);

        const increaseBps = amount
            .muln(10_000)
            .div(activeSupply);

        expect(
            parseInt(lockupDataPost.receiptToRewardExchangeRateBpsAccumulator.toString())
        ).approximately(
            parseInt(lockupDataPre.receiptToRewardExchangeRateBpsAccumulator.toString()) + increaseBps.toNumber(),
            10 // 0.1% difference delta
        );
    });

    it("Slashes lockup", async () => {
        const lockupId = new BN(0);

        const [lockup] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("lockup"),
                lockupId.toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        const lockupData = await Lockup.fromAccountAddress(
            provider.connection,
            lockup
        );

        const [assetLockup] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                lockup.toBuffer(),
                lockupData.assetMint.toBuffer(),
            ],
            program.programId
        );

        const slashId = new BN(lockupData.slashState.index);
        const slashAmount = new BN(LAMPORTS_PER_SOL * 200);

        const [slash] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("slash"),
                lockup.toBuffer(),
                slashId.toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        const {
            slashedAccounts: initialSlashedAccounts,
            slashedAmount: initialSlashedAmount,
            targetAccounts: initialTargetAccounts,
            targetAmount: initialTargetAmount,
        } = await Slash.fromAccountAddress(
            provider.connection,
            slash
        );

        expect(initialSlashedAccounts.toString()).eq("0");
        expect(initialSlashedAmount.toString()).eq("0");
        expect(initialTargetAccounts.toString()).eq(`${ lockupData.deposits.toString() }`);
        expect(initialTargetAmount.toString()).eq(slashAmount.toString());

        const targetDeposits = parseInt(lockupData.deposits.toString());
        const depositsToSlash: PublicKey[] = [];
        const depositsToSlashData: Deposit[] = [];
        for (let i = 0; i < targetDeposits; ++i) {
            const [deposit] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("deposit"),
                    lockup.toBuffer(),
                    new BN(i).toArrayLike(Buffer, "le", 8)
                ],
                program.programId
            );

            depositsToSlash.push(deposit);
            depositsToSlashData.push(
                await Deposit.fromAccountAddress(provider.connection, deposit)
            );
        }

        const coldWalletVault = getAssociatedTokenAddressSync(
            lockupData.assetMint,
            coldWallet,
            true
        );

        const {
            slashedAccounts: slashedDeposits,
            slashedAmount: slashedAmountFromDeposits,
            targetAccounts: slashedTargetAccounts
        } = await Slash.fromAccountAddress(
            provider.connection,
            slash
        );

        expect(new BN(slashedDeposits).toNumber())
            .eq(depositsToSlash.length)
            .eq(new BN(slashedTargetAccounts).toNumber());

        expect(new BN(slashedAmountFromDeposits).toNumber())
            .approximately(
                (slashAmount).toNumber(),
                1 * LAMPORTS_PER_SOL // 1 full token delta
            );

        const [lockupAssetVault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                lockup.toBuffer(),
                lockupData.assetMint.toBuffer(),
            ],
            program.programId
        );

        const {
            amount: lockupAssetVaultBalancePre
        } = await getAccount(
            provider.connection,
            lockupAssetVault
        );

        const {
            slashedAccounts,
            targetAccounts,
            slashedAmount,
            targetAmount
        } = await Slash.fromAccountAddress(
            provider.connection,
            slash
        );

        const {
            slashedColdWallet,
            transferSig
        } = await Slash.fromAccountAddress(
            provider.connection,
            slash
        );

        expect(transferSig).eq("test-signature");
        expect(slashedColdWallet).eq(true);

        const [asset] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("asset"),
                lockupData.assetMint.toBuffer()
            ],
            PROGRAM_ID
        );

        const assetLockupData = await getAccount(
            provider.connection,
            assetLockup
        );

        // Make sure there's enough funds in the hot wallet to slash.
        expect(parseInt(assetLockupData.amount.toString()))
            .gte(slashAmount.toNumber());

        const {
            amount: lockupAssetVaultBalancePost
        } = await getAccount(
            provider.connection,
            lockupAssetVault
        );

        expect(
            new BN(lockupAssetVaultBalancePost.toString()).toNumber()
        )
            .approximately(
                new BN(lockupAssetVaultBalancePre.toString()).toNumber() - new BN(slashedAmount).toNumber() * 0.3,
                1 * LAMPORTS_PER_SOL // 1 full token delta
            );
    });

    it("Awaits 10 second lockup time and requests withdrawal.", async () => {
        await sleep(10);

        const lockupId = new BN(0);
        const depositId = new BN(0);
        const amount = new BN(5 * LAMPORTS_PER_SOL);

        const [lockup] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("lockup"),
                new BN(lockupId).toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        const settingsData = await Settings.fromAccountAddress(
            provider.connection,
            settings
        );

        const lockupDataPre = await Lockup.fromAccountAddress(
            provider.connection,
            lockup
        );

        const [lockupAssetVault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                lockup.toBuffer(),
                lockupDataPre.assetMint.toBuffer(),
            ],
            program.programId
        );

        const [deposit] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("deposit"),
                lockup.toBuffer(),
                new BN(depositId).toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        const depositDataPre = await Deposit.fromAccountAddress(
            provider.connection,
            deposit
        );

        const [asset] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("asset"),
                lockupDataPre.assetMint.toBuffer()
            ],
            program.programId
        );

        const [assetRewardPool] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("reward_pool"),
                lockup.toBuffer(),
                rewardToken.toBuffer(),
            ],
            program.programId
        );

        const assetRewardPoolDataPre = await getAccount(provider.connection, assetRewardPool);

        const [cooldown] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("cooldown"),
                deposit.toBuffer()
            ],
            PROGRAM_ID
        );

        const timestampPre = Math.floor(Date.now() / 1000);

        await program
            .methods
            .requestWithdrawal({
                lockupId,
                depositId,
                rewardBoostId: null,
                mode: { exactIn: { "0": new BN(amount) } }
            })
            .accounts({
                user: user.publicKey,
                settings,
                lockup,
                deposit,
                asset,
                assetMint: lockupDataPre.assetMint,
                clock: SYSVAR_CLOCK_PUBKEY,
                tokenProgram: TOKEN_PROGRAM_ID,
                assetRewardPool,
                rewardBoost: program.programId,
                rewardMint: rewardToken,
                cooldown,
            })
            .signers([user])
            .rpc();

        const cooldownDataPost = await Cooldown.fromAccountAddress(
            provider.connection,
            cooldown
        );

        const totalRewardsPre = new BN(assetRewardPoolDataPre.amount.toString()).toNumber();

        expect(cooldownDataPost.user.toString()).eq(user.publicKey.toString());
        expect(cooldownDataPost.lockupId.toString()).eq(lockupId.toString());
        expect(cooldownDataPost.depositId.toString()).eq(depositId.toString());
        expect(parseInt(cooldownDataPost.unlockTs.toString()))
            .approximately(
                parseInt(settingsData.cooldownDuration.toString()) + timestampPre,
                2 // 2 second delta
            );
    });

    it("Waits cooldown duration & withdraws.", async () => {
        const {
            cooldownDuration
        } = await Settings.fromAccountAddress(
            provider.connection,
            settings
        );

        await sleep(parseInt(cooldownDuration.toString()));

        const lockupId = new BN(0);
        const depositId = new BN(0);

        const [lockup] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("lockup"),
                new BN(lockupId).toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        const lockupDataPre = await Lockup.fromAccountAddress(
            provider.connection,
            lockup
        );

        const [lockupAssetVault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                lockup.toBuffer(),
                lockupDataPre.assetMint.toBuffer(),
            ],
            program.programId
        );

        const lockupAssetVaultBalancePre = await getAccount(
            provider.connection,
            lockupAssetVault
        );

        const [deposit] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("deposit"),
                lockup.toBuffer(),
                new BN(depositId).toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        const [cooldown] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("cooldown"),
                deposit.toBuffer()
            ],
            PROGRAM_ID
        );

        const cooldownData = await Cooldown.fromAccountAddress(
            provider.connection,
            cooldown
        );

        const depositDataPre = await Deposit.fromAccountAddress(
            provider.connection,
            deposit
        );

        const [asset] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("asset"),
                lockupDataPre.assetMint.toBuffer()
            ],
            program.programId
        );

        const [assetRewardPool] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("reward_pool"),
                lockup.toBuffer(),
                rewardToken.toBuffer(),
            ],
            program.programId
        );

        const assetRewardPoolDataPre = await getAccount(
            provider.connection,
            assetRewardPool
        );

        const userAssetAta = getAssociatedTokenAddressSync(
            lockupDataPre.assetMint,
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

        const coldWalletVault = getAssociatedTokenAddressSync(
            lockupDataPre.assetMint,
            coldWallet,
            true
        );

        await program
            .methods
            .withdraw({
                lockupId,
                depositId
            })
            .accounts({
                settings,
                user: user.publicKey,
                lockup,
                deposit,
                cooldown,
                assetMint: lockupDataPre.assetMint,
                asset,
                userAssetAta,
                rewardMint: rewardToken,
                userRewardAta,
                assetRewardPool,
                tokenProgram: TOKEN_PROGRAM_ID,
                clock: SYSVAR_CLOCK_PUBKEY
            })
            .preInstructions([
                createAssociatedTokenAccountInstruction(
                    user.publicKey,
                    userRewardAta,
                    user.publicKey,
                    rewardToken
                )
            ])
            .signers([user])
            .rpc();

        const userRewardAtaDataPost = await getAccount(
            provider.connection,
            userRewardAta
        );

        const userAssetAtaPost = await getAccount(
            provider.connection,
            userAssetAta
        );

        const lockupAssetVaultBalancePost = await getAccount(
            provider.connection,
            lockupAssetVault
        );

        const depositDataPost = await Deposit.fromAccountAddress(
            provider.connection,
            deposit
        );

        const assetRewardPoolDataPost = await getAccount(
            provider.connection,
            assetRewardPool
        );

        const userRewards = parseInt((cooldownData.rewards.fields[0] as (number | BN)).toString());

        expect(userRewards)
            .approximately(
                parseInt(userRewardAtaDataPost.amount.toString()),
                1 * LAMPORTS_PER_SOL // 1 full token delta
            );

        expect(parseInt(assetRewardPoolDataPost.amount.toString()))
            .approximately(
                parseInt(assetRewardPoolDataPre.amount.toString()) - userRewards,
                1 * LAMPORTS_PER_SOL // 1 full token delta
            );

        expect(new BN(lockupAssetVaultBalancePost.amount.toString()).toNumber())
            .eq(new BN(lockupAssetVaultBalancePre.amount.toString()).toNumber() - 5 * LAMPORTS_PER_SOL);
    });

    it("Withdraws >30% of the insurance fund, should create intent.", async () => {
        const lockupId = new BN(0);
        const depositId = new BN(0);

        const [lockup] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("lockup"),
                lockupId.toArrayLike(Buffer, "le", 8)
            ],
            PROGRAM_ID
        );

        const [deposit] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("deposit"),
                lockup.toBuffer(),
                depositId.toArrayLike(Buffer, "le", 8)
            ],
            PROGRAM_ID
        );

        const depositData = await Deposit.fromAccountAddress(
            provider.connection,
            deposit
        );

        const {
            assetMint
        } = await Lockup.fromAccountAddress(
            provider.connection,
            lockup
        );

        const [asset] = PublicKey.findProgramAddressSync(
            [Buffer.from("asset"), assetMint.toBuffer()],
            PROGRAM_ID
        );

        const userAssetAta = getAssociatedTokenAddressSync(
            assetMint,
            user.publicKey,
            false
        );

        const [lockupAssetVault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                lockup.toBuffer(),
                assetMint.toBuffer(),
            ],
            program.programId
        );

        const [intent] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("intent"),
                deposit.toBuffer()
            ],
            PROGRAM_ID
        );

        const lockupAssetVaultData = await getAccount(
            provider.connection,
            lockupAssetVault
        );

        const {
            lockup: intentLockup,
            deposit: intentDeposit,
        } = await Intent.fromAccountAddress(
            provider.connection,
            intent
        );

        expect(lockup.toString()).eq(intentLockup.toString());
        expect(deposit.toString()).eq(intentDeposit.toString());
    });
});
