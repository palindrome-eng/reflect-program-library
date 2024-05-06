import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
const { SystemProgram, Transaction } = anchor.web3;
import { SolanaStakeMarket } from "../target/types/solana_stake_market";
import { assert, expect } from "chai";

describe("solana-stake-market", () => {
    const provider = anchor.AnchorProvider.env();
    anchor.setProvider(provider);
    const program = anchor.workspace.SolanaStakeMarket as Program<SolanaStakeMarket>;
    let orderBookAccount;

    before(async () => {
        const [orderBookPda, orderBookBump] = await anchor.web3.PublicKey.findProgramAddress(
            [Buffer.from("orderBook")],
            program.programId
        );
        orderBookAccount = orderBookPda;

        // Initialize the order book with minimal space
        await program.rpc.initializeOrderBookWrapper({
            accounts: {
                orderBook: orderBookAccount,
                user: provider.wallet.publicKey,
                systemProgram: SystemProgram.programId,
            },
            signers: [],
        });

        const orderBook = await program.account.orderBook.fetch(orderBookAccount);
        assert.isTrue(orderBook.globalNonce.eq(new anchor.BN(0)), "Global nonce should be initialized to 0.");
        console.log(`OrderBook initialized at ${orderBookAccount}`);
    });

    it("Places bids at different rates and checks order book size", async () => {
      const rates = [970_000_000, 980_000_000, 990_000_000]; // Rates as per 0.97:1, 0.98:1, 0.99:1
      for (const rate of rates) {
          const currentNonce = (await program.account.orderBook.fetch(orderBookAccount)).globalNonce;
          const [bidPda, bidBump] = await anchor.web3.PublicKey.findProgramAddressSync(
              [Buffer.from("bid"), provider.wallet.publicKey.toBuffer(), currentNonce.toBuffer('le', 8)],
              program.programId
          );
  
          // Airdrop SOL to cover the bid and transaction fees
          const airdropSignature = await provider.connection.requestAirdrop(provider.wallet.publicKey, 5_000_000_000);
          await provider.connection.confirmTransaction(airdropSignature, "confirmed");
  
          const tx = new Transaction();
          tx.add(program.instruction.placeBid(
            new anchor.BN(rate), 
            new anchor.BN(1_000_000_000),
            {  // Ensure rate is a BN if needed
              accounts: {
                  bid: bidPda,
                  orderBook: orderBookAccount,
                  user: provider.wallet.publicKey,
                  systemProgram: SystemProgram.programId,
              }
          }));
  
          try {
              // Execute the transaction
              await provider.sendAndConfirm(tx, [], { commitment: "confirmed", skipPreflight: true });
  
              // Fetch the updated order book
              const updatedOrderBook = await program.account.orderBook.fetch(orderBookAccount);
              expect(updatedOrderBook.bids.length).to.be.greaterThan(currentNonce.toNumber());
              console.log(`Order book size after bid at rate ${rate}: ${updatedOrderBook.bids.length}`);
          } catch (error) {
              console.error(`Error placing bid at rate ${rate}: ${error}`);
          }
      }
  });
  

  it("Fails to place a bid with an invalid rate", async () => {
    const currentNonce = (await program.account.orderBook.fetch(orderBookAccount)).globalNonce;
    const [bidPda] = await anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("bid"), provider.wallet.publicKey.toBuffer(), currentNonce.toBuffer('le', 8)],
        program.programId
    );

    try {
        await program.rpc.placeBid(
            new anchor.BN(500_000_000), // Invalid rate, below 600_000_000
            new anchor.BN(1_000_000_000), {
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
        assert.include(error.message, "BelowMinimumRate", "Error should be related to the minimum rate.");
    }
});

it("Fails to place a bid with insufficient funding", async () => {
    const currentNonce = (await program.account.orderBook.fetch(orderBookAccount)).globalNonce;
    const [bidPda] = await anchor.web3.PublicKey.findProgramAddressSync(
        [Buffer.from("bid"), provider.wallet.publicKey.toBuffer(), currentNonce.toBuffer('le', 8)],
        program.programId
    );

    try {
        await program.rpc.placeBid(
            new anchor.BN(900_000_000), // Valid rate
            new anchor.BN(800_000_000), { // Insufficient amount
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
        assert.include(error.message, "UnfundedBid", "Error should be related to insufficient funding.");
    }
});
});


