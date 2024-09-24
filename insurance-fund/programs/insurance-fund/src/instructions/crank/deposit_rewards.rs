use anchor_lang::prelude::*;
use anchor_spl::token;
use crate::constants::*;
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
        lockup_id
    } = args;

    let caller = &ctx.accounts.caller;
    let asset_mint = &ctx.accounts.asset_mint;
    let caller_asset_ata = &ctx.accounts.caller_asset_ata;
    let asset_reward_pool = &ctx.accounts.asset_reward_pool;
    let token_program = &ctx.accounts.token_program;

    transfer(
        CpiContext::new(
            token_program.to_account_info(), 
            Transfer {
                to: asset_reward_pool.to_account_info(),
                from: caller_asset_ata.to_account_info(),
                authority: caller.to_account_info()
            }
        ),
        amount
    )?;

    emit!(DepositRewardEvent {
        from: caller.key(),
        asset: asset_mint.key(),
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
            LOCKUP_SEED.as_bytes(),
            &args.lockup_id.to_le_bytes()
        ],
        bump
    )]
    pub lockup: Account<'info, Lockup>,

    #[account(
        mut,
        address = lockup.asset
    )]
    pub asset_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::authority = caller,
        associated_token::mint = asset_mint,
    )]
    pub caller_asset_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            REWARD_POOL_SEED.as_bytes(),
            lockup.key().as_ref(),
            asset_mint.key().as_ref(),
        ],
        bump,
        token::mint = asset_mint,
        token::authority = lockup,
    )]
    pub asset_reward_pool: Account<'info, TokenAccount>,

    #[account()]
    pub token_program: Program<'info, Token>,
}