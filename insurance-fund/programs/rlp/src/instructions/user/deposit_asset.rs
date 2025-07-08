use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token, transfer, Transfer};
use spl_math::precise_number::PreciseNumber;
use crate::errors::InsuranceFundError;
use crate::states::{Asset, LiquidityPool, Settings};
use crate::constants::*;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct DepositAssetArgs {
    pub liquidity_pool_index: u64,
    pub amount: u64,
    pub min_lp_tokens: u64,
}

pub fn deposit_asset<'info>(
    ctx: Context<'_, 'info, 'info, 'info, DepositAsset<'info>>,
    args: DepositAssetArgs
) -> Result<()> {
    let DepositAssetArgs { liquidity_pool_index: _, amount, min_lp_tokens } = args;

    let signer = &ctx.accounts.signer;
    let liquidity_pool = &ctx.accounts.liquidity_pool;
    let lp_token = &ctx.accounts.lp_token;
    let token_program = &ctx.accounts.token_program;

    require!(
        amount > 0,
        crate::errors::InsuranceFundError::InvalidInput
    );

    let clock = Clock::get()?;

    // Calculate total USD value of the entire liquidity pool
    let total_pool_value_before = calculate_total_pool_value(
        &ctx.remaining_accounts,
        liquidity_pool,
        &clock
    )?;

    // Transfer tokens to pool
    liquidity_pool.deposit(
        signer,
        amount,
        &ctx.accounts.user_asset_account,
        &ctx.accounts.pool_asset_account,
        token_program
    )?;

    // Get asset price from oracle for the deposited asset
    let asset = &ctx.accounts.asset;
    let oracle = &ctx.accounts.oracle;
    let deposit_asset_price = asset.get_price(oracle, &clock)?;

    let deposit_value_precise = PreciseNumber::new(deposit_asset_price.mul(amount)?).ok_or(InsuranceFundError::MathOverflow)?;

    // Calculate LP tokens to mint based on total pool value
    let lp_tokens_to_mint = if lp_token.supply == 0 {
        // First deposit - mint tokens equal to the deposit value
        deposit_value_precise.floor()
            .ok_or(crate::errors::InsuranceFundError::MathOverflow)?
            .to_imprecise()
            .ok_or(crate::errors::InsuranceFundError::MathOverflow)?
            .try_into()
            .map_err(|_| crate::errors::InsuranceFundError::MathOverflow)?
    } else {
        // Calculate based on the proportion of deposit value to total pool value
        let lp_supply_precise = PreciseNumber::new(lp_token.supply as u128)
            .ok_or(crate::errors::InsuranceFundError::MathOverflow)?;
        let deposit_ratio = deposit_value_precise
            .checked_mul(&lp_supply_precise)
            .ok_or(crate::errors::InsuranceFundError::MathOverflow)?
            .checked_div(&total_pool_value_before)
            .ok_or(crate::errors::InsuranceFundError::MathOverflow)?;
        
        deposit_ratio.floor()
            .ok_or(crate::errors::InsuranceFundError::MathOverflow)?
            .to_imprecise()
            .ok_or(crate::errors::InsuranceFundError::MathOverflow)?
            .try_into()
            .map_err(|_| crate::errors::InsuranceFundError::MathOverflow)?
    };

    require!(
        min_lp_tokens <= lp_tokens_to_mint,
        InsuranceFundError::SlippageExceeded
    );

    // Mint LP tokens to user using the liquidity pool as authority
    liquidity_pool.mint_lp_token(
        lp_tokens_to_mint,
        liquidity_pool,
        lp_token,
        &ctx.accounts.user_lp_account,
        token_program
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(args: DepositAssetArgs)]
pub struct DepositAsset<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes(),
        ],
        bump,
        constraint = !settings.frozen @ InsuranceFundError::Frozen,
    )]
    pub settings: Account<'info, Settings>,

    #[account(
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            &args.liquidity_pool_index.to_le_bytes()
        ],
        bump,
        constraint = liquidity_pool.index == args.liquidity_pool_index,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        mut,
        address = liquidity_pool.lp_token
    )]
    pub lp_token: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = lp_token,
        associated_token::authority = signer,
    )]
    pub user_lp_account: Box<Account<'info, TokenAccount>>,

    #[account(
        seeds = [
            ASSET_SEED.as_bytes(),
            &asset_mint.key().to_bytes()
        ],
        bump,
    )]
    pub asset: Account<'info, Asset>,

    #[account(
        mut,
        address = asset.mint
    )]
    pub asset_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = asset_mint,
        token::authority = signer,
    )]
    pub user_asset_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        associated_token::mint = asset_mint,
        associated_token::authority = liquidity_pool,
    )]
    pub pool_asset_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: Oracle account for price feed
    #[account(
        constraint = asset.oracle.key() == &oracle.key()
    )]
    pub oracle: UncheckedAccount<'info>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub system_program: Program<'info, System>,
}

