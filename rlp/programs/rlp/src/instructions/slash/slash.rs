use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;
use crate::errors::InsuranceFundError;
use anchor_spl::token::{
    Mint,
    TokenAccount,
    Token,
    transfer,
    Transfer
};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SlashArgs {
    liquidity_pool_id: u8,
    amount: u64,
}

pub fn slash(
    ctx: Context<Slash>,
    args: SlashArgs
) -> Result<()> {
    let token_program = &ctx.accounts.token_program;
    let destination = &ctx.accounts.destination;
    let liquidity_pool = &ctx.accounts.liquidity_pool;
    let liquidity_pool_token_account = &ctx.accounts.liquidity_pool_token_account;

    let SlashArgs {
        amount,
        liquidity_pool_id
    } = args;

    let seeds = &[
        LIQUIDITY_POOL_SEED.as_bytes(),
        &liquidity_pool_id.to_le_bytes(),
        &[liquidity_pool.bump]
    ];
    
    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                authority: liquidity_pool.to_account_info(),
                from: liquidity_pool_token_account.to_account_info(),
                to: destination.to_account_info(),
            },
            &[seeds]
        ),
        amount
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: SlashArgs
)]
pub struct Slash<'info> {
    #[account(
        mut,
    )]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump,
        constraint = permissions.can_perform_protocol_action(Action::Slash, &settings.access_control) @ InsuranceFundError::PermissionsTooLow,
    )]
    pub permissions: Account<'info, UserPermissions>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        constraint = !settings.access_control.killswitch.is_frozen(&Action::Slash) @ InsuranceFundError::Frozen,
    )]
    pub settings: Account<'info, Settings>,

    #[account(
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            &args.liquidity_pool_id.to_le_bytes()
        ],
        bump,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        mut
    )]
    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            ASSET_SEED.as_bytes(),
            mint.key().as_ref()
        ],
        bump
    )]
    pub asset: Account<'info, Asset>,

    #[account(
        mut,
        token::mint = mint,
        token::authority = liquidity_pool,
    )]
    pub liquidity_pool_token_account: Account<'info, TokenAccount>,

    #[account(
        token::mint = mint,
    )]
    pub destination: Account<'info, TokenAccount>,

    #[account()]
    pub token_program: Program<'info, Token>,
}