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
use anchor_spl::associated_token::AssociatedToken;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DepositRewardsArgs {
    pub amount: u64
}

pub fn deposit_rewards(
    ctx: Context<DepositRewards>,
    args: DepositRewardsArgs
) -> Result<()> {
    let DepositRewardsArgs {
        amount,
    } = args;

    let signer = &ctx.accounts.signer;
    let asset_mint = &ctx.accounts.asset_mint;
    let asset = &ctx.accounts.asset;
    let asset_pool = &ctx.accounts.asset_pool;
    let signer_asset_token_account = &ctx.accounts.signer_asset_token_account;
    let token_program = &ctx.accounts.token_program;

    transfer(
        CpiContext::new(
            token_program.to_account_info(), 
            Transfer {
                to: asset_pool.to_account_info(),
                from: signer_asset_token_account.to_account_info(),
                authority: signer.to_account_info()
            }
        ),
        amount
    )?;

    emit!(DepositRewardEvent {
        authority: signer.key(),
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
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump
    )]
    pub settings: Account<'info, Settings>,

    #[account(
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            &signer.key().to_bytes()
        ],
        bump,
        constraint = permissions.can_perform_protocol_action(Action::DepositRewards, &settings.access_control) @ InsuranceFundError::PermissionsTooLow
    )]
    pub permissions: Account<'info, UserPermissions>,

    #[account(
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            &liquidity_pool.index.to_le_bytes()
        ],
        bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        mut,
        token::authority = signer,
        token::mint = asset_mint,
    )]
    pub signer_asset_token_account: Account<'info, TokenAccount>,

    #[account(
        address = asset.mint
    )]
    pub asset_mint: Account<'info, Mint>,

    #[account(
        seeds = [
            ASSET_SEED.as_bytes(),
            &asset_mint.key().to_bytes()
        ],
        bump
    )]
    pub asset: Account<'info, Asset>,

    #[account(
        init_if_needed,
        payer = signer,
        associated_token::mint = asset_mint,
        associated_token::authority = liquidity_pool,
    )]
    pub asset_pool: Account<'info, TokenAccount>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account()]
    pub system_program: Program<'info, System>
}