import * as anchor from "@coral-xyz/anchor";
import { AnchorError, Program } from "@coral-xyz/anchor";
import { Rlp } from "../target/types/rlp";

import {
  PublicKey,
  Keypair,
  SystemProgram,
  LAMPORTS_PER_SOL,
  SYSVAR_CLOCK_PUBKEY,
  TransactionMessage,
  Connection,
  TransactionInstruction,
  TokenBalance,
  Transaction,
  MessageV0,
} from "@solana/web3.js";
import { before, it } from "mocha";
import {
  Asset,
  Cooldown,
  LiquidityPool,
  UserPermissions,
  PROGRAM_ID,
  Settings,
  Action,
  Role,
  Update,
  KillSwitch,
  AccessLevel,
} from "../sdk/src/generated";
import { expect } from "chai";
import createToken from "./helpers/createToken";
import BN from "bn.js";
import {
  AuthorityType,
  createAssociatedTokenAccountIdempotentInstruction,
  createAssociatedTokenAccountInstruction,
  createInitializeAccount3Instruction,
  createInitializeMint2Instruction,
  createSetAuthorityInstruction,
  getAccount,
  getAssociatedTokenAddressSync,
  getMint,
  MINT_SIZE,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import mintTokens from "./helpers/mintTokens";
import signAndSendTransaction from "./helpers/signAndSendTransaction";
import sleep from "./helpers/sleep";
import {
  PROGRAM_ID as METAPLEX_PROGRAM_ID,
  createCreateMetadataAccountV3Instruction,
} from "@metaplex-foundation/mpl-token-metadata";
import { Restaking } from "../sdk/src";
import getOraclePriceFromAccount from "./helpers/getOraclePriceFromAccount";
import { ASSOCIATED_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/utils/token";
import comparePubkeys from "./helpers/comparePubkeys";
import calculateLpToken from "./helpers/calculateLpToken";
import calculateReceiptToken from "./helpers/calculateReceiptToken";
import { AccountMeta } from "@solana/web3.js";
import { ComputeBudgetInstruction } from "@solana/web3.js";
import { ComputeBudgetProgram } from "@solana/web3.js";
import {
  calculateDepositValue,
  calculateLpTokensOnDeposit,
  calculateTotalPoolValue,
} from "./helpers/liquidityPoolMath";
import { AddressLookupTableProgram } from "@solana/web3.js";
import { VersionedTransaction } from "@solana/web3.js";
import getOraclePrice from "./helpers/getOraclePrice";
import { publicKey } from "@coral-xyz/anchor/dist/cjs/utils";

async function createLpToken(
  signer: PublicKey,
  liquidityPool: PublicKey,
  connection: Connection,
  withMetadata?: boolean,
) {
  const decimals = 9;

  const tokenKeypair = Keypair.generate();
  const instructions: TransactionInstruction[] = [];

  const createAccountIx = SystemProgram.createAccount({
    lamports: await connection.getMinimumBalanceForRentExemption(MINT_SIZE),
    space: MINT_SIZE,
    fromPubkey: signer,
    newAccountPubkey: tokenKeypair.publicKey,
    programId: TOKEN_PROGRAM_ID,
  });

  instructions.push(createAccountIx);

  const createMintIx = createInitializeMint2Instruction(
    tokenKeypair.publicKey,
    decimals,
    signer,
    null,
  );

  instructions.push(createMintIx);

  if (withMetadata) {
    const metadataData = {
      name: "Reflect | Insurance Fund Receipt",
      symbol: "RECEIPT",
      uri: "",
      sellerFeeBasisPoints: 0,
      creators: null,
      collection: null,
      uses: null,
    };

    const [metadata] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("metadata"),
        METAPLEX_PROGRAM_ID.toBuffer(),
        tokenKeypair.publicKey.toBuffer(),
      ],
      METAPLEX_PROGRAM_ID,
    );

    const createMetadataIx = createCreateMetadataAccountV3Instruction(
      {
        metadata,
        mint: tokenKeypair.publicKey,
        mintAuthority: signer,
        payer: signer,
        updateAuthority: signer,
      },
      {
        createMetadataAccountArgsV3: {
          data: metadataData,
          isMutable: true,
          collectionDetails: null,
        },
      },
    );

    instructions.push(createMetadataIx);
  }

  const setAuthorityIx = createSetAuthorityInstruction(
    tokenKeypair.publicKey,
    signer,
    AuthorityType.MintTokens,
    liquidityPool,
  );

  instructions.push(setAuthorityIx);

  return {
    instructions,
    mint: tokenKeypair,
  };
}

type WhitelistedAsset = {
  mint: PublicKey;
  oracle: PublicKey;
  accessLevel: AccessLevel;
};

