use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;
use crate::errors::InsuranceFundError;
use anchor_spl::token::{
    Token,
    TokenAccount,
    Mint
};

pub fn initialize_lockup_vaults(
    ctx: Context<InitializeLockupVaults>,
    lockup_id: u64
) -> Result<()> {
    msg!("check 0");
    
    let settings = &mut ctx.accounts.settings;
    let lockup = &mut ctx.accounts.lockup;
    let asset_mint = &ctx.accounts.asset_mint;
    let signer = &ctx.accounts.signer;
    let signer = &ctx.accounts.signer;
    let lockup_hot_vault = &ctx.accounts.lockup_hot_vault;
    let lockup_cold_vault = &ctx.accounts.lockup_cold_vault;
    let token_program = &ctx.accounts.token_program;
    let system_program = &ctx.accounts.system_program;

    msg!("check 1");

    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(165);

    msg!("check 2");

    lockup.initialize_hot_vault(
        &signer, 
        &lockup, 
        &asset_mint, 
        &lockup_hot_vault, 
        &token_program, 
        &system_program, 
        ctx.bumps.lockup_hot_vault, 
        lamports
    )?;

    lockup.initialize_cold_vault(
        &signer, 
        &lockup, 
        &asset_mint, 
        &lockup_cold_vault, 
        &token_program, 
        &system_program, 
        ctx.bumps.lockup_cold_vault, 
        lamports
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    lockup_id: u64
)]
pub struct InitializeLockupVaults<'info> {
    #[account(
        mut,
    )]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            ADMIN_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump,
        constraint = admin.has_permissions(Permissions::AssetsAndLockups) @ InsuranceFundError::InvalidSigner,
    )]
    pub admin: Box<Account<'info, Admin>>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        constraint = !settings.frozen @ InsuranceFundError::Frozen
    )]
    pub settings: Box<Account<'info, Settings>>,

    #[account(
        mut,
        seeds = [
            LOCKUP_SEED.as_bytes(),
            &lockup_id.to_le_bytes()
        ],
        bump,
    )]
    pub lockup: Box<Account<'info, Lockup>>,

    /// CHECK: Directly checking address
    #[account(
        mut,
        address = lockup.asset_mint
    )]
    pub asset_mint: AccountInfo<'info>,

    /// CHECK: Only checking seeds. Initializing it later.
    #[account(
        mut,
        seeds = [
            COLD_VAULT_SEED.as_bytes(),
            lockup.key().as_ref(),
            asset_mint.key().as_ref(),
        ],
        bump,
    )]
    pub lockup_cold_vault: AccountInfo<'info>,

    /// CHECK: Only checking seeds. Initializing it later.
    #[account(
        mut,
        seeds = [
            HOT_VAULT_SEED.as_bytes(),
            lockup.key().as_ref(),
            asset_mint.key().as_ref(),
        ],
        bump,
    )]
    pub lockup_hot_vault: AccountInfo<'info>,

    #[account(
        address = settings.reward_config.main
    )]
    pub reward_mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = signer,
        seeds = [
            REWARD_POOL_SEED.as_bytes(),
            lockup.key().as_ref(),
            reward_mint.key().as_ref(),
        ],
        bump,
        token::mint = reward_mint,
        token::authority = lockup,
    )]
    pub asset_reward_pool: Account<'info, TokenAccount>,

    #[account(
        address = Token::id()
    )]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub system_program: Program<'info, System>
}