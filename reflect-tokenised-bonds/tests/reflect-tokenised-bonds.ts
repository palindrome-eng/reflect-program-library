import * as anchor from "@coral-xyz/anchor";
import {getProvider, Program, Provider, AnchorProvider} from "@coral-xyz/anchor";
import { ReflectTokenisedBonds } from "../target/types/reflect_tokenised_bonds";
import {
    AuthorityType,
    createAssociatedTokenAccountIdempotent,
    createAssociatedTokenAccountIdempotentInstruction,
    createAssociatedTokenAccountInstruction,
    createInitializeMintInstruction,
    createMintToInstruction,
    createSetAuthorityInstruction, getAccount,
    getAssociatedTokenAddressSync,
    MINT_SIZE,
    TOKEN_PROGRAM_ID
} from "@solana/spl-token";
import {
    Connection,
    Keypair, Lockup,
    PublicKey,
    SystemProgram,
    SYSVAR_CLOCK_PUBKEY,
    SYSVAR_RENT_PUBKEY,
    Transaction
} from "@solana/web3.js";
import BN from "bn.js";
import {LockupState, RTBProtocol, Vault} from "../sdk";
import {expect, use} from "chai";

async function createToken(
    connection: Connection,
    payer: AnchorProvider
) {
    const keypair = Keypair.generate();

    const lamports = await connection.getMinimumBalanceForRentExemption(MINT_SIZE);
    const createAccountIx = SystemProgram.createAccount({
        newAccountPubkey: keypair.publicKey,
        fromPubkey: payer.publicKey,
        lamports,
        programId: TOKEN_PROGRAM_ID,
        space: MINT_SIZE
    });

    const ix = createInitializeMintInstruction(
        keypair.publicKey,
        9,
        payer.publicKey,
        payer.publicKey
    );

    const tx = new Transaction();
    tx.add(createAccountIx, ix);

    const {
        lastValidBlockHeight,
        blockhash
    } = await connection.getLatestBlockhash();

    tx.feePayer = payer.publicKey;
    tx.recentBlockhash = blockhash;

    tx.partialSign(keypair);
    const signed = await payer.wallet.signTransaction(tx);

    const sent = await connection.sendRawTransaction(signed.serialize());
    await connection.confirmTransaction({
        blockhash,
        lastValidBlockHeight,
        signature: sent
    }, "confirmed");

    return keypair.publicKey;
}

async function transferAuthority(
    mint: PublicKey,
    currentAuthority: AnchorProvider,
    type: AuthorityType,
    newAuthority: PublicKey | null
) {
    const ix = createSetAuthorityInstruction(
        mint,
        currentAuthority.publicKey,
        type,
        newAuthority
    );

    const tx = new Transaction();
    tx.add(ix);

    const {
        lastValidBlockHeight,
        blockhash
    } = await currentAuthority.connection.getLatestBlockhash();

    tx.feePayer = currentAuthority.publicKey;
    tx.recentBlockhash = blockhash;

    const signed = await currentAuthority.wallet.signTransaction(tx);

    const sent = await currentAuthority.connection.sendRawTransaction(signed.serialize());
    await currentAuthority.connection.confirmTransaction({
        blockhash,
        lastValidBlockHeight,
        signature: sent
    }, "confirmed");
}

async function mintTokens(
    mint: PublicKey,
    payer: AnchorProvider,
    amount: number,
    recipient?: Keypair
) {

    const ata = getAssociatedTokenAddressSync(
        mint,
        recipient?.publicKey || payer.publicKey
    );

    const ataIx = createAssociatedTokenAccountIdempotentInstruction(
        payer.publicKey,
        ata,
        recipient?.publicKey || payer.publicKey,
        mint,
    );

    const ix = createMintToInstruction(
        mint,
        ata,
        payer.publicKey,
        amount,
    );

    const tx = new Transaction();
    tx.add(ataIx, ix);

    const {
        lastValidBlockHeight,
        blockhash
    } = await payer.connection.getLatestBlockhash();

    tx.feePayer = payer.publicKey;
    tx.recentBlockhash = blockhash;

    const signed = await payer.wallet.signTransaction(tx);

    const sent = await payer.connection.sendRawTransaction(signed.serialize());
    await payer.connection.confirmTransaction({
        blockhash,
        lastValidBlockHeight,
        signature: sent
    }, "confirmed");
}

