import * as anchor from "@coral-xyz/anchor";
import {Program} from "@coral-xyz/anchor";
import {SolanaStakeMarket} from "../target/types/solana_stake_market";
import {assert, expect} from "chai";
import {
    Authorized, Keypair, LAMPORTS_PER_SOL, Lockup,
    Message,
    PublicKey,
    StakeProgram, SYSVAR_CLOCK_PUBKEY,
    SYSVAR_RENT_PUBKEY,
    TransactionMessage,
    VersionedMessage
} from "@solana/web3.js";
import {Bid, createSellStakeInstruction, OrderBook} from "../sdk";
import BN from "bn.js";

const { SystemProgram, Transaction } = anchor.web3;

const sleep = (seconds: number) => new Promise((resolve) => {
    setTimeout(() => {
        resolve(null);
    }, seconds * 1000);
});

function getBids(count: number, programId: PublicKey) {
    const bids: PublicKey[] = [];

    for (let i = 0; i < count; i++) {
        const [bidPda] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("bid"),
                new BN(i).toArrayLike(Buffer, 'le', 8)
            ],
            programId
        );

        bids.push(bidPda);
    }

    return bids;
}

describe("solana-stake-market", () => {
    const provider = anchor.AnchorProvider.local();
    anchor.setProvider(provider);
    const program = anchor.workspace.SolanaStakeMarket as Program<SolanaStakeMarket>;
    let orderBookAccount: PublicKey;
    let orderBookAccountData: OrderBook;
    let alice: Keypair;
    let aliceStakeAccount: Keypair;

    before(async () => {
        [orderBookAccount] = PublicKey.findProgramAddressSync(
            [Buffer.from("orderBook")],
            program.programId
        );

        alice = Keypair.generate();
        aliceStakeAccount = Keypair.generate();

        // Initialize the order book with minimal space
        await program
            .methods
            .initializeOrderBookWrapper()
            .accounts({
                orderBook: orderBookAccount,
                user: provider.wallet.publicKey,
                systemProgram: SystemProgram.programId,
            })
            .rpc().catch(err => console.error(err));

        const orderBook = await OrderBook.fromAccountAddress(
            provider.connection,
            orderBookAccount
        );

        orderBookAccountData = orderBook;

        console.log(`OrderBook initialized at ${orderBookAccount}`);
        expect(orderBook.tvl.toString()).eq("0");

        console.log(`current TVL =  ${orderBook.tvl}`);
        expect(orderBook.bids.toString()).eq("0");

        console.log(`current bids =  ${orderBook.bids}`);
        expect(orderBook.globalNonce.toString()).eq("0");
    });

    it("Places and closes bids correctly", async () => {
        const rate = new anchor.BN(970_000_000); // A valid rate greater than the minimum
        const amount = new anchor.BN(1_000_000_000); // Amount greater than the rate

        const toDeposit = amount.toNumber() / Math.pow(10, 9) * rate.toNumber();

        // Generate a bid account PDA
        const [bidPda] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("bid"),
                new BN(orderBookAccountData.globalNonce).toArrayLike(Buffer, 'le', 8)
            ],
            program.programId
        );

        const [bidVault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                bidPda.toBuffer()
            ],
            program.programId
        );

        const placeBid = program
            .methods
            .placeBid(
                rate,
                amount
            )
            .accounts({
                user: provider.wallet.publicKey,
                bid: bidPda,
                orderBook: orderBookAccount,
                systemProgram: SystemProgram.programId,
                bidVault
            });

        const vaultBalanceBeforePlacingBid = await provider.connection.getBalance(bidVault);
        expect(vaultBalanceBeforePlacingBid).eq(0);

        const balanceBeforePlacingBid = await provider.connection.getBalance(provider.wallet.publicKey);
        const { blockhash } = await provider.connection.getLatestBlockhash();
        const {
            value: expectedNetworkFeeInLamports
        } = await provider.connection.getFeeForMessage(
            (new TransactionMessage({
                payerKey: provider.wallet.publicKey,
                instructions: [await placeBid.instruction()],
                recentBlockhash: blockhash
            })).compileToV0Message()
        );

        await placeBid.rpc().catch(err => console.error(err));

        const vaultBalanceAfterPlacingBid = await provider.connection.getBalance(bidVault);
        const balanceAfterPlacingBid = await provider.connection.getBalance(provider.wallet.publicKey);

        expect(balanceAfterPlacingBid).to.approximately(
            balanceBeforePlacingBid - toDeposit - expectedNetworkFeeInLamports,
            5_000_000
        );

        expect(vaultBalanceAfterPlacingBid).eq(toDeposit);

        // Close the bid
        const closeBid = program
            .methods
            .closeBid(
                new BN(orderBookAccountData.globalNonce)
            )
            .accounts({
                bid: bidPda,
                user: provider.wallet.publicKey,
                orderBook: orderBookAccount,
                bidVault,
                systemProgram: SystemProgram.programId,
            });

        const { blockhash: blockhash2 } = await provider.connection.getLatestBlockhash();
        const {
            value: expectedNetworkFeeInLamports2
        } = await provider.connection.getFeeForMessage(
            (new TransactionMessage({
                payerKey: provider.wallet.publicKey,
                instructions: [await placeBid.instruction()],
                recentBlockhash: blockhash2
            })).compileToV0Message()
        );

        await closeBid.rpc().catch(err => console.error(err));

        const vaultBalanceAfterClosingBid = await provider.connection.getBalance(bidVault);
        const balanceAfterClosingBid = await provider.connection.getBalance(provider.wallet.publicKey);
        expect(balanceAfterClosingBid).to.approximately(
            balanceAfterPlacingBid + toDeposit - expectedNetworkFeeInLamports2,
            5_000_000
        );
        expect(vaultBalanceAfterClosingBid).eq(0);
    });

    it("Places bids at different rates and checks order book size", async () => {
        const rates = [970_000_000, 980_000_000, 990_000_000]; // Rates as per 0.97:1, 0.98:1, 0.99:1

        let orderBook = await OrderBook.fromAccountAddress(provider.connection, orderBookAccount);
        let previousBidsCount = typeof orderBook.bids === "number" ? orderBook.bids : orderBook.bids.toNumber();

        const refreshOrderBook = async () => orderBook = await OrderBook.fromAccountAddress(provider.connection, orderBookAccount);

        for (const rate of rates) {
            const currentNonce = orderBook.globalNonce;
            const [bidPda] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("bid"),
                    new BN(currentNonce).toArrayLike(Buffer, 'le', 8)
                ],
                program.programId
            );

            const [bidVault] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("vault"),
                    bidPda.toBuffer()
                ],
                program.programId
            );

            await program
                .methods
                .placeBid(
                    new anchor.BN(rate),
                    new anchor.BN(1_000_000_000),
                )
                .accounts({
                    user: provider.wallet.publicKey,
                    bid: bidPda,
                    orderBook: orderBookAccount,
                    systemProgram: SystemProgram.programId,
                    bidVault
                })
                .rpc().catch(err => console.error(err));

            await refreshOrderBook();
            expect(parseInt(orderBook.bids.toString())).eq(previousBidsCount + 1);
            previousBidsCount = typeof orderBook.bids === "number" ? orderBook.bids : orderBook.bids.toNumber();
        }
    });

    it("Fails to place a bid with an invalid rate", async () => {
        let orderBook = await OrderBook.fromAccountAddress(provider.connection, orderBookAccount);
        const currentNonce = orderBook.globalNonce;

        const [bidPda] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("bid"),
                new BN(currentNonce).toArrayLike(Buffer, 'le', 8)
            ],
            program.programId
        );

        const [bidVault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                bidPda.toBuffer()
            ],
            program.programId
        );

        let error: string = "";
        await program
            .methods
            .placeBid(
                new anchor.BN(500_000_000), // Invalid rate, below 600_000_000
                new anchor.BN(1_000_000_000)
            )
            .accounts({
                bid: bidPda,
                orderBook: orderBookAccount,
                user: provider.wallet.publicKey,
                systemProgram: SystemProgram.programId,
                bidVault
            })
            .rpc()
            .catch(err => error = err.message);

        expect(error).to.include(
            "BelowMinimumRate",
            "Error should be related to the minimum rate."
        );
    });

    // This test is not needed anymore, since there is no more checks for insufficient funding.
    // it("Fails to place a bid with insufficient funding", async () => {
    //     let orderBook = await OrderBook.fromAccountAddress(provider.connection, orderBookAccount);
    //     const currentNonce = orderBook.globalNonce;
    //
    //     const [bidPda] = PublicKey.findProgramAddressSync(
    //         [
    //             Buffer.from("bid"),
    //             new BN(currentNonce).toArrayLike(Buffer, 'le', 8)
    //         ],
    //         program.programId
    //     );
    //
    //     const [bidVault] = PublicKey.findProgramAddressSync(
    //         [
    //             Buffer.from("vault"),
    //             bidPda.toBuffer()
    //         ],
    //         program.programId
    //     );
    //
    //     let error = "";
    //     await program
    //         .methods
    //         .placeBid(
    //             new anchor.BN(900_000_000), // Valid rate
    //             new anchor.BN(800_000_000)
    //         )
    //         .accounts({
    //             bid: bidPda,
    //             orderBook: orderBookAccount,
    //             user: provider.wallet.publicKey,
    //             systemProgram: SystemProgram.programId,
    //             bidVault
    //         })
    //         .rpc()
    //         .catch(err => error = err.message);
    //
    //     expect(error).to.include("UnfundedBid", "Error should be related to insufficient funding.");
    // });

    it("successfully creates stake account and activates stake", async () => {
        const minimumRent = await provider.connection.getMinimumBalanceForRentExemption(
            StakeProgram.space
        );

        {
            let {
                blockhash,
                lastValidBlockHeight
            } = await provider.connection.getLatestBlockhash();

            const airdropTx = await provider.connection.requestAirdrop(alice.publicKey, 250 * LAMPORTS_PER_SOL);
            await provider.connection.confirmTransaction({
                blockhash,
                lastValidBlockHeight,
                signature: airdropTx
            });
        }

        const accountTx = StakeProgram.createAccount({
            authorized: new Authorized(
                alice.publicKey,
                alice.publicKey
            ),
            fromPubkey: alice.publicKey,
            lamports: 4 * LAMPORTS_PER_SOL + minimumRent,
            stakePubkey: aliceStakeAccount.publicKey,
            lockup: new Lockup(0,0, alice.publicKey)
        });

        let {
            blockhash,
            lastValidBlockHeight
        } = await provider.connection.getLatestBlockhash();

        accountTx.feePayer = alice.publicKey;
        accountTx.recentBlockhash = blockhash;

        accountTx.sign(alice, aliceStakeAccount);
        let txId = await provider.connection.sendRawTransaction(accountTx.serialize());
        await provider.connection.confirmTransaction({
            blockhash,
            lastValidBlockHeight,
            signature: txId
        });

        const stakeAccountBalance = await provider.connection.getBalance(aliceStakeAccount.publicKey);
        const {
            state,
            inactive,
            active
        } = await provider.connection.getStakeActivation(aliceStakeAccount.publicKey);

        expect(stakeAccountBalance).approximately(4 * LAMPORTS_PER_SOL, LAMPORTS_PER_SOL / 10);
        expect(state).eq("inactive");
        expect(inactive).eq(4 * LAMPORTS_PER_SOL);
        expect(active).eq(0);

        const validators = await provider.connection.getVoteAccounts();
        const selectedValidator = validators.current[0];
        const selectedValidatorPubkey = new PublicKey(selectedValidator.votePubkey);

        const delegateTx = StakeProgram.delegate({
            stakePubkey: aliceStakeAccount.publicKey,
            authorizedPubkey: alice.publicKey,
            votePubkey: selectedValidatorPubkey,
        });

        {
            let {
                blockhash,
                lastValidBlockHeight
            } = await provider.connection.getLatestBlockhash();

            delegateTx.feePayer = alice.publicKey;
            delegateTx.recentBlockhash = blockhash;
        }

        delegateTx.sign(alice);
        txId = await provider.connection.sendRawTransaction(delegateTx.serialize());

        {
            let {
                blockhash,
                lastValidBlockHeight
            } = await provider.connection.getLatestBlockhash();

            await provider.connection.confirmTransaction({
                blockhash,
                lastValidBlockHeight,
                signature: txId
            });
        }

        while (true) {
            await sleep(1);

            const {
                state,
                inactive,
                active
            } = await provider.connection.getStakeActivation(aliceStakeAccount.publicKey);

            if (state === "active") break;
            console.log(`Stake is ${state}.`);
        }

        console.log("Refreshing stake account.");
        {
            const {
                state,
                inactive,
                active
            } = await provider.connection.getStakeActivation(aliceStakeAccount.publicKey);

            expect(state).eq("active");
            expect(inactive).eq(0);
            expect(active).eq(4 * LAMPORTS_PER_SOL);
        }
    });

    it("Successfully places bids and sells stake into a bid (with splitting).", async () => {
        const rate = new anchor.BN(970_000_000); // A valid rate greater than the minimum
        const amount = new anchor.BN(1_000_000_000); // Amount greater than the rate

        let orderBook = await OrderBook.fromAccountAddress(provider.connection, orderBookAccount);
        const currentNonce = typeof orderBook.globalNonce === "number" ? orderBook.globalNonce : orderBook.globalNonce.toNumber();

        const bids: PublicKey[] = [];
        const cosigns: Keypair[] = [];
        for (let i = 0; i < 3; i++) {
            const [bidPda] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("bid"),
                    new BN(currentNonce + i).toArrayLike(Buffer, 'le', 8)
                ],
                program.programId
            );

            const [bidVault] = PublicKey.findProgramAddressSync(
                [
                    Buffer.from("vault"),
                    bidPda.toBuffer()
                ],
                program.programId
            );

            await program
                .methods
                .placeBid(
                    rate,
                    amount
                )
                .accounts({
                    orderBook: orderBookAccount,
                    systemProgram: SystemProgram.programId,
                    user: provider.wallet.publicKey,
                    bid: bidPda,
                    bidVault
                })
                .rpc();

            bids.push(bidPda);
            cosigns.push(Keypair.generate());
            console.log(`Initialized bid no ${i}`);
        }

        const rentIx = cosigns.map(keypair => {
            return SystemProgram.transfer({
                fromPubkey: alice.publicKey,
                lamports: 2282880,
                toPubkey: keypair.publicKey,
            });
        });

        const sellStakeIx = createSellStakeInstruction(
            {
                orderBook: orderBookAccount,
                systemProgram: SystemProgram.programId,
                stakeAccount: aliceStakeAccount.publicKey,
                rentSysvar: SYSVAR_RENT_PUBKEY,
                seller: alice.publicKey,
                clock: SYSVAR_CLOCK_PUBKEY,
                stakeProgram: StakeProgram.programId,
                anchorRemainingAccounts: bids.map((bid, index) => ([
                    {
                        pubkey: bid,
                        isSigner: false,
                        isWritable: true
                    },
                    {
                        pubkey: PublicKey.findProgramAddressSync([Buffer.from("vault"), bid.toBuffer()], program.programId)[0],
                        isSigner: false,
                        isWritable: true
                    },
                    {
                        pubkey: cosigns[index].publicKey,
                        isSigner: false,
                        isWritable: true
                    }
                ])).flat()
            },
            {
                totalStakeAmount: new BN(3 * LAMPORTS_PER_SOL)
            }
        );

        const tx = new Transaction();
        const {
            blockhash,
            lastValidBlockHeight
        } = await provider.connection.getLatestBlockhash();

        // // In case you want to make new staking accounts rent-exempt before running the sell_stake.
        // // Added this just for testing.
        // tx.add(...rentIx);

        tx.add(sellStakeIx);
        tx.feePayer = alice.publicKey;
        tx.recentBlockhash = blockhash;
        tx.sign(
            alice,
            ...cosigns
        );

        const sent = await provider
                .connection
                .sendRawTransaction(tx.serialize(), { skipPreflight: false });

        await provider.connection.confirmTransaction({
            signature: sent,
            lastValidBlockHeight,
            blockhash
        });

        await Promise.all(bids.map(async (bid) => {
            const accountInfo = await provider
                .connection
                .getAccountInfo(bid, "confirmed");

            const [{
                amount,
                fulfilled
            }] = Bid.fromAccountInfo(
                accountInfo,
            );

            const [vault]  = PublicKey.findProgramAddressSync([Buffer.from("vault"), bid.toBuffer()], program.programId);

            const vaultBalance = await provider.connection.getBalance(vault);
            const bidAmount = typeof amount === "number" ? amount : amount.toNumber();

            // Expect all bids to be fulfilled (since we're selling 3SOL into 3x 1SOL bids).
            expect(bidAmount).eq(0);
            expect(vaultBalance).eq(0);
            expect(fulfilled).eq(true);
        }));
    });
});