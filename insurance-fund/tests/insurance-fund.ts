import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { InsuranceFund } from "../target/types/insurance_fund";
import {
    PublicKey,
    Keypair,
    SystemProgram,
    LAMPORTS_PER_SOL,
    SYSVAR_CLOCK_PUBKEY,
    TransactionMessage
} from "@solana/web3.js";
import {before} from "mocha";
import {Asset, Deposit, Lockup, PROGRAM_ID, Settings, Slash} from "../sdk/generated";
import {expect} from "chai";
import createToken from "./helpers/createToken";
import BN from "bn.js";
import {
    createAssociatedTokenAccountInstruction, getAccount,
    getAssociatedTokenAddressSync,
    TOKEN_PROGRAM_ID
} from "@solana/spl-token";
import sleep from "./helpers/sleep";
import mintTokens from "./helpers/mintTokens";
import signAndSendTransaction from "./helpers/signAndSendTransaction";

describe("insurance-fund", () => {
    const provider = anchor.AnchorProvider.local();
    anchor.setProvider(provider);
    const program = anchor.workspace.InsuranceFund as Program<InsuranceFund>;

    let coldWallet: PublicKey;
    let settings: PublicKey;
    let user: Keypair;

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
            LAMPORTS_PER_SOL * 10
        );
    })

    it("Initializes insurance fund.", async () => {
        await program
            .methods
            .initializeInsuranceFund({
                coldWallet,
                coldWalletShareBps: new BN(0.7 * 10_000),
                hotWalletShareBps: new BN(0.3 * 10_000)
            })
            .accounts({
                superadmin: provider.publicKey,
                settings,
                systemProgram: SystemProgram.programId,
            })
            .rpc();

        const {
            coldWallet: setColdWallet,
            superadmin,
            bump,
            lockups,
            frozen,
            sharesConfig
        } = await Settings.fromAccountAddress(
            provider.connection,
            settings
        );

        expect(setColdWallet.toString()).eq(coldWallet.toString());
        expect(lockups.toString()).eq("0");
        expect(superadmin.toString()).eq(provider.publicKey.toString());
        expect(sharesConfig.coldWalletShareBps.toString()).eq(`${7_000}`);
        expect(sharesConfig.hotWalletShareBps.toString()).eq(`${3_000}`);
        expect(frozen).eq(false);
    });

    it('Freezes/unfreezes protocol', async () => {
        await program
            .methods
            .manageFreeze({
                freeze: true
            })
            .accounts({
                superadmin: provider.publicKey,
                settings
            })
            .rpc()

        let settingsData = await Settings.fromAccountAddress(
            provider.connection,
            settings
        );

        expect(settingsData.frozen).eq(true);

        await program
            .methods
            .manageFreeze({
                freeze: false
            })
            .accounts({
                superadmin: provider.publicKey,
                settings
            })
            .rpc();

        settingsData = await Settings.fromAccountAddress(
            provider.connection,
            settings
        );

        expect(settingsData.frozen).eq(false);
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
                    superadmin: provider.publicKey,
                    // Just to pass the test
                    oracle: new PublicKey("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG")
                })
                .rpc()
        }

        for (let asset of assets) {
            const assetData = await Asset.fromAccountAddress(
                provider.connection,
                asset,
            );

            expect(lsts.map(l => l.toString())).include(assetData.mint.toString());
            expect(assetData.tvl.toString()).eq("0");
        }
    });

    it("Initializes lockups", async () => {
        for (let [j, mint] of lsts.entries()) {
            const MONTH = 30 * 24 * 60 * 60;

            for (let i = 0; i < 4; i ++) {
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
                        mint.toBuffer(),
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

                await program
                    .methods
                    .initializeLockup({
                        asset: mint,
                        minDeposit: new BN(1), // 1 token for now
                        duration,
                        yieldBps: new BN(i * 3 * 100),
                        yieldMode: {
                            single: { 0: [new BN(1)] } // 1 rUSD per 1 unit of deposit [lamport] per one lockup period
                        },
                        depositCap: new BN(LAMPORTS_PER_SOL * 100_000)
                    })
                    .accounts({
                        superadmin: provider.publicKey,
                        settings,
                        lockup,
                        assetMint: mint,
                        lockupAssetVault: assetLockup,
                        tokenProgram: TOKEN_PROGRAM_ID,
                        systemProgram: SystemProgram.programId,
                        asset,
                        assetRewardPool
                    })
                    .rpc();

                const {
                    asset: assetMint,
                    bump,
                    duration: setDuration,
                    yieldBps,
                    minDeposit,
                    yieldMode,
                    index,
                    deposits,
                    depositCap,
                    rewardBoosts,
                    slashState
                } = await Lockup.fromAccountAddress(
                    provider.connection,
                    lockup
                );

                expect(assetMint.toString()).eq(mint.toString());
                expect(setDuration.toString()).eq(duration.toString());
                expect(yieldBps.toString()).eq(`${ i * 3 * 100 }`);
                expect(minDeposit.toString()).eq("1");
                expect(yieldMode.__kind).eq("Single");
                expect(yieldMode.fields.length).eq(1);
                expect(yieldMode.fields[0].toString()).eq("1");
                expect(index.toString()).eq(lockupId.toString());
                expect(deposits.toString()).eq("0");
                expect(depositCap.toString()).eq(`${ LAMPORTS_PER_SOL * 100_000 }`);
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

        const [assetLockup] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                lockup.toBuffer(),
                lockupData.asset.toBuffer(),
            ],
            program.programId
        );

        const [asset] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("asset"),
                lockupData.asset.toBuffer()
            ],
            program.programId
        );

        const amount = new BN(LAMPORTS_PER_SOL * 1_000);
        await mintTokens(
            lockupData.asset,
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
            lockupData.asset,
            coldWallet,
            true
        );

        const userAssetAta = getAssociatedTokenAddressSync(
            lockupData.asset,
            user.publicKey
        );

        const [lockupAssetVault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                lockup.toBuffer(),
                lockupData.asset.toBuffer(),
            ],
            program.programId
        );

        const coldWalletAtaIx = createAssociatedTokenAccountInstruction(
            user.publicKey,
            coldWalletVault,
            coldWallet,
            lockupData.asset
        );

        const ix = await program
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
                coldWallet,
                coldWalletVault,
                assetMint: lockupData.asset,
                userAssetAta,
                oracle: new PublicKey("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG"),
                lockupAssetVault,
                clock: SYSVAR_CLOCK_PUBKEY,
                tokenProgram: TOKEN_PROGRAM_ID,
                asset,
                systemProgram: SystemProgram.programId
            })
            .instruction();

        const { lastValidBlockHeight, blockhash } = await provider.connection.getLatestBlockhash();
        const message = new TransactionMessage({
            payerKey: user.publicKey,
            instructions: [coldWalletAtaIx, ix],
            recentBlockhash: blockhash
        }).compileToV0Message();

        const transactionTimestamp = Math.floor(Date.now() / 1000);
        await signAndSendTransaction(
            message,
            provider.connection,
            false,
            [user]
        );

        const lockupVaultData = await getAccount(
            provider.connection,
            lockupAssetVault
        );

        const coldWalletVaultData = await getAccount(
            provider.connection,
            coldWalletVault
        );

        expect(parseInt(lockupVaultData.amount.toString()))
            .approximately(
                Math.floor(0.3 * amount.toNumber()),
                1000
            );

        expect(parseInt(coldWalletVaultData.amount.toString()))
            .approximately(
                Math.floor(0.7 * amount.toNumber()),
                1000
            );

        const {
            user: userPointer,
            lockup: lockupPointer,
            amount: depositedAmount,
            initialUsdValue,
            lastSlashed,
            amountSlashed,
            unlockTs
        } = await Deposit.fromAccountAddress(
            provider.connection,
            deposit
        );

        expect(userPointer.toString()).eq(user.publicKey.toString());
        expect(lockupPointer.toString()).eq(lockup.toString());
        expect(depositedAmount.toString()).eq(amount.toString());
        // expect(initialUsdValue).eq("");
        expect(lastSlashed).eq(null);
        expect(amountSlashed.toString()).eq("0");
        expect(parseInt(unlockTs.toString()))
            .approximately(
                transactionTimestamp + parseInt(lockupData.duration.toString()),
                10 // 10 seconds delta
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
                lockupData.asset.toBuffer(),
            ],
            program.programId
        );

        const slashId = new BN(lockupData.slashState.index);
        const slashAmount = new BN(LAMPORTS_PER_SOL * 200);

        const assetLockupData = await getAccount(
            provider.connection,
            assetLockup
        );

        expect(parseInt(assetLockupData.amount.toString()))
            .gte(slashAmount.toNumber());

        const [slash] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("slash"),
                lockup.toBuffer(),
                slashId.toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        // Initialize slash
        await program
            .methods
            .initializeSlash({
                amount: slashAmount,
                lockupId
            })
            .accounts({
                settings,
                superadmin: provider.publicKey,
                lockup,
                assetMint: lockupData.asset,
                assetLockup,
                slash,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId
            })
            .rpc();

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
        const depositsToSlash = [];
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
        }


        // Slash individual deposits
        const tx = await program
            .methods
            .slashDeposits({
                lockupId,
                slashId,
                slashAmount
            })
            .accounts({
                settings,
                superadmin: provider.publicKey,
                lockup,
                assetMint: lockupData.asset,
                assetLockup,
                slash,
            })
            .remainingAccounts(depositsToSlash.map((deposit) => {
                return {
                    pubkey: deposit,
                    isSigner: false,
                    isWritable: true
                }
            }))
            .rpc();

        // await sleep(10);
        //
        // const {
        //     meta: {
        //         logMessages
        //     }
        // } = await program
        //     .provider
        //     .connection
        //     .getParsedTransaction(tx, "confirmed");

        const {
            slashedAccounts: slashedDeposits,
            slashedAmount: slashedAmountFromDeposits,
        } = await Slash.fromAccountAddress(
            provider.connection,
            slash
        );

        const coldWalletVault = getAssociatedTokenAddressSync(
            lockupData.asset,
            coldWallet,
            true
        );

        const [lockupAssetVault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                lockup.toBuffer(),
                lockupData.asset.toBuffer(),
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

        // Slash entire pool
        await program
            .methods
            .slashPool({
                lockupId,
                slashId
            })
            .accounts({
                settings,
                superadmin: provider.publicKey,
                lockup,
                asset: lockupData.asset,
                assetLockup,
                slash,
                tokenProgram: TOKEN_PROGRAM_ID,
                destination: coldWalletVault,
            })
            .rpc();

        const {
            amount: lockupAssetVaultBalancePost
        } = await getAccount(
            provider.connection,
            lockupAssetVault
        );
    });

    it("Withdraws & claims rewards.", async () => {
        await sleep(10);

        const lockupId = new BN(0);
        const depositId = new BN(0);

        const [lockup] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("lockup"),
                new BN(lockupId).toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        const lockupData = await Lockup.fromAccountAddress(
            provider.connection,
            lockup
        );

        const [lockupAssetVault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                lockup.toBuffer(),
                lockupData.asset.toBuffer(),
            ],
            program.programId
        );

        const {
            amount: lockupAssetVaultBalance
        } = await getAccount(
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

        const [asset] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("asset"),
                lockupData.asset.toBuffer()
            ],
            program.programId
        );

        const [assetRewardPool] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("reward_pool"),
                lockup.toBuffer(),
                lockupData.asset.toBuffer(),
            ],
            program.programId
        );

        const userAssetAta = getAssociatedTokenAddressSync(
            lockupData.asset,
            user.publicKey
        );

        await program
            .methods
            .withdraw({
                lockupId,
                depositId,
                rewardBoostId: null,
                amount: new BN(5 * LAMPORTS_PER_SOL)
            })
            .accounts({
                user: user.publicKey,
                settings,
                lockup,
                deposit,
                asset,
                assetMint: lockupData.asset,
                userAssetAta,
                lockupAssetVault,
                clock: SYSVAR_CLOCK_PUBKEY,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                assetRewardPool,
                rewardBoost: program.programId
            })
            .signers([user])
            .rpc()
    });

    it("Creates withdrawal intent for >30% of the insurance fund.", async () => {
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
            amount: leftToWithdraw
        } = depositData;

        const {
            asset: assetMint
        } = await Lockup.fromAccountAddress(
            provider.connection,
            lockup
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

        await program
            .methods
            .createIntent({
                amount: new BN(leftToWithdraw),
                lockupId,
                depositId,
            })
            .accounts({
                user: user.publicKey,
                settings,
                lockup,
                deposit,
                assetMint,
                userAssetAta,
                lockupAssetVault,
                systemProgram: SystemProgram.programId,
                tokenProgram: TOKEN_PROGRAM_ID,
                clock: SYSVAR_CLOCK_PUBKEY,
                intent
            })
            .signers([user])
            .rpc()
    });
});