describe("reflect-tokenised-bonds", () => {
    const provider = anchor.AnchorProvider.local();
    anchor.setProvider(provider);

    const program = anchor.workspace.ReflectTokenisedBonds as Program<ReflectTokenisedBonds>;

    let vaultId = 0;
    let [rtbProtocol] = PublicKey.findProgramAddressSync(
        [
            Buffer.from("rtb")
        ],
        program.programId
    );
    const user = Keypair.generate();
    const [userAccount] = PublicKey.findProgramAddressSync(
        [
            Buffer.from("user_account"),
            user.publicKey.toBuffer()
        ],
        program.programId
    );

    before(async () => {
        await provider.connection.requestAirdrop(user.publicKey, 2_000_000_000);
    });

    it('Initializes Reflect Tokenised Bonds Protocol', async () => {
        await program
            .methods
            .initializeProtocol()
            .accounts({
                rtbProtocol,
                payer: provider.publicKey,
                systemProgram: SystemProgram.programId
            })
            .rpc();

        const {
            nextVaultSeed
        } = await RTBProtocol.fromAccountAddress(
            provider.connection,
            rtbProtocol
        );

        expect(nextVaultSeed.toString()).eq("0");
    });

    it("Successfully initializes vault and vault pools.", async () => {
        const depositToken = await createToken(
            provider.connection,
            provider
        );

        const receiptToken = await createToken(
            provider.connection,
            provider
        );

        const [vault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                new BN(vaultId).toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        // Transfer mint authority to vault.
        await transferAuthority(
            receiptToken,
            provider,
            AuthorityType.MintTokens,
            vault
        );

        // Transfer mint authority to vault.
        await transferAuthority(
            receiptToken,
            provider,
            AuthorityType.FreezeAccount,
            null
        );

        const minimumDeposit = new BN(2 * 1_000_000_000);
        const minimumLockup = new BN(20); // 20 seconds
        const yieldPreset = new BN(5);

        await program
            .methods
            .createVault(
                minimumDeposit,
                minimumLockup,
                yieldPreset,
                new BN(vaultId)
            )
            .accounts({
                vault,
                rtbProtocol,
                admin: provider.publicKey,
                systemProgram: SystemProgram.programId,
                rent: SYSVAR_RENT_PUBKEY
            })
            .rpc();

        const [depositPool] = PublicKey.findProgramAddressSync(
            [
                vault.toBuffer(),
                Buffer.from("deposit_pool")
            ],
            program.programId
        );

        const [rewardPool] = PublicKey.findProgramAddressSync(
            [
                vault.toBuffer(),
                Buffer.from("reward_pool")
            ],
            program.programId
        );

        await program
            .methods
            .initVaultPools(
                new BN(vaultId)
            )
            .accounts({
                vault,
                admin: provider.publicKey,
                systemProgram: SystemProgram.programId,
                rent: SYSVAR_RENT_PUBKEY,
                depositTokenMint: depositToken,
                receiptTokenMint: receiptToken,
                depositPool,
                rewardPool,
                tokenProgram: TOKEN_PROGRAM_ID,
            })
            .rpc();

        const {
            depositPool: depositPoolAddress,
            admin,
            depositTokenMint,
            receiptTokenMint,
            rewardPool: rewardPoolAddress,
            minLockup,
            minDeposit,
            targetYieldRate,
            totalReceiptSupply
        } = await Vault.fromAccountAddress(
            provider.connection,
            vault
        );

        expect(depositPoolAddress.toString()).eq(depositPoolAddress.toString());
        expect(admin.toString()).eq(provider.wallet.publicKey.toString());
        expect(depositTokenMint.toString()).eq(depositToken.toString());
        expect(receiptTokenMint.toString()).eq(receiptToken.toString());
        expect(rewardPoolAddress.toString()).eq(rewardPool.toString());
        expect(minLockup.toString()).eq(minimumLockup.toString());
        expect(minDeposit.toString()).eq(minimumDeposit.toString());
        expect(targetYieldRate.toString()).eq(yieldPreset.toString());
        expect(totalReceiptSupply.toString()).eq("0");

        vaultId++;
    });

    it('Fails to create a vault with incorrect mint authority.', async () => {
        const depositToken = await createToken(
            provider.connection,
            provider
        );

        const receiptToken = await createToken(
            provider.connection,
            provider
        );

        const [vault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                new BN(vaultId).toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        const minimumDeposit = new BN(2 * 1_000_000_000);
        const minimumLockup = new BN(20);
        const yieldPreset = new BN(5);

        await program
            .methods
            .createVault(
                minimumDeposit,
                minimumLockup,
                yieldPreset,
                new BN(vaultId)
            )
            .accounts({
                vault,
                rtbProtocol,
                admin: provider.publicKey,
                systemProgram: SystemProgram.programId,
                rent: SYSVAR_RENT_PUBKEY
            })
            .rpc();

        const [depositPool] = PublicKey.findProgramAddressSync(
            [
                vault.toBuffer(),
                Buffer.from("deposit_pool")
            ],
            program.programId
        );

        const [rewardPool] = PublicKey.findProgramAddressSync(
            [
                vault.toBuffer(),
                Buffer.from("reward_pool")
            ],
            program.programId
        );

        let error: string = "";

        await program
            .methods
            .initVaultPools(
                new BN(vaultId)
            )
            .accounts({
                vault,
                admin: provider.publicKey,
                systemProgram: SystemProgram.programId,
                rent: SYSVAR_RENT_PUBKEY,
                depositTokenMint: depositToken,
                receiptTokenMint: receiptToken,
                depositPool,
                rewardPool,
                tokenProgram: TOKEN_PROGRAM_ID
            })
            .rpc()
            .catch(err => error = err.toString());

        expect(error).include("Invalid mint authority. Move mint authority of the receipt token to the vault PDA");

        vaultId++;
    });

    it('Fails to create a vault with incorrect freeze authority.', async () => {
        const depositToken = await createToken(
            provider.connection,
            provider
        );

        const receiptToken = await createToken(
            provider.connection,
            provider
        );

        const [vault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                new BN(vaultId).toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        const minimumDeposit = new BN(2 * 1_000_000_000);
        const minimumLockup = new BN(20);
        const yieldPreset = new BN(5);

        await program
            .methods
            .createVault(
                minimumDeposit,
                minimumLockup,
                yieldPreset,
                new BN(vaultId)
            )
            .accounts({
                vault,
                rtbProtocol,
                admin: provider.publicKey,
                systemProgram: SystemProgram.programId,
                rent: SYSVAR_RENT_PUBKEY
            })
            .rpc();

        const [depositPool] = PublicKey.findProgramAddressSync(
            [
                vault.toBuffer(),
                Buffer.from("deposit_pool")
            ],
            program.programId
        );

        const [rewardPool] = PublicKey.findProgramAddressSync(
            [
                vault.toBuffer(),
                Buffer.from("reward_pool")
            ],
            program.programId
        );

        // Transfer mint authority to vault.
        await transferAuthority(
            receiptToken,
            provider,
            AuthorityType.MintTokens,
            vault
        );

        let error: string = "";

        await program
            .methods
            .initVaultPools(
                new BN(vaultId)
            )
            .accounts({
                vault,
                admin: provider.publicKey,
                systemProgram: SystemProgram.programId,
                rent: SYSVAR_RENT_PUBKEY,
                depositTokenMint: depositToken,
                receiptTokenMint: receiptToken,
                depositPool,
                rewardPool,
                tokenProgram: TOKEN_PROGRAM_ID
            })
            .rpc()
            .catch(err => error = err.toString());

        expect(error).include("Invalid freeze authority. Move freeze authority of the receipt token to the vault PDA, or remove it completely.");

        vaultId++;
    });

    it('Fails to create a vault with pre-minted receipt tokens.', async () => {
        const depositToken = await createToken(
            provider.connection,
            provider
        );

        const receiptToken = await createToken(
            provider.connection,
            provider
        );

        await mintTokens(
            receiptToken,
            provider,
            1
        );

        const [vault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                new BN(vaultId).toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        const minimumDeposit = new BN(2 * 1_000_000_000);
        const minimumLockup = new BN(20);
        const yieldPreset = new BN(5);

        await program
            .methods
            .createVault(
                minimumDeposit,
                minimumLockup,
                yieldPreset,
                new BN(vaultId)
            )
            .accounts({
                vault,
                rtbProtocol,
                admin: provider.publicKey,
                systemProgram: SystemProgram.programId,
                rent: SYSVAR_RENT_PUBKEY
            })
            .rpc();

        const [depositPool] = PublicKey.findProgramAddressSync(
            [
                vault.toBuffer(),
                Buffer.from("deposit_pool")
            ],
            program.programId
        );

        const [rewardPool] = PublicKey.findProgramAddressSync(
            [
                vault.toBuffer(),
                Buffer.from("reward_pool")
            ],
            program.programId
        );

        // Transfer mint authority to vault.
        await transferAuthority(
            receiptToken,
            provider,
            AuthorityType.MintTokens,
            vault
        );

        // Transfer freeze authority to vault.
        await transferAuthority(
            receiptToken,
            provider,
            AuthorityType.FreezeAccount,
            null
        );

        let error: string = "";

        await program
            .methods
            .initVaultPools(
                new BN(vaultId)
            )
            .accounts({
                vault,
                admin: provider.publicKey,
                systemProgram: SystemProgram.programId,
                rent: SYSVAR_RENT_PUBKEY,
                depositTokenMint: depositToken,
                receiptTokenMint: receiptToken,
                depositPool,
                rewardPool,
                tokenProgram: TOKEN_PROGRAM_ID
            })
            .rpc()
            .catch(err => error = err.toString());

        expect(error).include("Supply of the receipt token has to be 0. Pre-minting is not allowed.");

        vaultId++;
    });

    it('Successfully deposits base token.', async () => {
        // Vault with ID = 0, created in the 2nd test.
        const [vault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                new BN(0).toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        let vaultData  = await Vault.fromAccountAddress(
            provider.connection,
            vault
        );

        let {
            depositPool,
            rewardPool,
            depositTokenMint,
            receiptTokenMint,
            minDeposit
        } = vaultData;

        const amount = typeof minDeposit === "number" ? minDeposit : minDeposit.toNumber();

        await mintTokens(
            depositTokenMint,
            provider,
            amount,
            user
        );

        const depositTokenAccount = getAssociatedTokenAddressSync(
            depositTokenMint,
            user.publicKey
        );

        const receiptTokenAccount = getAssociatedTokenAddressSync(
            receiptTokenMint,
            user.publicKey
        );

        const ataIx = createAssociatedTokenAccountInstruction(
            user.publicKey,
            receiptTokenAccount,
            user.publicKey,
            receiptTokenMint
        );

        const depositIx = await program
            .methods
            .deposit(
                new BN(amount),
                new BN(0)
            )
            .accounts({
                vault,
                rewardPool,
                depositPool,
                receiptTokenMint,
                user: user.publicKey,
                tokenProgram: TOKEN_PROGRAM_ID,
                depositTokenAccount,
                receiptTokenAccount,
            })
            .instruction();

        const transaction = new Transaction();
        transaction.add(ataIx);
        transaction.add(depositIx);

        const {
            lastValidBlockHeight,
            blockhash
        } = await provider.connection.getLatestBlockhash();

        transaction.recentBlockhash = blockhash;
        transaction.feePayer = user.publicKey;

        transaction.sign(user);
        const signature = await provider.connection.sendRawTransaction(transaction.serialize());
        await provider.connection.confirmTransaction({
            blockhash,
            lastValidBlockHeight,
            signature
        }, "confirmed");

        vaultData  = await Vault.fromAccountAddress(
            provider.connection,
            vault
        );

        const userReceiptTokenAccountData = await getAccount(
            provider.connection,
            receiptTokenAccount,
        );

        const userDepositTokenAccountData = await getAccount(
            provider.connection,
            depositTokenAccount
        );

        const depositPoolData = await getAccount(
            provider.connection,
            depositPool
        );

        expect(vaultData.totalReceiptSupply.toString()).eq(amount.toString());
        expect(userReceiptTokenAccountData.amount.toString()).eq(amount.toString());
        expect(userDepositTokenAccountData.amount.toString()).eq("0");
        expect(depositPoolData.amount.toString()).eq(amount.toString());
    });

    it("Locks receipts tokens up.", async () => {
        // Vault with ID = 0, created in the 2nd test.
        const [vault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                new BN(0).toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        const [lockup] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("lockup"),
                (new BN(0)).toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        let {
            receiptTokenMint,
            minLockup
        }  = await Vault.fromAccountAddress(
            provider.connection,
            vault
        );

        const lockupReceiptTokenAccount = getAssociatedTokenAddressSync(
            receiptTokenMint,
            lockup,
            true
        );

        const userReceiptTokenAccount = getAssociatedTokenAddressSync(
            receiptTokenMint,
            user.publicKey
        );

        const ataIx = createAssociatedTokenAccountInstruction(
            user.publicKey,
            lockupReceiptTokenAccount,
            lockup,
            receiptTokenMint
        );

        const {
            amount: userReceiptTokenBalance
        } = await getAccount(
            provider.connection,
            userReceiptTokenAccount,
        );

        const lockupIx = await program
            .methods
            .lockup(
                new BN(userReceiptTokenBalance.toString())
            )
            .accounts({
                user: user.publicKey,
                systemProgram: SystemProgram.programId,
                tokenProgram: TOKEN_PROGRAM_ID,
                vault,
                userAccount,
                lockup,
                clock: SYSVAR_CLOCK_PUBKEY,
                lockupReceiptTokenAccount,
                userReceiptTokenAccount,
            })
            .instruction();

        const transaction = new Transaction();
        transaction.add(ataIx);
        transaction.add(lockupIx);

        const {
            lastValidBlockHeight,
            blockhash
        } = await provider.connection.getLatestBlockhash();

        transaction.recentBlockhash = blockhash;
        transaction.feePayer = user.publicKey;

        transaction.sign(user);
        const signature = await provider.connection.sendRawTransaction(transaction.serialize());
        const sendTransactionTimestamp = Math.floor(Date.now() / 1000);

        await provider.connection.confirmTransaction({
            blockhash,
            lastValidBlockHeight,
            signature
        }, "confirmed");

        const lockupAccountData = await LockupState.fromAccountAddress(
            provider.connection,
            lockup
        );

        const userReceiptTokenAccountData = await getAccount(
            provider.connection,
            userReceiptTokenAccount,
        );

        const lockupReceiptTokenAccountData = await getAccount(
            provider.connection,
            lockupReceiptTokenAccount,
        );

        expect(userReceiptTokenAccountData.amount.toString()).eq("0");
        expect(lockupAccountData.receiptAmount.toString()).eq(userReceiptTokenBalance.toString());
        expect(lockupReceiptTokenAccountData.amount.toString()).eq(lockupAccountData.receiptAmount.toString());
        expect(lockupAccountData.user.toString()).eq(user.publicKey.toString());

        // Expect unlock date to be approximately equal to timestamp of the
        // transaction + minimum lockup. Delta of 3 seconds due to latencies.
        expect(
            typeof lockupAccountData.unlockDate == "number"
                ? lockupAccountData.unlockDate
                : lockupAccountData.unlockDate.toNumber()
        ).approximately(
            sendTransactionTimestamp + (typeof minLockup == "number" ? minLockup : minLockup.toNumber()),
            3
        );
    });
});
