import { describe, it, before } from "mocha";
import { expect } from "chai";
import {
  LiteSVM,
  FailedTransactionMetadata,
  TransactionMetadata,
} from "litesvm";
import {
  Address,
  address,
  lamports,
  TransactionSigner,
  Instruction,
} from "@solana/kit";
import { getCreateAccountInstruction } from "@solana-program/system";
import {
  getInitializeMintInstruction,
  getMintToInstruction,
  getCreateAssociatedTokenIdempotentInstruction,
  findAssociatedTokenPda,
  TOKEN_PROGRAM_ADDRESS,
  getMintSize,
} from "@solana-program/token";
import * as path from "path";

import {
  Transaction as LegacyTransaction,
  PublicKey,
  Keypair,
  ComputeBudgetProgram,
  TransactionInstruction,
  LAMPORTS_PER_SOL,
} from "@solana/web3.js";

// SDK imports
import {
  RLP_PROGRAM_ADDRESS as PROGRAM_ID,
  getInitializeRlpInstructionAsync,
  getAddAssetInstructionAsync,
  getInitializeLpInstructionAsync,
  getFreezeFunctionalityInstructionAsync,
  getUpdateActionRoleInstructionAsync,
  getCreatePermissionAccountInstructionAsync,
  getUpdateRoleHolderInstructionAsync,
  getUpdateDepositCapInstructionAsync,
  getRestakeInstructionAsync,
  getRequestWithdrawalInstructionAsync,
  getWithdrawInstructionAsync,
  getSlashInstructionAsync,
  getSwapInstructionAsync,
  AccessLevel,
  Action,
  Role,
  Update,
} from "../src/generated";
import { PdaClient } from "../src/classes/PdaClient";
import { Insurance, type AccountWithAddress } from "../src/classes/Insurance";
import type { LiquidityPool, Asset, Settings } from "../src/generated";

const RLP_SO_PATH = path.join(__dirname, "../../target/deploy/rlp.so");
const PYTH_PROGRAM_ID = "rec5EKMGg6MxZYaMdyBfgwp4d5rB9T1VQH5pJv5LtFJ";

// ============================================================================
// TYPES & HELPERS
// ============================================================================

interface TransactionResult {
  success: boolean;
  signature: string;
  logs: string[];
  computeUnits: number;
  error?: string;
}

function isFailedTransaction(
  result: TransactionMetadata | FailedTransactionMetadata,
): result is FailedTransactionMetadata {
  return "err" in result && typeof (result as any).err === "function";
}

function toPublicKey(addr: Address): PublicKey {
  return new PublicKey(addr);
}

function toAddress(pubkey: PublicKey): Address {
  return address(pubkey.toBase58());
}

function generateKeypair(): { address: Address; keypair: Keypair } {
  const keypair = Keypair.generate();
  return { address: toAddress(keypair.publicKey), keypair };
}

