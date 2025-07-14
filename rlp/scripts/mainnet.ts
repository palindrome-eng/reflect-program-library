import {
    Connection,
    Keypair,
    LAMPORTS_PER_SOL,
    PublicKey,
    SystemProgram,
    Transaction,
    TransactionMessage
} from "@solana/web3.js";
import {Restaking} from "../sdk";
import bs58 from "bs58";
import BN from "bn.js";
import {AnchorProvider} from "@coral-xyz/anchor";
import {
    createAssociatedTokenAccountIdempotentInstruction, createInitializeMintInstruction,
    createMintToInstruction,
    getAssociatedTokenAddressSync, MINT_SIZE, TOKEN_PROGRAM_ID
} from "@solana/spl-token";
import {createGetUserBalanceAndRewardInstruction, Deposit, depositDiscriminator, Settings} from "../sdk/src/generated";
import sleep from "../tests/helpers/sleep";
import signAndSendTransaction from "./helpers/signAndSendTransaction";

const RPC_URL = "https://devnet.helius-rpc.com/?api-key=c9d75796-3865-44a8-baa7-4ed4dc0da671";
console.log(RPC_URL);
const connection = new Connection(RPC_URL);
const restaking = new Restaking(connection);
const ADMIN_SECRET_KEY = process.env.ADMIN_SECRET_KEY;
const keypair = Keypair.fromSecretKey(
    bs58.decode(ADMIN_SECRET_KEY)
);
const coldWalletKeypair = Keypair.generate();

console.log(keypair.publicKey.toString());

export default async function mintTokens(
    mint: PublicKey,
    payer: Keypair,
    amount: number,
    recipient?: PublicKey
) {

    const ata = getAssociatedTokenAddressSync(
        mint,
        recipient || payer.publicKey,
        true
    );

    const ataIx = createAssociatedTokenAccountIdempotentInstruction(
        payer.publicKey,
        ata,
        recipient || payer.publicKey,
        mint,
    );

    const ix = createMintToInstruction(
        mint,
        ata,
        payer.publicKey,
        amount,
    );

    const tx = new Transaction();
    tx.add(ataIx, ix);

    const {
        lastValidBlockHeight,
        blockhash
    } = await connection.getLatestBlockhash();

    tx.feePayer = payer.publicKey;
    tx.recentBlockhash = blockhash;
    tx.sign(payer);

    const sent = await connection.sendRawTransaction(tx.serialize());
    await connection.confirmTransaction({
        blockhash,
        lastValidBlockHeight,
        signature: sent
    }, "confirmed");
}

async function createToken(
    connection: Connection,
    payer: Keypair
) {
    const keypair = Keypair.generate();

    const lamports = await connection.getMinimumBalanceForRentExemption(MINT_SIZE);
    const createAccountIx = SystemProgram.createAccount({
        newAccountPubkey: keypair.publicKey,
        fromPubkey: payer.publicKey,
        lamports,
        programId: TOKEN_PROGRAM_ID,
        space: MINT_SIZE
    });

    const ix = createInitializeMintInstruction(
        keypair.publicKey,
        9,
        payer.publicKey,
        payer.publicKey
    );

    const tx = new Transaction();
    tx.add(createAccountIx, ix);

    const {
        lastValidBlockHeight,
        blockhash
    } = await connection.getLatestBlockhash();

    tx.feePayer = payer.publicKey;
    tx.recentBlockhash = blockhash;
    tx.sign(payer, keypair);

    const sent = await connection.sendRawTransaction(tx.serialize());
    await connection.confirmTransaction({
        blockhash,
        lastValidBlockHeight,
        signature: sent
    }, "confirmed");

    return keypair.publicKey;
}

async function createRestakingAndRewardToken() {
    const reward = await createToken(connection, keypair);

    const ix = await restaking.initializeInsuranceFund(
        keypair.publicKey,
        {
            coldWallet: coldWalletKeypair.publicKey,
            rewardMint: reward,
            coldWalletShareBps: 7_000,
            hotWalletShareBps: 3_000,
            cooldownDuration: new BN(30) // 30 seconds
        }
    );

    const { lastValidBlockHeight, blockhash } = await connection.getLatestBlockhash();
    const message = new TransactionMessage({
        recentBlockhash: blockhash,
        instructions: [ix],
        payerKey: keypair.publicKey
    }).compileToV0Message();

    await signAndSendTransaction(message, connection, true, [keypair]);

    return {
        reward
    };
}

async function createLockup(
    assetMint: PublicKey,
    cap: BN,
    minDeposit: BN,
    duration: BN
) {
    const {
        instructions,
        signer
    } = await restaking.initializeLockup(
        keypair.publicKey,
        assetMint,
        cap,
        minDeposit,
        duration,
    );

    const { lastValidBlockHeight, blockhash } = await connection.getLatestBlockhash();
    const message = new TransactionMessage({
        recentBlockhash: blockhash,
        instructions: [...instructions],
        payerKey: keypair.publicKey
    }).compileToV0Message();

    await signAndSendTransaction(message, connection, true, [keypair, signer]);
}

