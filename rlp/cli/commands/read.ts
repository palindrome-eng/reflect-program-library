import { Command } from "commander";
import { Insurance } from "../../sdk/src/classes/Insurance";
import { createRpc } from "../utils/connection";
import { resolveRpcUrl } from "../utils/keypair";
import { printSuccess, printError, printTable } from "../utils/format";

export function registerReadCommands(program: Command) {
  program
    .command("get-settings")
    .description("Fetch and display protocol settings")
    .action(async (_opts, cmd) => {
      try {
        const globals = cmd.optsWithGlobals();
        const rpc = createRpc(resolveRpcUrl(globals));
        const insurance = new Insurance(rpc);
        const settings = await insurance.getSettingsData();
        printSuccess("Settings", settings);
      } catch (e) {
        printError(e);
      }
    });

  program
    .command("get-pools")
    .description("Fetch all liquidity pools")
    .action(async (_opts, cmd) => {
      try {
        const globals = cmd.optsWithGlobals();
        const rpc = createRpc(resolveRpcUrl(globals));
        const insurance = new Insurance(rpc);
        const pools = await insurance.getLiquidityPools();
        printTable(
          pools.map((p) => ({
            address: p.address,
            index: p.data.index,
            lpToken: p.data.lpToken,
            cooldowns: p.data.cooldowns.toString(),
            cooldownDuration: p.data.cooldownDuration.toString(),
            depositCap: p.data.depositCap?.toString() ?? "unlimited",
          })),
        );
      } catch (e) {
        printError(e);
      }
    });

  program
    .command("get-assets")
    .description("Fetch all registered assets")
    .action(async (_opts, cmd) => {
      try {
        const globals = cmd.optsWithGlobals();
        const rpc = createRpc(resolveRpcUrl(globals));
        const insurance = new Insurance(rpc);
        console.log("ok1");
        const assets = await insurance.getAssets();
        console.log("ok");
        printTable(
          assets.map((a) => ({
            address: a.address,
            index: a.data.index,
            mint: a.data.mint,
            oracle: (a.data.oracle as any).fields[0],
            accessLevel: a.data.accessLevel,
          })),
        );
      } catch (e) {
        printError(e);
      }
    });

  program
    .command("fetch-oracle-price")
    .description("Fetch oracle price for an asset")
    .requiredOption("--mint <address>", "Asset mint address")
    .action(async (opts, cmd) => {
      try {
        const globals = cmd.optsWithGlobals();
        const rpc = createRpc(resolveRpcUrl(globals));
        const insurance = new Insurance(rpc);
        await insurance.load();
        const price = await insurance.fetchOraclePrice(opts.mint);
        printSuccess("Oracle Price", {
          price: price.price.toString(),
          exponent: price.exponent,
          publishTime: price.publishTime.toString(),
        });
      } catch (e) {
        printError(e);
      }
    });
}
