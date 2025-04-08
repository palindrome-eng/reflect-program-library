import {Connection} from "@solana/web3.js";

export default async function debugLogs(
    connection: Connection,
    signature: string,
) {

    const { blockhash, lastValidBlockHeight } = await connection.getLatestBlockhash();
    await connection.confirmTransaction({
        blockhash,
        lastValidBlockHeight,
        signature
    }, "confirmed");

    const {
        meta: {
            logMessages,
            err
        }
    } = await connection.getParsedTransaction(
        signature,
        "confirmed"
    );

    return logMessages;
}