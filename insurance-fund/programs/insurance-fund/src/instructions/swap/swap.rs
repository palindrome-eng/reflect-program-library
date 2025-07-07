use std::ops::Div;

use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{transfer, Mint, Token, TokenAccount, Transfer}};
use crate::{constants::{ASSET_SEED, LIQUIDITY_POOL_SEED}, errors::InsuranceFundError, states::{liquidity_pool, Admin, Asset, LiquidityPool}};

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct SwapLpArgs {
    pub amount_in: u64,
    pub min_out: Option<u64>
}

pub fn swap_lp(
    ctx: Context<SwapLp>,
    args: SwapLpArgs
) -> Result<()> {
    let SwapLpArgs {
        min_out,
        amount_in
    } = args;

    let clock = &Clock::get()?;

    let signer = &ctx.accounts.signer;
    let liquidity_pool = &ctx.accounts.liquidity_pool;

    let token_a = &ctx.accounts.token_a;
    let token_b = &ctx.accounts.token_b;

    let token_a_asset = &ctx.accounts.token_a_asset;
    let token_b_asset = &ctx.accounts.token_b_asset;

    let token_a_oracle = &ctx.accounts.token_a_oracle;
    let token_b_oracle = &ctx.accounts.token_b_oracle;

    let token_a_price = token_a_asset.get_price(token_a_oracle, clock)?;
    let token_b_price = token_b_asset.get_price(token_b_oracle, clock)?;

    let token_a_signer_account = &ctx.accounts.token_a_signer_account;
    let token_b_signer_account = &ctx.accounts.token_b_signer_account;

    let token_a_pool = &ctx.accounts.token_a_pool;
    let token_b_pool = &ctx.accounts.token_b_pool;

    let token_program = &ctx.accounts.token_program;

    let (
        in_price, 
        out_price,
    ) = if a_to_b { 
        (
            &token_a_price, 
            &token_b_price,
        ) 
    } else { 
        (
            &token_b_price, 
            &token_a_price,
        ) 
    };

    let amount_out: u64 = in_price
        .mul(amount_in)?
        .div(out_price
            .mul(1)?
        )
        .try_into()
        .map_err(|_| InsuranceFundError::MathOverflow)?;

    match min_out {
        Some(amount) => {
            require!(
                amount_out >= amount,
                InsuranceFundError::SlippageExceeded
            );
        },
        None => {}
    }

    let token_a_key = token_a.key();
    let token_b_key = token_b.key();

    let lp_seeds = &[
        LIQUIDITY_POOL_SEED.as_bytes(),
        token_a_key.as_ref(),
        token_b_key.as_ref()
    ];

    if a_to_b {
        transfer(
            CpiContext::new(
                token_program.to_account_info(), 
                Transfer { 
                    from: token_a_signer_account.to_account_info(), 
                    to: token_a_pool.to_account_info(), 
                    authority: signer.to_account_info() 
                }
            ),
            amount_in
        )?;

        transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(), 
                Transfer { 
                    from: token_b_pool.to_account_info(), 
                    to: token_b_signer_account.to_account_info(), 
                    authority: liquidity_pool.to_account_info()
                }, 
                &[lp_seeds]
            ), 
            amount_out
        )?;
    } else {
        transfer(
            CpiContext::new(
                token_program.to_account_info(), 
                Transfer { 
                    from: token_b_signer_account.to_account_info(), 
                    to: token_b_pool.to_account_info(), 
                    authority: signer.to_account_info() 
                }
            ),
            amount_in
        )?;

        transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(), 
                Transfer { 
                    from: token_a_pool.to_account_info(), 
                    to: token_a_signer_account.to_account_info(), 
                    authority: liquidity_pool.to_account_info() 
                }, 
                &[lp_seeds]
            ), 
            amount_out
        )?;
    }

    Ok(())
}

#[derive(Accounts)]
pub struct SwapLp<'info> {
    #[account(
        mut
    )]
    pub signer: Signer<'info>,

    #[account()]
    pub admin: Account<'info, Admin>,

    #[account(
        has_one = token_a,
        has_one = token_b
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        address = liquidity_pool.token_a
    )]
    pub token_a: Account<'info, Mint>,

    #[account(
        seeds = [
            ASSET_SEED.as_bytes(),
            token_a.key().as_ref()
        ],
        bump
    )]
    pub token_a_asset: Account<'info, Asset>,

    /// CHECK: Directly checking the address
    #[account(
        address = *token_a_asset.oracle.key()
    )]
    pub token_a_oracle: AccountInfo<'info>,

    #[account(
        address = liquidity_pool.token_b
    )]
    pub token_b: Account<'info, Mint>,

    #[account(
        seeds = [
            ASSET_SEED.as_bytes(),
            token_b.key().as_ref()
        ],
        bump
    )]
    pub token_b_asset: Account<'info, Asset>,

    /// CHECK: Directly checking the address
    #[account(
        address = *token_b_asset.oracle.key()
    )]
    pub token_b_oracle: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::authority = liquidity_pool,
        associated_token::mint = token_a
    )]
    pub token_a_pool: Account<'info, TokenAccount>,

    #[account(
        mut,
        associated_token::authority = liquidity_pool,
        associated_token::mint = token_b
    )]
    pub token_b_pool: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = token_a,
        token::authority = signer,
    )]
    pub token_a_signer_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::mint = token_b,
        token::authority = signer,
    )]
    pub token_b_signer_account: Account<'info, TokenAccount>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub associated_token_program: Program<'info, AssociatedToken>,
}