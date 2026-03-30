import { Command } from "commander";
import { address } from "@solana/kit";
import { Insurance } from "../../sdk/src/classes/Insurance";
import { printSuccess, printError, formatTokenAmount } from "../utils/format";

export function registerSimulateCommands(program: Command) {
  program
    .command("simulate-deposit")
    .description("Simulate deposit math (offline)")
    .requiredOption("--amount <raw>", "Deposit amount in base units")
    .requiredOption("--price <i64>", "Oracle price (integer)")
    .requiredOption("--exponent <i32>", "Oracle exponent (e.g. -8)")
    .requiredOption("--decimals <n>", "Token decimals")
    .requiredOption("--lp-supply <raw>", "Current LP token supply")
    .requiredOption("--lp-decimals <n>", "LP token decimals")
    .requiredOption(
      "--reserves <json>",
      'Pool reserves JSON: [{"balance":"...","price":"...","exponent":...,"decimals":...}, ...]',
    )
    .action(async (opts) => {
      try {
        const reserves = JSON.parse(opts.reserves).map((r: any) => ({
          balance: BigInt(r.balance),
          price: BigInt(r.price),
          exponent: Number(r.exponent),
          decimals: Number(r.decimals),
        }));

        const result = Insurance.simulateDepositMath({
          depositAmount: BigInt(opts.amount),
          depositAssetPrice: {
            price: BigInt(opts.price),
            exponent: Number(opts.exponent),
          },
          depositAssetDecimals: Number(opts.decimals),
          poolReserves: reserves,
          lpTokenSupply: BigInt(opts.lpSupply),
          lpTokenDecimals: Number(opts.lpDecimals),
        });

        printSuccess("Simulated Deposit", {
          lpTokensReceived: result.toString(),
          lpTokensFormatted: formatTokenAmount(
            result,
            Number(opts.lpDecimals),
          ),
        });
      } catch (e) {
        printError(e);
      }
    });

  program
    .command("simulate-withdraw")
    .description("Simulate withdrawal math (offline)")
    .requiredOption("--lp-amount <raw>", "LP tokens to redeem")
    .requiredOption("--lp-supply <raw>", "Current LP token supply")
    .requiredOption(
      "--reserves <json>",
      'Pool reserves JSON: [{"mint":"...","balance":"..."}, ...]',
    )
    .action(async (opts) => {
      try {
        const reserves = JSON.parse(opts.reserves).map((r: any) => ({
          mint: address(r.mint),
          balance: BigInt(r.balance),
        }));

        const result = Insurance.simulateWithdrawMath({
          lpTokenAmount: BigInt(opts.lpAmount),
          lpTokenSupply: BigInt(opts.lpSupply),
          reserves,
        });

        printSuccess("Simulated Withdrawal");
        for (const r of result) {
          console.log(`  ${r.mint}: ${r.amount.toString()}`);
        }
      } catch (e) {
        printError(e);
      }
    });
}
