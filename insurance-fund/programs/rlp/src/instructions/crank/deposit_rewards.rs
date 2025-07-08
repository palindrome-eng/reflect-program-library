use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::InsuranceFundError;
use crate::states::*;
use crate::events::*;
use anchor_spl::token::{
    Mint,
    TokenAccount,
    transfer,
    Transfer,
    Token
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DepositRewardsArgs {
    pub lockup_id: u64,
    pub amount: u64
}

pub fn deposit_rewards(
    ctx: Context<DepositRewards>,
    args: DepositRewardsArgs
) -> Result<()> {
    let DepositRewardsArgs {
        amount,
        lockup_id: _
    } = args;

    let lockup = &mut ctx.accounts.lockup;
    let caller = &ctx.accounts.caller;
    let reward_mint = &ctx.accounts.reward_mint;
    let caller_reward_ata = &ctx.accounts.caller_reward_ata;
    let asset_reward_pool = &ctx.accounts.asset_reward_pool;
    let token_program = &ctx.accounts.token_program;
    let receipt_token_mint = &ctx.accounts.receipt_token_mint;
    let lockup_cooldown_vault = &ctx.accounts.lockup_cooldown_vault;

    let total_receipts = receipt_token_mint.supply;
    let cooldown_receipts = lockup_cooldown_vault.amount;

    let active_receipts = total_receipts
        .checked_sub(cooldown_receipts)
        .ok_or(InsuranceFundError::MathOverflow)?;


    lockup.increase_exchange_rate_accumulator(
        active_receipts, 
        amount
    )?;

    transfer(
        CpiContext::new(
            token_program.to_account_info(), 
            Transfer {
                to: asset_reward_pool.to_account_info(),
                from: caller_reward_ata.to_account_info(),
                authority: caller.to_account_info()
            }
        ),
        amount
    )?;

    emit!(DepositRewardEvent {
        from: caller.key(),
        asset: reward_mint.key(),
        amount: amount
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: DepositRewardsArgs
)]
pub struct DepositRewards<'info> {
    #[account(
        mut
    )]
    pub caller: Signer<'info>,

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
        bump
    )]
    pub lockup: Account<'info, Lockup>,

    #[account(
        mut,
        address = settings.reward_config.main
    )]
    pub reward_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::authority = caller,
        associated_token::mint = reward_mint,
    )]
    pub caller_reward_ata: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = caller,
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
        mut,
        address = lockup.receipt_mint
    )]
    pub receipt_token_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            COOLDOWN_VAULT_SEED.as_bytes(),
            lockup.key().as_ref(),
            receipt_token_mint.key().as_ref(),
        ],
        bump,
        token::mint = receipt_token_mint,
        token::authority = lockup,
    )]
    pub lockup_cooldown_vault: Account<'info, TokenAccount>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub system_program: Program<'info, System>
}