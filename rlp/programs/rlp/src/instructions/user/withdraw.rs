use anchor_lang::prelude::*;
use anchor_spl::token::close_account;
use anchor_spl::token::CloseAccount;
use anchor_spl::token::Token;
use spl_math::precise_number::PreciseNumber;
use crate::errors::RlpError;
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
use crate::helpers::{
    load_assets,
    load_reserves,
    load_user_token_accounts
};
use crate::events::WithdrawEvent;

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
    let signer = &ctx.accounts.signer;

    let lp_token_amount = cooldown_lp_token_account.amount;
    let lp_token_supply = lp_token_mint.supply;

    let clock = Clock::get()?;

    require!(
        clock.unix_timestamp as u64 >= cooldown.unlock_ts,
        RlpError::CooldownInForce
    );

    require!(
        lp_token_amount > 0 && lp_token_supply > 0,
        RlpError::InvalidInput
    );

    let remaining_accounts = &ctx.remaining_accounts;
    require!(
        remaining_accounts.len() == settings.assets as usize * 3,
        RlpError::InvalidInput
    );

    let lp_seeds = &[
        LIQUIDITY_POOL_SEED.as_bytes(),
        &liquidity_pool.index.to_le_bytes(),
        &[liquidity_pool.bump]
    ];

    let assets: Vec<(Pubkey, Asset)> = load_assets(settings, remaining_accounts)?;
    let asset_datas = assets.iter().map(|(_, asset)| asset).collect::<Vec<&Asset>>();
    let reserves = load_reserves(liquidity_pool, &asset_datas, remaining_accounts)?;
    let user_token_accounts = load_user_token_accounts(signer, &asset_datas, remaining_accounts)?;

    for i in 0..assets.len() {
        let (reserve_key, reserve) = &reserves[i];
        let (user_token_account_key, _) = &user_token_accounts[i];

        // Note: DEAD_SHARES are now actually minted and included in lp_token_supply
        // so the calculation is correct without virtual additions
        let user_pool_share_amount = PreciseNumber::new(reserve.amount as u128)
            .ok_or(RlpError::MathOverflow)?
            .checked_mul(
                &PreciseNumber::new(lp_token_amount as u128)
                .ok_or(RlpError::MathOverflow)?
            )
            .ok_or(RlpError::MathOverflow)?
            .checked_div(
                &PreciseNumber::new(lp_token_supply as u128)
                .ok_or(RlpError::MathOverflow)?
            )
            .ok_or(RlpError::MathOverflow)?
            .to_imprecise()
            .ok_or(RlpError::MathOverflow)?;

        let user_pool_share_amount: u64 = user_pool_share_amount
            .try_into()
            .map_err(|_| error!(RlpError::MathOverflow))?;

        if user_pool_share_amount > 0 {
            let reserve_account = remaining_accounts
                .iter()
                .find(|account| account.key().eq(reserve_key))
                .ok_or(RlpError::InvalidInput)?;

            let user_token_account_info = remaining_accounts
                .iter()
                .find(|account| account.key().eq(user_token_account_key))
                .ok_or(RlpError::InvalidInput)?;

            transfer(
                CpiContext::new_with_signer(
                    token_program.to_account_info(), 
                    Transfer {
                        from: reserve_account.to_account_info(),
                        to: user_token_account_info.to_account_info(),
                        authority: liquidity_pool.to_account_info()
                    }, 
                    &[lp_seeds]
                ), 
                user_pool_share_amount
            )?;
        }
    }

    let cooldown_seeds = &[
        COOLDOWN_SEED.as_bytes(),
        &liquidity_pool.index.to_le_bytes(),
        &cooldown_id.to_le_bytes(),
        &[cooldown.bump]
    ];

    burn(
        CpiContext::new_with_signer(
            token_program.to_account_info(), 
            Burn {
                authority: cooldown.to_account_info(),
                from: cooldown_lp_token_account.to_account_info(),
                mint: lp_token_mint.to_account_info()
            }, 
            &[cooldown_seeds]
        ),
        lp_token_amount
    )?;

    close_account(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            CloseAccount {
                account: cooldown_lp_token_account.to_account_info(),
                destination: signer.to_account_info(),
                authority: cooldown.to_account_info()
            },
            &[cooldown_seeds]
        )
    )?;

    emit!(WithdrawEvent {
        amount: lp_token_amount,
        from: signer.key(),
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: WithdrawArgs
)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump = settings.bump,
        constraint = !settings.access_control.killswitch.is_frozen(&Action::Withdraw) @ RlpError::Frozen
    )]
    pub settings: Box<Account<'info, Settings>>,

    #[account(
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump = permissions.bump,
    )]
    pub permissions: Option<Account<'info, UserPermissions>>,

    #[account(
        mut,
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            &args.liquidity_pool_id.to_le_bytes()
        ],
        bump = liquidity_pool.bump,
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
            &liquidity_pool.index.to_le_bytes(),
            &args.cooldown_id.to_le_bytes()
        ],
        bump = cooldown.bump,
        close = signer,
        constraint = cooldown.liquidity_pool_id == args.liquidity_pool_id,
        constraint = cooldown.authority == signer.key()
    )]
    pub cooldown: Account<'info, Cooldown>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub system_program: Option<Program<'info, System>>,
}