describe("rlp", () => {
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);
  const program = anchor.workspace.Rlp as Program<Rlp>;

  let settings: PublicKey;
  let user: Keypair;
  let publicAssets: WhitelistedAsset[] = [];
  let admin: PublicKey;

  const whitelistedAssets: WhitelistedAsset[] = [];

  before(async () => {
    user = Keypair.generate();

    settings = Restaking.deriveSettings();

    await provider.connection.requestAirdrop(
      user.publicKey,
      LAMPORTS_PER_SOL * 1000,
    );

    admin = Restaking.deriveUserPermissions(provider.publicKey);

    await getOraclePrice(
      "ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
    );
    await getOraclePrice(
      "eaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a",
    );
  });

  function isActionFrozen(killswitch: KillSwitch, action: Action) {
    const mask = 1 << action;
    return (killswitch.frozen & mask) !== 0;
  }

  it.only("Initializes RLP.", async () => {
    const fee = 30;
    const cooldownDuration = new BN(30); // 30 seconds

    await program.methods
      .initializeRlp({
        feeBps: fee,
        cooldownDuration,
      })
      .accounts({
        signer: provider.publicKey,
        permissions: admin,
        settings,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const { frozen, liquidityPools, assets } =
      await Settings.fromAccountAddress(provider.connection, settings);

    expect(liquidityPools).eq(0);
    expect(assets).eq(0);
    expect(frozen).eq(false);
    // This will take effect once we generate the SDK again.
    // expect(swapFeeBps).eq(fee);
  });

  it.only("Adds public assets to the RLP.", async () => {
    for (let i = 0; i < 3; i++) {
      const token = await createToken(
        provider.connection,
        provider,
        i % 2 ? 9 : 6,
      );

      const oracleString =
        i % 2
          ? "7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE"
          : "Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX";

      publicAssets.push({
        mint: token,
        oracle: new PublicKey(oracleString),
        accessLevel: AccessLevel.Public,
      });

      const asset = Restaking.deriveAsset(token);

      let tx: string;
      try {
        tx = await program.methods
          .addAsset({
            accessLevel: { public: {} },
          })
          .accounts({
            admin,
            asset,
            assetMint: token,
            oracle: new PublicKey(oracleString),
            settings,
            signer: provider.publicKey,
            systemProgram: SystemProgram.programId,
          })
          .rpc();
      } catch (err) {
        console.log("[tx failed with logs]", err);
      }

      const assetData = await Asset.fromAccountAddress(
        provider.connection,
        asset,
      );

      expect(assetData.mint.toString()).eq(token.toString());
      expect(assetData.oracle.fields[0].toString()).eq(oracleString);
      expect(assetData.accessLevel).eq(AccessLevel.Public);
    }
  });

  it("Freezes protocol.", async () => {
    await program.methods
      .freezeFunctionality({
        action: { freezeRestake: {} },
        freeze: true,
      })
      .accounts({
        admin: provider.publicKey,
        adminPermissions: admin,
        settings,
      })
      .rpc();

    let settingsData = await Settings.fromAccountAddress(
      provider.connection,
      settings,
    );

    const killswitch = settingsData.accessControl.killswitch;
    expect(isActionFrozen(killswitch, Action.Restake)).eq(true);
  });

  it("Tries to interact with frozen protocol. Succeeds on errors", async () => {
    let error: AnchorError;

    const wrappedSol = new PublicKey(
      "So11111111111111111111111111111111111111112",
    );
    const asset = Restaking.deriveAsset(wrappedSol);

    await program.methods
      .addAsset({
        accessLevel: { private: {} },
      })
      .accounts({
        assetMint: wrappedSol,
        asset,
        systemProgram: SystemProgram.programId,
        settings,
        signer: provider.publicKey,
        admin,
        // Just to pass the test
        oracle: new PublicKey("7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE"),
      })
      .rpc()
      .catch((err: AnchorError) => (error = err));
  });

  it("Unfreezes protocol.", async () => {
    await program.methods
      .freezeFunctionality({
        action: { freezeRestake: {} },
        freeze: false,
      })
      .accounts({
        admin: provider.publicKey,
        adminPermissions: admin,
        settings,
      })
      .rpc();

    let settingsData = await Settings.fromAccountAddress(
      provider.connection,
      settings,
    );

    const killswitch = settingsData.accessControl.killswitch;
    expect(killswitch.frozen).eq(0);
  });

  it.only("Mints and adds private assets to insurance pool.", async () => {
    const assets: PublicKey[] = [];

    for (let i = 0; i < 2; i++) {
      const oracleString =
        i % 2
          ? "7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE"
          : "Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX";

      const token = await createToken(
        provider.connection,
        provider,
        i % 2 ? 9 : 6,
      );

      whitelistedAssets.push({
        mint: token,
        oracle: new PublicKey(oracleString),
        accessLevel: AccessLevel.Private,
      });

      const asset = Restaking.deriveAsset(token);

      assets.push(asset);

      await program.methods
        .addAsset({
          accessLevel: { private: {} },
        })
        .accounts({
          assetMint: token,
          asset,
          systemProgram: SystemProgram.programId,
          settings,
          signer: provider.publicKey,
          admin,
          // Just to pass the test
          oracle: new PublicKey(oracleString),
        })
        .rpc();
    }

    for (let i = 0; i < assets.length; i++) {
      const asset = assets[i];
      const assetData = await Asset.fromAccountAddress(
        provider.connection,
        asset,
      );

      const oracleString =
        i % 2
          ? "7UVimffxr9ow1uXYxsr4LHAcV58mLzhmwaeKvJ1pjLiE"
          : "Dpw1EAVrSB1ibxiDQyTAW6Zip3J4Btk2x4SgApQCeFbX";

      expect(whitelistedAssets[i].mint.toString()).eq(
        assetData.mint.toString(),
      );
      expect(assetData.oracle.__kind).eq("Pyth");
      expect(assetData.oracle.fields[0].toString()).eq(oracleString);
      expect(assetData.accessLevel).eq(AccessLevel.Private);
    }
  });

  it.only("Initializes liquidity pool", async () => {
    const liquidityPool = Restaking.deriveLiquidityPool(0);

    const { mint: lpTokenMint, instructions: preInstructions } =
      await createLpToken(
        provider.publicKey,
        liquidityPool,
        provider.connection,
        false,
      );

    await program.methods
      .initializeLp({
        cooldownDuration: new BN(30),
        cooldowns: new BN(0),
      })
      .preInstructions(preInstructions)
      .accounts({
        permissions: admin,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        liquidityPool,
        lpTokenMint: lpTokenMint.publicKey,
        signer: provider.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        settings: Restaking.deriveSettings(),
      })
      .signers([lpTokenMint])
      .rpc();

    const { lpToken, index, cooldownDuration, cooldowns } =
      await LiquidityPool.fromAccountAddress(
        provider.connection,
        liquidityPool,
      );

    expect(cooldownDuration.toString()).eq("30");
    expect(cooldowns.toString()).eq("0");
    expect(lpToken.toString()).eq(lpTokenMint.publicKey.toString());
    expect(index).eq(0);
  });

  it.only("Initializes LP-owned token accounts.", async () => {
    const restaking = new Restaking(provider.connection);
    const assets = await restaking.getAssets();

    const liquidityPool = Restaking.deriveLiquidityPool(0);

    await Promise.all(
      assets.map(async ({ pubkey, account }) => {
        const lpMintTokenAccount = getAssociatedTokenAddressSync(
          account.mint,
          liquidityPool,
          true,
        );

        const instruction = await program.methods
          .initializeLpTokenAccount({
            liquidityPoolIndex: 0,
          })
          .accounts({
            admin,
            asset: pubkey,
            associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
            liquidityPool,
            lpMintTokenAccount,
            mint: account.mint,
            settings,
            signer: provider.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .instruction();

        await program.methods
          .initializeLpTokenAccount({
            liquidityPoolIndex: 0,
          })
          .accounts({
            admin,
            asset: pubkey,
            associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
            liquidityPool,
            lpMintTokenAccount,
            mint: account.mint,
            settings,
            signer: provider.publicKey,
            systemProgram: SystemProgram.programId,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .rpc();
      }),
    );
  });

  it.only("Sets the restaking action to public.", async () => {
    await program.methods
      // Necessary to ignore fucked-up anchor uppercasing the enum.
      // @ts-ignore
      .updateActionRole({
        action: { restake: {} },
        role: { public: {} },
        update: { add: {} },
      })
      .accounts({
        admin: provider.publicKey,
        settings,
        adminPermissions: admin,
      })
      .rpc();

    const settingsData = await Settings.fromAccountAddress(
      provider.connection,
      settings,
    );

    const actionPermissionMap =
      settingsData.accessControl.accessMap.actionPermissions.find(
        (mapping) => mapping.action === Action.Restake,
      );
    expect(actionPermissionMap.action).eq(Action.Restake);
    expect(actionPermissionMap.allowedRoles).include(Role.PUBLIC);
  });

  it.only("Restakes tokens in liquidity pool.", async () => {
    const liquidityPoolId = 0;
    const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);

    const liquidityPoolData = await LiquidityPool.fromAccountAddress(
      provider.connection,
      liquidityPool,
    );

    // Create two fresh users for isolated LP token tracking
    const userA = Keypair.generate();
    const userB = Keypair.generate();

    await provider.connection.requestAirdrop(
      userA.publicKey,
      LAMPORTS_PER_SOL * 10,
    );
    await provider.connection.requestAirdrop(
      userB.publicKey,
      LAMPORTS_PER_SOL * 10,
    );
    await sleep(1);

    const restaking = new Restaking(provider.connection);
    const assets = await restaking.getAssets();

    const anchorRemainingAccounts: AccountMeta[] = assets
      .map(
        ({ account }) =>
          [
            {
              isSigner: false,
              isWritable: true,
              pubkey: getAssociatedTokenAddressSync(
                account.mint,
                liquidityPool,
                true,
              ),
            },
            {
              isSigner: false,
              isWritable: false,
              pubkey: Restaking.deriveAsset(account.mint),
            },
            {
              isSigner: false,
              isWritable: false,
              pubkey: account.oracle.fields[0],
            },
            { isSigner: false, isWritable: false, pubkey: account.mint },
          ] as AccountMeta[],
      )
      .flat();

    // Fetch oracle prices
    // whitelistedAssets[0] = 6 decimals (USDC-like)
    // whitelistedAssets[1] = 9 decimals (SOL-like)
    const usdcOraclePrice = await getOraclePrice(
      "eaa020c61cc479712813461ce153894a96a6c00b21ed0cfc2798d1f9a9e9c94a",
    );
    const solOraclePrice = await getOraclePrice(
      "ef0d8b6fda2ceba41da15d4095d1da392a0d2f8ed0c6c7bc0f4cfac8c280b56d",
    );

    console.log(
      "USDC price:",
      usdcOraclePrice.price.toString(),
      "exponent:",
      usdcOraclePrice.precision.toString(),
    );
    console.log(
      "SOL price:",
      solOraclePrice.price.toString(),
      "exponent:",
      solOraclePrice.precision.toString(),
    );

    const usdcDecimals = 6;
    const solDecimals = 9;

    // USDC amount: 1000 USDC
    const usdcAmount = new BN(1000).mul(new BN(10).pow(new BN(usdcDecimals)));

    // Calculate SOL amount for same USD value
    const solAmount = usdcAmount
      .mul(usdcOraclePrice.price)
      .mul(new BN(10).pow(new BN(solDecimals)))
      .div(solOraclePrice.price.mul(new BN(10).pow(new BN(usdcDecimals))));

    console.log(
      "USDC raw amount:",
      usdcAmount.toString(),
      "(",
      usdcAmount.toNumber() / 10 ** usdcDecimals,
      "USDC )",
    );
    console.log(
      "SOL raw amount:",
      solAmount.toString(),
      "(",
      solAmount.toNumber() / 10 ** solDecimals,
      "SOL )",
    );

    // Mint tokens to users
    await mintTokens(
      whitelistedAssets[0].mint,
      provider,
      usdcAmount.toNumber(),
      userA.publicKey,
    );
    await mintTokens(
      whitelistedAssets[1].mint,
      provider,
      solAmount.toNumber(),
      userB.publicKey,
    );

    // === User A deposits USDC-like token (6 decimals) ===
    const assetA = Restaking.deriveAsset(whitelistedAssets[0].mint);
    const assetAData = await Asset.fromAccountAddress(
      provider.connection,
      assetA,
    );
    const userAAssetAta = getAssociatedTokenAddressSync(
      whitelistedAssets[0].mint,
      userA.publicKey,
    );
    const poolAssetAccountA = getAssociatedTokenAddressSync(
      whitelistedAssets[0].mint,
      liquidityPool,
      true,
    );
    const userALpAccount = getAssociatedTokenAddressSync(
      liquidityPoolData.lpToken,
      userA.publicKey,
    );

    await program.methods
      .restake({
        liquidityPoolIndex: liquidityPoolId,
        amount: usdcAmount,
        minLpTokens: new BN(0),
      })
      .accounts({
        signer: userA.publicKey,
        settings,
        liquidityPool,
        lpToken: liquidityPoolData.lpToken,
        userLpAccount: userALpAccount,
        asset: assetA,
        assetMint: whitelistedAssets[0].mint,
        userAssetAccount: userAAssetAta,
        poolAssetAccount: poolAssetAccountA,
        oracle: assetAData.oracle.fields[0],
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        permissions: null,
      })
      .remainingAccounts(anchorRemainingAccounts)
      .signers([userA])
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 1_000_000 }),
        createAssociatedTokenAccountInstruction(
          userA.publicKey,
          userALpAccount,
          userA.publicKey,
          liquidityPoolData.lpToken,
        ),
      ])
      .rpc();

    await sleep(2);
    const userALpAccountData = await getAccount(
      provider.connection,
      userALpAccount,
    );
    const lpTokensA = BigInt(userALpAccountData.amount.toString());
    console.log(
      "\nUser A (USDC-like, 6 decimals) LP tokens:",
      lpTokensA.toString(),
    );

    const poolAssetAAccount = await getAccount(
      provider.connection,
      poolAssetAccountA,
    );
    const poolAssetAAmount = poolAssetAAccount.amount;
    console.log(
      "\Pool (USDC-like, 6 decimals) amount:",
      poolAssetAAmount.toString(),
    );
    expect(Number(poolAssetAAmount)).to.be.eq(Number(usdcAmount));

    // === User B deposits SOL-like token (9 decimals) ===
    const assetB = Restaking.deriveAsset(whitelistedAssets[1].mint);
    const assetBData = await Asset.fromAccountAddress(
      provider.connection,
      assetB,
    );
    const userBAssetAta = getAssociatedTokenAddressSync(
      whitelistedAssets[1].mint,
      userB.publicKey,
    );
    const poolAssetAccountB = getAssociatedTokenAddressSync(
      whitelistedAssets[1].mint,
      liquidityPool,
      true,
    );
    const userBLpAccount = getAssociatedTokenAddressSync(
      liquidityPoolData.lpToken,
      userB.publicKey,
    );

    await program.methods
      .restake({
        liquidityPoolIndex: liquidityPoolId,
        amount: solAmount,
        minLpTokens: new BN(0),
      })
      .accounts({
        signer: userB.publicKey,
        settings,
        liquidityPool,
        lpToken: liquidityPoolData.lpToken,
        userLpAccount: userBLpAccount,
        asset: assetB,
        assetMint: whitelistedAssets[1].mint,
        userAssetAccount: userBAssetAta,
        poolAssetAccount: poolAssetAccountB,
        oracle: assetBData.oracle.fields[0],
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        permissions: null,
      })
      .remainingAccounts(anchorRemainingAccounts)
      .signers([userB])
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 1_000_000 }),
        createAssociatedTokenAccountInstruction(
          userB.publicKey,
          userBLpAccount,
          userB.publicKey,
          liquidityPoolData.lpToken,
        ),
      ])
      .rpc();

    await sleep(2);
    const userBLpAccountData = await getAccount(
      provider.connection,
      userBLpAccount,
    );
    const lpTokensB = BigInt(userBLpAccountData.amount.toString());
    console.log(
      "User B (SOL-like, 9 decimals) LP tokens:",
      lpTokensB.toString(),
    );

    const poolAssetBAccount = await getAccount(
      provider.connection,
      poolAssetAccountB,
    );
    const poolAssetBAmount = poolAssetBAccount.amount;
    console.log(
      "\Pool (WSOL-like, 9 decimals) amount:",
      poolAssetBAmount.toString(),
    );
    expect(Number(poolAssetBAmount)).to.be.eq(Number(solAmount));

    // Compare LP tokens
    const ratio = Number(lpTokensA) / Number(lpTokensB);
    console.log("\n=== RESULTS ===");
    console.log("LP tokens for equal USD value deposits:");
    console.log("  User A (6 decimals):", lpTokensA.toString());
    console.log("  User B (9 decimals):", lpTokensB.toString());
    console.log("  Ratio (A/B):", ratio.toFixed(6));

    expect(Number(lpTokensA)).gt(0);
    expect(Number(lpTokensB)).gt(0);

    const tolerance = 0.05;
    expect(ratio).to.be.closeTo(1.0, tolerance);
  });

  it.only("Rejects restake with duplicate assets in remaining_accounts", async () => {
    const liquidityPoolId = 0;
    const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);

    const liquidityPoolData = await LiquidityPool.fromAccountAddress(
      provider.connection,
      liquidityPool,
    );

    // Create attacker
    const attacker = Keypair.generate();
    await provider.connection.requestAirdrop(
      attacker.publicKey,
      LAMPORTS_PER_SOL * 10,
    );
    await sleep(1);

    const depositAmount = new BN(1000 * 10 ** 6);

    // Mint tokens to attacker
    await mintTokens(
      whitelistedAssets[0].mint,
      provider,
      depositAmount.toNumber(),
      attacker.publicKey,
    );

    const assetA = Restaking.deriveAsset(whitelistedAssets[0].mint);
    const assetAData = await Asset.fromAccountAddress(
      provider.connection,
      assetA,
    );

    const attackerAssetAta = getAssociatedTokenAddressSync(
      whitelistedAssets[0].mint,
      attacker.publicKey,
    );
    const poolAssetAccountA = getAssociatedTokenAddressSync(
      whitelistedAssets[0].mint,
      liquidityPool,
      true,
    );
    const attackerLpAccount = getAssociatedTokenAddressSync(
      liquidityPoolData.lpToken,
      attacker.publicKey,
    );

    const restaking = new Restaking(provider.connection);
    const assets = await restaking.getAssets();

    // Build MALICIOUS remaining_accounts: duplicate assets
    const maliciousRemainingAccounts: AccountMeta[] = assets
      .map(
        () =>
          [
            {
              isSigner: false,
              isWritable: true,
              pubkey: getAssociatedTokenAddressSync(
                whitelistedAssets[0].mint,
                liquidityPool,
                true,
              ),
            },
            {
              isSigner: false,
              isWritable: false,
              pubkey: Restaking.deriveAsset(whitelistedAssets[0].mint),
            },
            {
              isSigner: false,
              isWritable: false,
              pubkey: whitelistedAssets[0].oracle,
            },
            {
              isSigner: false,
              isWritable: false,
              pubkey: whitelistedAssets[0].mint,
            },
          ] as AccountMeta[],
      )
      .flat();

    console.log(
      "Attempting restake with duplicated assets in remaining_accounts...",
    );

    let errorThrown = false;
    let errorMessage = "";

    try {
      await program.methods
        .restake({
          liquidityPoolIndex: liquidityPoolId,
          amount: depositAmount,
          minLpTokens: new BN(0),
        })
        .accounts({
          signer: attacker.publicKey,
          settings,
          liquidityPool,
          lpToken: liquidityPoolData.lpToken,
          userLpAccount: attackerLpAccount,
          asset: assetA,
          assetMint: whitelistedAssets[0].mint,
          userAssetAccount: attackerAssetAta,
          poolAssetAccount: poolAssetAccountA,
          oracle: assetAData.oracle.fields[0],
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          permissions: null,
        })
        .remainingAccounts(maliciousRemainingAccounts)
        .signers([attacker])
        .preInstructions([
          ComputeBudgetProgram.setComputeUnitLimit({ units: 1_000_000 }),
          createAssociatedTokenAccountInstruction(
            attacker.publicKey,
            attackerLpAccount,
            attacker.publicKey,
            liquidityPoolData.lpToken,
          ),
        ])
        .rpc();
    } catch (err: any) {
      errorThrown = true;
      errorMessage = err.message || err.toString();
      console.log("Transaction failed as expected:", errorMessage);
    }

    expect(errorThrown).to.be.true;
  });

  it.only("Verifies swap price impact: larger trades get worse rates", async () => {
    const liquidityPoolId = 0;
    const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);

    // Create two test users
    // const smallTrader = Keypair.generate();
    // const largeTrader = Keypair.generate();
    const smallTrader = provider;
    const largeTrader = provider;

    // admin = Restaking.deriveUserPermissions(provider.publicKey);

    await provider.connection.requestAirdrop(
      smallTrader.publicKey,
      LAMPORTS_PER_SOL * 10,
    );
    await provider.connection.requestAirdrop(
      largeTrader.publicKey,
      LAMPORTS_PER_SOL * 10,
    );
    await sleep(1);

    const fromMint = whitelistedAssets[0].mint;
    const toMint = whitelistedAssets[1].mint;

    const fromAsset = Restaking.deriveAsset(fromMint);
    const toAsset = Restaking.deriveAsset(toMint);

    const { oracle: fromOracle } = await Asset.fromAccountAddress(
      provider.connection,
      fromAsset,
    );
    const { oracle: toOracle } = await Asset.fromAccountAddress(
      provider.connection,
      toAsset,
    );

    // Check pool liquidity first
    const poolFromAccount = getAssociatedTokenAddressSync(
      fromMint,
      liquidityPool,
      true,
    );
    const poolFromData = await getAccount(provider.connection, poolFromAccount);
    const poolLiquidity = Number(poolFromData.amount);
    console.log("\nPool liquidity (token_from):", poolLiquidity);

    // Small trade: 1% of pool
    const smallAmount = new BN(Math.floor(poolLiquidity * 0.01));
    // Large trade: 30% of pool
    const largeAmount = new BN(Math.floor(poolLiquidity * 0.3));

    console.log("Small trade amount:", smallAmount.toString(), "(1% of pool)");
    console.log("Large trade amount:", largeAmount.toString(), "(30% of pool)");

    // Mint tokens to traders
    await mintTokens(
      fromMint,
      provider,
      smallAmount.toNumber() * 2,
      smallTrader.publicKey,
    );
    await mintTokens(
      fromMint,
      provider,
      largeAmount.toNumber() * 2,
      largeTrader.publicKey,
    );

    // Setup token accounts
    const smallTraderFrom = getAssociatedTokenAddressSync(
      fromMint,
      smallTrader.publicKey,
      true,
    );
    const smallTraderTo = getAssociatedTokenAddressSync(
      toMint,
      smallTrader.publicKey,
      true,
    );
    const largeTraderFrom = getAssociatedTokenAddressSync(
      fromMint,
      largeTrader.publicKey,
      true,
    );
    const largeTraderTo = getAssociatedTokenAddressSync(
      toMint,
      largeTrader.publicKey,
      true,
    );

    // Get pre-balances
    const smallTraderToPreBalance = await getAccount(
      provider.connection,
      smallTraderTo,
    ).catch(() => ({ amount: BigInt(0) }));
    const largeTraderToPreBalance = await getAccount(
      provider.connection,
      largeTraderTo,
    ).catch(() => ({ amount: BigInt(0) }));

    // === Small Trade ===
    console.log("\n--- Executing small trade ---");
    await program.methods
      .swap({
        amountIn: smallAmount,
        minOut: new BN(0),
      })
      .preInstructions([
        createAssociatedTokenAccountIdempotentInstruction(
          smallTrader.publicKey,
          smallTraderTo,
          smallTrader.publicKey,
          toMint,
        ),
      ])
      .accounts({
        settings,
        signer: smallTrader.publicKey,
        admin,
        tokenProgram: TOKEN_PROGRAM_ID,
        liquidityPool,
        tokenFrom: fromMint,
        tokenFromAsset: fromAsset,
        tokenFromOracle: fromOracle.fields[0],
        tokenTo: toMint,
        tokenToAsset: toAsset,
        tokenToOracle: toOracle.fields[0],
        tokenFromPool: getAssociatedTokenAddressSync(
          fromMint,
          liquidityPool,
          true,
        ),
        tokenToPool: getAssociatedTokenAddressSync(toMint, liquidityPool, true),
        tokenFromSignerAccount: smallTraderFrom,
        tokenToSignerAccount: smallTraderTo,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
      })
      // .signers([smallTrader])
      .rpc()
      .catch((err) => {
        console.error("Small trade failed:", err.logs?.join("\n"));
        throw err;
      });

    await sleep(1);

    // === Large Trade ===
    console.log("\n--- Executing large trade ---");
    const largeTradeTx = await program.methods
      .swap({
        amountIn: largeAmount,
        minOut: new BN(0),
      })
      .preInstructions([
        createAssociatedTokenAccountIdempotentInstruction(
          largeTrader.publicKey,
          largeTraderTo,
          largeTrader.publicKey,
          toMint,
        ),
      ])
      .accounts({
        settings,
        signer: largeTrader.publicKey,
        admin,
        tokenProgram: TOKEN_PROGRAM_ID,
        liquidityPool,
        tokenFrom: fromMint,
        tokenFromAsset: fromAsset,
        tokenFromOracle: fromOracle.fields[0],
        tokenTo: toMint,
        tokenToAsset: toAsset,
        tokenToOracle: toOracle.fields[0],
        tokenFromPool: getAssociatedTokenAddressSync(
          fromMint,
          liquidityPool,
          true,
        ),
        tokenToPool: getAssociatedTokenAddressSync(toMint, liquidityPool, true),
        tokenFromSignerAccount: largeTraderFrom,
        tokenToSignerAccount: largeTraderTo,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
      })
      // .signers([largeTrader])
      .rpc()
      .catch((err) => {
        console.error("Large trade failed:", err.logs?.join("\n"));
        throw err;
      });

    await sleep(1);

    // Get post-balances
    const smallTraderToPostBalance = await getAccount(
      provider.connection,
      smallTraderTo,
    );
    const largeTraderToPostBalance = await getAccount(
      provider.connection,
      largeTraderTo,
    );

    const smallTraderReceived =
      Number(smallTraderToPostBalance.amount) -
      Number(smallTraderToPreBalance.amount);
    const largeTraderReceived =
      Number(largeTraderToPostBalance.amount) -
      Number(largeTraderToPreBalance.amount);

    // Calculate effective rates (output per unit input)
    const smallTradeRate = smallTraderReceived / smallAmount.toNumber();
    const largeTradeRate = largeTraderReceived / largeAmount.toNumber();

    console.log("\n=== PRICE IMPACT TEST RESULTS ===");
    console.log("Small trade (1% of pool):");
    console.log("  Input:", smallAmount.toString());
    console.log("  Output:", smallTraderReceived);
    console.log("  Rate:", smallTradeRate.toFixed(6), "per unit");

    console.log("\nLarge trade (30% of pool):");
    console.log("  Input:", largeAmount.toString());
    console.log("  Output:", largeTraderReceived);
    console.log("  Rate:", largeTradeRate.toFixed(6), "per unit");

    const rateRatio = smallTradeRate / largeTradeRate;
    console.log("\nRate ratio (small/large):", rateRatio.toFixed(4));
    console.log("Expected: > 1.0 (small trades should get better rates)");

    // Verify price impact: small trade should get better rate than large trade
    expect(smallTradeRate).to.be.greaterThan(
      largeTradeRate,
      "Small trade should get better rate than large trade due to price impact",
    );

    // Verify the ratio is meaningful (at least 5% better for small trade)
    expect(rateRatio).to.be.greaterThan(
      1.05,
      "Price impact should cause at least 5% rate difference",
    );

    // const tx = await provider.connection.getTransaction(largeTradeTx, {
    //   commitment: "confirmed",
    // });
    // console.log(tx.meta.logMessages);
  });

  it("Restakes token B in liquidity pool to allow swaps later", async () => {
    const liquidityPoolId = 0;
    const amount = new BN(LAMPORTS_PER_SOL * 1_000);

    const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);

    const liquidityPoolData = await LiquidityPool.fromAccountAddress(
      provider.connection,
      liquidityPool,
    );

    const asset = Restaking.deriveAsset(whitelistedAssets[1].mint);

    await mintTokens(
      whitelistedAssets[1].mint,
      provider,
      amount.toNumber(),
      user.publicKey,
    );

    const userAssetAta = getAssociatedTokenAddressSync(
      whitelistedAssets[1].mint,
      user.publicKey,
    );

    const poolAssetAccount = getAssociatedTokenAddressSync(
      whitelistedAssets[1].mint,
      liquidityPool,
      true,
    );

    const userLpAccount = getAssociatedTokenAddressSync(
      liquidityPoolData.lpToken,
      user.publicKey,
    );

    const assetData = await Asset.fromAccountAddress(
      provider.connection,
      asset,
    );

    const restaking = new Restaking(provider.connection);
    const assets = await restaking.getAssets();

    const anchorRemainingAccounts: AccountMeta[] = assets
      .map(({ account, pubkey }) => {
        return [
          {
            isSigner: false,
            isWritable: true,
            pubkey: getAssociatedTokenAddressSync(
              account.mint,
              liquidityPool,
              true,
            ),
          },
          {
            isSigner: false,
            isWritable: false,
            pubkey: Restaking.deriveAsset(account.mint),
          },
          {
            isSigner: false,
            isWritable: false,
            pubkey: account.oracle.fields[0],
          },
        ] as AccountMeta[];
      })
      .flat();

    await program.methods
      .restake({
        liquidityPoolIndex: liquidityPoolId,
        amount,
        minLpTokens: new BN(0),
      })
      .accounts({
        signer: user.publicKey,
        settings,
        liquidityPool,
        lpToken: liquidityPoolData.lpToken,
        userLpAccount,
        asset,
        assetMint: whitelistedAssets[1].mint,
        userAssetAccount: userAssetAta,
        poolAssetAccount,
        oracle: assetData.oracle.fields[0],
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        permissions: null,
      })
      .remainingAccounts(anchorRemainingAccounts)
      .signers([user])
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 1_000_000 }),
      ])
      .rpc()
      .catch((err) => {
        console.log(err);
        throw err;
      });

    await new Promise((resolve) => setTimeout(resolve, 15000));

    const userLpAccountData = await getAccount(
      provider.connection,
      userLpAccount,
    );

    expect(parseInt(userLpAccountData.amount.toString())).gt(0);
  });

  it("Deposits rewards A", async () => {
    const liquidityPoolId = 0;
    const amount = new BN(100_000 * LAMPORTS_PER_SOL);

    const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);

    const signerAssetTokenAccount = getAssociatedTokenAddressSync(
      publicAssets[0].mint,
      provider.publicKey,
    );

    await mintTokens(
      publicAssets[0].mint,
      provider,
      100_000 * LAMPORTS_PER_SOL,
      provider.publicKey,
    );

    const liquidityPoolDataPre = await LiquidityPool.fromAccountAddress(
      provider.connection,
      liquidityPool,
    );

    await program.methods
      .depositRewards({
        amount,
      })
      .accounts({
        signer: provider.publicKey,
        permissions: admin,
        settings,
        systemProgram: SystemProgram.programId,
        liquidityPool,
        tokenProgram: TOKEN_PROGRAM_ID,
        assetPool: getAssociatedTokenAddressSync(
          publicAssets[0].mint,
          liquidityPool,
          true,
        ),
        asset: Restaking.deriveAsset(publicAssets[0].mint),
        assetMint: publicAssets[0].mint,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        signerAssetTokenAccount,
      })
      .rpc()
      .catch((err) => {
        console.log(err.logs);
        throw err;
      });

    const liquidityPoolDataPost = await LiquidityPool.fromAccountAddress(
      provider.connection,
      liquidityPool,
    );

    // Verify rewards were deposited
    const assetPoolData = await getAccount(
      provider.connection,
      getAssociatedTokenAddressSync(publicAssets[0].mint, liquidityPool, true),
    );

    expect(parseInt(assetPoolData.amount.toString())).eq(amount.toNumber());
  });

  it("Deposits rewards B", async () => {
    const liquidityPoolId = 0;
    const amount = new BN(100_000 * LAMPORTS_PER_SOL);

    const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);

    const signerAssetTokenAccount = getAssociatedTokenAddressSync(
      publicAssets[1].mint,
      provider.publicKey,
    );

    await mintTokens(
      publicAssets[1].mint,
      provider,
      100_000 * LAMPORTS_PER_SOL,
      provider.publicKey,
    );

    const liquidityPoolDataPre = await LiquidityPool.fromAccountAddress(
      provider.connection,
      liquidityPool,
    );

    await program.methods
      .depositRewards({
        amount,
      })
      .accounts({
        signer: provider.publicKey,
        permissions: admin,
        settings,
        systemProgram: SystemProgram.programId,
        liquidityPool,
        tokenProgram: TOKEN_PROGRAM_ID,
        assetPool: getAssociatedTokenAddressSync(
          publicAssets[1].mint,
          liquidityPool,
          true,
        ),
        asset: Restaking.deriveAsset(publicAssets[1].mint),
        assetMint: publicAssets[1].mint,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        signerAssetTokenAccount,
      })
      .rpc()
      .catch((err) => {
        console.log(err.logs);
        throw err;
      });

    const liquidityPoolDataPost = await LiquidityPool.fromAccountAddress(
      provider.connection,
      liquidityPool,
    );

    // Verify rewards were deposited
    const assetPoolData = await getAccount(
      provider.connection,
      getAssociatedTokenAddressSync(publicAssets[1].mint, liquidityPool, true),
    );

    expect(parseInt(assetPoolData.amount.toString())).eq(amount.toNumber());
  });

  it("Slashes liquidity pool", async () => {
    const liquidityPoolId = 0;
    const slashAmount = new BN(LAMPORTS_PER_SOL * 200);

    const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);

    const liquidityPoolData = await LiquidityPool.fromAccountAddress(
      provider.connection,
      liquidityPool,
    );

    const asset = Restaking.deriveAsset(whitelistedAssets[0].mint);

    const destination = Keypair.generate();
    const destinationAta = getAssociatedTokenAddressSync(
      whitelistedAssets[0].mint,
      destination.publicKey,
    );

    const destinationAtaIx = createAssociatedTokenAccountInstruction(
      provider.publicKey,
      destinationAta,
      destination.publicKey,
      whitelistedAssets[0].mint,
    );

    const poolAssetAccount = getAssociatedTokenAddressSync(
      whitelistedAssets[0].mint,
      liquidityPool,
      true,
    );

    const poolAssetAccountPre = await getAccount(
      provider.connection,
      poolAssetAccount,
    );

    await program.methods
      .slash({
        liquidityPoolId,
        amount: slashAmount,
      })
      .accounts({
        signer: provider.publicKey,
        permissions: admin,
        settings,
        liquidityPool,
        tokenProgram: TOKEN_PROGRAM_ID,
        asset,
        mint: whitelistedAssets[0].mint,
        liquidityPoolTokenAccount: poolAssetAccount,
        destination: destinationAta,
      })
      .preInstructions([destinationAtaIx])
      .rpc();

    const poolAssetAccountPost = await getAccount(
      provider.connection,
      poolAssetAccount,
    );

    expect(parseInt(poolAssetAccountPost.amount.toString())).eq(
      parseInt(poolAssetAccountPre.amount.toString()) - slashAmount.toNumber(),
    );
  });

  let alice = Keypair.generate();
  it("Makes second deposit.", async () => {
    const liquidityPoolId = 0;
    const amount = new BN(LAMPORTS_PER_SOL * 1_000);

    const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);

    const liquidityPoolData = await LiquidityPool.fromAccountAddress(
      provider.connection,
      liquidityPool,
    );

    const asset = Restaking.deriveAsset(whitelistedAssets[0].mint);

    await provider.connection.requestAirdrop(
      alice.publicKey,
      5 * LAMPORTS_PER_SOL,
    );

    await mintTokens(
      whitelistedAssets[0].mint,
      provider,
      amount.toNumber(),
      alice.publicKey,
    );

    const userAssetAta = getAssociatedTokenAddressSync(
      whitelistedAssets[0].mint,
      alice.publicKey,
    );

    const poolAssetAccount = getAssociatedTokenAddressSync(
      whitelistedAssets[0].mint,
      liquidityPool,
      true,
    );

    const userLpAccount = getAssociatedTokenAddressSync(
      liquidityPoolData.lpToken,
      alice.publicKey,
    );

    const assetDataPre = await Asset.fromAccountAddress(
      provider.connection,
      asset,
    );

    const transactionTimestamp = Math.floor(Date.now() / 1000);

    const poolAssetAccountPre = await getAccount(
      provider.connection,
      poolAssetAccount,
    );

    const totalDepositsPre = new BN(poolAssetAccountPre.amount.toString());
    const lpToken = await getMint(
      provider.connection,
      liquidityPoolData.lpToken,
    );
    const totalLpSupply = new BN(lpToken.supply.toString());

    const restaking = new Restaking(provider.connection);
    const assets = await restaking.getAssets();

    const totalPoolValue = await calculateTotalPoolValue(
      await Promise.all(
        assets.map(async (asset) => ({
          asset: {
            mint: asset.account.mint.toString(),
            oracle: asset.account.oracle.fields[0].toString(),
            tokenBalance: new BN(
              (
                await getAccount(
                  provider.connection,
                  getAssociatedTokenAddressSync(
                    asset.account.mint,
                    liquidityPool,
                    true,
                  ),
                )
              ).amount.toString(),
            ),
          },
        })),
      ),
    );

    const oraclePrice = await getOraclePriceFromAccount(
      assetDataPre.oracle.fields[0].toString(),
    );
    const depositValue = calculateDepositValue(amount, oraclePrice);

    const expectedLpTokens = calculateLpTokensOnDeposit(
      totalLpSupply,
      totalPoolValue,
      depositValue,
    );

    const anchorRemainingAccounts: AccountMeta[] = assets
      .map(({ account, pubkey }) => {
        return [
          {
            isSigner: false,
            isWritable: true,
            pubkey: getAssociatedTokenAddressSync(
              account.mint,
              liquidityPool,
              true,
            ),
          },
          {
            isSigner: false,
            isWritable: false,
            pubkey: Restaking.deriveAsset(account.mint),
          },
          {
            isSigner: false,
            isWritable: false,
            pubkey: account.oracle.fields[0],
          },
        ] as AccountMeta[];
      })
      .flat();

    await program.methods
      .restake({
        liquidityPoolIndex: liquidityPoolId,
        amount,
        minLpTokens: new BN(0),
      })
      .accounts({
        signer: alice.publicKey,
        settings,
        liquidityPool,
        lpToken: liquidityPoolData.lpToken,
        userLpAccount,
        asset,
        assetMint: whitelistedAssets[0].mint,
        userAssetAccount: userAssetAta,
        poolAssetAccount,
        oracle: assetDataPre.oracle.fields[0],
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        permissions: null,
      })
      .remainingAccounts(anchorRemainingAccounts)
      .signers([alice])
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 1_000_000 }),
      ])
      .rpc()
      .catch((err) => {
        console.log(err);
        throw err;
      });

    await new Promise((resolve) => setTimeout(resolve, 15000));

    const userLpAccountData = await getAccount(
      provider.connection,
      userLpAccount,
    );

    expect(parseInt(userLpAccountData.amount.toString())).approximately(
      expectedLpTokens.toNumber(),
      0.1 * LAMPORTS_PER_SOL, // 0.1 full LP token delta
    );
  });

  it("Sets the withdraw action to public.", async () => {
    await program.methods
      // @ts-ignore
      .updateActionRole({
        action: { withdraw: {} },
        role: { public: {} },
        update: { add: {} },
      })
      .accounts({
        admin: provider.publicKey,
        settings,
        adminPermissions: admin,
      })
      .rpc();

    const settingsData = await Settings.fromAccountAddress(
      provider.connection,
      settings,
    );

    const actionPermissionMap =
      settingsData.accessControl.accessMap.actionPermissions.find(
        (mapping) => mapping.action === Action.Withdraw,
      );
    expect(actionPermissionMap.action).eq(Action.Withdraw);
    expect(actionPermissionMap.allowedRoles).include(Role.PUBLIC);
  });

  it("Requests withdrawal for the first user.", async () => {
    const liquidityPoolId = 0;
    const amount = new BN(5 * LAMPORTS_PER_SOL);

    const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);

    const liquidityPoolData = await LiquidityPool.fromAccountAddress(
      provider.connection,
      liquidityPool,
    );

    const userLpAccount = getAssociatedTokenAddressSync(
      liquidityPoolData.lpToken,
      user.publicKey,
    );

    const userLpAccountPre = await getAccount(
      provider.connection,
      userLpAccount,
    );

    const [cooldown] = PublicKey.findProgramAddressSync(
      [Buffer.from("cooldown"), new BN(0).toArrayLike(Buffer, "le", 8)],
      PROGRAM_ID,
    );

    const cooldownLpTokenAccount = getAssociatedTokenAddressSync(
      liquidityPoolData.lpToken,
      cooldown,
      true,
    );

    const timestampPre = Math.floor(Date.now() / 1000);

    await program.methods
      .requestWithdrawal({
        liquidityPoolId,
        amount,
      })
      .accounts({
        signer: user.publicKey,
        settings,
        liquidityPool,
        lpTokenMint: liquidityPoolData.lpToken,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
        signerLpTokenAccount: userLpAccount,
        cooldown,
        cooldownLpTokenAccount,
        permissions: null,
      })
      .signers([user])
      .rpc();

    const cooldownDataPost = await Cooldown.fromAccountAddress(
      provider.connection,
      cooldown,
    );

    expect(cooldownDataPost.authority.toString()).eq(user.publicKey.toString());
    expect(cooldownDataPost.liquidityPoolId.toString()).eq(
      liquidityPoolId.toString(),
    );
    expect(parseInt(cooldownDataPost.unlockTs.toString())).approximately(
      parseInt(liquidityPoolData.cooldownDuration.toString()) + timestampPre,
      2, // 2 second delta
    );
  });

  // let lookupTablePubkey: PublicKey;
  // it("Creates lookup table with asset datas", async () => {
  //     let done = false;
  //     while (!done) {
  //         try {
  //             const restaking = new Restaking(provider.connection);
  //             const assets = await restaking.getAssets();

  //             const [ix, pubkey] = AddressLookupTableProgram
  //                 .createLookupTable({
  //                     authority: provider.publicKey,
  //                     payer: provider.publicKey,
  //                     recentSlot: await provider.connection.getSlot()
  //                 });

  //             const tx = new Transaction();
  //             tx.add(ix);

  //             const { blockhash, lastValidBlockHeight } = await provider.connection.getLatestBlockhash();
  //             tx.recentBlockhash = blockhash;
  //             tx.feePayer = provider.publicKey;

  //             const signedTx = await provider.wallet.signTransaction(tx);
  //             const signature = await provider.connection.sendRawTransaction(signedTx.serialize());
  //             await provider.connection.confirmTransaction({
  //                 blockhash,
  //                 lastValidBlockHeight,
  //                 signature
  //             });

  //             const ix2 = AddressLookupTableProgram
  //                 .extendLookupTable({
  //                     addresses: assets.map(asset => [asset.pubkey, asset.account.mint, asset.account.oracle.fields[0]]).flat(),
  //                     authority: provider.publicKey,
  //                     payer: provider.publicKey,
  //                     lookupTable: pubkey
  //                 });

  //             const tx2 = new Transaction();
  //             tx2.add(ix2);

  //             const blockData2 = await provider.connection.getLatestBlockhash();
  //             tx2.recentBlockhash = blockData2.blockhash;
  //             tx2.feePayer = provider.publicKey;

  //             await sleep(10);

  //             const signedTx2 = await provider.wallet.signTransaction(tx2);
  //             const signature2 = await provider.connection.sendRawTransaction(signedTx2.serialize());
  //             await provider.connection.confirmTransaction({
  //                 blockhash: blockData2.blockhash,
  //                 lastValidBlockHeight: blockData2.lastValidBlockHeight,
  //                 signature: signature2
  //             });

  //             await sleep(10);
  //             lookupTablePubkey = pubkey;
  //             done = true;
  //         } catch (err) {
  //             console.log("failed to create lookup table. retrying...");
  //             await sleep(3);
  //         }
  //     }
  // });

  it("Waits cooldown duration & withdraws.", async () => {
    const liquidityPoolId = 0;
    const cooldownId = 0;

    const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);

    const liquidityPoolData = await LiquidityPool.fromAccountAddress(
      provider.connection,
      liquidityPool,
    );

    const [cooldown] = PublicKey.findProgramAddressSync(
      [
        Buffer.from("cooldown"),
        new BN(cooldownId).toArrayLike(Buffer, "le", 8),
      ],
      PROGRAM_ID,
    );

    const cooldownData = await Cooldown.fromAccountAddress(
      provider.connection,
      cooldown,
    );

    await sleep(
      new BN(liquidityPoolData.cooldownDuration.toString()).toNumber(),
    );

    const asset = Restaking.deriveAsset(whitelistedAssets[0].mint);

    const userAssetAta = getAssociatedTokenAddressSync(
      whitelistedAssets[0].mint,
      user.publicKey,
    );

    const userRewardAta = getAssociatedTokenAddressSync(
      publicAssets[0].mint,
      user.publicKey,
    );

    const userAssetAtaPre = await getAccount(provider.connection, userAssetAta);

    const poolAssetAccount = getAssociatedTokenAddressSync(
      whitelistedAssets[0].mint,
      liquidityPool,
      true,
    );

    const cooldownLpTokenAccount = getAssociatedTokenAddressSync(
      liquidityPoolData.lpToken,
      cooldown,
      true,
    );

    const restaking = new Restaking(provider.connection);
    const assets = await restaking.getAssets();

    const anchorRemainingAccounts: AccountMeta[] = assets
      .map(({ account, pubkey }) => {
        return [
          {
            isSigner: false,
            isWritable: true,
            pubkey: getAssociatedTokenAddressSync(
              account.mint,
              liquidityPool,
              true,
            ),
          },
          {
            isSigner: false,
            isWritable: false,
            pubkey: Restaking.deriveAsset(account.mint),
          },
          {
            isSigner: false,
            isWritable: true,
            pubkey: getAssociatedTokenAddressSync(
              account.mint,
              user.publicKey,
              true,
            ),
          },
        ] as AccountMeta[];
      })
      .flat();

    const { blockhash, lastValidBlockHeight } =
      await provider.connection.getLatestBlockhash();
    const msg = new TransactionMessage({
      instructions: [
        ...assets.map((asset) =>
          createAssociatedTokenAccountIdempotentInstruction(
            user.publicKey,
            getAssociatedTokenAddressSync(
              asset.account.mint,
              user.publicKey,
              true,
            ),
            user.publicKey,
            asset.account.mint,
          ),
        ),
      ],
      payerKey: user.publicKey,
      recentBlockhash: blockhash,
    }).compileToV0Message();

    const tx = new VersionedTransaction(msg);
    tx.sign([user]);
    const signature = await provider.connection.sendRawTransaction(
      tx.serialize(),
    );
    await provider.connection.confirmTransaction({
      blockhash,
      lastValidBlockHeight,
      signature,
    });

    await sleep(10);

    await program.methods
      .withdraw({
        liquidityPoolId,
        cooldownId: new BN(cooldownId),
      })
      .accounts({
        settings,
        systemProgram: SystemProgram.programId,
        liquidityPool,
        lpTokenMint: liquidityPoolData.lpToken,
        tokenProgram: TOKEN_PROGRAM_ID,
        cooldown,
        cooldownLpTokenAccount,
        signer: user.publicKey,
        permissions: null,
      })
      .signers([user])
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 1_000_000 }),
      ])
      .remainingAccounts(anchorRemainingAccounts)
      .rpc();

    const userRewardAtaPost = await getAccount(
      provider.connection,
      userRewardAta,
    );

    expect(parseInt(userRewardAtaPost.amount.toString())).gt(0);
  });

  it("Performs a swap between two private assets by an admin in a liquidity pool", async () => {
    const amountIn = new BN(1000 * LAMPORTS_PER_SOL);

    const liquidityPoolId = 0;
    const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);

    const restaking = new Restaking(provider.connection);
    const assets = await restaking.getAssets();

    const fromMint = whitelistedAssets[0];
    const toMint = whitelistedAssets[1];

    const fromAsset = Restaking.deriveAsset(fromMint.mint);
    const toAsset = Restaking.deriveAsset(toMint.mint);

    const { oracle: fromOracle } = await Asset.fromAccountAddress(
      provider.connection,
      fromAsset,
    );

    const { oracle: toOracle } = await Asset.fromAccountAddress(
      provider.connection,
      toAsset,
    );

    const fromSignerTokenAccount = getAssociatedTokenAddressSync(
      fromMint.mint,
      provider.publicKey,
      true,
    );

    const toSignerTokenAccount = getAssociatedTokenAddressSync(
      toMint.mint,
      provider.publicKey,
      true,
    );

    await mintTokens(
      whitelistedAssets[0].mint,
      provider,
      10000 * LAMPORTS_PER_SOL,
      provider.publicKey,
    );

    await mintTokens(
      whitelistedAssets[1].mint,
      provider,
      100000 * LAMPORTS_PER_SOL,
      provider.publicKey,
    );

    const admin = Restaking.deriveUserPermissions(provider.publicKey);

    const { price: fromTokenPrice, precision: fromTokenPrecision } =
      await getOraclePriceFromAccount(fromOracle.fields[0].toString());

    const { price: toTokenPrice, precision: toTokenPrecision } =
      await getOraclePriceFromAccount(toOracle.fields[0].toString());

    const { amount: fromSignerTokenAccountBalancePre } = await getAccount(
      provider.connection,
      fromSignerTokenAccount,
    );
    const { amount: toSignerTokenAccountBalancePre } = await getAccount(
      provider.connection,
      toSignerTokenAccount,
    );

    await program.methods
      .swap({
        amountIn,
        minOut: new BN(0),
      })
      .preInstructions([
        ...assets.map((asset) =>
          createAssociatedTokenAccountIdempotentInstruction(
            provider.publicKey,
            getAssociatedTokenAddressSync(
              asset.account.mint,
              provider.publicKey,
              true,
            ),
            provider.publicKey,
            asset.account.mint,
          ),
        ),
      ])
      .accounts({
        admin,
        settings,
        signer: provider.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        liquidityPool,
        tokenFrom: fromMint.mint,
        tokenFromAsset: fromAsset,
        tokenFromOracle: fromOracle.fields[0],
        tokenTo: toMint.mint,
        tokenToAsset: toAsset,
        tokenToOracle: toOracle.fields[0],
        tokenFromPool: getAssociatedTokenAddressSync(
          fromMint.mint,
          liquidityPool,
          true,
        ),
        tokenToPool: getAssociatedTokenAddressSync(
          toMint.mint,
          liquidityPool,
          true,
        ),
        tokenFromSignerAccount: fromSignerTokenAccount,
        tokenToSignerAccount: toSignerTokenAccount,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
      })
      .rpc()
      .catch((err) => {
        console.error(err);
        throw err;
      });

    const { amount: fromSignerTokenAccountBalancePost } = await getAccount(
      provider.connection,
      fromSignerTokenAccount,
    );

    const { amount: toSignerTokenAccountBalancePost } = await getAccount(
      provider.connection,
      toSignerTokenAccount,
    );

    const fromUsdValue = amountIn
      .mul(fromTokenPrice)
      .div(new BN(10).pow(fromTokenPrecision));

    const toAmount = fromUsdValue
      .mul(new BN(10).pow(toTokenPrecision))
      .div(toTokenPrice);

    expect(toAmount.toNumber()).approximately(
      new BN(toSignerTokenAccountBalancePost.toString())
        .sub(new BN(toSignerTokenAccountBalancePre.toString()))
        .abs()
        .toNumber(),
      0.25 * LAMPORTS_PER_SOL,
    );

    expect(amountIn.toNumber()).eq(
      new BN(fromSignerTokenAccountBalancePre.toString())
        .sub(new BN(fromSignerTokenAccountBalancePost.toString()))
        .abs()
        .toNumber(),
    );
  });

  const randomUser = Keypair.generate();
  it("Performs a swap between two public assets by a random user in a liquidity pool", async () => {
    const amountIn = new BN(1 * LAMPORTS_PER_SOL);

    await provider.connection.requestAirdrop(
      randomUser.publicKey,
      10000 * LAMPORTS_PER_SOL,
    );

    await mintTokens(
      publicAssets[0].mint,
      provider,
      10000 * LAMPORTS_PER_SOL,
      randomUser.publicKey,
    );

    const liquidityPoolId = 0;
    const liquidityPool = Restaking.deriveLiquidityPool(liquidityPoolId);

    const restaking = new Restaking(provider.connection);
    const assets = await restaking.getAssets();

    const fromMint = publicAssets[0];
    const toMint = publicAssets[1];

    const fromAsset = Restaking.deriveAsset(fromMint.mint);
    const toAsset = Restaking.deriveAsset(toMint.mint);

    const { oracle: fromOracle, accessLevel: fromAccessLevel } =
      await Asset.fromAccountAddress(provider.connection, fromAsset);

    const { oracle: toOracle, accessLevel: toAccessLevel } =
      await Asset.fromAccountAddress(provider.connection, toAsset);

    console.log({
      toAccessLevel,
      fromAccessLevel,
    });

    const fromSignerTokenAccount = getAssociatedTokenAddressSync(
      fromMint.mint,
      randomUser.publicKey,
      true,
    );

    const toSignerTokenAccount = getAssociatedTokenAddressSync(
      toMint.mint,
      randomUser.publicKey,
      true,
    );

    await mintTokens(
      publicAssets[0].mint,
      provider,
      10000 * LAMPORTS_PER_SOL,
      randomUser.publicKey,
    );

    await mintTokens(
      publicAssets[1].mint,
      provider,
      100000 * LAMPORTS_PER_SOL,
      randomUser.publicKey,
    );

    const { price: fromTokenPrice, precision: fromTokenPrecision } =
      await getOraclePriceFromAccount(fromOracle.fields[0].toString());

    const { price: toTokenPrice, precision: toTokenPrecision } =
      await getOraclePriceFromAccount(toOracle.fields[0].toString());

    const { amount: fromSignerTokenAccountBalancePre } = await getAccount(
      provider.connection,
      fromSignerTokenAccount,
    );
    const { amount: toSignerTokenAccountBalancePre } = await getAccount(
      provider.connection,
      toSignerTokenAccount,
    );

    await program.methods
      .swap({
        amountIn,
        minOut: new BN(0),
      })
      .preInstructions([
        ...assets.map((asset) =>
          createAssociatedTokenAccountIdempotentInstruction(
            randomUser.publicKey,
            getAssociatedTokenAddressSync(
              asset.account.mint,
              randomUser.publicKey,
              true,
            ),
            randomUser.publicKey,
            asset.account.mint,
          ),
        ),
      ])
      .accounts({
        settings,
        signer: randomUser.publicKey,
        admin: null,
        tokenProgram: TOKEN_PROGRAM_ID,
        liquidityPool,
        tokenFrom: fromMint.mint,
        tokenFromAsset: fromAsset,
        tokenFromOracle: fromOracle.fields[0],
        tokenTo: toMint.mint,
        tokenToAsset: toAsset,
        tokenToOracle: toOracle.fields[0],
        tokenFromPool: getAssociatedTokenAddressSync(
          fromMint.mint,
          liquidityPool,
          true,
        ),
        tokenToPool: getAssociatedTokenAddressSync(
          toMint.mint,
          liquidityPool,
          true,
        ),
        tokenFromSignerAccount: fromSignerTokenAccount,
        tokenToSignerAccount: toSignerTokenAccount,
        associatedTokenProgram: ASSOCIATED_PROGRAM_ID,
      })
      .signers([randomUser])
      .rpc()
      .catch((err) => {
        console.error((err.logs as string[]).join("\n"));
        throw err;
      });

    const { amount: fromSignerTokenAccountBalancePost } = await getAccount(
      provider.connection,
      fromSignerTokenAccount,
    );

    const { amount: toSignerTokenAccountBalancePost } = await getAccount(
      provider.connection,
      toSignerTokenAccount,
    );

    const fromUsdValue = amountIn
      .mul(fromTokenPrice)
      .div(new BN(10).pow(fromTokenPrecision));

    const toAmount = fromUsdValue
      .mul(new BN(10).pow(toTokenPrecision))
      .div(toTokenPrice);

    expect(toAmount.toNumber()).approximately(
      new BN(toSignerTokenAccountBalancePost.toString())
        .sub(new BN(toSignerTokenAccountBalancePre.toString()))
        .abs()
        .toNumber(),
      0.25 * LAMPORTS_PER_SOL,
    );

    expect(amountIn.toNumber()).eq(
      new BN(fromSignerTokenAccountBalancePre.toString())
        .sub(new BN(fromSignerTokenAccountBalancePost.toString()))
        .abs()
        .toNumber(),
    );
  });

  // Admin functionality tests
  it("Creates a new permission account", async () => {
    const newAdmin = Keypair.generate();
    const newAdminPermissions = Restaking.deriveUserPermissions(
      newAdmin.publicKey,
    );

    await program.methods
      .createPermissionAccount(newAdmin.publicKey)
      .accounts({
        settings,
        newCreds: newAdminPermissions,
        caller: provider.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const newAdminPermissionsData = await UserPermissions.fromAccountAddress(
      provider.connection,
      newAdminPermissions,
    );

    expect(newAdminPermissionsData.authority.toString()).eq(
      newAdmin.publicKey.toString(),
    );
    expect(newAdminPermissionsData.protocolRoles.roles.length).eq(0);
  });

  it("Updates action role - adds role to action", async () => {
    await program.methods
      // @ts-ignore
      .updateActionRole({
        action: { suspendDeposits: {} },
        role: { testee: {} },
        update: { add: {} },
      })
      .accounts({
        admin: provider.publicKey,
        adminPermissions: admin,
        settings,
      })
      .rpc();

    const settingsData = await Settings.fromAccountAddress(
      provider.connection,
      settings,
    );

    const actionPermissionMapping =
      settingsData.accessControl.accessMap.actionPermissions.find(
        (mapping) => mapping.action === Action.SuspendDeposits,
      );
    expect(actionPermissionMapping.allowedRoles.includes(Role.TESTEE)).eq(true);
  });

  it("Updates action role - removes role from action", async () => {
    await program.methods
      // @ts-ignore
      .updateActionRole({
        action: { suspendDeposits: {} },
        role: { testee: {} },
        update: { remove: {} },
      })
      .accounts({
        admin: provider.publicKey,
        adminPermissions: admin,
        settings,
      })
      .rpc();

    const settingsData = await Settings.fromAccountAddress(
      provider.connection,
      settings,
    );

    const actionPermissionMapping =
      settingsData.accessControl.accessMap.actionPermissions.find(
        (mapping) => mapping.action === Action.SuspendDeposits,
      );
    expect(actionPermissionMapping.allowedRoles.includes(Role.TESTEE)).eq(
      false,
    );
  });

  it("Updates role holder - adds role to user", async () => {
    const targetUser = Keypair.generate();
    const targetUserPermissions = Restaking.deriveUserPermissions(
      targetUser.publicKey,
    );

    // First create the permission account for the target user
    await program.methods
      .createPermissionAccount(targetUser.publicKey)
      .accounts({
        settings,
        newCreds: targetUserPermissions,
        caller: provider.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    await program.methods
      // @ts-ignore
      .updateRoleHolder({
        address: targetUser.publicKey,
        role: { crank: {} },
        update: { add: {} },
      })
      .accounts({
        admin: provider.publicKey,
        settings,
        adminPermissions: admin,
        updateAdminPermissions: targetUserPermissions,
        strategy: provider.publicKey, // Placeholder
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const targetUserPermissionsData = await UserPermissions.fromAccountAddress(
      provider.connection,
      targetUserPermissions,
    );

    // Verify the role was added
    expect(targetUserPermissionsData.authority.toString()).eq(
      targetUser.publicKey.toString(),
    );
  });

  it("Updates role holder - removes role from user", async () => {
    const targetUser = Keypair.generate();
    const targetUserPermissions = Restaking.deriveUserPermissions(
      targetUser.publicKey,
    );

    // First create the permission account for the target user
    await program.methods
      .createPermissionAccount(targetUser.publicKey)
      .accounts({
        settings,
        newCreds: targetUserPermissions,
        caller: provider.publicKey,
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    // Add a role first
    await program.methods
      // @ts-ignore
      .updateRoleHolder({
        address: targetUser.publicKey,
        role: { freeze: {} },
        update: { add: {} },
      })
      .accounts({
        admin: provider.publicKey,
        settings,
        adminPermissions: admin,
        updateAdminPermissions: targetUserPermissions,
        strategy: provider.publicKey, // Placeholder
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    // Then remove it
    await program.methods
      // @ts-ignore
      .updateRoleHolder({
        address: targetUser.publicKey,
        role: { freeze: {} },
        update: { remove: {} },
      })
      .accounts({
        admin: provider.publicKey,
        settings,
        adminPermissions: admin,
        updateAdminPermissions: targetUserPermissions,
        strategy: provider.publicKey, // Placeholder
        systemProgram: SystemProgram.programId,
      })
      .rpc();

    const targetUserPermissionsData = await UserPermissions.fromAccountAddress(
      provider.connection,
      targetUserPermissions,
    );

    // Verify the role was removed
    expect(targetUserPermissionsData.authority.toString()).eq(
      targetUser.publicKey.toString(),
    );
  });
});
