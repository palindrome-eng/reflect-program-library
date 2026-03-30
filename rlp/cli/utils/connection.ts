import { createSolanaRpc, type Rpc, type SolanaRpcApi } from "@solana/kit";

export function createRpc(url: string): Rpc<SolanaRpcApi> {
  return createSolanaRpc(url);
}
