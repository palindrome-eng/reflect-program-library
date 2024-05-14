import * as anchor from "@coral-xyz/anchor";
import { Program, web3, BN } from "@coral-xyz/anchor";
const { SystemProgram, Transaction, PublicKey, Keypair } = anchor.web3;
import { SolanaStakeMarket } from "../target/types/solana_stake_market";
import { assert, expect } from "chai";
import crypto from "crypto";
const LAMPORTS_PER_SOL = 1000000000;

describe("solana-stake-market", () => {
	const provider = anchor.AnchorProvider.env();
	anchor.setProvider(provider);
	const program = anchor.workspace
		.SolanaStakeMarket as Program<SolanaStakeMarket>;
	let orderBookAccount;
	// fetching 10 vote accounts from network
	console.log("fetching some vote accounts from network...");
	let voteAccounts: web3.PublicKey[] = [];
	let tempKeypair = Keypair.generate();

	before(async () => {
		const [orderBookPda, _] = PublicKey.findProgramAddressSync(
			[Buffer.from("orderBook")],
			program.programId
		);
		orderBookAccount = orderBookPda;

		// Initialize the order book with minimal space
		await program.rpc.initializeOrderBook({
			accounts: {
				orderBook: orderBookAccount,
				user: provider.wallet.publicKey,
				systemProgram: SystemProgram.programId,
			},
			signers: [],
		});

		const orderBook = await program.account.orderBook.fetch(orderBookAccount);
		console.log(`OrderBook initialized at ${orderBookAccount}`);
		assert.isTrue(orderBook.tvl.eq(new BN(0)), "TVL should initialize as 0");
		console.log(`current TVL =  ${orderBook.tvl}`);
		assert.isTrue(orderBook.bids.eq(new BN(0)), "there should be 0 bids.");
		console.log(`current bids =  ${orderBook.bids}`);
		assert.isTrue(
			orderBook.globalNonce.eq(new BN(0)),
			"Global nonce should be initialized to 0."
		);
		console.log(`global nonce = ${orderBook.globalNonce}`);
	});

	it("Places and closes bids correctly", async () => {
		const rate = new BN(970_000_000); // A valid rate greater than the minimum
		const amount = new BN(1_000_000_000); // Amount greater than the rate

		// Generate a bid account PDA
		const currentNonce = (
			await program.account.orderBook.fetch(orderBookAccount)
		).globalNonce;
		const [bidPda, _] = PublicKey.findProgramAddressSync(
			[
				Buffer.from("bid"),
				provider.wallet.publicKey.toBuffer(),
				currentNonce.toBuffer("le", 8),
			],
			program.programId
		);

		// Place the bid
		await program.rpc.placeBid(rate, amount, {
			accounts: {
				user: provider.wallet.publicKey,
				bid: bidPda,
				orderBook: orderBookAccount,
				systemProgram: SystemProgram.programId,
			},
			signers: [],
		});

		// Check balances before closing the bid
		const balanceBefore = await provider.connection.getBalance(
			provider.wallet.publicKey
		);
		console.log(`Balance before closing the bid: ${balanceBefore}`);

		// Close the bid
		await program.rpc.closeBid({
			accounts: {
				bid: bidPda,
				user: provider.wallet.publicKey,
				orderBook: orderBookAccount,
			},
			signers: [],
		});

		// Check balances after closing the bid
		const balanceAfter = await provider.connection.getBalance(
			provider.wallet.publicKey
		);
		console.log(`Balance after closing the bid: ${balanceAfter}`);

		assert(
			balanceAfter > balanceBefore,
			"User balance should increase after closing the bid"
		);
		console.log(
			`Bid closed and funds returned. Balance increased by ${
				balanceAfter - balanceBefore
			}`
		);
	});

	it("Places bids at different rates and checks order book size", async () => {
		const rates = [970_000_000, 980_000_000, 990_000_000]; // Rates as per 0.97:1, 0.98:1, 0.99:1
		let previousBidsCount = (
			await program.account.orderBook.fetch(orderBookAccount)
		).bids.toNumber();

		for (const rate of rates) {
			const currentNonce = (
				await program.account.orderBook.fetch(orderBookAccount)
			).globalNonce;
			const [bidPda, _] = PublicKey.findProgramAddressSync(
				[
					Buffer.from("bid"),
					provider.wallet.publicKey.toBuffer(),
					currentNonce.toBuffer("le", 8),
				],
				program.programId
			);

			// Airdrop SOL to cover the bid and transaction fees
			const airdropSignature = await provider.connection.requestAirdrop(
				provider.wallet.publicKey,
				5_000_000_000
			);
			await provider.connection.confirmTransaction(
				airdropSignature,
				"confirmed"
			);

			const tx = new Transaction();
			tx.add(
				program.instruction.placeBid(new BN(rate), new BN(1_000_000_000), {
					accounts: {
						user: provider.wallet.publicKey,
						bid: bidPda,
						orderBook: orderBookAccount,
						systemProgram: SystemProgram.programId,
					},
				})
			);

			try {
				// Execute the transaction
				await provider.sendAndConfirm(tx, [], { commitment: "confirmed" });

				// Fetch the updated order book
				const updatedOrderBook = await program.account.orderBook.fetch(
					orderBookAccount
				);
				expect(updatedOrderBook.bids.toNumber()).to.equal(
					previousBidsCount + 1
				);
				console.log(
					`Order book size after bid at rate ${rate}: ${updatedOrderBook.bids}`
				);

				// Update previousBidsCount for the next iteration
				previousBidsCount = updatedOrderBook.bids.toNumber();
			} catch (error) {
				console.error(`Error placing bid at rate ${rate}: ${error}`);
			}
		}
	});

	it("Fails to place a bid with an invalid rate", async () => {
		const currentNonce = (
			await program.account.orderBook.fetch(orderBookAccount)
		).globalNonce;
		const [bidPda] = PublicKey.findProgramAddressSync(
			[
				Buffer.from("bid"),
				provider.wallet.publicKey.toBuffer(),
				currentNonce.toBuffer("le", 8),
			],
			program.programId
		);

		try {
			await program.rpc.placeBid(
				new BN(500_000_000), // Invalid rate, below 600_000_000
				new BN(1_000_000_000),
				{
					accounts: {
						bid: bidPda,
						orderBook: orderBookAccount,
						user: provider.wallet.publicKey,
						systemProgram: SystemProgram.programId,
					},
					signers: [],
				}
			);
			assert.fail("Bid with an invalid rate should have failed.");
		} catch (error) {
			assert.include(
				error.message,
				"BelowMinimumRate",
				"Error should be related to the minimum rate."
			);
		}
	});

	it("Fails to place a bid with insufficient funding", async () => {
		const currentNonce = (
			await program.account.orderBook.fetch(orderBookAccount)
		).globalNonce;
		const [bidPda] = PublicKey.findProgramAddressSync(
			[
				Buffer.from("bid"),
				provider.wallet.publicKey.toBuffer(),
				currentNonce.toBuffer("le", 8),
			],
			program.programId
		);

		try {
			await program.rpc.placeBid(
				new BN(900_000_000), // Valid rate
				new BN(800_000_000),
				{
					// Insufficient amount
					accounts: {
						bid: bidPda,
						orderBook: orderBookAccount,
						user: provider.wallet.publicKey,
						systemProgram: SystemProgram.programId,
					},
					signers: [],
				}
			);
			assert.fail("Bid with insufficient funding should have failed.");
		} catch (error) {
			assert.include(
				error.message,
				"UnfundedBid",
				"Error should be related to insufficient funding."
			);
		}
	});

	it("fetch some vote accounts and airdrop sol to tempKeypair", async () => {
		const res = await provider.connection.getVoteAccounts(
			provider.connection.commitment
		);
		//for live network
		// voteAccounts = res.current
		// 	.slice(0, 11)
		// 	.map((validator) => new PublicKey(validator.votePubkey));
		voteAccounts = [new PublicKey(res.current[0].votePubkey)];

		const airdropTx = await provider.connection.requestAirdrop(
			tempKeypair.publicKey,
			1 * LAMPORTS_PER_SOL
		);
		await provider.connection.confirmTransaction(airdropTx);

		// assert.equal(10, voteAccounts.length);

		// creating #numberOfStakeAccounts stake accounts and delegating
		// create random string for seedPrefix
		const seedPrefix = crypto.randomBytes(4).toString("hex");
		console.log("seedPrefix", seedPrefix);

		// initial index for stake pubkey seeds
		let initialIndex = -1;
		// total amount of lamports to be staked
		// deducting a little bit to cover TX fees
		// const totalStakeAmount = 0.8 * LAMPORTS_PER_SOL;
		let stakeAccounts: web3.AccountMeta[] = [];

		let calculatedStakePubkeyNum = 0;
		while (calculatedStakePubkeyNum < 1) {
			initialIndex += 1;

			// creating #numberOfStakeAccounts stake accounts and delegating
			for (let i = 0; i < 1; i++) {
				let seedPostFix = (initialIndex + i).toString();
				let stakePubkey = await web3.PublicKey.createWithSeed(
					tempKeypair.publicKey,
					`${seedPrefix}-${seedPostFix}`,
					program.programId
				);

				if (web3.PublicKey.isOnCurve(stakePubkey)) {
					stakeAccounts = [];
					calculatedStakePubkeyNum = 0;
					break;
				}

				stakeAccounts.splice(i, 0, {
					pubkey: voteAccounts[i], // vote account pubkey
					isSigner: false,
					isWritable: false,
				});
				stakeAccounts.splice(i + 1, 0, {
					pubkey: stakePubkey, //stake account pubkey
					isSigner: false,
					isWritable: true,
				});
				calculatedStakePubkeyNum += 1;
			}
		}
		console.log("stakeAccounts", stakeAccounts);
		const currentNonce = (
			await program.account.orderBook.fetch(orderBookAccount)
		).globalNonce;
		const [bidPda, _] = PublicKey.findProgramAddressSync(
			[
				Buffer.from("bid"),
				provider.wallet.publicKey.toBuffer(),
				currentNonce.sub(new BN(1)).toBuffer("le", 8),
			],
			program.programId
		);
		let remainingAccounts: web3.AccountMeta[] = [
			{
				pubkey: bidPda, //stake account pubkey
				isSigner: false,
				isWritable: true,
			},
		];
		let tx = await program.methods
			.sellStake(new BN(1))
			.accounts({
				stakeAccount: stakeAccounts[1].pubkey,
				orderBook: orderBookAccount,
				seller: tempKeypair.publicKey,
				stakeProgram: web3.StakeProgram.programId,
				systemProgram: SystemProgram.programId,
				rentSysvar: web3.SYSVAR_RENT_PUBKEY,
			})
			.remainingAccounts(remainingAccounts)
			.signers([tempKeypair])
			.rpc();
		//Error processing Instruction 0: An account required by the instruction is missing TODO
		const confirmation = await provider.connection.confirmTransaction(
			tx,
			"confirmed"
		);
		console.log("Transaction confirmation status:", confirmation.value);
	});
});
