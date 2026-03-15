import { findAssociatedTokenPda, TOKEN_PROGRAM_ADDRESS } from "@solana-program/token";
import {
  AccountRole,
  Rpc,
  SolanaRpcApi,
  Address,
  TransactionSigner,
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

export class Restaking {
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

  async restake(
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
}
