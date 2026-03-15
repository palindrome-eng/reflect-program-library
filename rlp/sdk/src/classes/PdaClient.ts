import {
  Address,
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
}
