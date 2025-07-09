import {Connection, Keypair, PublicKey, Transaction} from "@solana/web3.js";
import {
    createAssociatedTokenAccountIdempotentInstruction,
    createMintToInstruction,
    getAssociatedTokenAddressSync
} from "@solana/spl-token";

export default async function mintTokens(
    mint: PublicKey,
    payer: Keypair,
    connection: Connection,
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