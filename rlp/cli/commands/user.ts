import { Command } from "commander";
import { address } from "@solana/kit";
import { JuniorTranche } from "../../sdk/src/classes/JuniorTranche";
import { createRpc } from "../utils/connection";
import {
  loadKeypairFile,
  keypairToSigner,
  resolveKeypairPath,
  resolveRpcUrl,
} from "../utils/keypair";
import { sendAndConfirm } from "../utils/send";
import { printSuccess, printError } from "../utils/format";

export function registerUserCommands(program: Command) {
  program
    .command("deposit")
    .description("Deposit tokens into a liquidity pool")
    .requiredOption("--pool-id <n>", "Liquidity pool index")
    .requiredOption("--mint <address>", "Asset mint to deposit")
    .requiredOption("--amount <raw>", "Amount in base units")
    .option("--min-lp-tokens <raw>", "Minimum LP tokens to receive (slippage protection)")
    .action(async (opts, cmd) => {
      try {
        const globals = cmd.optsWithGlobals();
        const kp = loadKeypairFile(resolveKeypairPath(globals));
        const signer = keypairToSigner(kp);
        const rpc = createRpc(resolveRpcUrl(globals));
        const juniorTranche = new JuniorTranche(rpc);
        await juniorTranche.load();
        const ix = await juniorTranche.deposit(
          signer,
          BigInt(opts.amount),
          address(opts.mint),
          Number(opts.poolId),
          opts.minLpTokens ? BigInt(opts.minLpTokens) : null,
        );
        const sig = await sendAndConfirm(resolveRpcUrl(globals), kp, ix);
        printSuccess(`Deposited. Signature: ${sig}`);
      } catch (e) {
        printError(e);
      }
    });

  program
    .command("request-withdrawal")
    .description("Request a withdrawal (starts cooldown)")
    .requiredOption("--pool-id <n>", "Liquidity pool index")
    .requiredOption("--amount <raw>", "LP token amount to withdraw")
    .action(async (opts, cmd) => {
      try {
        const globals = cmd.optsWithGlobals();
        const kp = loadKeypairFile(resolveKeypairPath(globals));
        const signer = keypairToSigner(kp);
        const rpc = createRpc(resolveRpcUrl(globals));
        const juniorTranche = new JuniorTranche(rpc);
        await juniorTranche.load();
        const ix = await juniorTranche.requestWithdrawal(
          signer,
          Number(opts.poolId),
          BigInt(opts.amount),
        );
        const sig = await sendAndConfirm(resolveRpcUrl(globals), kp, ix);
        printSuccess(`Withdrawal requested. Signature: ${sig}`);
      } catch (e) {
        printError(e);
      }
    });

  program
    .command("withdraw")
    .description("Complete a withdrawal after cooldown")
    .requiredOption("--pool-id <n>", "Liquidity pool index")
    .requiredOption("--cooldown-id <n>", "Cooldown ID")
    .action(async (opts, cmd) => {
      try {
        const globals = cmd.optsWithGlobals();
        const kp = loadKeypairFile(resolveKeypairPath(globals));
        const signer = keypairToSigner(kp);
        const rpc = createRpc(resolveRpcUrl(globals));
        const juniorTranche = new JuniorTranche(rpc);
        await juniorTranche.load();
        const ix = await juniorTranche.withdraw(
          signer,
          Number(opts.poolId),
          BigInt(opts.cooldownId),
        );
        const sig = await sendAndConfirm(resolveRpcUrl(globals), kp, ix);
        printSuccess(`Withdrawn. Signature: ${sig}`);
      } catch (e) {
        printError(e);
      }
    });

  program
    .command("swap")
    .description("Swap tokens within a liquidity pool")
    .requiredOption("--pool-id <n>", "Liquidity pool index")
    .requiredOption("--token-from <address>", "Mint of token to sell")
    .requiredOption("--token-to <address>", "Mint of token to buy")
    .requiredOption("--amount-in <raw>", "Amount to swap in base units")
    .option("--min-out <raw>", "Minimum output (slippage protection)")
    .action(async (opts, cmd) => {
      try {
        const globals = cmd.optsWithGlobals();
        const kp = loadKeypairFile(resolveKeypairPath(globals));
        const signer = keypairToSigner(kp);
        const rpc = createRpc(resolveRpcUrl(globals));
        const juniorTranche = new JuniorTranche(rpc);
        await juniorTranche.load();
        const ix = await juniorTranche.swap(
          signer,
          Number(opts.poolId),
          address(opts.tokenFrom),
          address(opts.tokenTo),
          BigInt(opts.amountIn),
          opts.minOut ? BigInt(opts.minOut) : null,
        );
        const sig = await sendAndConfirm(resolveRpcUrl(globals), kp, ix);
        printSuccess(`Swapped. Signature: ${sig}`);
      } catch (e) {
        printError(e);
      }
    });
}
