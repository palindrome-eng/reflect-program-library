import {PublicKey, Transaction} from "@solana/web3.js";
import {AnchorProvider} from "@coral-xyz/anchor";
import {
    createAssociatedTokenAccountIdempotentInstruction,
    createMintToInstruction,
    getAssociatedTokenAddressSync
} from "@solana/spl-token";

export default async function mintTokens(
    mint: PublicKey,
    payer: AnchorProvider,
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
    } = await payer.connection.getLatestBlockhash();

    tx.feePayer = payer.publicKey;
    tx.recentBlockhash = blockhash;

    const signed = await payer.wallet.signTransaction(tx);

    const sent = await payer.connection.sendRawTransaction(signed.serialize());
    await payer.connection.confirmTransaction({
        blockhash,
        lastValidBlockHeight,
        signature: sent
    }, "confirmed");
}