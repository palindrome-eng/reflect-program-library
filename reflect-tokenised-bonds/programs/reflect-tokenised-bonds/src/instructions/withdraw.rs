use anchor_lang::prelude::*;
use anchor_spl::token::{
    Token, 
    TokenAccount, 
    Mint,
};
use crate::state::*;
use crate::errors::ReflectError;
use crate::constants::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct WithdrawArgs {
    vault_id: u64,
    amount: u64,
}

pub fn withdraw(
    ctx: Context<Withdraw>,
    args: WithdrawArgs
) -> Result<()> {
    let WithdrawArgs {
        amount,
        vault_id: _
    } = args;

    require!(
        amount > 0,
        ReflectError::AmountTooLow
    );

    let Withdraw {
        signer,
        deposit_mint: _,
        pool,
        receipt_mint,
        signer_deposit_token_account,
        signer_receipt_token_account,
        token_program,
        vault
    } = ctx.accounts;

    let base_token_amount: u64 = vault.compute_base_token(
        amount,
        pool.amount,
        receipt_mint.supply
    )?;

    vault.burn_receipt_tokens(
        amount,
        signer,
        signer_receipt_token_account,
        receipt_mint,
        token_program
    )?;

    vault.withdraw(
        base_token_amount,
        vault,
        signer_deposit_token_account,
        pool,
        token_program
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: WithdrawArgs
)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            VAULT_SEED.as_bytes(),
            &args.vault_id.to_le_bytes(),
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
        mut,
        address = vault.receipt_token_mint
    )]
    pub receipt_mint: Account<'info, Mint>,

    #[account(
        address = vault.deposit_token_mint
    )]
    pub deposit_mint: Account<'info, Mint>,

    #[account(
        mut,
        token::authority = signer,
        token::mint = deposit_mint
    )]
    pub signer_deposit_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::authority = signer,
        token::mint = receipt_mint
    )]
    pub signer_receipt_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}
