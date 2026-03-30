import { Command } from "commander";
import { address } from "@solana/kit";
import { Insurance } from "../../sdk/src/classes/Insurance";
import { createRpc } from "../utils/connection";
import {
  loadKeypairFile,
  keypairToSigner,
  resolveKeypairPath,
  resolveRpcUrl,
} from "../utils/keypair";
import { sendAndConfirm } from "../utils/send";
import { printSuccess, printError } from "../utils/format";

export function registerCrankCommands(program: Command) {
  program
    .command("slash")
    .description("Slash tokens from a liquidity pool")
    .requiredOption("--pool-id <n>", "Liquidity pool index")
    .requiredOption("--mint <address>", "Asset mint to slash")
    .requiredOption("--amount <raw>", "Amount to slash in base units")
    .requiredOption("--destination <address>", "Destination token account")
    .action(async (opts, cmd) => {
      try {
        const globals = cmd.optsWithGlobals();
        const kp = loadKeypairFile(resolveKeypairPath(globals));
        const signer = keypairToSigner(kp);
        const rpc = createRpc(resolveRpcUrl(globals));
        const insurance = new Insurance(rpc);
        await insurance.load();
        const ix = await insurance.slash(
          address(opts.mint),
          BigInt(opts.amount),
          signer,
          Number(opts.poolId),
          address(opts.destination),
        );
        const sig = await sendAndConfirm(resolveRpcUrl(globals), kp, ix);
        printSuccess(`Slashed. Signature: ${sig}`);
      } catch (e) {
        printError(e);
      }
    });
}
