use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use crate::errors::InsuranceFundError;
use crate::states::*;
use crate::constants::*;
use anchor_spl::token::{ Mint, TokenAccount, transfer, Transfer };

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub struct WithdrawArgs {
    pub lockup_id: u64,
    pub deposit_id: u64,
}

pub fn withdraw(
    ctx: Context<Withdraw>,
    args: WithdrawArgs
) -> Result<()> {

    let WithdrawArgs {
        deposit_id: _,
        lockup_id,
    } = args;

    let token_program = &ctx.accounts.token_program;
    let lockup = &ctx.accounts.lockup;
    let asset_reward_pool = &ctx.accounts.asset_reward_pool;
    let user_reward_ata = &ctx.accounts.user_reward_ata;
    let cooldown = &ctx.accounts.cooldown;
    let lockup_asset_vault = &ctx.accounts.lockup_asset_vault;
    let user_asset_ata = &ctx.accounts.user_asset_ata;

    let seeds = &[
        LOCKUP_SEED.as_bytes(),
        &lockup_id.to_le_bytes(),
        &[ctx.bumps.lockup]
    ];

    let reward = match cooldown.rewards {
        CooldownRewards::Single(base) => base,
        CooldownRewards::Dual([base, _]) => base
    };

    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(), 
            Transfer {
                authority: lockup.to_account_info(),
                from: asset_reward_pool.to_account_info(),
                to: user_reward_ata.to_account_info()
            }, 
        &[seeds]
        ), 
        reward
    )?;

    // Transfer base amount
    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(), 
            Transfer {
                from: lockup_asset_vault.to_account_info(),
                to: user_asset_ata.to_account_info(),
                authority: lockup.to_account_info()
            }, 
            &[seeds]
        ), 
        cooldown.base_amount
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: WithdrawArgs
)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        constraint = !settings.frozen @ InsuranceFundError::Frozen
    )]
    pub settings: Account<'info, Settings>,

    #[account(
        mut,
        seeds = [
            LOCKUP_SEED.as_bytes(),
            &args.lockup_id.to_le_bytes()
        ],
        bump,
        constraint = !lockup.locked @ InsuranceFundError::DepositsLocked,
    )]
    pub lockup: Account<'info, Lockup>,

    // All these checks were run during initializing cooldown
    // not sure if we need to check them again, likely not.
    #[account(
        mut,
        seeds = [
            DEPOSIT_SEED.as_bytes(),
            lockup.key().as_ref(),
            &args.deposit_id.to_le_bytes()
        ],
        bump,
        // Cannot withdraw before unlock timestamp
        constraint = deposit.unlock_ts <= (clock.unix_timestamp as u64) @ InsuranceFundError::LockupInForce,
        // Enforce account ownership
        has_one = user
    )]
    pub deposit: Account<'info, Deposit>,

    #[account(
        mut,
        seeds = [
            COOLDOWN_SEED.as_bytes(),
            &deposit.key().to_bytes(),
        ],
        bump,
    )]
    pub cooldown: Account<'info, Cooldown>,

    #[account(
        mut,
        address = lockup.asset,
    )]
    pub asset_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [
            ASSET_SEED.as_bytes(),
            &asset_mint.key().to_bytes()
        ],
        bump,
        constraint = asset.mint == asset_mint.key()
    )]
    pub asset: Account<'info, Asset>,

    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = asset_mint
    )]
    pub user_asset_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        address = settings.reward_config.main
    )]
    pub reward_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = reward_mint
    )]
    pub user_reward_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [
            VAULT_SEED.as_bytes(),
            lockup.key().as_ref(),
            asset_mint.key().as_ref(),
        ],
        bump,
        // This should never happen
        constraint = lockup_asset_vault.amount >= cooldown.base_amount @ InsuranceFundError::NotEnoughFunds,
        token::mint = asset_mint,
        token::authority = lockup
    )]
    pub lockup_asset_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [
            REWARD_POOL_SEED.as_bytes(),
            lockup.key().as_ref(),
            reward_mint.key().as_ref(),
        ],
        bump,
        token::mint = reward_mint,
        token::authority = lockup,
    )]
    pub asset_reward_pool: Box<Account<'info, TokenAccount>>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub clock: Sysvar<'info, Clock>,
}