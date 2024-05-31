use anchor_lang::prelude::*;
use crate::state::vault::Vault;

#[derive(Accounts)]
#[instruction(
    min_deposit: u64,
    min_lockup: i64,
    target_yield_rate: u64,
    vault_seed: u64,
)]
pub struct CreateVault<'info> {
    #[account(
        init,
        seeds = [admin.key().as_ref(), vault_seed.to_le_bytes().as_ref()],
        bump,
        payer = admin,
        space = Vault::LEN
    )]
    pub vault: Box<Account<'info, Vault>>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn create_vault(
    ctx: Context<CreateVault>,
    min_deposit: u64,
    min_lockup: i64,
    target_yield_rate: u64,
    vault_seed: u64,
) -> Result<()> {
    let vault = &mut ctx.accounts.vault;

    vault.admin = *ctx.accounts.admin.key;
    vault.min_deposit = min_deposit;
    vault.min_lockup = min_lockup;
    vault.target_yield_rate = target_yield_rate;
    vault.total_receipt_supply = 0;

    Ok(())
}
