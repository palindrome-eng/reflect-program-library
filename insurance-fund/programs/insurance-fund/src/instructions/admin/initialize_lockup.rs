use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;
use crate::states::*;
use crate::constants::*;
use crate::errors::InsuranceFundError;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct InitializeLockupArgs {
    pub asset: Pubkey,
    pub min_deposit: u64,
    pub deposit_cap: u64,
    pub duration: u64,
    pub yield_bps: u64,
    pub yield_mode: YieldMode,
}

pub fn initialize_lockup(
    ctx: Context<InitializeLockup>,
    args: InitializeLockupArgs
) -> Result<()> {
    let settings = &mut ctx.accounts.settings;
    let lockup = &mut ctx.accounts.lockup;

    let InitializeLockupArgs {
        asset,
        duration,
        min_deposit,
        yield_bps,
        yield_mode,
        deposit_cap
    } = args;

    lockup.bump = ctx.bumps.lockup;
    lockup.index = settings.lockups;
    lockup.asset = asset;
    lockup.duration = duration;
    lockup.min_deposit = min_deposit;
    lockup.yield_bps = yield_bps;
    lockup.yield_mode = yield_mode;
    lockup.deposit_cap = if deposit_cap > 0 { Some(deposit_cap) } else { None };
    lockup.slash_state = SlashState {
        index: 0,
        amount: 0
    };
    lockup.reward_boosts = 0;

    settings.lockups += 1;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: InitializeLockupArgs
)]
pub struct InitializeLockup<'info> {
    #[account(
        mut,
        constraint = settings.superadmin == superadmin.key() @ InsuranceFundError::InvalidSigner
    )]
    pub superadmin: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        has_one = superadmin,
        constraint = !settings.frozen @ InsuranceFundError::Frozen
    )]
    pub settings: Account<'info, Settings>,

    #[account(
        init,
        seeds = [
            LOCKUP_SEED.as_bytes(),
            &settings.lockups.to_le_bytes()
        ],
        bump,
        payer = superadmin,
        space = Lockup::SIZE,
    )]
    pub lockup: Account<'info, Lockup>,

    #[account(
        mut,
        seeds = [
            ASSET_SEED.as_bytes(),
            &asset_mint.key().to_bytes()
        ],
        bump,
    )]
    pub asset: Account<'info, Asset>,

    #[account(
        mut,
        address = asset.mint,
    )]
    pub asset_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = superadmin,
        seeds = [
            VAULT_SEED.as_bytes(),
            lockup.key().as_ref(),
            asset_mint.key().as_ref(),
        ],
        bump,
        token::mint = asset_mint,
        token::authority = lockup,
    )]
    pub lockup_asset_vault: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = superadmin,
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

    #[account(
        address = Token::id()
    )]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub system_program: Program<'info, System>
}