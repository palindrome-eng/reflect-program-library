import * as anchor from "@coral-xyz/anchor";
import {getProvider, Program, Provider, AnchorProvider} from "@coral-xyz/anchor";
import { ReflectTokenisedBonds } from "../target/types/reflect_tokenised_bonds";
import {
    AuthorityType,
    createInitializeMintInstruction,
    createSetAuthorityInstruction, MINT_SIZE,
    TOKEN_PROGRAM_ID
} from "@solana/spl-token";
import {Connection, Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, Transaction} from "@solana/web3.js";
import BN from "bn.js";

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

describe("reflect-tokenised-bonds", () => {
    const provider = anchor.AnchorProvider.local();
    anchor.setProvider(provider);

    const program = anchor.workspace.ReflectTokenisedBonds as Program<ReflectTokenisedBonds>;

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
                new BN(0).toArrayLike(Buffer, "le", 8)
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

        await program
            .methods
            .createVault(
                new BN(2 * 1_000_000_000),
                new BN(20),
                new BN(5),
                new BN(0)
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
                new BN(0)
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
    });
});
