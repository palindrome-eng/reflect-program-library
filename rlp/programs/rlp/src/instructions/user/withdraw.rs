use anchor_lang::prelude::*;
use anchor_spl::token::Token;
use spl_math::precise_number::PreciseNumber;
use switchboard_solana::rust_decimal::prelude::ToPrimitive;
use crate::errors::InsuranceFundError;
use crate::states::*;
use crate::constants::*;
use anchor_spl::token::{ 
    Mint, 
    TokenAccount, 
    transfer,
    Transfer,
    burn,
    Burn
 };

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Copy)]
pub struct WithdrawArgs {
    pub liquidity_pool_id: u8,
    pub cooldown_id: u64,
}

pub fn withdraw<'a>(
    ctx: Context<'_, '_, 'a, 'a, Withdraw<'a>>,
    args: WithdrawArgs
) -> Result<()> {

    let WithdrawArgs {
        liquidity_pool_id: _,
        cooldown_id,
    } = args;

    let settings = &ctx.accounts.settings;
    let cooldown = &ctx.accounts.cooldown;
    let liquidity_pool = &ctx.accounts.liquidity_pool;
    let cooldown_lp_token_account = &ctx.accounts.cooldown_lp_token_account;
    let lp_token_mint = &ctx.accounts.lp_token_mint;
    let token_program = &ctx.accounts.token_program;
    let user = &ctx.accounts.user;

    let lp_token_amount = cooldown_lp_token_account.amount;
    let lp_token_supply = lp_token_mint.supply;

    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp as u64 > cooldown.unlock_ts,
        InsuranceFundError::CooldownInForce
    );

    require!(
        lp_token_amount > 0 && lp_token_supply > 0,
        InsuranceFundError::InvalidInput
    );

    let remaining_accounts = &ctx.remaining_accounts;
    require!(
        remaining_accounts.len() == settings.assets as usize * 3,
        InsuranceFundError::InvalidInput
    );

    // Liquidity pool signer seeds for transfers
    let lp_seeds = &[
        LIQUIDITY_POOL_SEED.as_bytes(),
        &liquidity_pool.index.to_le_bytes(),
        &[liquidity_pool.bump]
    ];

    // Iterate through remaining accounts to calculate and transfer user's share of each asset
    let mut i = 0;
    while i < remaining_accounts.len() {
        let pool_token_account_info = &remaining_accounts[i];
        
        require!(
            pool_token_account_info.owner == &anchor_spl::token::ID,
            InsuranceFundError::InvalidInput
        );

        let pool_token_account = TokenAccount::try_deserialize(&mut pool_token_account_info.try_borrow_mut_data()?.as_ref())
            .map_err(|_| InsuranceFundError::InvalidInput)?;

        require!(
            pool_token_account.owner == liquidity_pool.key(),
            InsuranceFundError::InvalidInput
        );

        let asset_info = &remaining_accounts[i + 1];
        
        require!(
            asset_info.owner == &crate::ID,
            InsuranceFundError::InvalidInput
        );
        
        let asset = Asset::try_deserialize(&mut asset_info.try_borrow_mut_data()?.as_ref())
            .map_err(|_| InsuranceFundError::InvalidInput)?;

        require!(
            asset.mint == pool_token_account.mint,
            InsuranceFundError::InvalidInput
        );
        
        let (expected_asset_pda, _) = Pubkey::find_program_address(
            &[
                ASSET_SEED.as_bytes(),
                &asset.mint.to_bytes()
            ],
            &crate::ID
        );

        require!(
            asset_info.key() == expected_asset_pda,
            InsuranceFundError::InvalidInput
        );

        // Verify this is the correct associated token account for the liquidity pool and asset
        let (expected_pool_token_account, _) = Pubkey::find_program_address(
            &[
                anchor_spl::associated_token::ID.as_ref(),
                liquidity_pool.key().as_ref(),
                anchor_spl::token::ID.as_ref(),
                asset.mint.as_ref(),
            ],
            &anchor_spl::associated_token::ID
        );

        require!(
            pool_token_account_info.key() == expected_pool_token_account,
            InsuranceFundError::InvalidInput
        );

        let user_token_account_info = &remaining_accounts[i + 2];
        
        require!(
            user_token_account_info.owner == &anchor_spl::token::ID,
            InsuranceFundError::InvalidInput
        );

        let user_token_account = TokenAccount::try_deserialize(&mut user_token_account_info.try_borrow_mut_data()?.as_ref())
            .map_err(|_| InsuranceFundError::InvalidInput)?;

        require!(
            user_token_account.owner == user.key(),
            InsuranceFundError::InvalidInput
        );

        require!(
            user_token_account.mint == asset.mint,
            InsuranceFundError::InvalidInput
        );

        let user_pool_share_amount = PreciseNumber::new(pool_token_account.amount as u128)
            .ok_or(InsuranceFundError::MathOverflow)?
            .checked_mul(
                &PreciseNumber::new(lp_token_amount as u128)
                .ok_or(InsuranceFundError::MathOverflow)?
            )
            .ok_or(InsuranceFundError::MathOverflow)?
            .checked_div(
                &PreciseNumber::new(lp_token_supply as u128)
                .ok_or(InsuranceFundError::MathOverflow)?
            )
            .ok_or(InsuranceFundError::MathOverflow)?
            .to_imprecise()
            .ok_or(InsuranceFundError::MathOverflow)?
            .to_u64()
            .ok_or(InsuranceFundError::MathOverflow)?;

        // Transfer user's share from pool to user
        if user_pool_share_amount > 0 {
            transfer(
                CpiContext::new_with_signer(
                    token_program.to_account_info(), 
                    Transfer {
                        from: pool_token_account_info.to_account_info(),
                        to: user_token_account_info.to_account_info(),
                        authority: liquidity_pool.to_account_info()
                    }, 
                    &[lp_seeds]
                ), 
                user_pool_share_amount
            )?;
        }

        i += 3;
    }

    // Burn the user's LP tokens
    burn(
        CpiContext::new_with_signer(
            token_program.to_account_info(), 
            Burn {
                authority: cooldown.to_account_info(),
                from: cooldown_lp_token_account.to_account_info(),
                mint: lp_token_mint.to_account_info()
            }, 
            &[&[
                COOLDOWN_SEED.as_bytes(),
                &cooldown_id.to_le_bytes(),
                &[ctx.bumps.cooldown]
            ]]
        ),
        lp_token_amount
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
    pub lp_token_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        associated_token::mint = lp_token_mint,
        associated_token::authority = cooldown,
    )]
    pub cooldown_lp_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        seeds = [
            COOLDOWN_SEED.as_bytes(),
            &args.cooldown_id.to_le_bytes(),
        ],
        bump,
        close = user,
        constraint = cooldown.liquidity_pool_id == args.liquidity_pool_id,
        constraint = cooldown.authority == user.key()
    )]
    pub cooldown: Account<'info, Cooldown>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub system_program: Option<Program<'info, System>>,
}