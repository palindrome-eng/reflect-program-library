use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token};
use crate::states::{liquidity_pool, Asset, Deposit, LiquidityPool, LpLockup};
use crate::constants::*;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct DepositAndLockLpArgs {
    pub token_a_amount: u64,
    pub token_b_amount: u64,
}

pub fn deposit_and_lock_lp(
    ctx: Context<DepositAndLockLp>,
    args: DepositAndLockLpArgs
) -> Result<()> {

    let DepositAndLockLpArgs {
        token_a_amount,
        token_b_amount
    } = args;

    let signer = &ctx.accounts.signer;
    let token_a_user_account = &ctx.accounts.user_token_a_account;
    let token_b_user_account = &ctx.accounts.user_token_b_account;
    let token_a_pool = &ctx.accounts.token_a_pool;
    let token_b_pool = &ctx.accounts.token_b_pool;
    let liquidity_pool = &ctx.accounts.liquidity_pool;
    let lp_token = &ctx.accounts.lp_token;
    let token_program = &ctx.accounts.token_program;
    let system_program = &ctx.accounts.system_program;
    let lockup_lp_token_vault = &ctx.accounts.lockup_lp_token_vault;
    
    let deposit = &ctx.accounts.position;
    let receipt_token = &ctx.accounts.receipt_token;
    let deposit_receipt_token_account = &ctx.accounts.deposit_receipt_token_account;

    let clock = &Clock::get()?;

    let token_a_asset = &ctx.accounts.token_a_asset;
    let token_a_oracle = &ctx.accounts.token_a_oracle;
    let token_a_price = token_a_asset.get_price(&token_a_oracle, &clock)?;

    let token_b_asset = &ctx.accounts.token_b_asset;
    let token_b_oracle = &ctx.accounts.token_b_oracle;
    let token_b_price = token_b_asset.get_price(&token_b_oracle, &clock)?;

    liquidity_pool.deposit(
        signer, 
        token_a_amount, 
        token_a_user_account, 
        token_a_pool, 
        token_b_amount, 
        token_b_user_account, 
        token_b_pool, 
        token_program
    )?;

    let lp_tokens = liquidity_pool.compute_lp_tokens_on_deposit(
        lp_token.supply, 
        token_a_pool.amount, 
        token_b_pool.amount, 
        token_a_amount, 
        token_b_amount, 
        token_a_price, 
        token_b_price
    )?;

    // deposit.create_deposit_receipt_token_account(
    //     signer, 
    //     deposit_receipt_token_account, 
    //     ctx.bumps.deposit_receipt_token_account, 
    //     deposit, 
    //     receipt_token, 
    //     token_program, 
    //     system_program
    // )?;

    liquidity_pool.mint_lp_token(
        lp_tokens, 
        liquidity_pool, 
        lp_token, 
        lockup_lp_token_vault, 
        token_program
    )?;

    Ok(())
}

#[derive(Accounts)]
pub struct DepositAndLockLp<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            token_a.key().as_ref(),
            token_b.key().as_ref(),
        ],
        bump,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        mut,
        seeds = [
            LIQUIDITY_POOL_LOCKUP_SEED.as_bytes(),
            liquidity_pool.key().as_ref(),
            &lp_lockup.duration.to_le_bytes(),
        ],
        bump
    )]
    pub lp_lockup: Account<'info, LpLockup>,

    #[account(
        init,
        payer = signer,
        seeds = [
            DEPOSIT_SEED.as_bytes(),
            lp_lockup.key().as_ref(),
            &lp_lockup.deposits.to_le_bytes()
        ],
        bump,
        space = 8 + Deposit::INIT_SPACE
    )]
    pub position: Account<'info, Deposit>,

    #[account(
        address = lp_lockup.receipt_token
    )]
    pub receipt_token: Box<Account<'info, Mint>>,

    /// CHECK: initializing this manually
    #[account(
        mut,
        seeds = [
            DEPOSIT_RECEIPT_VAULT_SEED.as_bytes(),
            position.key().as_ref(),
            receipt_token.key().as_ref(),
        ],
        bump
    )]
    pub deposit_receipt_token_account: AccountInfo<'info>,

    #[account(
        mut,
        address = liquidity_pool.lp_token
    )]
    pub lp_token: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = lp_token,
        associated_token::authority = lp_lockup
    )]
    pub lockup_lp_token_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        address = liquidity_pool.token_a
    )]
    pub token_a: Box<Account<'info, Mint>>,

    #[account(
        seeds = [
            ASSET_SEED.as_bytes(),
            &token_a.key().to_bytes()
        ],
        bump
    )]
    pub token_a_asset: Account<'info, Asset>,

    /// CHECK: Directly validating address
    #[account(
        constraint = token_a_asset.oracle.key().eq(&token_a_oracle.key())
    )]
    pub token_a_oracle: UncheckedAccount<'info>,

    #[account(
        seeds = [
            ASSET_SEED.as_bytes(),
            &token_b.key().to_bytes()
        ],
        bump
    )]
    pub token_b_asset: Account<'info, Asset>,

    /// CHECK: Directly validating address
    #[account(
        constraint = token_a_asset.oracle.key().eq(&token_a_oracle.key())
    )]
    pub token_b_oracle: UncheckedAccount<'info>,

    #[account(
        address = liquidity_pool.token_b
    )]
    pub token_b: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = token_a,
        associated_token::authority = liquidity_pool
    )]
    pub token_a_pool: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = token_b,
        associated_token::authority = liquidity_pool
    )]
    pub token_b_pool: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = token_a,
        token::authority = signer,
    )]
    pub user_token_a_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = token_b,
        token::authority = signer,
    )]
    pub user_token_b_account: Box<Account<'info, TokenAccount>>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub system_program: Program<'info, System>,
}