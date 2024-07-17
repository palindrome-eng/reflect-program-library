use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::CustomError;
use crate::constants::{
    RTB_SEED, VAULT_SEED
};

pub fn create_vault(
    ctx: Context<CreateVault>,
    min_deposit: u64,
    min_lockup: i64,
    target_yield_rate: u64,
    vault_seed: u64,
) -> Result<()> {
    let rtb_protocol = &mut ctx.accounts.rtb_protocol;
    let vault = &mut ctx.accounts.vault;

    rtb_protocol.next_vault_seed += 1;

    vault.admin = *ctx.accounts.admin.key;
    vault.min_deposit = min_deposit;
    vault.min_lockup = min_lockup;
    vault.target_yield_rate = target_yield_rate;
    vault.total_receipt_supply = 0;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    min_deposit: u64,
    min_lockup: i64,
    target_yield_rate: u64,
    vault_seed: u64,
)]
pub struct CreateVault<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            RTB_SEED.as_bytes()
        ],
        constraint = vault_seed == rtb_protocol.next_vault_seed @ CustomError::InvalidVaultSeed,
        bump
    )]
    rtb_protocol: Account<'info, RTBProtocol>,

    #[account(
        init,
        seeds = [
            VAULT_SEED.as_bytes(),
            vault_seed.to_le_bytes().as_ref()
        ],
        bump,
        payer = admin,
        space = Vault::LEN
    )]
    pub vault: Box<Account<'info, Vault>>,

    pub system_program: Program<'info, System>,

    pub rent: Sysvar<'info, Rent>,
}
