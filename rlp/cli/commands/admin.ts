import { Command } from "commander";
import { address } from "@solana/kit";
import { Insurance } from "../../sdk/src/classes/Insurance";
import { AccessLevel, Action, Role, Update } from "../../sdk/src/generated";
import { createRpc } from "../utils/connection";
import {
  loadKeypairFile,
  keypairToSigner,
  resolveKeypairPath,
  resolveRpcUrl,
} from "../utils/keypair";
import { sendAndConfirm } from "../utils/send";
import { printSuccess, printError } from "../utils/format";

function parseAction(s: string): Action {
  const val = (Action as any)[s];
  if (val === undefined) throw new Error(`Unknown action: ${s}. Valid: ${Object.keys(Action).filter(k => isNaN(Number(k))).join(", ")}`);
  return val;
}

function parseRole(s: string): Role {
  const val = (Role as any)[s];
  if (val === undefined) throw new Error(`Unknown role: ${s}. Valid: ${Object.keys(Role).filter(k => isNaN(Number(k))).join(", ")}`);
  return val;
}

function parseUpdate(s: string): Update {
  const val = (Update as any)[s];
  if (val === undefined) throw new Error(`Unknown update: ${s}. Valid: Add, Remove`);
  return val;
}

function parseAccessLevel(s: string): AccessLevel {
  const val = (AccessLevel as any)[s];
  if (val === undefined) throw new Error(`Unknown access level: ${s}. Valid: Public, Private`);
  return val;
}

