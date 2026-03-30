import { findAssociatedTokenPda, TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";
import {
  AccountRole,
  Rpc,
  SolanaRpcApi,
  Address,
  TransactionSigner,
  fetchEncodedAccount,
} from "@solana/kit";
import {
  type Settings,
  type LiquidityPool,
  type Asset,
  type Cooldown,
  type UserPermissions,
  type AccessLevel,
  RLP_PROGRAM_ADDRESS,
  LIQUIDITY_POOL_DISCRIMINATOR,
  USER_PERMISSIONS_DISCRIMINATOR,
  fetchSettings,
  fetchLiquidityPool,
  getLiquidityPoolDecoder,
  getAssetEncoder,
  getAssetDecoder,
  getCooldownEncoder,
  getCooldownDecoder,
  getUserPermissionsDecoder,
  getInitializeRlpInstructionAsync,
  getInitializeLpInstructionAsync,
  getAddAssetInstructionAsync,
  getSlashInstructionAsync,
  getRestakeInstructionAsync,
  getRequestWithdrawalInstructionAsync,
  getWithdrawInstructionAsync,
} from "../generated";
import { PdaClient } from "./PdaClient";

export type AccountWithAddress<T> = {
  data: T;
  address: Address;
};

export class Insurance {
  private connection: Rpc<SolanaRpcApi>;
  private settings!: Settings;
  private liquidityPools!: AccountWithAddress<LiquidityPool>[];
  private assets!: AccountWithAddress<Asset>[];

  constructor(connection: Rpc<SolanaRpcApi>) {
    this.connection = connection;
  }

  async load(): Promise<void> {
    this.settings = await this.getSettingsData();
    this.liquidityPools = await this.getLiquidityPools();
    this.assets = await this.getAssets();
  }

  async getSettingsData(): Promise<Settings> {
    const [settingsAddress] = await PdaClient.deriveSettings();
    const account = await fetchSettings(this.connection, settingsAddress);
    return account.data;
  }

  async getLiquidityPools(): Promise<AccountWithAddress<LiquidityPool>[]> {
    const decoder = getLiquidityPoolDecoder();

    const programAccounts = await (this.connection as any)
      .getProgramAccounts(RLP_PROGRAM_ADDRESS, {
        encoding: "base64",
        withContext: false,
        filters: [
          {
            memcmp: {
              encoding: "base64",
              offset: BigInt(0),
              bytes: Buffer.from(LIQUIDITY_POOL_DISCRIMINATOR).toString(
                "base64",
              ),
            },
          },
        ],
      })
      .send();

    return programAccounts.map((account: any) => {
      const [b64] = account.account.data;
      const bytes = new Uint8Array(Buffer.from(b64, "base64"));
      return {
        address: account.pubkey,
        data: decoder.decode(bytes),
      };
    });
  }

  async getLiquidityPoolData(liquidityPoolId: number): Promise<LiquidityPool> {
    const [liquidityPoolAddress] =
      await PdaClient.deriveLiquidityPool(liquidityPoolId);
    const account = await fetchLiquidityPool(
      this.connection,
      liquidityPoolAddress,
    );
    return account.data;
  }

  async getAssets(): Promise<AccountWithAddress<Asset>[]> {
    if (this.assets?.length > 0) {
      return this.assets;
    }

    const encoder = getAssetEncoder();
    const decoder = getAssetDecoder();

    const programAccounts = await (this.connection as any)
      .getProgramAccounts(RLP_PROGRAM_ADDRESS, {
        encoding: "base64",
        withContext: false,
        filters: [
          { dataSize: BigInt((encoder as any).fixedSize) },
        ],
      })
      .send();

    const result = programAccounts
      .map((account: any) => {
        const [b64] = account.account.data;
        const bytes = new Uint8Array(Buffer.from(b64, "base64"));
        return {
          address: account.pubkey,
          data: decoder.decode(bytes),
        };
      })
      .sort((a: any, b: any) => a.data.index - b.data.index);

    this.assets = result;
    return result;
  }

  async getCooldowns(): Promise<AccountWithAddress<Cooldown>[]> {
    const encoder = getCooldownEncoder();
    const decoder = getCooldownDecoder();

    const programAccounts = await (this.connection as any)
      .getProgramAccounts(RLP_PROGRAM_ADDRESS, {
        encoding: "base64",
        withContext: false,
        filters: [
          { dataSize: BigInt((encoder as any).fixedSize) },
        ],
      })
      .send();

    return programAccounts.map((account: any) => {
      const [b64] = account.account.data;
      const bytes = new Uint8Array(Buffer.from(b64, "base64"));
      return {
        address: account.pubkey,
        data: decoder.decode(bytes),
      };
    });
  }

  async getCooldownsByUser(
    user: Address,
  ): Promise<AccountWithAddress<Cooldown>[]> {
    const allCooldowns = await this.getCooldowns();
    return allCooldowns.filter((c) => c.data.authority === user);
  }

  async getUserPermissions(): Promise<AccountWithAddress<UserPermissions>[]> {
    const decoder = getUserPermissionsDecoder();

    const programAccounts = await (this.connection as any)
      .getProgramAccounts(RLP_PROGRAM_ADDRESS, {
        encoding: "base64",
        withContext: false,
        filters: [
          {
            memcmp: {
              encoding: "base64",
              offset: BigInt(0),
              bytes: Buffer.from(USER_PERMISSIONS_DISCRIMINATOR).toString(
                "base64",
              ),
            },
          },
        ],
      })
      .send();

    return programAccounts.map((account: any) => {
      const [b64] = account.account.data;
      const bytes = new Uint8Array(Buffer.from(b64, "base64"));
      return {
        address: account.pubkey,
        data: decoder.decode(bytes),
      };
    });
  }

  async getUserPermissionsFromAddress(
    address: Address,
  ): Promise<AccountWithAddress<UserPermissions>[]> {
    const allPermissions = await this.getUserPermissions();
    return allPermissions.filter((p) => p.data.authority === address);
  }

  async initializeRlp(signer: TransactionSigner, swapFeeBps: number) {
    return getInitializeRlpInstructionAsync({
      signer,
      swapFeeBps,
    });
  }

  async initializeLiquidityPool(
    signer: TransactionSigner,
    args: {
      lpTokenMint: Address;
      cooldownDuration: number | bigint;
      depositCap: number | bigint | null;
      assets: number[];
    },
  ) {
    const settings = await this.getSettingsData();
    const [liquidityPoolAddress] = await PdaClient.deriveLiquidityPool(
      settings.liquidityPools,
    );

    return getInitializeLpInstructionAsync({
      signer,
      liquidityPool: liquidityPoolAddress,
      lpTokenMint: args.lpTokenMint,
      cooldownDuration: args.cooldownDuration,
      depositCap: args.depositCap,
      assets: new Uint8Array(args.assets),
    });
  }

  async addAsset(
    signer: TransactionSigner,
    assetMint: Address,
    oracle: Address,
    accessLevel: AccessLevel,
  ) {
    return getAddAssetInstructionAsync({
      signer,
      assetMint,
      oracle,
      accessLevel,
    });
  }

  async slash(
    mint: Address,
    amount: number | bigint,
    signer: TransactionSigner,
    liquidityPoolId: number,
    destination: Address,
  ) {
    const [liquidityPoolAddress] =
      await PdaClient.deriveLiquidityPool(liquidityPoolId);

    const assets = await this.getAssets();
    const assetEntry = assets.find((a) => a.data.mint === mint);
    if (!assetEntry) throw new Error(`Asset not found for mint ${mint}`);

    const [liquidityPoolTokenAccount] = await findAssociatedTokenPda({
      mint,
      owner: liquidityPoolAddress,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });

    return getSlashInstructionAsync({
      signer,
      liquidityPool: liquidityPoolAddress,
      mint,
      asset: assetEntry.address,
      liquidityPoolTokenAccount,
      destination,
      liquidityPoolId,
      amount,
      assetId: assetEntry.data.index,
    });
  }

  /**
   * Inject pre-fetched state (useful for testing without an RPC connection).
   */
  loadFromCache(
    settings: Settings,
    liquidityPools: AccountWithAddress<LiquidityPool>[],
    assets: AccountWithAddress<Asset>[],
  ): void {
    this.settings = settings;
    this.liquidityPools = liquidityPools;
    this.assets = assets;
  }

  /**
   * Build the remaining accounts needed by calculate_total_pool_value().
   * Per asset (sorted by index): [pool_ata, asset_pda, oracle, mint]
   */
  private async buildPoolValueRemainingAccounts(
    liquidityPoolAddress: Address,
  ) {
    const assets = await this.getAssets();
    const remaining: { address: Address; role: AccountRole }[] = [];

    for (const asset of assets) {
      const [poolAta] = await findAssociatedTokenPda({
        mint: asset.data.mint,
        owner: liquidityPoolAddress,
        tokenProgram: TOKEN_PROGRAM_ADDRESS,
      });
      const oracleAddress = (asset.data.oracle as any).fields[0] as Address;

      remaining.push(
        { address: poolAta, role: AccountRole.READONLY },
        { address: asset.address, role: AccountRole.READONLY },
        { address: oracleAddress, role: AccountRole.READONLY },
        { address: asset.data.mint, role: AccountRole.READONLY },
      );
    }

    return remaining;
  }

  /**
   * Build the remaining accounts needed by withdraw's load_assets,
   * load_reserves, and load_user_token_accounts.
   *
   * Layout:
   *   - First N: user token ATAs (one per asset in index order)
   *   - Then per asset: asset_pda, pool_reserve_ata
   */
  private async buildWithdrawRemainingAccounts(
    signer: TransactionSigner,
    liquidityPoolAddress: Address,
  ) {
    const assets = await this.getAssets();
    const userAtas: { address: Address; role: AccountRole }[] = [];
    const assetAndReserves: { address: Address; role: AccountRole }[] = [];

    for (const asset of assets) {
      const [userAta] = await findAssociatedTokenPda({
        mint: asset.data.mint,
        owner: signer.address,
        tokenProgram: TOKEN_PROGRAM_ADDRESS,
      });
      userAtas.push({ address: userAta, role: AccountRole.WRITABLE });

      const [poolReserve] = await findAssociatedTokenPda({
        mint: asset.data.mint,
        owner: liquidityPoolAddress,
        tokenProgram: TOKEN_PROGRAM_ADDRESS,
      });
      assetAndReserves.push(
        { address: asset.address, role: AccountRole.READONLY },
        { address: poolReserve, role: AccountRole.WRITABLE },
      );
    }

    return [...userAtas, ...assetAndReserves];
  }

  /**
   * Append remaining account metas to a frozen instruction object,
   * returning a new instruction with the extra accounts.
   */
  private appendRemainingAccounts<T extends { accounts: readonly any[]; data: any; programAddress: any }>(
    ix: T,
    remaining: { address: Address; role: AccountRole }[],
  ): T {
    const extraMetas = remaining.map((r) =>
      Object.freeze({ address: r.address, role: r.role }),
    );
    return Object.freeze({
      ...ix,
      accounts: [...ix.accounts, ...extraMetas],
    }) as T;
  }

  async deposit(
    signer: TransactionSigner,
    amount: number | bigint,
    mint: Address,
    liquidityPoolId: number,
    minLpTokens?: number | bigint | null,
  ) {
    const assets = await this.getAssets();
    const assetEntry = assets.find((a) => a.data.mint === mint);
    if (!assetEntry) throw new Error(`Asset not found for mint ${mint}`);

    const oracleAddress = (assetEntry.data.oracle as any).fields[0];

    const lpEntry = this.liquidityPools.find(
      (lp) => lp.data.index === liquidityPoolId,
    );
    if (!lpEntry)
      throw new Error(`Liquidity pool ${liquidityPoolId} not found`);

    const [userAssetAccount] = await findAssociatedTokenPda({
      mint,
      owner: signer.address,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });

    const ix = await getRestakeInstructionAsync({
      signer,
      liquidityPool: lpEntry.address,
      lpToken: lpEntry.data.lpToken,
      assetMint: mint,
      userAssetAccount,
      oracle: oracleAddress,
      liquidityPoolIndex: liquidityPoolId,
      amount,
      minLpTokens: (minLpTokens ?? null) as any,
    });

    const remaining = await this.buildPoolValueRemainingAccounts(
      lpEntry.address,
    );

    return this.appendRemainingAccounts(ix, remaining);
  }

  async requestWithdrawal(
    signer: TransactionSigner,
    liquidityPoolId: number,
    amount: number | bigint,
  ) {
    const lpEntry = this.liquidityPools.find(
      (lp) => lp.data.index === liquidityPoolId,
    );
    if (!lpEntry)
      throw new Error(`Liquidity pool ${liquidityPoolId} not found`);

    const [cooldownAddress] = await PdaClient.deriveCooldown(
      liquidityPoolId,
      lpEntry.data.cooldowns,
    );

    const [signerLpTokenAccount] = await findAssociatedTokenPda({
      mint: lpEntry.data.lpToken,
      owner: signer.address,
      tokenProgram: TOKEN_PROGRAM_ADDRESS,
    });

    return getRequestWithdrawalInstructionAsync({
      signer,
      liquidityPool: lpEntry.address,
      lpTokenMint: lpEntry.data.lpToken,
      signerLpTokenAccount,
      cooldown: cooldownAddress,
      liquidityPoolId,
      amount,
    });
  }

  async withdraw(
    signer: TransactionSigner,
    liquidityPoolId: number,
    cooldownId: number | bigint,
  ) {
    const lpEntry = this.liquidityPools.find(
      (lp) => lp.data.index === liquidityPoolId,
    );
    if (!lpEntry)
      throw new Error(`Liquidity pool ${liquidityPoolId} not found`);

    const [cooldownAddress] = await PdaClient.deriveCooldown(
      liquidityPoolId,
      cooldownId,
    );

    const ix = await getWithdrawInstructionAsync({
      signer,
      liquidityPool: lpEntry.address,
      lpTokenMint: lpEntry.data.lpToken,
      cooldown: cooldownAddress,
      liquidityPoolId,
      cooldownId,
    });

    const remaining = await this.buildWithdrawRemainingAccounts(
      signer,
      lpEntry.address,
    );

    return this.appendRemainingAccounts(ix, remaining);
  }

  async findAssetFromMint(mint: Address): Promise<Address> {
    const assets = await this.getAssets();
    const entry = assets.find((a) => a.data.mint === mint);
    if (!entry) throw new Error(`Asset not found for mint ${mint}`);
    return entry.address;
  }

  getSettings(): Settings {
    return this.settings;
  }

  getCachedLiquidityPools(): AccountWithAddress<LiquidityPool>[] {
    return this.liquidityPools;
  }

  getCachedAssets(): AccountWithAddress<Asset>[] {
    return this.assets;
  }

  // ========================================================================
  // Oracle
  // ========================================================================

  /**
   * Fetch and deserialize the Pyth PriceUpdateV2 oracle account for a given
   * asset, returning `{ price, exponent }` ready to pass into
   * `simulateDepositMath`.
   *
   * @param mint  The asset mint whose oracle to read, OR a direct oracle
   *              address.  When a mint is provided the oracle address is
   *              looked up from the cached asset list.
   */
  async fetchOraclePrice(
    mintOrOracle: Address,
  ): Promise<{ price: bigint; exponent: number; publishTime: bigint }> {
    // Resolve oracle address — try cached assets first, fall back to treating
    // the input as a direct oracle address.
    let oracleAddress: Address = mintOrOracle;
    const assets = await this.getAssets();
    const assetEntry = assets.find((a) => a.data.mint === mintOrOracle);
    if (assetEntry) {
      oracleAddress = (assetEntry.data.oracle as any).fields[0] as Address;
    }

    const account = await fetchEncodedAccount(this.connection, oracleAddress);
    if (!account.exists) {
      throw new Error(`Oracle account not found: ${oracleAddress}`);
    }

    return Insurance.deserializePythPrice(new Uint8Array(account.data));
  }

  /**
   * Deserialize a raw Pyth PriceUpdateV2 account buffer into price fields.
   *
   * Borsh layout (after 8-byte Anchor discriminator):
   *   write_authority: Pubkey (32)
   *   verification_level: Full=[1] (1 byte) | Partial=[0,u8] (2 bytes)
   *   price_message {
   *     feed_id: [u8;32], price: i64, conf: u64, exponent: i32,
   *     publish_time: i64, ...
   *   }
   */
  static deserializePythPrice(
    data: Uint8Array,
  ): { price: bigint; exponent: number; publishTime: bigint } {
    const dv = new DataView(data.buffer, data.byteOffset, data.byteLength);

    // Skip 8-byte discriminator + 32-byte write_authority = offset 40
    const verificationVariant = data[40];
    // Full = 1 byte, Partial = 2 bytes (variant + num_signatures)
    const verificationSize = verificationVariant === 0 ? 2 : 1;

    const pmOffset = 40 + verificationSize; // start of price_message
    // feed_id: 32 bytes, then price(i64), conf(u64), exponent(i32), publish_time(i64)
    const priceOffset = pmOffset + 32;
    const exponentOffset = priceOffset + 8 + 8; // after price + conf
    const publishTimeOffset = exponentOffset + 4;

    return {
      price: dv.getBigInt64(priceOffset, true),
      exponent: dv.getInt32(exponentOffset, true),
      publishTime: dv.getBigInt64(publishTimeOffset, true),
    };
  }

  // ========================================================================
  // Simulation math
  // ========================================================================

  /**
   * Simulate deposit math off-chain.
   *
   * Mirrors the on-chain logic in:
   *   - `OraclePrice::mul`          (oracle_price.rs)
   *   - `LiquidityPool::calculate_total_pool_value`
   *   - `LiquidityPool::calculate_lp_tokens_on_deposit`
   *
   * @param depositAmount   Raw token amount to deposit (base units).
   * @param depositAssetPrice  Oracle price of the deposit asset (`{ price, exponent }`).
   * @param depositAssetDecimals  Decimals of the deposit token mint.
   * @param poolReserves    Array of { balance, price, exponent, decimals } for every
   *                        asset currently in the pool (BEFORE the deposit).
   * @param lpTokenSupply   Current total supply of the LP token (includes dead shares).
   * @param lpTokenDecimals Decimals of the LP token mint (typically 9).
   * @returns               The number of LP tokens the depositor would receive (bigint).
   */
  static simulateDepositMath(params: {
    depositAmount: bigint;
    depositAssetPrice: { price: bigint; exponent: number };
    depositAssetDecimals: number;
    poolReserves: {
      balance: bigint;
      price: bigint;
      exponent: number;
      decimals: number;
    }[];
    lpTokenSupply: bigint;
    lpTokenDecimals: number;
  }): bigint {
    const {
      depositAmount,
      depositAssetPrice,
      depositAssetDecimals,
      poolReserves,
      lpTokenSupply,
      lpTokenDecimals,
    } = params;

    // --- PreciseNumber constants (mirrors spl-math) ---
    const ONE = 1_000_000_000_000n; // 10^12
    const HALF = ONE / 2n;
    const PRECISION = 18;

    // PreciseNumber::new(x) → x * ONE
    const pNew = (x: bigint) => x * ONE;
    // PreciseNumber::to_imprecise(v) → (v + HALF) / ONE
    const pToU64 = (v: bigint) => (v + HALF) / ONE;
    // PreciseNumber::checked_mul(a, b) → (a * b + HALF) / ONE
    const pMul = (a: bigint, b: bigint) => (a * b + HALF) / ONE;
    // PreciseNumber::checked_div(a, b) → (a * ONE + HALF) / b
    const pDiv = (a: bigint, b: bigint) => (a * ONE + HALF) / b;
    // PreciseNumber::checked_add(a, b) → a + b
    const pAdd = (a: bigint, b: bigint) => a + b;

    // --- OraclePrice::mul(amount, token_decimals) ---
    const oracleMul = (
      price: bigint,
      exponent: number,
      amount: bigint,
      tokenDecimals: number,
    ): bigint => {
      const decimalAdj = PRECISION - tokenDecimals;
      const normalizedAmount = amount * 10n ** BigInt(decimalAdj);
      if (exponent >= 0) {
        return normalizedAmount * price * 10n ** BigInt(exponent);
      } else {
        return (normalizedAmount * price) / 10n ** BigInt(-exponent);
      }
    };

    // --- calculate_total_pool_value ---
    let totalPoolValue = pNew(0n); // precise
    for (const r of poolReserves) {
      if (r.balance > 0n) {
        const rawValue = oracleMul(r.price, r.exponent, r.balance, r.decimals);
        totalPoolValue = pAdd(totalPoolValue, pNew(rawValue));
      }
    }

    // --- deposit_value ---
    const rawDepositValue = oracleMul(
      depositAssetPrice.price,
      depositAssetPrice.exponent,
      depositAmount,
      depositAssetDecimals,
    );
    const depositValue = pNew(rawDepositValue); // precise

    // --- calculate_lp_tokens_on_deposit ---
    const poolValueImprecise = pToU64(totalPoolValue);
    if (lpTokenSupply === 0n || poolValueImprecise === 0n) {
      // Initial deposit formula: deposit_value / 10^(PRECISION - lp_decimals)
      const scaleDown = pNew(10n ** BigInt(PRECISION - lpTokenDecimals));
      return BigInt(pToU64(pDiv(depositValue, scaleDown)));
    } else {
      // Proportional: deposit_value * lp_supply / total_pool_value
      const lpSupplyPrecise = pNew(lpTokenSupply);
      const ratio = pDiv(pMul(depositValue, lpSupplyPrecise), totalPoolValue);
      return BigInt(pToU64(ratio));
    }
  }

  /**
   * Simulate withdrawal math off-chain.
   *
   * Mirrors the on-chain logic in `withdraw()` which computes:
   *   `user_share_per_asset = reserve_amount * lp_token_amount / lp_token_supply`
   *
   * Uses PreciseNumber arithmetic to match on-chain rounding.
   *
   * @param lpTokenAmount   Amount of LP tokens the user is redeeming.
   * @param lpTokenSupply   Current total supply of the LP token.
   * @param reserves        Array of { mint, balance } for each pool asset.
   * @returns               Array of { mint, amount } the user would receive per asset.
   */
  static simulateWithdrawMath(params: {
    lpTokenAmount: bigint;
    lpTokenSupply: bigint;
    reserves: { mint: Address; balance: bigint }[];
  }): { mint: Address; amount: bigint }[] {
    const { lpTokenAmount, lpTokenSupply, reserves } = params;

    const ONE = 1_000_000_000_000n;
    const HALF = ONE / 2n;

    const pNew = (x: bigint) => x * ONE;
    const pToU64 = (v: bigint) => (v + HALF) / ONE;
    const pMul = (a: bigint, b: bigint) => (a * b + HALF) / ONE;
    const pDiv = (a: bigint, b: bigint) => (a * ONE + HALF) / b;

    return reserves.map((r) => {
      // PreciseNumber(reserve) * PreciseNumber(lp_amount) / PreciseNumber(lp_supply)
      const reservePrecise = pNew(r.balance);
      const amountPrecise = pNew(lpTokenAmount);
      const supplyPrecise = pNew(lpTokenSupply);

      const share = pDiv(pMul(reservePrecise, amountPrecise), supplyPrecise);
      return {
        mint: r.mint,
        amount: BigInt(pToU64(share)),
      };
    });
  }
}
