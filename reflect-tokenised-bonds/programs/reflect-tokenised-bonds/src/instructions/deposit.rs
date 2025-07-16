use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token};
use crate::constants::{VAULT_SEED, VAULT_POOL_SEED};
use crate::state::Vault;
use crate::errors::ReflectError;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DepositArgs {
    pub vault_id: u64,
    pub amount: u64,
    pub is_rewards: bool,
}

pub fn deposit(
    ctx: Context<Deposit>,
    args: DepositArgs
) -> Result<()> {

    let DepositArgs {
        amount,
        is_rewards,
        vault_id: _
    } = args;
    
    let Deposit {
        signer,
        receipt_token,
        signer_receipt_token_account,
        pool,
        signer_deposit_token_account,
        vault,
        token_program,
        deposit_token: _,
    } = ctx.accounts;

    msg!("depositing");

    vault.deposit(
        amount,
        signer,
        signer_deposit_token_account,
        pool,
        token_program
    )?;

    msg!("deposited");

    if !is_rewards {
        let receipts: u64 = vault.compute_receipt_token(
            amount,
            pool.amount,
            receipt_token.supply
        )?;

        vault.mint_receipt_tokens(
            receipts,
            vault,
            signer_receipt_token_account,
            receipt_token,
            token_program
        )?
    }

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: DepositArgs
)]
pub struct Deposit<'info> {
    #[account()]
    pub signer: Signer<'info>,

    #[account(
        seeds = [
            VAULT_SEED.as_bytes(),
            &args.vault_id.to_le_bytes()
        ],
        bump
    )]
    pub vault: Account<'info, Vault>,

    #[account(
        mut,
        seeds = [
            VAULT_POOL_SEED.as_bytes(),
            vault.key().as_ref()
        ],
        bump
    )]
    pub pool: Account<'info, TokenAccount>,

    #[account(
        address = vault.deposit_token_mint
    )]
    pub deposit_token: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = deposit_token,
        token::authority = signer
    )]
    pub signer_deposit_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        address = vault.receipt_token_mint,
    )]
    pub receipt_token: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = receipt_token,
        token::authority = signer,
    )]
    pub signer_receipt_token_account: Account<'info, TokenAccount>,

    #[account()]
    pub token_program: Program<'info, Token>,
}