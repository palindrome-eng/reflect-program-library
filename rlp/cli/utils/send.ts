import {
  Transaction as LegacyTransaction,
  PublicKey,
  Keypair,
  ComputeBudgetProgram,
  TransactionInstruction,
  Connection,
} from "@solana/web3.js";
import type { Address, Instruction } from "@solana/kit";

function toPublicKey(addr: Address): PublicKey {
  return new PublicKey(addr);
}

function convertToLegacyInstruction(ix: Instruction): TransactionInstruction {
  return new TransactionInstruction({
    programId: toPublicKey(ix.programAddress),
    keys: ((ix as any).accounts ?? []).map((acc: any) => ({
      pubkey: toPublicKey(acc.address),
      isSigner: acc.role === 3 || acc.role === 2,
      isWritable: acc.role === 1 || acc.role === 3,
    })),
    data: (ix as any).data
      ? Buffer.from((ix as any).data as Uint8Array)
      : Buffer.alloc(0),
  });
}

export async function sendAndConfirm(
  rpcUrl: string,
  payer: Keypair,
  instruction: Instruction | Instruction[],
  extraSigners: Keypair[] = [],
): Promise<string> {
  const conn = new Connection(rpcUrl, "confirmed");

  const ixArray = Array.isArray(instruction) ? instruction : [instruction];
  const legacyIxs = ixArray.map(convertToLegacyInstruction);

  const tx = new LegacyTransaction();
  tx.recentBlockhash = (await conn.getLatestBlockhash()).blockhash;
  tx.feePayer = payer.publicKey;
  tx.add(
    ComputeBudgetProgram.setComputeUnitLimit({ units: 400_000 }),
    ...legacyIxs,
  );

  const signers = [
    payer,
    ...extraSigners.filter(
      (s) => s.publicKey.toBase58() !== payer.publicKey.toBase58(),
    ),
  ];
  tx.sign(...signers);

  const sig = await conn.sendRawTransaction(tx.serialize(), {
    skipPreflight: false,
    preflightCommitment: "confirmed",
  });

  await conn.confirmTransaction(sig, "confirmed");
  return sig;
}
