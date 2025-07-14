use anchor_lang::prelude::*;
use crate::constants::*;
use crate::events::RequestWithdrawEvent;
use crate::states::*;
use crate::errors::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{
    Mint,
    TokenAccount,
    Token,
    transfer,
    Transfer
};
use crate::helpers::action_check_protocol;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct RequestWithdrawalArgs {
    pub liquidity_pool_id: u8,
    pub amount: u64
}

pub fn request_withdrawal(
    ctx: Context<RequestWithdrawal>,
    args: RequestWithdrawalArgs
) -> Result<()> {
    let RequestWithdrawalArgs {
        liquidity_pool_id,
        amount
    } = args;

    let settings = &ctx.accounts.settings;
    let permissions = &ctx.accounts.permissions;

    action_check_protocol(
        Action::Withdraw,
        permissions.as_deref(),
        &settings.access_control
    )?;

    let signer = &ctx.accounts.signer;
    let liquidity_pool = &ctx.accounts.liquidity_pool;
    let cooldown = &mut ctx.accounts.cooldown;
    let token_program = &ctx.accounts.token_program;

    cooldown.set_inner(Cooldown {
        liquidity_pool_id,
        authority: signer.key(),
        ..Default::default()
    });

    cooldown.lock(liquidity_pool.cooldown_duration)?;

    let signer_lp_token_account = &ctx.accounts.signer_lp_token_account;
    let cooldown_lp_token_account = &ctx.accounts.cooldown_lp_token_account;

    transfer(
        CpiContext::new(
            token_program.to_account_info(), 
            Transfer {
                from: signer_lp_token_account.to_account_info(),
                to: cooldown_lp_token_account.to_account_info(),
                authority: signer.to_account_info()
            }
        ),
        amount
    )?;

    emit!(RequestWithdrawEvent {
        amount,
        authority: signer.key(),
        liquidity_pool_id
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: RequestWithdrawalArgs
)]
pub struct RequestWithdrawal<'info> {
    #[account(
        mut,
    )]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        constraint = !settings.access_control.killswitch.is_frozen(&Action::Withdraw) @ InsuranceFundError::Frozen
    )]
    pub settings: Account<'info, Settings>,

    #[account(
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump = permissions.bump
    )]
    pub permissions: Option<Account<'info, UserPermissions>>,

    #[account(
        mut,
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            &args.liquidity_pool_id.to_le_bytes()
        ],
        bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        mut,
        address = liquidity_pool.lp_token
    )]
    pub lp_token_mint: Account<'info, Mint>,

    #[account(
        mut,
        token::mint = lp_token_mint,
        token::authority = signer,
    )]
    pub signer_lp_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        seeds = [
            COOLDOWN_SEED.as_bytes(),
            &liquidity_pool.cooldowns.to_le_bytes(),
        ],
        bump,
        payer = signer,
        space = 8 + Cooldown::INIT_SPACE,
    )]
    pub cooldown: Account<'info, Cooldown>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = lp_token_mint,
        associated_token::authority = cooldown,
    )]
    pub cooldown_lp_token_account: Account<'info, TokenAccount>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub associated_token_program: Program<'info, AssociatedToken>,

    #[account()]
    pub system_program: Program<'info, System>,
}