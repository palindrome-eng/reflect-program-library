import {
  Address,
  address,
  getAddressEncoder,
  getProgramDerivedAddress,
  getU8Encoder,
  getU64Encoder,
} from "@solana/kit";
import {
  SETTINGS_SEED,
  PERMISSIONS_SEED,
  LIQUIDITY_POOL_SEED,
  ASSET_SEED,
  COOLDOWN_SEED,
  EVENT_AUTHORITY_SEED,
  PROXY_STATE_SEED,
  PROXY_PROGRAM_ADDRESS,
} from "../constants";
import { RLP_PROGRAM_ADDRESS } from "../generated";

export class PdaClient {
  static async deriveSettings() {
    return getProgramDerivedAddress({
      programAddress: RLP_PROGRAM_ADDRESS,
      seeds: [SETTINGS_SEED],
    });
  }

  static async deriveUserPermissions(address: Address) {
    return getProgramDerivedAddress({
      programAddress: RLP_PROGRAM_ADDRESS,
      seeds: [PERMISSIONS_SEED, getAddressEncoder().encode(address)],
    });
  }

  static async deriveLiquidityPool(liquidityPoolId: number) {
    return getProgramDerivedAddress({
      programAddress: RLP_PROGRAM_ADDRESS,
      seeds: [
        LIQUIDITY_POOL_SEED,
        getU8Encoder().encode(liquidityPoolId),
      ],
    });
  }

  static async deriveAsset(assetMint: Address) {
    return getProgramDerivedAddress({
      programAddress: RLP_PROGRAM_ADDRESS,
      seeds: [ASSET_SEED, getAddressEncoder().encode(assetMint)],
    });
  }

  static async deriveCooldown(
    liquidityPoolId: number,
    cooldownId: number | bigint,
  ) {
    return getProgramDerivedAddress({
      programAddress: RLP_PROGRAM_ADDRESS,
      seeds: [
        COOLDOWN_SEED,
        getU8Encoder().encode(liquidityPoolId),
        getU64Encoder().encode(cooldownId),
      ],
    });
  }

  /**
   * Anchor's standard `event_authority` PDA. The codama-generated async
   * builders auto-derive this when omitted, so most callers won't need it
   * directly — exposed for low-level instruction inspection.
   */
  static async deriveEventAuthority() {
    return getProgramDerivedAddress({
      programAddress: RLP_PROGRAM_ADDRESS,
      seeds: [EVENT_AUTHORITY_SEED],
    });
  }

  /**
   * Derive the senior tranche's ProxyState address from its branded mint.
   * Uses the proxy program ID, not the RLP program ID. Used by NAV slash
   * setup (audit-M06).
   */
  static async deriveProxyState(brandedMint: Address) {
    return getProgramDerivedAddress({
      programAddress: address(PROXY_PROGRAM_ADDRESS),
      seeds: [PROXY_STATE_SEED, getAddressEncoder().encode(brandedMint)],
    });
  }
}
