use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};
use crate::states::*;
use crate::constants::*;
use crate::errors::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeLpTokenAccountArgs {
    pub liquidity_pool_index: u8,
}

pub fn initialize_lp_token_account(
    ctx: Context<InitializeLpTokenAccount>,
    args: InitializeLpTokenAccountArgs,
) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
#[instruction(args: InitializeLpTokenAccountArgs)]
pub struct InitializeLpTokenAccount<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump,
        constraint = admin.can_perform_protocol_action(Action::InitializeLiquidityPool, &settings.access_control) @ InsuranceFundError::InvalidSigner,
    )]
    pub admin: Account<'info, UserPermissions>,

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
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            &args.liquidity_pool_index.to_le_bytes()
        ],
        bump,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        constraint = asset.mint == mint.key() @ InsuranceFundError::InvalidInput,
        seeds = [
            ASSET_SEED.as_bytes(),
            &asset.mint.to_bytes()
        ],
        bump,
    )]
    pub asset: Account<'info, Asset>,

    #[account(
        constraint = mint.key() == asset.mint @ InsuranceFundError::InvalidInput,
    )]
    pub mint: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = signer,
        associated_token::mint = mint,
        associated_token::authority = liquidity_pool,
    )]
    pub lp_mint_token_account: Box<Account<'info, TokenAccount>>,

    #[account()]
    pub system_program: Program<'info, System>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub associated_token_program: Program<'info, AssociatedToken>,
}