/// Calculate total USD value of the entire liquidity pool
fn calculate_total_pool_value<'a>(
    remaining_accounts: &'a [AccountInfo<'a>],
    liquidity_pool: &'a Account<'a, LiquidityPool>,
    clock: &Clock,
) -> Result<PreciseNumber> {
    let mut total_pool_value = PreciseNumber::new(0)
        .ok_or(crate::errors::InsuranceFundError::MathOverflow)?;
    
    // Validate that remaining_accounts length is a multiple of 3
    require!(
        remaining_accounts.len() % 3 == 0,
        crate::errors::InsuranceFundError::InvalidInput
    );
    
    // Iterate over remaining_accounts to get all pool token accounts and their asset info
    // The remaining_accounts should be structured as: [token_account, asset, oracle, token_account, asset, oracle, ...]
    let mut i = 0;
    while i < remaining_accounts.len() {
        let token_account_info = &remaining_accounts[i];
        
        // Verify this is a token account
        require!(
            token_account_info.owner == &anchor_spl::token::ID,
            crate::errors::InsuranceFundError::InvalidInput
        );

        // Deserialize as TokenAccount - fail if invalid
        let token_account = TokenAccount::try_deserialize(&mut token_account_info.try_borrow_mut_data()?.as_ref())
            .map_err(|_| crate::errors::InsuranceFundError::InvalidInput)?;

        // Verify token account belongs to the liquidity pool
        require!(
            token_account.owner == liquidity_pool.key(),
            crate::errors::InsuranceFundError::InvalidInput
        );

        // Get asset info (next account)
        let asset_info = &remaining_accounts[i + 1];
        
        // Verify asset account is owned by our program
        require!(
            asset_info.owner == &crate::ID,
            crate::errors::InsuranceFundError::InvalidInput
        );
        
        // Deserialize as Asset - fail if invalid
        let asset = Asset::try_deserialize(&mut asset_info.try_borrow_mut_data()?.as_ref())
            .map_err(|_| crate::errors::InsuranceFundError::InvalidInput)?;

        // Verify asset mint matches token account mint
        require!(
            asset.mint == token_account.mint,
            crate::errors::InsuranceFundError::InvalidInput
        );
        
        // Verify asset account can be derived from the mint
        let (expected_asset_pda, _) = Pubkey::find_program_address(
            &[
                crate::constants::ASSET_SEED.as_bytes(),
                &asset.mint.to_bytes()
            ],
            &crate::ID
        );

        require!(
            asset_info.key() == expected_asset_pda,
            crate::errors::InsuranceFundError::InvalidInput
        );

        // Get oracle info (next account)
        let oracle_info = &remaining_accounts[i + 2];

        // Verify oracle account matches the one stored in the asset
        require!(
            oracle_info.key() == *asset.oracle.key(),
            crate::errors::InsuranceFundError::InvalidInput
        );

        // Get price for this asset - fail if invalid
        let asset_price = asset.get_price(oracle_info, clock)
            .map_err(|_| crate::errors::InsuranceFundError::InvalidInput)?;

        // Calculate value for this token using PreciseNumber
        let token_balance = token_account.amount;
        if token_balance > 0 {
            let token_value_precise = PreciseNumber::new(asset_price.mul(token_balance)?)
                .ok_or(crate::errors::InsuranceFundError::MathOverflow)?;
            total_pool_value = total_pool_value.checked_add(&token_value_precise)
                .ok_or(crate::errors::InsuranceFundError::MathOverflow)?;
        }

        // Move to next set of accounts (token_account, asset, oracle)
        i += 3;
    }

    Ok(total_pool_value)
} 