function createSignerFromKeypair(keypair: Keypair): TransactionSigner {
  return {
    address: address(keypair.publicKey.toBase58()),
    modifyAndSignTransactions: async (txs: any) => txs,
  } as unknown as TransactionSigner;
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


function createMockPythPriceData(
  price: bigint,
  exponent: number,
  publishTime: bigint,
): Uint8Array {
  // PriceUpdateV2 Borsh layout (after 8-byte Anchor discriminator):
  //   write_authority: Pubkey (32)
  //   verification_level: VerificationLevel::Full = [1] (1 byte, no padding!)
  //   price_message: PriceFeedMessage {
  //     feed_id: [u8; 32], price: i64, conf: u64, exponent: i32,
  //     publish_time: i64, prev_publish_time: i64, ema_price: i64, ema_conf: u64
  //   }
  //   posted_slot: u64
  // Total: 8 + 32 + 1 + (32+8+8+4+8+8+8+8) + 8 = 133
  const buf = Buffer.alloc(133);
  let offset = 0;
  // Anchor discriminator
  Buffer.from([34, 241, 35, 99, 157, 126, 244, 205]).copy(buf, offset);
  offset += 8;
  // write_authority (32 zeros)
  offset += 32;
  // verification_level: Full = variant index 1, no fields
  buf[offset++] = 1;
  // feed_id (32 bytes)
  Buffer.alloc(32, 1).copy(buf, offset);
  offset += 32;
  // price (i64 LE)
  buf.writeBigInt64LE(price, offset);
  offset += 8;
  // conf (u64 LE)
  buf.writeBigUInt64LE(100n, offset);
  offset += 8;
  // exponent (i32 LE)
  buf.writeInt32LE(exponent, offset);
  offset += 4;
  // publish_time (i64 LE)
  buf.writeBigInt64LE(publishTime, offset);
  offset += 8;
  // prev_publish_time (i64 LE)
  buf.writeBigInt64LE(publishTime - 1n, offset);
  offset += 8;
  // ema_price (i64 LE)
  buf.writeBigInt64LE(price, offset);
  offset += 8;
  // ema_conf (u64 LE)
  buf.writeBigUInt64LE(100n, offset);
  offset += 8;
  // posted_slot (u64 LE)
  buf.writeBigUInt64LE(1n, offset);
  return new Uint8Array(buf);
}

// ============================================================================
// TEST SUITE
// ============================================================================

describe("RLP SDK Full Flow Test", function () {
  this.timeout(120000);

  let svm: LiteSVM;

  let admin: { address: Address; keypair: Keypair; signer: TransactionSigner };
  let user: { address: Address; keypair: Keypair; signer: TransactionSigner };

  let assetMint1: { address: Address; keypair: Keypair };
  let assetMint2: { address: Address; keypair: Keypair };
  let lpTokenMint: { address: Address; keypair: Keypair };

  let oracle1Address: Address;
  let oracle2Address: Address;

  let settingsPda: Address;
  let adminPermissionsPda: Address;
  let userPermissionsPda: Address;
  let liquidityPoolPda: Address;
  let asset1Pda: Address;
  let asset2Pda: Address;

  // Pool ATAs
  let poolAsset1Ata: Address;
  let poolAsset2Ata: Address;

  let programLoaded = false;
  let createdCooldownPda: Address;

  // SDK class instance (used for restake/withdraw which need remaining accounts)
  let restaking: Insurance;

  // ========================================================================
  // Transaction helpers
  // ========================================================================

  function sendTransaction(
    instructions: TransactionInstruction[],
    signers: Keypair[] = [],
    payer: Keypair = admin.keypair,
  ): TransactionResult {
    const tx = new LegacyTransaction();
    tx.recentBlockhash = svm.latestBlockhash();
    tx.feePayer = payer.publicKey;
    tx.add(
      ComputeBudgetProgram.setComputeUnitLimit({ units: 1_400_000 }),
      ...instructions,
    );
    const allSigners = [
      payer,
      ...signers.filter(
        (s) => s.publicKey.toBase58() !== payer.publicKey.toBase58(),
      ),
    ];
    tx.sign(...allSigners);
    const result = svm.sendTransaction(tx);
    if (isFailedTransaction(result)) {
      const meta = result.meta();
      return {
        success: false,
        signature: "",
        logs: meta.logs(),
        computeUnits:
          typeof meta.computeUnitsConsumed() === "bigint"
            ? Number(meta.computeUnitsConsumed())
            : (meta.computeUnitsConsumed() ?? 0),
        error: result.err()?.toString() || "Unknown error",
      };
    }
    const meta = (result as any).meta ? (result as any).meta() : result;
    const logs =
      typeof meta.logs === "function" ? meta.logs() : (meta.logs ?? []);
    const cu =
      typeof meta.computeUnitsConsumed === "function"
        ? meta.computeUnitsConsumed()
        : (meta.computeUnitsConsumed ?? 0);
    return {
      success: true,
      signature:
        typeof result.signature === "function"
          ? result.signature().toString()
          : "",
      logs,
      computeUnits: typeof cu === "bigint" ? Number(cu) : cu,
    };
  }

  function sendSdkInstruction(
    instruction: Instruction | Instruction[],
    signers: Keypair[] = [],
    payer: Keypair = admin.keypair,
  ): TransactionResult {
    const ixArray = Array.isArray(instruction) ? instruction : [instruction];
    return sendTransaction(
      ixArray.map(convertToLegacyInstruction),
      signers,
      payer,
    );
  }


  function assertSuccess(result: TransactionResult, desc: string): void {
    if (!result.success) {
      console.error(`  ✗ ${desc} FAILED`);
      console.error(`    Error: ${result.error}`);
      result.logs.forEach((l) => console.error(`      ${l}`));
    }
    expect(result.success, `${desc} should succeed. Error: ${result.error}`).to
      .be.true;
    console.log(`  ✓ ${desc} (CU: ${result.computeUnits.toLocaleString()})`);
  }

  function accountExists(addr: Address): boolean {
    return svm.getAccount(toPublicKey(addr)) !== null;
  }

  function getAccountOwner(addr: Address): string | null {
    return svm.getAccount(toPublicKey(addr))?.owner.toBase58() ?? null;
  }

  function getTokenBalance(ata: Address): bigint {
    const account = svm.getAccount(toPublicKey(ata));
    if (!account) return 0n;
    const dv = new DataView(
      account.data.buffer,
      account.data.byteOffset,
      account.data.byteLength,
    );
    return dv.getBigUint64(64, true);
  }

  async function createMintNoFreeze(
    mintKeypair: Keypair,
    decimals: number,
    mintAuthority: Address,
  ): Promise<TransactionResult> {
    const mintSigner = createSignerFromKeypair(mintKeypair);
    const payerSigner = createSignerFromKeypair(admin.keypair);
    return sendSdkInstruction(
      [
        getCreateAccountInstruction({
          payer: payerSigner,
          newAccount: mintSigner,
          lamports: lamports(10_000_000_000n),
          space: getMintSize(),
          programAddress: TOKEN_PROGRAM_ADDRESS,
        }),
        getInitializeMintInstruction({
          mint: mintSigner.address,
          decimals,
          mintAuthority,
        }),
      ],
      [mintKeypair],
    );
  }

  async function createMint(
    mintKeypair: Keypair,
    decimals: number,
    mintAuthority: Address,
  ): Promise<TransactionResult> {
    const mintSigner = createSignerFromKeypair(mintKeypair);
    const payerSigner = createSignerFromKeypair(admin.keypair);
    return sendSdkInstruction(
      [
        getCreateAccountInstruction({
          payer: payerSigner,
          newAccount: mintSigner,
          lamports: lamports(10_000_000_000n),
          space: getMintSize(),
          programAddress: TOKEN_PROGRAM_ADDRESS,
        }),
        getInitializeMintInstruction({
          mint: mintSigner.address,
          decimals,
          mintAuthority,
          freezeAuthority: mintAuthority,
        }),
      ],
      [mintKeypair],
    );
  }

  async function createAtaAndMint(
    mint: Address,
    owner: Address,
    mintAuthority: Keypair,
    amount: bigint,
  ): Promise<{ ata: Address; result: TransactionResult }> {
    const [ata] = await findAssociatedTokenPda({
      mint,
      owner,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });
    return {
      ata,
      result: sendSdkInstruction(
        [
          getCreateAssociatedTokenIdempotentInstruction({
            payer: createSignerFromKeypair(admin.keypair),
            ata,
            owner,
            mint,
            tokenProgram: TOKEN_PROGRAM_ADDRESS,
          }),
          getMintToInstruction({
            mint,
            token: ata,
            mintAuthority: createSignerFromKeypair(mintAuthority),
            amount,
          }),
        ],
        [mintAuthority],
      ),
    };
  }

  async function createAta(
    mint: Address,
    owner: Address,
    payer: Keypair = admin.keypair,
  ): Promise<Address> {
    const [ata] = await findAssociatedTokenPda({
      mint,
      owner,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });
    sendSdkInstruction(
      getCreateAssociatedTokenIdempotentInstruction({
        payer: createSignerFromKeypair(payer),
        ata,
        owner,
        mint,
        tokenProgram: TOKEN_PROGRAM_ADDRESS,
      }),
      [],
      payer,
    );
    return ata;
  }

  function createPythOracle(price: bigint, exponent: number): Address {
    const kp = Keypair.generate();
    const publishTime = BigInt(Math.floor(Date.now() / 1000));
    svm.setAccount(kp.publicKey, {
      lamports: 1_000_000,
      data: Buffer.from(createMockPythPriceData(price, exponent, publishTime)),
      owner: new PublicKey(PYTH_PROGRAM_ID),
      executable: false,
    });
    return toAddress(kp.publicKey);
  }


  // ========================================================================
  // Setup
  // ========================================================================

  before(async function () {
    svm = new LiteSVM();

    const adminKp = generateKeypair();
    admin = { ...adminKp, signer: createSignerFromKeypair(adminKp.keypair) };

    const userKp = generateKeypair();
    user = { ...userKp, signer: createSignerFromKeypair(userKp.keypair) };

    assetMint1 = generateKeypair();
    assetMint2 = generateKeypair();
    lpTokenMint = generateKeypair();

    svm.airdrop(toPublicKey(admin.address), BigInt(1000 * LAMPORTS_PER_SOL));
    svm.airdrop(toPublicKey(user.address), BigInt(100 * LAMPORTS_PER_SOL));

    try {
      svm.addProgramFromFile(toPublicKey(PROGRAM_ID), RLP_SO_PATH);
      programLoaded = true;
      console.log("  ✓ RLP program loaded");
    } catch (e) {
      console.error(`  ✗ Failed to load RLP program: ${e}`);
    }

    // Derive PDAs
    [settingsPda] = await PdaClient.deriveSettings();
    [adminPermissionsPda] = await PdaClient.deriveUserPermissions(admin.address);
    [userPermissionsPda] = await PdaClient.deriveUserPermissions(user.address);
    [liquidityPoolPda] = await PdaClient.deriveLiquidityPool(0);
    [asset1Pda] = await PdaClient.deriveAsset(assetMint1.address);
    [asset2Pda] = await PdaClient.deriveAsset(assetMint2.address);

    // Create oracles
    oracle1Address = createPythOracle(100_00000000n, -8); // $100
    oracle2Address = createPythOracle(50_00000000n, -8); // $50

    // Create asset mints (admin is mint authority)
    let r = await createMint(assetMint1.keypair, 9, admin.address);
    expect(r.success, "create assetMint1").to.be.true;

    r = await createMint(assetMint2.keypair, 9, admin.address);
    expect(r.success, "create assetMint2").to.be.true;

    // Create LP token mint: authority = pool PDA, NO freeze authority
    r = await createMintNoFreeze(lpTokenMint.keypair, 9, liquidityPoolPda);
    expect(r.success, "create lpTokenMint").to.be.true;

    // Derive pool ATAs
    [poolAsset1Ata] = await findAssociatedTokenPda({
      mint: assetMint1.address,
      owner: liquidityPoolPda,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });
    [poolAsset2Ata] = await findAssociatedTokenPda({
      mint: assetMint2.address,
      owner: liquidityPoolPda,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });

    console.log("  ✓ Setup complete");
  });

  // ========================================================================
  // 1. initializeRlp
  // ========================================================================
  it("should initialize RLP", async function () {
    if (!programLoaded) return this.skip();
    const ix = await getInitializeRlpInstructionAsync({
      signer: admin.signer,
      swapFeeBps: 30,
    });
    const result = sendSdkInstruction(ix);
    assertSuccess(result, "initializeRlp");
    expect(accountExists(settingsPda)).to.be.true;
    expect(getAccountOwner(settingsPda)).to.equal(PROGRAM_ID);
  });

  // ========================================================================
  // 2. addAsset
  // ========================================================================
  it("should add asset 1 (public)", async function () {
    if (!programLoaded) return this.skip();
    const ix = await getAddAssetInstructionAsync({
      signer: admin.signer,
      assetMint: assetMint1.address,
      oracle: oracle1Address,
      accessLevel: AccessLevel.Public,
    });
    const result = sendSdkInstruction(ix);
    assertSuccess(result, "addAsset (asset 1, public)");
    expect(accountExists(asset1Pda)).to.be.true;
  });

  it("should add asset 2 (public)", async function () {
    if (!programLoaded) return this.skip();
    const ix = await getAddAssetInstructionAsync({
      signer: admin.signer,
      assetMint: assetMint2.address,
      oracle: oracle2Address,
      accessLevel: AccessLevel.Public,
    });
    const result = sendSdkInstruction(ix);
    assertSuccess(result, "addAsset (asset 2, public)");
    expect(accountExists(asset2Pda)).to.be.true;
  });

  // ========================================================================
  // 3. updateActionRole
  // ========================================================================
  it("should set Restake, Withdraw, Swap, Slash to PUBLIC", async function () {
    if (!programLoaded) return this.skip();
    for (const action of [Action.Restake, Action.Withdraw, Action.Swap, Action.Slash]) {
      const ix = await getUpdateActionRoleInstructionAsync({
        admin: admin.signer,
        action,
        role: Role.PUBLIC,
        update: Update.Add,
      });
      assertSuccess(sendSdkInstruction(ix), `updateActionRole ${Action[action]} -> PUBLIC`);
    }
  });

  it("should add and remove a role from an action", async function () {
    if (!programLoaded) return this.skip();
    let ix = await getUpdateActionRoleInstructionAsync({
      admin: admin.signer,
      action: Action.SuspendDeposits,
      role: Role.TESTEE,
      update: Update.Add,
    });
    assertSuccess(sendSdkInstruction(ix), "add TESTEE to SuspendDeposits");

    ix = await getUpdateActionRoleInstructionAsync({
      admin: admin.signer,
      action: Action.SuspendDeposits,
      role: Role.TESTEE,
      update: Update.Remove,
    });
    assertSuccess(sendSdkInstruction(ix), "remove TESTEE from SuspendDeposits");
  });

  // ========================================================================
  // 4. createPermissionAccount
  // ========================================================================
  it("should create permission account for user", async function () {
    if (!programLoaded) return this.skip();
    const ix = await getCreatePermissionAccountInstructionAsync({
      caller: admin.signer,
      newAdmin: user.address,
    });
    assertSuccess(sendSdkInstruction(ix), "createPermissionAccount");
    expect(accountExists(userPermissionsPda)).to.be.true;
  });

  // ========================================================================
  // 5. updateRoleHolder
  // ========================================================================
  it("should add PUBLIC role to user", async function () {
    if (!programLoaded) return this.skip();
    const ix = await getUpdateRoleHolderInstructionAsync({
      admin: admin.signer,
      updateAdminPermissions: userPermissionsPda,
      address: user.address,
      role: Role.PUBLIC,
      update: Update.Add,
    });
    assertSuccess(sendSdkInstruction(ix), "updateRoleHolder add PUBLIC");
  });

  it("should add and remove CRANK role from user", async function () {
    if (!programLoaded) return this.skip();
    let ix = await getUpdateRoleHolderInstructionAsync({
      admin: admin.signer,
      updateAdminPermissions: userPermissionsPda,
      address: user.address,
      role: Role.CRANK,
      update: Update.Add,
    });
    assertSuccess(sendSdkInstruction(ix), "add CRANK");

    ix = await getUpdateRoleHolderInstructionAsync({
      admin: admin.signer,
      updateAdminPermissions: userPermissionsPda,
      address: user.address,
      role: Role.CRANK,
      update: Update.Remove,
    });
    assertSuccess(sendSdkInstruction(ix), "remove CRANK");
  });

  // ========================================================================
  // 6. freezeFunctionality
  // ========================================================================
  it("should freeze and unfreeze Restake", async function () {
    if (!programLoaded) return this.skip();
    let ix = await getFreezeFunctionalityInstructionAsync({
      admin: admin.signer, action: Action.FreezeRestake, freeze: true,
    });
    assertSuccess(sendSdkInstruction(ix), "freeze Restake");

    ix = await getFreezeFunctionalityInstructionAsync({
      admin: admin.signer, action: Action.FreezeRestake, freeze: false,
    });
    assertSuccess(sendSdkInstruction(ix), "unfreeze Restake");
  });

  // ========================================================================
  // 7. initializeLp
  // ========================================================================
  it("should initialize liquidity pool", async function () {
    if (!programLoaded) return this.skip();
    const ix = await getInitializeLpInstructionAsync({
      signer: admin.signer,
      liquidityPool: liquidityPoolPda,
      lpTokenMint: lpTokenMint.address,
      cooldownDuration: 0n, // 0 for instant withdrawals in test
      depositCap: null,
      assets: new Uint8Array([0, 1]), // asset indices to include in the pool
    });
    assertSuccess(sendSdkInstruction(ix), "initializeLp");
    expect(accountExists(liquidityPoolPda)).to.be.true;

    // Initialize Insurance SDK instance with cached state from LiteSVM
    restaking = new Insurance(null as any); // no RPC needed — we inject cached data
    restaking.loadFromCache(
      { liquidityPools: 1, assets: 2 } as Settings,
      [
        {
          address: liquidityPoolPda,
          data: { bump: 0, index: 0, lpToken: lpTokenMint.address, cooldowns: 0n, cooldownDuration: 0n, depositCap: null } as any,
        },
      ],
      [
        { address: asset1Pda, data: { bump: 0, index: 0, mint: assetMint1.address, oracle: { __kind: "Pyth" as const, fields: [oracle1Address] }, accessLevel: AccessLevel.Public } as any },
        { address: asset2Pda, data: { bump: 0, index: 1, mint: assetMint2.address, oracle: { __kind: "Pyth" as const, fields: [oracle2Address] }, accessLevel: AccessLevel.Public } as any },
      ],
    );
  });

  // ========================================================================
  // 8. updateDepositCap
  // ========================================================================
  it("should update and remove deposit cap", async function () {
    if (!programLoaded) return this.skip();
    let ix = await getUpdateDepositCapInstructionAsync({
      signer: admin.signer,
      liquidityPool: liquidityPoolPda,
      lockupId: 0n,
      newCap: 1_000_000_000_000n,
    });
    assertSuccess(sendSdkInstruction(ix), "updateDepositCap (set)");

    ix = await getUpdateDepositCapInstructionAsync({
      signer: admin.signer,
      liquidityPool: liquidityPoolPda,
      lockupId: 0n,
      newCap: null,
    });
    assertSuccess(sendSdkInstruction(ix), "updateDepositCap (remove)");
  });

  // ========================================================================
  // 9. restake
  // ========================================================================
  it("should restake tokens into the liquidity pool", async function () {
    if (!programLoaded) return this.skip();

    const amount = 1_000_000_000n; // 1 token

    // Mint asset tokens to user
    await createAtaAndMint(
      assetMint1.address, user.address, admin.keypair, amount * 10n,
    );

    // Create pool ATAs for BOTH assets
    await createAta(assetMint1.address, liquidityPoolPda);
    await createAta(assetMint2.address, liquidityPoolPda);

    // Create user LP ATA
    await createAta(lpTokenMint.address, user.address, user.keypair);

    // Insurance.restake() returns instruction with remaining accounts included
    const ix = await restaking.restake(user.signer, amount, assetMint1.address, 0, 0n);
    const result = sendSdkInstruction(ix, [user.keypair], user.keypair);
    assertSuccess(result, "restake");

    // Verify LP tokens were minted
    const [userLpAta] = await findAssociatedTokenPda({
      mint: lpTokenMint.address,
      owner: user.address,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });
    const lpBalance = getTokenBalance(userLpAta);
    expect(lpBalance).to.be.greaterThan(0n);
    console.log(`    LP tokens minted: ${lpBalance}`);

    // Verify tokens moved to pool
    const poolBalance = getTokenBalance(poolAsset1Ata);
    expect(poolBalance).to.equal(amount);
  });

  // ========================================================================
  // 10. requestWithdrawal
  // ========================================================================
  it("should request withdrawal (cooldown)", async function () {
    if (!programLoaded) return this.skip();

    const [userLpAta] = await findAssociatedTokenPda({
      mint: lpTokenMint.address,
      owner: user.address,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });

    const lpBalance = getTokenBalance(userLpAta);
    expect(lpBalance).to.be.greaterThan(0n, "user should have LP tokens");

    const withdrawAmount = lpBalance / 2n;
    const [cooldownPda] = await PdaClient.deriveCooldown(0, 0);

    const ix = await getRequestWithdrawalInstructionAsync({
      signer: user.signer,
      liquidityPool: liquidityPoolPda,
      lpTokenMint: lpTokenMint.address,
      signerLpTokenAccount: userLpAta,
      cooldown: cooldownPda,
      liquidityPoolId: 0,
      amount: withdrawAmount,
    });

    assertSuccess(
      sendSdkInstruction(ix, [user.keypair], user.keypair),
      "requestWithdrawal",
    );
    expect(accountExists(cooldownPda)).to.be.true;
    createdCooldownPda = cooldownPda;
  });

  // ========================================================================
  // 11. withdraw
  // ========================================================================
  it("should withdraw after cooldown", async function () {
    if (!programLoaded) return this.skip();

    const [cooldownPda] = await PdaClient.deriveCooldown(0, 0);
    expect(accountExists(cooldownPda), "cooldown account should exist").to.be
      .true;

    // Create user ATAs for all assets (withdraw transfers assets back to user)
    await createAta(assetMint1.address, user.address, user.keypair);
    await createAta(assetMint2.address, user.address, user.keypair);

    // Insurance.withdraw() returns instruction with remaining accounts included
    const ix = await restaking.withdraw(user.signer, 0, 0n);
    const result = sendSdkInstruction(ix, [user.keypair], user.keypair);

    assertSuccess(result, "withdraw");
    expect(accountExists(cooldownPda)).to.be.false;
  });

  // ========================================================================
  // 12. slash
  // ========================================================================
  it("should slash tokens from the liquidity pool", async function () {
    if (!programLoaded) return this.skip();

    // First restake more so there's tokens in the pool to slash
    const amount = 2_000_000_000n;
    await createAtaAndMint(
      assetMint1.address, user.address, admin.keypair, amount,
    );

    const restakeIx = await restaking.restake(user.signer, amount, assetMint1.address, 0, 0n);
    assertSuccess(
      sendSdkInstruction(restakeIx, [user.keypair], user.keypair),
      "restake for slash test",
    );

    const poolBalanceBefore = getTokenBalance(poolAsset1Ata);
    expect(poolBalanceBefore).to.be.greaterThan(0n);

    // MAX_SLASH_BPS = 1000 (10%), slash within limit
    const slashAmount = poolBalanceBefore / 20n; // 5%

    // Create destination ATA
    const destKp = generateKeypair();
    svm.airdrop(toPublicKey(destKp.address), BigInt(LAMPORTS_PER_SOL));
    const destAta = await createAta(assetMint1.address, destKp.address);

    const ix = await getSlashInstructionAsync({
      signer: admin.signer,
      liquidityPool: liquidityPoolPda,
      mint: assetMint1.address,
      asset: asset1Pda,
      liquidityPoolTokenAccount: poolAsset1Ata,
      destination: destAta,
      liquidityPoolId: 0,
      amount: slashAmount,
      assetId: 0,
    });

    // The program's destination account needs writable for CPI transfer,
    // but the IDL marks it readonly. Fix the account meta manually.
    const legacyIx = convertToLegacyInstruction(ix);
    const destIdx = legacyIx.keys.findIndex(
      (k) => k.pubkey.toBase58() === destAta,
    );
    if (destIdx >= 0) legacyIx.keys[destIdx].isWritable = true;
    assertSuccess(sendTransaction([legacyIx]), "slash");

    const poolBalanceAfter = getTokenBalance(poolAsset1Ata);
    expect(poolBalanceAfter).to.equal(poolBalanceBefore - slashAmount);
    const destBalance = getTokenBalance(destAta);
    expect(destBalance).to.equal(slashAmount);
    console.log(`    Slashed: ${slashAmount}, pool: ${poolBalanceBefore} -> ${poolBalanceAfter}`);
  });

  // ========================================================================
  // 13. swap
  // ========================================================================
  it("should swap tokens between assets", async function () {
    if (!programLoaded) return this.skip();

    // Seed pool with asset2 liquidity (direct mint since admin is mint authority)
    await createAtaAndMint(
      assetMint2.address,
      liquidityPoolPda,
      admin.keypair,
      5_000_000_000n, // 5 tokens of asset2
    );

    const swapAmount = 100_000_000n; // 0.1 token of asset1

    // User needs asset1 tokens
    const [userAsset1Ata] = await findAssociatedTokenPda({
      mint: assetMint1.address,
      owner: user.address,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });
    // Mint more asset1 tokens to user
    await createAtaAndMint(
      assetMint1.address, user.address, admin.keypair, swapAmount * 10n,
    );

    // User needs asset2 ATA
    const userAsset2Ata = await createAta(
      assetMint2.address, user.address, user.keypair,
    );

    const asset2BalBefore = getTokenBalance(userAsset2Ata);

    const ix = await getSwapInstructionAsync({
      signer: user.signer,
      liquidityPool: liquidityPoolPda,
      tokenFrom: assetMint1.address,
      tokenFromOracle: oracle1Address,
      tokenTo: assetMint2.address,
      tokenToOracle: oracle2Address,
      tokenFromSignerAccount: userAsset1Ata,
      tokenToSignerAccount: userAsset2Ata,
      amountIn: swapAmount,
      minOut: null,
    });

    assertSuccess(
      sendSdkInstruction(ix, [user.keypair], user.keypair),
      "swap",
    );

    const asset2BalAfter = getTokenBalance(userAsset2Ata);
    expect(asset2BalAfter).to.be.greaterThan(asset2BalBefore);
    console.log(`    Swapped ${swapAmount} asset1 -> ${asset2BalAfter - asset2BalBefore} asset2`);
  });

  // ========================================================================
  // 14. deserializePythPrice
  // ========================================================================
  it("should deserialize Pyth oracle price from raw account data", function () {
    // Build mock oracle data with known values
    const knownPrice = 12345_00000000n; // $12345 with 8 decimals
    const knownExponent = -8;
    const knownPublishTime = BigInt(Math.floor(Date.now() / 1000));
    const rawData = createMockPythPriceData(knownPrice, knownExponent, knownPublishTime);

    const parsed = Insurance.deserializePythPrice(rawData);

    expect(parsed.price).to.equal(knownPrice);
    expect(parsed.exponent).to.equal(knownExponent);
    expect(parsed.publishTime).to.equal(knownPublishTime);
    console.log(`    Parsed: price=${parsed.price}, exp=${parsed.exponent}, ts=${parsed.publishTime}`);
  });

  // ========================================================================
  // 15. simulateDepositMath
  // ========================================================================
  it("should simulate deposit math matching on-chain restake", async function () {
    if (!programLoaded) return this.skip();

    // Use the actual pool state to simulate a deposit, then execute it
    // and compare the simulation result with the actual LP tokens minted.
    const depositAmount = 500_000_000n; // 0.5 token

    // Mint tokens for the deposit
    await createAtaAndMint(
      assetMint1.address, user.address, admin.keypair, depositAmount,
    );

    // Read current pool state
    const [userLpAta] = await findAssociatedTokenPda({
      mint: lpTokenMint.address,
      owner: user.address,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });
    const lpBalBefore = getTokenBalance(userLpAta);

    // Read current LP supply from mint account
    const lpMintAcct = svm.getAccount(toPublicKey(lpTokenMint.address))!;
    const lpSupply = new DataView(
      lpMintAcct.data.buffer,
      lpMintAcct.data.byteOffset,
    ).getBigUint64(36, true); // supply at offset 36 in Mint layout

    // Read pool reserve balances
    const pool1Bal = getTokenBalance(poolAsset1Ata);
    const pool2Bal = getTokenBalance(poolAsset2Ata);

    // Simulate
    const simulated = Insurance.simulateDepositMath({
      depositAmount,
      depositAssetPrice: { price: 100_00000000n, exponent: -8 },
      depositAssetDecimals: 9,
      poolReserves: [
        { balance: pool1Bal, price: 100_00000000n, exponent: -8, decimals: 9 },
        { balance: pool2Bal, price: 50_00000000n, exponent: -8, decimals: 9 },
      ],
      lpTokenSupply: lpSupply,
      lpTokenDecimals: 9,
    });

    console.log(`    Simulated LP tokens: ${simulated}`);

    // Execute the actual restake
    const ix = await restaking.restake(
      user.signer, depositAmount, assetMint1.address, 0, 0n,
    );
    assertSuccess(
      sendSdkInstruction(ix, [user.keypair], user.keypair),
      "restake for simulation comparison",
    );

    const lpBalAfter = getTokenBalance(userLpAta);
    const actualMinted = lpBalAfter - lpBalBefore;
    console.log(`    Actual LP tokens:    ${actualMinted}`);

    // They should match exactly (same math, same rounding)
    expect(simulated).to.equal(actualMinted);
  });

  // ========================================================================
  // 15. simulateWithdrawMath
  // ========================================================================
  it("should simulate withdraw math matching on-chain proportions", async function () {
    if (!programLoaded) return this.skip();

    // Read current state
    const [userLpAta] = await findAssociatedTokenPda({
      mint: lpTokenMint.address,
      owner: user.address,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });
    const userLpBal = getTokenBalance(userLpAta);
    expect(userLpBal).to.be.greaterThan(0n);

    const lpMintAcct = svm.getAccount(toPublicKey(lpTokenMint.address))!;
    const lpSupply = new DataView(
      lpMintAcct.data.buffer,
      lpMintAcct.data.byteOffset,
    ).getBigUint64(36, true);

    const pool1Bal = getTokenBalance(poolAsset1Ata);
    const pool2Bal = getTokenBalance(poolAsset2Ata);

    const redeemAmount = userLpBal / 4n;

    const simulated = Insurance.simulateWithdrawMath({
      lpTokenAmount: redeemAmount,
      lpTokenSupply: lpSupply,
      reserves: [
        { mint: assetMint1.address, balance: pool1Bal },
        { mint: assetMint2.address, balance: pool2Bal },
      ],
    });

    console.log(`    Simulated withdraw for ${redeemAmount} LP tokens:`);
    for (const s of simulated) {
      console.log(`      ${s.mint}: ${s.amount}`);
    }

    // Verify proportionality: share ≈ redeemAmount / lpSupply * reserve
    for (const s of simulated) {
      const reserve =
        s.mint === assetMint1.address ? pool1Bal : pool2Bal;
      // Expected = reserve * redeemAmount / lpSupply (integer div)
      // Allow ±1 for rounding
      const expectedApprox = (reserve * redeemAmount) / lpSupply;
      const diff =
        s.amount > expectedApprox
          ? s.amount - expectedApprox
          : expectedApprox - s.amount;
      expect(diff).to.be.lessThanOrEqual(1n);
    }
  });
});
