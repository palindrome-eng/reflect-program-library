import {Connection, Keypair, MessageV0, TransactionMessage, VersionedTransaction} from "@solana/web3.js";
import {AnchorProvider} from "@coral-xyz/anchor";

export default async function signAndSendTransaction(
    message: MessageV0,
    connection: Connection,
    skipPreflight?: boolean,
    signers?: Keypair[]
) {
    const {
        lastValidBlockHeight,
        blockhash
    } = await connection.getLatestBlockhash();

    const transaction = new VersionedTransaction(message);

    if (signers && signers.length) transaction.sign(signers);

    const txid = await connection.sendRawTransaction(
        transaction.serialize(),
        { skipPreflight }
    );

    await connection.confirmTransaction({
        lastValidBlockHeight,
        blockhash,
        signature: txid
    }, "confirmed");
    //
    // const {
    //     meta: {
    //         err,
    //         logMessages
    //     }
    // } = await provider.connection.getParsedTransaction(txid, "confirmed");
    //
    // console.log({
    //     logMessages,
    //     err
    // });
    //
    // if (err) throw err;

    return txid;
}