import BN from 'bn.js';

type Price = {
    price: BN;
    precision: BN;
}

export default function calculateLpToken(
  lpTokenSupply: BN,
  tokenALiquidity: BN,
  tokenBLiquidity: BN,
  tokenADeposit: BN,
  tokenBDeposit: BN,
  tokenAPrice: Price,
  tokenBPrice: Price
): BN {
  const depositAValue = tokenAPrice
    .price
    .mul(tokenADeposit)
    .div(new BN(10).pow(tokenAPrice.precision));
  const depositBValue = tokenBPrice
    .price
    .mul(tokenBDeposit)
    .div(new BN(10).pow(tokenBPrice.precision));

  const totalDepositValue = depositAValue.add(depositBValue);
  
  if (lpTokenSupply.isZero()) {
    return totalDepositValue;
  }
  
  const tokenALiquidityValue = tokenAPrice
    .price
    .mul(tokenALiquidity)
    .div(
        new BN(10).pow(tokenAPrice.precision)
    );
    
  const tokenBLiquidityValue = tokenBPrice
    .price
    .mul(tokenBLiquidity)
    .div(
        new BN(10).pow(tokenBPrice.precision)
    );

  const totalLpValue = tokenALiquidityValue.add(tokenBLiquidityValue);
  
  const valuePerLpToken = totalLpValue.div(lpTokenSupply);
  const lpTokenToMint = totalDepositValue.div(valuePerLpToken);
  
  return lpTokenToMint;
}