#!/usr/bin/env tsx
import { Command } from "commander";
import { registerAdminCommands } from "./commands/admin";
import { registerUserCommands } from "./commands/user";
import { registerCrankCommands } from "./commands/crank";
import { registerReadCommands } from "./commands/read";
import { registerSimulateCommands } from "./commands/simulate";

const program = new Command();

program
  .name("rlp")
  .description("RLP Insurance Protocol CLI")
  .version("1.0.0")
  .option("--keypair <path>", "Path to keypair file", process.env.KEYPAIR)
  .option("--rpc <url>", "Solana RPC URL", process.env.RPC_URL);

registerAdminCommands(program);
registerUserCommands(program);
registerCrankCommands(program);
registerReadCommands(program);
registerSimulateCommands(program);

program.parseAsync(process.argv).catch((err) => {
  console.error(err);
  process.exit(1);
});
