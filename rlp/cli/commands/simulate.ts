import { Command } from "commander";
import { address, fetchEncodedAccount } from "@solana/kit";
import { findAssociatedTokenPda, TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";
import { Insurance } from "../../sdk/src/classes/Insurance";
import { createRpc } from "../utils/connection";
import { resolveRpcUrl } from "../utils/keypair";
import { printSuccess, printError, formatTokenAmount } from "../utils/format";

/**
 * Decode an SPL Mint account from raw bytes.
 * Layout: 36 bytes (mint authority option) + 8 supply + 1 decimals + ...
 */
function decodeMint(data: Uint8Array): { supply: bigint; decimals: number } {
  const dv = new DataView(data.buffer, data.byteOffset, data.byteLength);
  const supply = dv.getBigUint64(36, true);
  const decimals = data[44];
  return { supply, decimals };
}

/**
 * Decode an SPL Token account from raw bytes.
 * Layout: 32 mint + 32 owner + 8 amount + ...
 */
function decodeTokenAccount(data: Uint8Array): { mint: string; amount: bigint } {
  const dv = new DataView(data.buffer, data.byteOffset, data.byteLength);
  const amount = dv.getBigUint64(64, true);
  // extract mint as base58 isn't trivial, so we just return the amount
  return { mint: "", amount };
}

export function registerSimulateCommands(program: Command) {
  program
    .command("simulate-deposit")
    .description("Simulate deposit math using live on-chain data")
    .requiredOption("--mint <address>", "Deposit asset mint address")
    .requiredOption("--pool-id <n>", "Liquidity pool index")
    .requiredOption("--amount <tokens>", "Deposit amount in human-readable units (e.g. 10)")
    .action(async (opts, cmd) => {
      try {
        const globals = cmd.optsWithGlobals();
        const rpc = createRpc(resolveRpcUrl(globals));
        const insurance = new Insurance(rpc);
        await insurance.load();

        const depositMint = address(opts.mint);
        const poolId = Number(opts.poolId);

        // Find the pool
        const pools = insurance.getCachedLiquidityPools();
        const pool = pools.find((p) => p.data.index === poolId);
        if (!pool) throw new Error(`Pool ${poolId} not found`);

        // Find the deposit asset
        const assets = insurance.getCachedAssets();
        const depositAsset = assets.find((a) => a.data.mint === depositMint);
        if (!depositAsset) throw new Error(`Asset not found for mint ${depositMint}`);

        // Fetch deposit mint info (decimals)
        const depositMintAccount = await fetchEncodedAccount(rpc, depositMint);
        if (!depositMintAccount.exists) throw new Error(`Mint account not found: ${depositMint}`);
        const depositMintInfo = decodeMint(new Uint8Array(depositMintAccount.data));

        // Convert human-readable amount to raw base units
        const depositAmount = BigInt(
          Math.round(Number(opts.amount) * 10 ** depositMintInfo.decimals),
        );

        // Fetch LP token mint info (supply, decimals)
        const lpMintAccount = await fetchEncodedAccount(rpc, pool.data.lpToken);
        if (!lpMintAccount.exists) throw new Error(`LP mint not found: ${pool.data.lpToken}`);
        const lpMintInfo = decodeMint(new Uint8Array(lpMintAccount.data));

        // Fetch deposit asset oracle price
        const oraclePrice = await insurance.fetchOraclePrice(depositMint);

        // Get pool asset indices
        const poolAssetIndices = Array.from(pool.data.assets).slice(0, pool.data.assetCount);

        // Build reserves: for each pool asset, get the ATA balance, oracle price, and mint decimals
        const reserves: {
          mint: string;
          balance: bigint;
          price: bigint;
          exponent: number;
          decimals: number;
        }[] = [];

        for (const assetIndex of poolAssetIndices) {
          const asset = assets.find((a) => a.data.index === assetIndex);
          if (!asset) throw new Error(`Asset index ${assetIndex} not found`);

          // Fetch mint decimals
          const mintAcc = await fetchEncodedAccount(rpc, asset.data.mint);
          if (!mintAcc.exists) throw new Error(`Mint not found: ${asset.data.mint}`);
          const mintInfo = decodeMint(new Uint8Array(mintAcc.data));

          // Fetch pool ATA balance
          const [poolAta] = await findAssociatedTokenPda({
            mint: asset.data.mint,
            owner: pool.address,
            tokenProgram: TOKEN_PROGRAM_ADDRESS,
          });
          const ataAcc = await fetchEncodedAccount(rpc, poolAta);
          let balance = 0n;
          if (ataAcc.exists) {
            balance = decodeTokenAccount(new Uint8Array(ataAcc.data)).amount;
          }

          // Fetch oracle price for this asset
          const assetOraclePrice = await insurance.fetchOraclePrice(asset.data.mint);

          reserves.push({
            mint: asset.data.mint,
            balance,
            price: assetOraclePrice.price,
            exponent: assetOraclePrice.exponent,
            decimals: mintInfo.decimals,
          });
        }

        // Run simulation
        const lpTokensReceived = Insurance.simulateDepositMath({
          depositAmount,
          depositAssetPrice: {
            price: oraclePrice.price,
            exponent: oraclePrice.exponent,
          },
          depositAssetDecimals: depositMintInfo.decimals,
          poolReserves: reserves,
          lpTokenSupply: lpMintInfo.supply,
          lpTokenDecimals: lpMintInfo.decimals,
        });

        // Print detailed breakdown
        console.log("\n--- Deposit Simulation ---\n");
        console.log(`Pool:              #${poolId} (${pool.address})`);
        console.log(`Deposit asset:     ${depositMint}`);
        console.log(`  decimals:        ${depositMintInfo.decimals}`);
        console.log(`  oracle price:    ${oraclePrice.price} (exp ${oraclePrice.exponent})`);
        console.log(`  effective price:  $${Number(oraclePrice.price) * 10 ** oraclePrice.exponent}`);
        console.log(`Deposit amount:    ${opts.amount} tokens (${depositAmount} raw)`);
        console.log("");
        console.log(`LP token:          ${pool.data.lpToken}`);
        console.log(`  decimals:        ${lpMintInfo.decimals}`);
        console.log(`  current supply:  ${formatTokenAmount(lpMintInfo.supply, lpMintInfo.decimals)}`);
        console.log("");
        console.log("Pool reserves:");
        for (const r of reserves) {
          const mintAcc = await fetchEncodedAccount(rpc, address(r.mint));
          const mintDec = mintAcc.exists ? decodeMint(new Uint8Array(mintAcc.data)).decimals : r.decimals;
          console.log(`  ${r.mint}`);
          console.log(`    balance:  ${formatTokenAmount(r.balance, mintDec)} (${r.balance} raw)`);
          console.log(`    price:    ${r.price} (exp ${r.exponent})`);
          console.log(`    decimals: ${r.decimals}`);
        }
        console.log("");
        printSuccess("Result", {
          lpTokensReceived: lpTokensReceived.toString(),
          lpTokensFormatted: formatTokenAmount(lpTokensReceived, lpMintInfo.decimals),
        });
      } catch (e) {
        printError(e);
      }
    });

  program
    .command("simulate-withdraw")
    .description("Simulate withdrawal math using live on-chain data")
    .requiredOption("--pool-id <n>", "Liquidity pool index")
    .requiredOption("--amount <tokens>", "LP tokens to redeem in human-readable units")
    .action(async (opts, cmd) => {
      try {
        const globals = cmd.optsWithGlobals();
        const rpc = createRpc(resolveRpcUrl(globals));
        const insurance = new Insurance(rpc);
        await insurance.load();

        const poolId = Number(opts.poolId);

        const pools = insurance.getCachedLiquidityPools();
        const pool = pools.find((p) => p.data.index === poolId);
        if (!pool) throw new Error(`Pool ${poolId} not found`);

        const assets = insurance.getCachedAssets();

        // Fetch LP mint info
        const lpMintAccount = await fetchEncodedAccount(rpc, pool.data.lpToken);
        if (!lpMintAccount.exists) throw new Error(`LP mint not found`);
        const lpMintInfo = decodeMint(new Uint8Array(lpMintAccount.data));

        const lpAmount = BigInt(
          Math.round(Number(opts.amount) * 10 ** lpMintInfo.decimals),
        );

        // Build reserves
        const poolAssetIndices = Array.from(pool.data.assets).slice(0, pool.data.assetCount);
        const reserves: { mint: string; balance: bigint }[] = [];

        for (const assetIndex of poolAssetIndices) {
          const asset = assets.find((a) => a.data.index === assetIndex);
          if (!asset) throw new Error(`Asset index ${assetIndex} not found`);

          const [poolAta] = await findAssociatedTokenPda({
            mint: asset.data.mint,
            owner: pool.address,
            tokenProgram: TOKEN_PROGRAM_ADDRESS,
          });

          const ataAcc = await fetchEncodedAccount(rpc, poolAta);
          let balance = 0n;
          if (ataAcc.exists) {
            balance = decodeTokenAccount(new Uint8Array(ataAcc.data)).amount;
          }

          reserves.push({ mint: asset.data.mint, balance });
        }

        const result = Insurance.simulateWithdrawMath({
          lpTokenAmount: lpAmount,
          lpTokenSupply: lpMintInfo.supply,
          reserves: reserves.map((r) => ({ mint: address(r.mint), balance: r.balance })),
        });

        console.log("\n--- Withdrawal Simulation ---\n");
        console.log(`Pool:           #${poolId}`);
        console.log(`LP amount:      ${opts.amount} (${lpAmount} raw)`);
        console.log(`LP supply:      ${formatTokenAmount(lpMintInfo.supply, lpMintInfo.decimals)}`);
        console.log(`LP decimals:    ${lpMintInfo.decimals}`);
        console.log("");
        console.log("Tokens received:");
        for (const r of result) {
          const mintAcc = await fetchEncodedAccount(rpc, r.mint);
          const decimals = mintAcc.exists ? decodeMint(new Uint8Array(mintAcc.data)).decimals : 0;
          console.log(`  ${r.mint}: ${formatTokenAmount(r.amount, decimals)} (${r.amount} raw)`);
        }
      } catch (e) {
        printError(e);
      }
    });
}