async function createUserMintTokensAndDeposit(
    lockupId: number,
    amount: number,
    mint: PublicKey
) {
    const user = Keypair.generate();
    await connection.requestAirdrop(user.publicKey, 2 * LAMPORTS_PER_SOL);

    await mintTokens(
        mint,
        keypair,
        100_000 * LAMPORTS_PER_SOL,
        user.publicKey
    );

    const ix = await restaking.restake(
        user.publicKey,
        new BN(amount),
        new BN(lockupId)
    );

    return {
        user
    }
}

// async function createDestinationAndSlash(
//     lockupId: BN,
//     percentageSlash: BN
// ) {
//     const destination = Keypair.generate();
//     await connection.requestAirdrop(destination.publicKey, 2 * LAMPORTS_PER_SOL);
//
//    const ix = await restaking.slash(
//         amount,
//         keypair.publicKey,
//         lockupId,
//         destination.publicKey
//     );
//
//    return {
//        destination
//    };
// }

async function requestWithdrawWaitCooldownAndWithdraw(
    user: Keypair,
    lockupId: BN,
    depositId: BN,
    amount: BN
) {
    const ix1 = restaking.requestWithdrawal(
        user.publicKey,
        lockupId,
        depositId,
        "ExactIn",
        amount
    );

    const {
        cooldownDuration
    } = await Settings.fromAccountAddress(
        connection,
        Restaking.deriveSettings()
    );

    await sleep(parseInt(cooldownDuration.toString()));

    const ix2 = await restaking.withdrawCooldown(
        user.publicKey,
        lockupId,
        depositId
    );
}

async function getAllUsersPerLockup(
    lockupId: BN
) {
    const lockup = Restaking.deriveLockup(lockupId);
    const deposits = await Deposit
        .gpaBuilder()
        .addFilter("accountDiscriminator", depositDiscriminator)
        .addFilter("lockup", lockup)
        .run(connection);

    return deposits.map(({ account, pubkey }) => ({
        account: Deposit.deserialize(
            account.data
        )[0],
        pubkey
    }));
}

async function depositRewards(
    lockupId: BN,
    amount: BN
) {
    const ix = await restaking.depositRewards(
        lockupId,
        amount,
        keypair.publicKey
    );
}

// Flow:
// Perpetually create users. After every user, deposit new reward and slash 20% of the pool.
// After each check, reiterate all users and check if their values are correct.
// After 100 users, request withdrawal, wait cooldown and withdraw all of them.
//
// async function testFullFlow() {
//     const baseAsset = await createToken(
//         connection,
//         keypair
//     );
//
//     const {
//         reward
//     } = await createRestakingAndRewardToken();
//
//     await createLockup(
//         baseAsset,
//         new BN("1000000000000"),
//         new BN("0"),
//         new BN(30)
//     );
//
//     const users: Keypair[] = [];
//     const slashed: number[] = [];
//     const slashDestinations: Keypair[] = [];
//
//     for (let i = 0; i < 100; i++) {
//         const {
//             user
//         } = await createUserMintTokensAndDeposit(
//             0,
//             1000,
//             baseAsset
//         );
//
//         users.push(user);
//
//         const slash = !!Math.floor(Math.random() * 2);
//         if (slash) {
//             slashed.push(i);
//             await createDestinationAndSlash(
//                 new BN(0),
//                 new BN(20)
//             );
//         }
//     }
// }

async function setup() {
    const baseAsset = new PublicKey("J39SsqkKVUQmkhkipAFeZ6265ZBNKgPY6d9p1EXAamCt");
    //
    // await sleep(30);
    //
    // const {
    //     reward
    // } = await createRestakingAndRewardToken();

    // await sleep(30);

    // const instruction = await restaking.addAsset(
    //     keypair.publicKey,
    //     baseAsset,
    //     new PublicKey("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE")
    // );
    //
    // const {
    //     lastValidBlockHeight,
    //     blockhash
    // } = await connection.getLatestBlockhash();
    // await signAndSendTransaction(
    //     new TransactionMessage({
    //         recentBlockhash: blockhash,
    //         instructions: [instruction],
    //         payerKey: keypair.publicKey
    //     }).compileToV0Message(),
    //     connection,
    //     true,
    //     [keypair]
    // );
    //
    // await sleep(30);

    await createLockup(
        baseAsset,
        new BN("1000000000000"),
        new BN("0"),
        new BN(30)
    );
}

async function testFullFlow(asset: PublicKey) {
    for (let i = 0; i < 100; i++) {
        const {
            user
        } = await createUserMintTokensAndDeposit(
            0,
            1000 * LAMPORTS_PER_SOL,
            asset
        );
    }

    // const depositsPreSlash = await getAllUsersPerLockup(
    //     new BN(0)
    // );
    //
    // // await createDestinationAndSlash(new BN(0), new BN(20));
    //
    // const depositsPostSlash = await getAllUsersPerLockup(
    //     new BN(0)
    // );
    //
    // // for (let { account: { initialReceiptExchangeRateBps }, pubkey } of depositsPostSlash) {
    // //
    // // }
}

(async () => {
    await testFullFlow(
        new PublicKey("J39SsqkKVUQmkhkipAFeZ6265ZBNKgPY6d9p1EXAamCt")
    );
})();