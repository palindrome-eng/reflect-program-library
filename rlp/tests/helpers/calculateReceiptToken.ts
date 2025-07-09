import BN from 'bn.js';

export default function calculateReceiptToken(
  receiptTokenMintSupply: BN,
  deposit: BN,
  totalDeposits: BN
): BN {
  if (receiptTokenMintSupply.isZero()) {
    return deposit;
  } else {
    const result = deposit.mul(totalDeposits).div(receiptTokenMintSupply);
    return result;
  }
}