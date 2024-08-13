use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;
use crate::borsh::*;
use crate::states::*;
use crate::constants::*;
use crate::errors::InsuranceFundError;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct InitializeLockupArgs {
    pub asset: Pubkey,
    pub min_deposit: u64,
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
        yield_mode
    } = args;

    lockup.index = settings.lockups;
    lockup.asset = asset;
    lockup.duration = duration;
    lockup.min_deposit = min_deposit;
    lockup.yield_bps = yield_bps;
    lockup.yield_mode = yield_mode;

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
        has_one = superadmin
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
        space = Lockup::SIZE
    )]
    pub lockup: Account<'info, Lockup>,

    #[account(
        mut,
        constraint = settings.whitelisted_assets.contains(&asset_mint.key()) @ InsuranceFundError::AssetNotWhitelisted
    )]
    pub asset_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = superadmin,
        token::mint = asset_mint,
        token::authority = lockup,
    )]
    pub asset_lockup: Account<'info, TokenAccount>,

    #[account(
        address = Token::id()
    )]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub system_program: Program<'info, System>
}