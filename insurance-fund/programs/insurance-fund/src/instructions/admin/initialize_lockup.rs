use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use anchor_spl::token::Token;
use anchor_spl::token::TokenAccount;
use crate::states::*;
use crate::constants::*;
use crate::events::InitializeLockupEvent;
use crate::errors::InsuranceFundError;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct InitializeLockupArgs {
    pub min_deposit: u64,
    pub deposit_cap: u64,
    pub duration: u64,
    pub yield_mode: YieldMode,
}

pub fn initialize_lockup(
    ctx: Context<InitializeLockup>,
    args: InitializeLockupArgs
) -> Result<()> {
    let settings = &mut ctx.accounts.settings;
    let lockup = &mut ctx.accounts.lockup;
    let asset_mint = &ctx.accounts.asset_mint;
    let asset = &mut ctx.accounts.asset;
    let receipt_mint = &mut ctx.accounts.pool_share_receipt;
    let signer = &ctx.accounts.signer;

    // lockup.initialize_hot_vault(
    //     &signer, 
    //     &lockup, 
    //     &asset_mint, 
    //     &lockup_hot_vault, 
    //     &token_program, 
    //     &system_program, 
    //     ctx.bumps.lockup_hot_vault, 
    //     lamports
    // )?;

    // lockup.initialize_cold_vault(
    //     &signer, 
    //     &lockup, 
    //     &asset_mint, 
    //     &lockup_cold_vault, 
    //     &token_program, 
    //     &system_program, 
    //     ctx.bumps.lockup_cold_vault, 
    //     lamports
    // )?;

    let InitializeLockupArgs {
        duration,
        min_deposit,
        yield_mode,
        deposit_cap
    } = args;

    lockup.bump = ctx.bumps.lockup;
    lockup.index = settings.lockups;
    lockup.asset_mint = asset_mint.key();
    lockup.duration = duration;
    lockup.yield_mode = yield_mode;
    lockup.deposits = 0;
    lockup.min_deposit = min_deposit;
    lockup.deposit_cap = if deposit_cap > 0 { Some(deposit_cap) } else { None };
    lockup.slash_state = SlashState {
        index: 0,
        amount: 0
    };
    lockup.reward_boosts = 0;
    lockup.receipt_to_reward_exchange_rate_bps_accumulator = 0;
    lockup.receipt_mint = receipt_mint.key();

    settings.lockups += 1;
    asset.add_lockup()?;

    emit!(InitializeLockupEvent {
        admin: signer.key(),
        asset: asset_mint.key(),
        duration: duration,
        lockup: lockup.key()
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: InitializeLockupArgs
)]
pub struct InitializeLockup<'info> {
    #[account(
        mut,
    )]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            ADMIN_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump,
        constraint = admin.has_permissions(Permissions::AssetsAndLockups) @ InsuranceFundError::InvalidSigner,
    )]
    pub admin: Box<Account<'info, Admin>>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        constraint = !settings.frozen @ InsuranceFundError::Frozen
    )]
    pub settings: Box<Account<'info, Settings>>,

    #[account(
        init,
        seeds = [
            LOCKUP_SEED.as_bytes(),
            &settings.lockups.to_le_bytes()
        ],
        bump,
        payer = signer,
        space = 8 + Lockup::INIT_SPACE,
    )]
    pub lockup: Box<Account<'info, Lockup>>,

    #[account(
        mut,
        seeds = [
            ASSET_SEED.as_bytes(),
            &asset_mint.key().to_bytes()
        ],
        bump,
    )]
    pub asset: Box<Account<'info, Asset>>,

    #[account(
        mut,
        address = asset.mint
    )]
    pub asset_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        address = settings.reward_config.main
    )]
    pub reward_mint: Box<Account<'info, Mint>>,

    /// CHECK: Directly checking the address.
    #[account(
        address = settings.cold_wallet
    )]
    pub cold_wallet: AccountInfo<'info>,

    #[account(
        mut,
        constraint = pool_share_receipt.supply == 0 &&
            pool_share_receipt.mint_authority.unwrap() == lockup.key() &&
            pool_share_receipt.freeze_authority.is_none() &&
            pool_share_receipt.is_initialized &&
            pool_share_receipt.decimals == 9 @ InsuranceFundError::InvalidReceiptTokenSetup
    )]
    pub pool_share_receipt: Box<Account<'info, Mint>>,

    #[account(
        init,
        payer = signer,
        seeds = [
            COOLDOWN_VAULT_SEED.as_bytes(),
            lockup.key().as_ref(),
            pool_share_receipt.key().as_ref(),
        ],
        bump,
        token::mint = pool_share_receipt,
        token::authority = lockup,
    )]
    pub lockup_cooldown_vault: Box<Account<'info, TokenAccount>>,

    #[account(
        address = Token::id()
    )]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub system_program: Program<'info, System>
}