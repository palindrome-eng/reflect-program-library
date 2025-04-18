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

    vault.deposit(
        amount,
        signer,
        signer_deposit_token_account,
        pool,
        token_program
    )?;

    // If normal deposit, compute and mint receipts.
    // If rewards, just transfer tokens into the pool.
    if !is_rewards {
        require!(
            receipt_token.is_some(),
            ReflectError::MissingAccounts
        );

        require!(
            signer_receipt_token_account.is_some(),
            ReflectError::MissingAccounts
        );

        let receipts: u64 = vault.compute_receipt_token(
            amount,
            pool.amount,
            receipt_token
                .as_ref()
                .unwrap()
                .supply
        )?;

        vault.mint_receipt_tokens(
            receipts,
            vault,
            signer_receipt_token_account.as_ref().unwrap(),
            receipt_token.as_ref().unwrap(),
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
        address = vault.receipt_token_mint,
    )]
    pub receipt_token: Option<Account<'info, Mint>>,

    #[account(
        mut,
        constraint = receipt_token.is_some() @ ReflectError::MissingAccounts,
        token::mint = receipt_token,
        token::authority = signer,
    )]
    pub signer_receipt_token_account: Option<Account<'info, TokenAccount>>,

    #[account()]
    pub token_program: Program<'info, Token>,
}