export function registerAdminCommands(program: Command) {
  program
    .command("initialize-rlp")
    .description("Initialize the RLP protocol (one-time)")
    .requiredOption("--swap-fee-bps <n>", "Swap fee in basis points")
    .action(async (opts, cmd) => {
      try {
        const globals = cmd.optsWithGlobals();
        const kp = loadKeypairFile(resolveKeypairPath(globals));
        const signer = keypairToSigner(kp);
        const rpc = createRpc(resolveRpcUrl(globals));
        const insurance = new Insurance(rpc);
        const ix = await insurance.initializeRlp(signer, Number(opts.swapFeeBps));
        const sig = await sendAndConfirm(resolveRpcUrl(globals), kp, ix);
        printSuccess(`Protocol initialized. Signature: ${sig}`);
      } catch (e) { printError(e); }
    });

  program
    .command("initialize-lp")
    .description("Initialize a liquidity pool")
    .requiredOption("--lp-token-mint <address>", "LP token mint address")
    .requiredOption("--cooldown-duration <seconds>", "Cooldown duration in seconds")
    .requiredOption("--assets <indices...>", "Asset indices to include (e.g. 0 1 2)")
    .option("--deposit-cap <amount>", "Deposit cap in raw LP token units (omit for unlimited)")
    .action(async (opts, cmd) => {
      try {
        const globals = cmd.optsWithGlobals();
        const kp = loadKeypairFile(resolveKeypairPath(globals));
        const signer = keypairToSigner(kp);
        const rpc = createRpc(resolveRpcUrl(globals));
        const insurance = new Insurance(rpc);
        const ix = await insurance.initializeLiquidityPool(signer, {
          lpTokenMint: address(opts.lpTokenMint),
          cooldownDuration: BigInt(opts.cooldownDuration),
          depositCap: opts.depositCap ? BigInt(opts.depositCap) : null,
          assets: new Uint8Array(opts.assets.map(Number)),
        });
        const sig = await sendAndConfirm(resolveRpcUrl(globals), kp, ix);
        printSuccess(`Liquidity pool initialized. Signature: ${sig}`);
      } catch (e) { printError(e); }
    });

  program
    .command("add-asset")
    .description("Register a new asset")
    .requiredOption("--mint <address>", "Asset token mint")
    .requiredOption("--oracle <address>", "Pyth oracle address")
    .requiredOption("--access-level <Public|Private>", "Access level")
    .action(async (opts, cmd) => {
      try {
        const globals = cmd.optsWithGlobals();
        const kp = loadKeypairFile(resolveKeypairPath(globals));
        const signer = keypairToSigner(kp);
        const rpc = createRpc(resolveRpcUrl(globals));
        const insurance = new Insurance(rpc);
        const ix = await insurance.addAsset(
          signer,
          address(opts.mint),
          address(opts.oracle),
          parseAccessLevel(opts.accessLevel),
        );
        const sig = await sendAndConfirm(resolveRpcUrl(globals), kp, ix);
        printSuccess(`Asset added. Signature: ${sig}`);
      } catch (e) { printError(e); }
    });

  program
    .command("create-permission-account")
    .description("Create a permission account for an address")
    .requiredOption("--new-admin <address>", "Address to create permissions for")
    .action(async (opts, cmd) => {
      try {
        const globals = cmd.optsWithGlobals();
        const kp = loadKeypairFile(resolveKeypairPath(globals));
        const signer = keypairToSigner(kp);
        const rpc = createRpc(resolveRpcUrl(globals));
        const insurance = new Insurance(rpc);
        const ix = await insurance.createPermissionAccount(signer, address(opts.newAdmin));
        const sig = await sendAndConfirm(resolveRpcUrl(globals), kp, ix);
        printSuccess(`Permission account created. Signature: ${sig}`);
      } catch (e) { printError(e); }
    });

  program
    .command("update-role-holder")
    .description("Add or remove a role from a user")
    .requiredOption("--address <address>", "Target user address")
    .requiredOption("--role <Role>", "Role name (PUBLIC, TESTEE, FREEZE, CRANK, MANAGER, SUPREMO)")
    .requiredOption("--update <Add|Remove>", "Add or Remove")
    .action(async (opts, cmd) => {
      try {
        const globals = cmd.optsWithGlobals();
        const kp = loadKeypairFile(resolveKeypairPath(globals));
        const signer = keypairToSigner(kp);
        const rpc = createRpc(resolveRpcUrl(globals));
        const insurance = new Insurance(rpc);
        const ix = await insurance.updateRoleHolder(
          signer,
          address(opts.address),
          parseRole(opts.role),
          parseUpdate(opts.update),
        );
        const sig = await sendAndConfirm(resolveRpcUrl(globals), kp, ix);
        printSuccess(`Role updated. Signature: ${sig}`);
      } catch (e) { printError(e); }
    });

  program
    .command("update-action-role")
    .description("Add or remove a role from an action")
    .requiredOption("--action <Action>", "Action name (Restake, Withdraw, Slash, Swap, ...)")
    .requiredOption("--role <Role>", "Role name")
    .requiredOption("--update <Add|Remove>", "Add or Remove")
    .action(async (opts, cmd) => {
      try {
        const globals = cmd.optsWithGlobals();
        const kp = loadKeypairFile(resolveKeypairPath(globals));
        const signer = keypairToSigner(kp);
        const rpc = createRpc(resolveRpcUrl(globals));
        const insurance = new Insurance(rpc);
        const ix = await insurance.updateActionRole(
          signer,
          parseAction(opts.action),
          parseRole(opts.role),
          parseUpdate(opts.update),
        );
        const sig = await sendAndConfirm(resolveRpcUrl(globals), kp, ix);
        printSuccess(`Action role updated. Signature: ${sig}`);
      } catch (e) { printError(e); }
    });

  program
    .command("update-deposit-cap")
    .description("Update or remove deposit cap for a pool")
    .requiredOption("--pool-id <n>", "Liquidity pool index")
    .option("--new-cap <amount>", "New cap in raw units (omit to remove cap)")
    .action(async (opts, cmd) => {
      try {
        const globals = cmd.optsWithGlobals();
        const kp = loadKeypairFile(resolveKeypairPath(globals));
        const signer = keypairToSigner(kp);
        const rpc = createRpc(resolveRpcUrl(globals));
        const insurance = new Insurance(rpc);
        await insurance.load();
        const ix = await insurance.updateDepositCap(
          signer,
          Number(opts.poolId),
          opts.newCap ? BigInt(opts.newCap) : null,
        );
        const sig = await sendAndConfirm(resolveRpcUrl(globals), kp, ix);
        printSuccess(`Deposit cap updated. Signature: ${sig}`);
      } catch (e) { printError(e); }
    });

  program
    .command("freeze")
    .description("Freeze an action")
    .requiredOption("--action <Action>", "Action to freeze (FreezeRestake, FreezeWithdraw, FreezeSlash, FreezeSwap)")
    .action(async (opts, cmd) => {
      try {
        const globals = cmd.optsWithGlobals();
        const kp = loadKeypairFile(resolveKeypairPath(globals));
        const signer = keypairToSigner(kp);
        const rpc = createRpc(resolveRpcUrl(globals));
        const insurance = new Insurance(rpc);
        const ix = await insurance.freezeFunctionality(signer, parseAction(opts.action), true);
        const sig = await sendAndConfirm(resolveRpcUrl(globals), kp, ix);
        printSuccess(`Action frozen. Signature: ${sig}`);
      } catch (e) { printError(e); }
    });

  program
    .command("unfreeze")
    .description("Unfreeze an action")
    .requiredOption("--action <Action>", "Action to unfreeze")
    .action(async (opts, cmd) => {
      try {
        const globals = cmd.optsWithGlobals();
        const kp = loadKeypairFile(resolveKeypairPath(globals));
        const signer = keypairToSigner(kp);
        const rpc = createRpc(resolveRpcUrl(globals));
        const insurance = new Insurance(rpc);
        const ix = await insurance.freezeFunctionality(signer, parseAction(opts.action), false);
        const sig = await sendAndConfirm(resolveRpcUrl(globals), kp, ix);
        printSuccess(`Action unfrozen. Signature: ${sig}`);
      } catch (e) { printError(e); }
    });
}
