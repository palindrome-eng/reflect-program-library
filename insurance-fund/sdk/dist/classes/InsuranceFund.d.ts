import { AccountInfo, Connection, PublicKey, TransactionInstruction } from "@solana/web3.js";
import { Admin, Asset, Cooldown, Deposit, InitializeInsuranceFundArgs, Intent, Lockup, RewardBoost, Settings, Slash } from "../generated";
import BN from "bn.js";
type InsuranceFundAccount = Asset | Admin | Cooldown | Deposit | Intent | Lockup | RewardBoost | Settings | Slash;
export declare class InsuranceFund {
    private connection;
    constructor(connection: Connection);
    accountFromBuffer<T extends InsuranceFundAccount>(schema: {
        fromAccountInfo: (accountInfo: AccountInfo<Buffer>) => [T, number];
    }, accountInfo: AccountInfo<Buffer>): T;
    getLockups(): Promise<{
        pubkey: PublicKey;
        account: Lockup;
    }[]>;
    getLockupsByAsset(asset: PublicKey): Promise<{
        pubkey: PublicKey;
        account: Lockup;
    }[]>;
    getAssets(): Promise<{
        pubkey: PublicKey;
        account: Asset;
    }[]>;
    getDeposits(): Promise<{
        pubkey: PublicKey;
        account: Deposit;
    }[]>;
    getDepositsByUser(user: PublicKey): Promise<{
        pubkey: PublicKey;
        account: Deposit;
    }[]>;
    getCooldowns(): Promise<{
        pubkey: PublicKey;
        account: Cooldown;
    }[]>;
    getCooldownsByDeposit(depositId: BN | number): Promise<{
        pubkey: PublicKey;
        account: Cooldown;
    }[]>;
    getCooldownsByUser(user: PublicKey): Promise<{
        pubkey: PublicKey;
        account: Cooldown;
    }[]>;
    getRewardBoostsForLockup(lockup: PublicKey): Promise<{
        pubkey: PublicKey;
        account: RewardBoost;
    }[]>;
    getRewardBoostsForLockupByDepositSize(lockup: PublicKey, depositSize: BN): Promise<{
        pubkey: PublicKey;
        account: RewardBoost;
    }[]>;
    getIntents(): Promise<{
        pubkey: PublicKey;
        account: Intent;
    }[]>;
    deriveAdmin(index: number | BN): PublicKey;
    getAdmins(): Promise<{
        pubkey: PublicKey;
        account: Admin;
    }[]>;
    getAdminFromPublicKey(address: PublicKey): Promise<{
        pubkey: PublicKey;
        account: Admin;
    }[]>;
    deriveSettings(): PublicKey;
    initializeInsuranceFund(admin: PublicKey, args: InitializeInsuranceFundArgs): Promise<TransactionInstruction>;
    getSettingsData(): Promise<Settings>;
    deriveLockup(index: number | BN): PublicKey;
    deriveAsset(mint: PublicKey): PublicKey;
    deriveAssetPool(type: "vault" | "reward_pool", lockup: PublicKey, assetMint: PublicKey): PublicKey;
    initializeLockup(signer: PublicKey, assetMint: PublicKey, rewardMint: PublicKey, depositCap: BN, minDeposit: BN, duration: BN, governanceYield?: BN): Promise<TransactionInstruction>;
    addAsset(signer: PublicKey, assetMint: PublicKey, oracle: PublicKey): Promise<TransactionInstruction>;
    deriveRewardBoost(lockup: PublicKey, boostId: number | BN): PublicKey;
    getLockup(lockup: PublicKey): Promise<Lockup>;
    boostRewards(signer: PublicKey, lockupId: BN, minUsdValue: BN, boostBps: BN): Promise<TransactionInstruction>;
    depositRewards(lockupId: BN, amount: BN, signer: PublicKey): Promise<TransactionInstruction>;
    deriveIntent(deposit: PublicKey): PublicKey;
    deriveDeposit(lockup: PublicKey, depositId: BN | number): PublicKey;
    getDeposit(deposit: PublicKey): Promise<Deposit>;
    createIntent(lockupId: BN | number, depositId: BN | number, amount: BN): Promise<TransactionInstruction>;
    getIntent(intent: PublicKey): Promise<Intent>;
    processIntent(deposit: PublicKey, signer: PublicKey): Promise<TransactionInstruction>;
    private manageFreeze;
    freeze(signer: PublicKey): Promise<void>;
    unfreeze(signer: PublicKey): Promise<void>;
    deriveSlash(lockup: PublicKey, slashId: number | BN): PublicKey;
    initializeSlash(signer: PublicKey, lockupId: BN | number, amount: BN): Promise<TransactionInstruction>;
    getSlash(slash: PublicKey): Promise<Slash>;
    slashPool(signer: PublicKey, lockupId: BN | number, destination: PublicKey): Promise<TransactionInstruction>;
    slashDepositsBatch(signer: PublicKey, lockupId: BN | number): Promise<TransactionInstruction>;
    slashAllRemainingDeposits(signer: PublicKey, lockupId: BN | number): Promise<void>;
    getAsset(asset: PublicKey): Promise<Asset>;
    restake(signer: PublicKey, amount: BN, lockupId: BN): Promise<TransactionInstruction>;
    deriveCooldown(deposit: PublicKey): PublicKey;
    requestWithdrawal(signer: PublicKey, lockupId: BN | number, depositId: BN | number, amount: BN, rewardBoostId?: BN | number): Promise<TransactionInstruction>;
    requestWithdrawalWithAutoBoostDetection(signer: PublicKey, depositId: BN | number, lockupId: BN | number, amount: BN): Promise<TransactionInstruction>;
    slashColdWalletAndTransferFunds(signer: PublicKey, lockupId: BN | number, destination: PublicKey): Promise<TransactionInstruction>;
    slashColdWalletWithTransferSignature(signer: PublicKey, lockupId: BN | number, transferSig: string): Promise<TransactionInstruction>;
    withdrawCooldown(signer: PublicKey, lockupId: BN | number, depositId: BN | number): Promise<TransactionInstruction>;
}
export {};
