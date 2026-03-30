import { readFileSync } from "fs";
import { resolve } from "path";
import { homedir } from "os";
import { Keypair } from "@solana/web3.js";
import { address, type Address, type TransactionSigner } from "@solana/kit";

export function loadKeypairFile(path: string): Keypair {
  const resolved = path.startsWith("~")
    ? resolve(homedir(), path.slice(2))
    : resolve(path);
  const raw = JSON.parse(readFileSync(resolved, "utf-8"));
  return Keypair.fromSecretKey(Uint8Array.from(raw));
}

export function keypairToSigner(kp: Keypair): TransactionSigner {
  return {
    address: address(kp.publicKey.toBase58()),
    modifyAndSignTransactions: async (txs: any) => txs,
  } as unknown as TransactionSigner;
}

export function keypairAddress(kp: Keypair): Address {
  return address(kp.publicKey.toBase58());
}

export function resolveKeypairPath(opts: any): string {
  return (
    opts.keypair ||
    process.env.KEYPAIR ||
    `${homedir()}/.config/solana/id.json`
  );
}

export function resolveRpcUrl(opts: any): string {
  return (
    opts.rpc ||
    process.env.RPC_URL ||
    "https://api.mainnet-beta.solana.com"
  );
}
