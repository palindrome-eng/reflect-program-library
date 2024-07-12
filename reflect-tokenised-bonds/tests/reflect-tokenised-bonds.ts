import * as anchor from "@coral-xyz/anchor";
import {getProvider, Program, Provider, AnchorProvider} from "@coral-xyz/anchor";
import { ReflectTokenisedBonds } from "../target/types/reflect_tokenised_bonds";
import {
    AuthorityType, createAssociatedTokenAccountIdempotent, createAssociatedTokenAccountIdempotentInstruction,
    createInitializeMintInstruction, createMintToInstruction,
    createSetAuthorityInstruction, getAssociatedTokenAddressSync, MINT_SIZE,
    TOKEN_PROGRAM_ID
} from "@solana/spl-token";
import {Connection, Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction} from "@solana/web3.js";
import BN from "bn.js";
import {Vault} from "../sdk";
import {expect} from "chai";

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
    amount: number
) {

    const ata = getAssociatedTokenAddressSync(
        mint,
        payer.publicKey
    );

    const ataIx = createAssociatedTokenAccountIdempotentInstruction(
        payer.publicKey,
        ata,
        payer.publicKey,
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
                provider.publicKey.toBuffer(),
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
                tokenProgram: TOKEN_PROGRAM_ID
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
                provider.publicKey.toBuffer(),
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
                provider.publicKey.toBuffer(),
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
                provider.publicKey.toBuffer(),
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
});
