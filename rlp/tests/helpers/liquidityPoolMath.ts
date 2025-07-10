import BN from 'bn.js';
import getOraclePriceFromAccount from './getOraclePriceFromAccount';

export type OraclePrice = {
  price: BN;
  precision: BN;
};

export type AssetData = {
  mint: string;
  oracle: string;
  tokenBalance: BN;
};

export type PoolAsset = {
  asset: AssetData;
};

/**
 * Calculate total pool value from assets
 * Automatically fetches oracle prices for each asset
 */
export async function calculateTotalPoolValue(
  poolAssets: PoolAsset[]
): Promise<BN> {
  let totalPoolValue = new BN(0);
  
  for (const poolAsset of poolAssets) {
    const { asset } = poolAsset;
    
    if (asset.tokenBalance.gt(new BN(0))) {
      // Fetch the oracle price
      const oraclePrice = await getOraclePriceFromAccount(asset.oracle);
      
      // Calculate token value: price * balance / precision
      const tokenValue = oraclePrice.price
        .mul(asset.tokenBalance)
        .div(new BN(10).pow(oraclePrice.precision));
      
      totalPoolValue = totalPoolValue.add(tokenValue);
    }
  }
  
  return totalPoolValue;
}

/**
 * Calculate LP tokens to mint on deposit
 * Simplified version that focuses only on the math calculation
 */
export function calculateLpTokensOnDeposit(
  lpTokenSupply: BN,
  totalPoolValue: BN,
  depositValue: BN
): BN {
  if (lpTokenSupply.isZero()) {
    // First deposit: mint tokens equal to deposit value
    return depositValue;
  } else {
    // Subsequent deposits: mint based on ratio
    // Formula: (deposit_value * lp_supply) / total_pool_value
    const lpTokensToMint = depositValue
      .mul(lpTokenSupply)
      .div(totalPoolValue);
    
    return lpTokensToMint;
  }
}

/**
 * Calculate deposit value from asset amount and price
 */
export function calculateDepositValue(
  assetAmount: BN,
  oraclePrice: OraclePrice
): BN {
  return oraclePrice.price
    .mul(assetAmount)
    .div(new BN(10).pow(oraclePrice.precision));
}

/**
 * Helper function to get oracle price from account
 * Uses the existing getOraclePriceFromAccount function
 */
export async function getAssetPrice(
  oracleAccount: any
): Promise<OraclePrice> {
  return await getOraclePriceFromAccount(oracleAccount);
}

/**
 * Calculate total deposit value from multiple assets
 */
export function calculateTotalDepositValue(
  deposits: Array<{ amount: BN; oraclePrice: OraclePrice }>
): BN {
  let totalDepositValue = new BN(0);
  
  for (const deposit of deposits) {
    const depositValue = calculateDepositValue(deposit.amount, deposit.oraclePrice);
    totalDepositValue = totalDepositValue.add(depositValue);
  }
  
  return totalDepositValue;
} 