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
    getMint,
    MINT_SIZE,
    TOKEN_PROGRAM_ID
} from "@solana/spl-token";
import {
    Connection,
    Keypair, LAMPORTS_PER_SOL, Lockup,
    PublicKey,
    SystemProgram,
    SYSVAR_CLOCK_PUBKEY,
    SYSVAR_RENT_PUBKEY,
    Transaction
} from "@solana/web3.js";
import BN, {min} from "bn.js";
import {Config, Vault} from "../sdk/src";
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

function sleep(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
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
    recipient?: PublicKey
) {

    const ata = getAssociatedTokenAddressSync(
        mint,
        recipient || payer.publicKey,
        true
    );

    const ataIx = createAssociatedTokenAccountIdempotentInstruction(
        payer.publicKey,
        ata,
        recipient || payer.publicKey,
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
    const [config] = PublicKey.findProgramAddressSync(
        [
            Buffer.from("config")
        ],
        program.programId
    );
    const [admin] = PublicKey.findProgramAddressSync(
        [
            Buffer.from("admin"),
            provider.publicKey.toBuffer()
        ],
        program.programId
    );
    const user = Keypair.generate();
    const user2 = Keypair.generate();
    const [userAccount] = PublicKey.findProgramAddressSync(
        [
            Buffer.from("user_account"),
            user.publicKey.toBuffer()
        ],
        program.programId
    );

    before(async () => {
        await provider.connection.requestAirdrop(user.publicKey, 2_000_000_000);
        await provider.connection.requestAirdrop(user2.publicKey, 2_000_000_000);
    });

    it('Initializes Reflect Tokenised Bonds Protocol', async () => {
        await program
            .methods
            .initialize()
            .accounts({
                admin,
                config,
                program: program.programId,
                signer: provider.publicKey,
                systemProgram: SystemProgram.programId,
            })
            .rpc();

        const {
            vaults,
            frozen,
            bump
        } = await Config.fromAccountAddress(
            provider.connection,
            config
        );

        expect(vaults.toString()).eq("0");
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

        const [vaultPool] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault_pool"),
                vault.toBuffer()
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
            .createVault()
            .accounts({
                vault,
                config,
                signer: provider.publicKey,
                depositMint: depositToken,
                receiptMint: receiptToken,
                tokenProgram: TOKEN_PROGRAM_ID,
                vaultPool,
                systemProgram: SystemProgram.programId,
            })
            .rpc()
            .catch(err => {
                console.log(err);
                throw err;
            });

        const {
            bump,
            creator,
            depositTokenMint,
            receiptTokenMint,
            index
        } = await Vault.fromAccountAddress(
            provider.connection,
            vault
        );

        expect(index.toString()).eq("0");
        expect(creator.toString()).eq(provider.wallet.publicKey.toString());
        expect(depositTokenMint.toString()).eq(depositToken.toString());
        expect(receiptTokenMint.toString()).eq(receiptToken.toString());

        const {
            vaults
        } = await Config.fromAccountAddress(
            provider.connection,
            config
        );

        expect(vaults.toString()).eq("1");

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

        const [vaultPool] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault_pool"),
                vault.toBuffer()
            ],
            program.programId
        );

        let error: string = "";
        await program
            .methods
            .createVault()
            .accounts({
                vault,
                config,
                signer: provider.publicKey,
                depositMint: depositToken,
                receiptMint: receiptToken,
                tokenProgram: TOKEN_PROGRAM_ID,
                vaultPool,
                systemProgram: SystemProgram.programId,
            })
            .rpc()
            .catch(err => error = err.toString());

        expect(error).include("InvalidReceiptTokenMintAuthority");
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

        const [vaultPool] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault_pool"),
                vault.toBuffer()
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
            .createVault()
            .accounts({
                vault,
                config,
                signer: provider.publicKey,
                depositMint: depositToken,
                receiptMint: receiptToken,
                tokenProgram: TOKEN_PROGRAM_ID,
                vaultPool,
                systemProgram: SystemProgram.programId,
            })
            .rpc()
            .catch(err => error = err.toString());

        expect(error).include("InvalidReceiptTokenSupply");
    });

    it('Deposits base token.', async () => {
        // Vault with ID = 0, created in the 2nd test.
        const [vault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                new BN(0).toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        const [vaultPool] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault_pool"),
                vault.toBuffer()
            ],
            program.programId
        );

        const {
            bump,
            creator,
            depositTokenMint,
            index,
            receiptTokenMint
        }  = await Vault.fromAccountAddress(
            provider.connection,
            vault
        );

        const amount = LAMPORTS_PER_SOL * 100;

        await mintTokens(
            depositTokenMint,
            provider,
            amount,
            user.publicKey
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

        const vaultPoolPre = await getAccount(
            provider.connection,
            vaultPool
        );

        const signerDepositTokenAccountPre = await getAccount(
            provider.connection,
            depositTokenAccount
        );

        await program
            .methods
            .deposit({
                amount: new BN(amount),
                isRewards: false,
                vaultId: new BN(0)
            })
            .accounts({
                vault,
                tokenProgram: TOKEN_PROGRAM_ID,
                depositToken: depositTokenMint,
                pool: vaultPool,
                receiptToken: receiptTokenMint,
                signer: user.publicKey,
                signerDepositTokenAccount: depositTokenAccount,
                signerReceiptTokenAccount: receiptTokenAccount
            })
            .preInstructions([ataIx])
            .signers([user])
            .rpc()
            .catch(err => {
                console.log(err);
                throw err;
            });

        const vaultPoolPost = await getAccount(
            provider.connection,
            vaultPool
        );

        const signerReceiptTokenAccountPost = await getAccount(
            provider.connection,
            receiptTokenAccount
        );

        const signerDepositTokenAccountPost = await getAccount(
            provider.connection,
            depositTokenAccount
        );

        expect(
            new BN(signerDepositTokenAccountPost.amount.toString())
                .add(new BN(amount.toString()))
                .toNumber()
        ).eq(new BN(signerDepositTokenAccountPre.amount.toString()).toNumber());

        expect(
            new BN(vaultPoolPost.amount.toString())
                .sub(new BN(amount))
                .toNumber()
        ).eq(new BN(vaultPoolPre.amount.toString()).toNumber());

        // Expect to have same amount of receipts inted as the amount deposited, since it's the first deposit.
        expect(
            new BN(signerReceiptTokenAccountPost.amount.toString()).toNumber()
        ).eq(amount);
    });

    it("Deposits 500,000 reward token into the reward pool.", async () => {
        const [vault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                new BN(0).toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        const [vaultPool] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault_pool"),
                vault.toBuffer()
            ],
            program.programId
        );

        const {
            depositTokenMint,
            receiptTokenMint
        } = await Vault.fromAccountAddress(
            provider.connection,
            vault
        );

        const signerDepositTokenAccount = getAssociatedTokenAddressSync(
            depositTokenMint,
            provider.publicKey
        );

        const signerReceiptTokenAccount = getAssociatedTokenAddressSync(
            receiptTokenMint,
            provider.publicKey
        );

        const ataIx = createAssociatedTokenAccountInstruction(
            provider.publicKey,
            signerReceiptTokenAccount,
            provider.publicKey,
            receiptTokenMint
        );

        await mintTokens(
            depositTokenMint,
            provider,
            50 * LAMPORTS_PER_SOL,
            provider.publicKey
        );

        await program
            .methods
            .deposit({
                amount: new BN(50 * LAMPORTS_PER_SOL),
                isRewards: true,
                vaultId: new BN(0)
            })
            .accounts({
                signer: provider.publicKey,
                signerDepositTokenAccount,
                signerReceiptTokenAccount,
                vault,
                tokenProgram: TOKEN_PROGRAM_ID,
                depositToken: depositTokenMint,
                pool: vaultPool,
                receiptToken: receiptTokenMint,
            })
            .preInstructions([ataIx])
            .rpc();
    });

    
    it('Deposits base token for user2.', async () => {
        // Vault with ID = 0, created in the 2nd test.
        const [vault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                new BN(0).toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        const [vaultPool] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault_pool"),
                vault.toBuffer()
            ],
            program.programId
        );

        const {
            bump,
            creator,
            depositTokenMint,
            index,
            receiptTokenMint
        }  = await Vault.fromAccountAddress(
            provider.connection,
            vault
        );

        const amount = LAMPORTS_PER_SOL * 100;

        await mintTokens(
            depositTokenMint,
            provider,
            amount,
            user2.publicKey
        );

        const depositTokenAccount = getAssociatedTokenAddressSync(
            depositTokenMint,
            user2.publicKey
        );

        const receiptTokenAccount = getAssociatedTokenAddressSync(
            receiptTokenMint,
            user2.publicKey
        );

        const ataIx = createAssociatedTokenAccountInstruction(
            user2.publicKey,
            receiptTokenAccount,
            user2.publicKey,
            receiptTokenMint
        );

        const vaultPoolPre = await getAccount(
            provider.connection,
            vaultPool
        );

        const signerDepositTokenAccountPre = await getAccount(
            provider.connection,
            depositTokenAccount
        );

        const receiptTokenMintDataPre = await getMint(
            provider.connection,
            receiptTokenMint
        );

        await program
            .methods
            .deposit({
                amount: new BN(amount),
                isRewards: false,
                vaultId: new BN(0)
            })
            .accounts({
                vault,
                tokenProgram: TOKEN_PROGRAM_ID,
                depositToken: depositTokenMint,
                pool: vaultPool,
                receiptToken: receiptTokenMint,
                signer: user2.publicKey,
                signerDepositTokenAccount: depositTokenAccount,
                signerReceiptTokenAccount: receiptTokenAccount
            })
            .preInstructions([ataIx])
            .signers([user2])
            .rpc()
            .catch(err => {
                console.log(err);
                throw err;
            });

        const vaultPoolPost = await getAccount(
            provider.connection,
            vaultPool
        );

        const signerReceiptTokenAccountPost = await getAccount(
            provider.connection,
            receiptTokenAccount
        );

        const signerDepositTokenAccountPost = await getAccount(
            provider.connection,
            depositTokenAccount
        );

        const receiptTokenMintDataPost = await getMint(
            provider.connection,
            receiptTokenMint
        );

        expect(
            new BN(signerReceiptTokenAccountPost.amount.toString())
                .mul(new BN(vaultPoolPost.amount.toString()))
                .div(new BN(receiptTokenMintDataPost.supply.toString()))
                .toNumber()
        )
        .approximately(
            amount,
            1
        );

        expect(
            new BN(vaultPoolPre.amount.toString())
                .add(new BN(amount.toString()))
                .toNumber()
        ).eq(
            new BN(vaultPoolPost.amount.toString())
                .toNumber()
        );

        expect(
            new BN(signerDepositTokenAccountPost.amount.toString())
                .add(new BN(amount.toString()))
                .toNumber()
        ).eq(new BN(signerDepositTokenAccountPre.amount.toString()).toNumber());

        expect(
            new BN(vaultPoolPost.amount.toString())
                .sub(new BN(amount))
                .toNumber()
        ).eq(new BN(vaultPoolPre.amount.toString()).toNumber());
    });

    it("Withdraws tokens and reward for user 1.", async () => {
        const [vault] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault"),
                new BN(0).toArrayLike(Buffer, "le", 8)
            ],
            program.programId
        );

        const [vaultPool] = PublicKey.findProgramAddressSync(
            [
                Buffer.from("vault_pool"),
                vault.toBuffer()
            ],
            program.programId
        );

        let {
            depositTokenMint,
            receiptTokenMint,
        }  = await Vault.fromAccountAddress(
            provider.connection,
            vault
        );

        const signerDepositTokenAccount = getAssociatedTokenAddressSync(
            depositTokenMint,
            user.publicKey
        );

        const signerDepositTokenAccountPre = await getAccount(
            provider.connection,
            signerDepositTokenAccount
        );

        const signerReceiptTokenAccount = getAssociatedTokenAddressSync(
            receiptTokenMint,
            user.publicKey
        );

        const signerReceiptTokenAccountPre = await getAccount(
            provider.connection,
            signerReceiptTokenAccount
        );

        const vaultPoolPre = await getAccount(
            provider.connection,
            vaultPool
        );

        const amount = new BN(signerReceiptTokenAccountPre.amount.toString());
        const receiptTokenMintDataPre = await getMint(
            provider.connection,
            receiptTokenMint
        );

        await program
            .methods
            .withdraw({
                amount,
                vaultId: new BN(0)
            })
            .accounts({
                signer: user.publicKey,
                vault,
                receiptMint: receiptTokenMint,
                tokenProgram: TOKEN_PROGRAM_ID,
                depositMint: depositTokenMint,
                pool: vaultPool,
                signerDepositTokenAccount,
                signerReceiptTokenAccount
            })
            .signers([user])
            .rpc();

        const receiptTokenMintDataPost = await getMint(
            provider.connection,
            receiptTokenMint
        );

        const signerReceiptTokenAccountPost = await getAccount(
             provider.connection,
             signerReceiptTokenAccount
        );

        const signerDepositTokenAccountPost = await getAccount(
            provider.connection,
            signerDepositTokenAccount
        );

        // Check that tokens have been burned.
        expect(
            new BN(receiptTokenMintDataPost.supply.toString())
                .toNumber()
        ).eq(
            new BN(receiptTokenMintDataPre.supply.toString())
                .sub(amount)
                .toNumber()
        );
        
        // Check that tokens have been burned from the user's account.
        expect(
            new BN(signerReceiptTokenAccountPre.amount.toString())
                .sub(new BN(signerReceiptTokenAccountPost.amount.toString()))
                .toNumber()
        ).eq(amount.toNumber());

        // Check that user has received the correct share of the vault pool, based on the amount of receipts they burned.
        expect(
            new BN(signerReceiptTokenAccountPre.amount.toString())
                .mul(new BN(vaultPoolPre.amount.toString()))
                .div(new BN(receiptTokenMintDataPre.supply.toString()))
                .toNumber()
        ).approximately(
            new BN(signerDepositTokenAccountPost.amount.toString())
                .sub(new BN(signerDepositTokenAccountPre.amount.toString()))
                .toNumber(),
            0.01 * LAMPORTS_PER_SOL
        );
    });
});
