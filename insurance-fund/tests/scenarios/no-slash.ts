import * as anchor from "@coral-xyz/anchor";
import {AnchorProvider, Program} from "@coral-xyz/anchor";
import {InsuranceFund} from "../../target/types/insurance_fund";
import { Keypair, LAMPORTS_PER_SOL, PublicKey, SystemProgram, SYSVAR_CLOCK_PUBKEY} from "@solana/web3.js";
import {before} from "mocha";
import BN from "bn.js";
import {Asset, PROGRAM_ID, Settings} from "../../sdk/generated";
import {expect} from "chai";
import createToken from "../helpers/createToken";
import mintTokens from "../helpers/mintTokens";
import {getAccount, getAssociatedTokenAddressSync, TOKEN_PROGRAM_ID} from "@solana/spl-token";
import NodeWallet from "@coral-xyz/anchor/dist/cjs/nodewallet";
import sleep from "../helpers/sleep";

describe("insurance-fund", () => {
    const provider = AnchorProvider.local("http://127.0.0.1:8899");
    anchor.setProvider(provider);

    console.log(provider.connection.rpcEndpoint);

    const program = anchor.workspace.InsuranceFund as Program<InsuranceFund>;

    const oracle = new PublicKey("H6ARHf6YXhGYeQfUzQNGk6rDNnLBQKrenN712K4AQJEG");
    let coldWallet: PublicKey;
    let settings: PublicKey;
    let users: Keypair[] = [];

    let lst: PublicKey | null = null;
    let asset: PublicKey | null = null;
    let lockup: PublicKey | null = null;
    let lockupAssetVault: PublicKey | null = null;
    let assetRewardPool: PublicKey | null = null;

    before(async () => {
        coldWallet = Keypair.generate().publicKey;
        // initialize cold wallet
        await provider.connection.requestAirdrop(coldWallet, 10 * LAMPORTS_PER_SOL);
        // await sleep(5);

        for (let i = 0; i < 100; i++) {
            const keypair = Keypair.generate();
            try {
                await provider.connection.requestAirdrop(keypair.publicKey, 10 * LAMPORTS_PER_SOL);
            } catch (err) {
                console.log({ err });
                throw err;
            }
            users.push(keypair);
        }

        [settings] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("settings")
            ],
            program.programId
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

    it("Mints and adds asset to insurance pool.", async () => {
        const token = await createToken(
            provider.connection,
            provider
        );

        lst = token;

        // initialize cold wallet ata
        await mintTokens(lst, provider, 500 * LAMPORTS_PER_SOL, coldWallet);

        [asset] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("asset"),
                token.toBuffer()
            ],
            PROGRAM_ID
        );

        const tx = await program
            .methods
            .addAsset()
            .accounts({
                assetMint: token,
                asset,
                systemProgram: SystemProgram.programId,
                settings,
                superadmin: provider.publicKey,
                // Just to pass the test
                oracle,
            })
            .rpc();

        const assetData = await Asset.fromAccountAddress(
            provider.connection,
            asset,
        );

        expect(assetData.tvl.toString()).eq("0");
        expect(assetData.mint.toString()).eq(lst.toString());
        expect(assetData.oracle.__kind).eq("Switchboard");
        expect(assetData.oracle.fields[0].toString()).eq(oracle.toString());
    });

    it("Creates lockup", async () => {

        const settingsData = await Settings.fromAccountAddress(
            provider.connection,
            settings
        );

        [lockup] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("lockup"),
                new BN(settingsData.lockups).toArrayLike(Buffer, "le", 8)
            ],
            PROGRAM_ID
        );

        [lockupAssetVault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                lockup.toBuffer(),
                lst.toBuffer()
            ],
            PROGRAM_ID
        );

        [assetRewardPool] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("reward_pool"),
                lockup.toBuffer(),
                lst.toBuffer()
            ],
            PROGRAM_ID
        );

        await program
            .methods
            .initializeLockup({
                asset: lst,
                minDeposit: new BN(0),
                duration: new BN(1),
                yieldBps: new BN(1000),
                yieldMode: {
                    single: { 0: [new BN(1)] } // 1 rUSD per 1 unit of deposit [lamport] per one lockup period
                },
                depositCap: new BN(0)
            })
            .accounts({
                superadmin: provider.publicKey,
                settings,
                lockup,
                asset,
                assetMint: lst,
                lockupAssetVault,
                tokenProgram: TOKEN_PROGRAM_ID,
                systemProgram: SystemProgram.programId,
                assetRewardPool
            })
            .rpc()
    });

    it('Mints tokens to 100 users and restakes them', async () => {
        await Promise.all(users.map(async (user) => {
            await mintTokens(
                lst,
                provider,
                100_000 * LAMPORTS_PER_SOL,
                user.publicKey
            );
        }));

        for (let [index, user] of users.entries()) {
            const ata = getAssociatedTokenAddressSync(
                lst,
                user.publicKey,
                true
            );

            const {
                amount
            } = await getAccount(provider.connection, ata);

            expect(amount.toString()).eq(`${100_000 * LAMPORTS_PER_SOL}`);

            const [deposit] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("deposit"),
                    lockup.toBuffer(),
                    new BN(index).toArrayLike(Buffer, "le", 8)
                ],
                PROGRAM_ID
            );

            const coldWalletVault = getAssociatedTokenAddressSync(
                lst,
                coldWallet,
                true
            );

            await program
                .methods
                .restake({
                    lockupId: new BN(0),
                    amount: new BN(100_000 * LAMPORTS_PER_SOL)
                })
                .accounts({
                    settings,
                    user: user.publicKey,
                    lockup,
                    deposit,
                    coldWallet,
                    coldWalletVault,
                    asset,
                    assetMint: lst,
                    userAssetAta: getAssociatedTokenAddressSync(lst, user.publicKey, true),
                    oracle,
                    clock: SYSVAR_CLOCK_PUBKEY,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    systemProgram: SystemProgram.programId,
                    lockupAssetVault
                })
                .signers([user])
                .rpc()

            const {
                amount: postAmount
            } = await getAccount(provider.connection, ata);

            expect(postAmount.toString()).eq(`${0}`);
        }
    });

    it("Cranks rewards to the pool", async () => {

        // 500 per user
        const amount = new BN(100 * 500 * LAMPORTS_PER_SOL);

        await mintTokens(
            lst,
            provider,
            amount.toNumber()
        );

        await program
            .methods
            .depositRewards({
                lockupId: new BN(0),
                amount
            })
            .accounts({
                caller: provider.publicKey,
                lockup,
                assetMint: lst,
                callerAssetAta: getAssociatedTokenAddressSync(lst, provider.publicKey),
                assetRewardPool,
                tokenProgram: TOKEN_PROGRAM_ID
            })
            .rpc();
    });

    it(`Waits 1 second until the lockup is over.`, async () => {
        await sleep(1);
    });

    it("Claims deposits + rewards for all users.", async () => {
        for (let [index, user] of users.slice(0, 30).entries()) {

            const [deposit] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("deposit"),
                    lockup.toBuffer(),
                    new BN(index).toArrayLike(Buffer, "le", 8)
                ],
                PROGRAM_ID
            );

            const userAssetAta = getAssociatedTokenAddressSync(lst, user.publicKey);
            const {
                amount: balancePre
            } = await getAccount(
                provider.connection,
                userAssetAta
            );

            expect(balancePre.toString()).eq("0");

            const coldWalletVault = getAssociatedTokenAddressSync(
                lst,
                coldWallet,
                true
            );

            await program
                .methods
                .withdraw({
                    lockupId: new BN(0),
                    depositId: new BN(index),
                    rewardBoostId: null,
                    amount: new BN(100_000 * LAMPORTS_PER_SOL)
                })
                .accounts({
                    settings,
                    user: user.publicKey,
                    lockup,
                    deposit,
                    rewardBoost: PROGRAM_ID,
                    asset,
                    assetMint: lst,
                    userAssetAta,
                    lockupAssetVault,
                    assetRewardPool,
                    clock: SYSVAR_CLOCK_PUBKEY,
                    tokenProgram: TOKEN_PROGRAM_ID,
                    systemProgram: SystemProgram.programId,
                    coldWallet,
                    coldWalletVault
                })
                .signers([user])
                .rpc();

            const {
                amount: balancePost
            } = await getAccount(
                provider.connection,
                userAssetAta
            );

            // Fails after 30th attempt, when 30% of the insurance fund (in hot wallet)
            // is entirely gone.

            console.log({ processed: index, total: 100 });
            expect(new BN(balancePost.toString()).toNumber())
                .approximately(
                    100_500 * LAMPORTS_PER_SOL,
                    // Allow for 0.1 difference in calculation
                    // Due to type casting from float to u64
                    0.1 * LAMPORTS_PER_SOL
                );
        }
    });
});