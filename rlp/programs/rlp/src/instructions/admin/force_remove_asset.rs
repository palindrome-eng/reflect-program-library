use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::token::spl_token::state::AccountState;
use anchor_spl::token::{Mint, TokenAccount};

use crate::constants::*;
use crate::errors::RlpError;
use crate::events::ForceRemoveAssetEvent;
use crate::states::*;

/// Admin recovery instruction for permanently frozen pool assets.
///
/// When a pool asset's reserve token account is frozen by its mint's freeze
/// authority (e.g., a compliance freeze on USDC), `deposit` and `withdraw`
/// both reject so the pool isn't trapped one-sided (audit M02). If the
/// freeze is temporary, normal operations resume once it is lifted. If the
/// freeze is permanent (compliance forfeiture, issuer migration without a
/// return path), an authorized caller can use this instruction to drop the
/// asset from the pool's reserve list — the frozen tokens stay in the pool
/// ATA but are no longer counted in valuation or distributed in withdraws.
///
/// The remaining LP holders effectively socialize the loss across themselves
/// (their LP claims are now backed only by the remaining unfrozen assets).
///
/// Gated by `Action::RemovePoolAsset`. Not added to `MANAGER`'s default
/// permissions; SUPREMO is the only caller by default.
pub fn force_remove_asset(ctx: Context<ForceRemoveAsset>) -> Result<()> {
    let liquidity_pool = &mut ctx.accounts.liquidity_pool;
    let asset = &ctx.accounts.asset;
    let pool_token_account = &ctx.accounts.pool_token_account;

    require!(
        liquidity_pool.has_asset(asset.index),
        RlpError::AssetNotWhitelisted
    );

    // The asset's pool ATA must actually be frozen — this is an admin
    // recovery path, not a generic asset-removal tool.
    require!(
        pool_token_account.state == AccountState::Frozen,
        RlpError::PoolAssetNotFrozen
    );

    require!(
        liquidity_pool.asset_count > 1,
        RlpError::CannotRemoveLastAsset
    );

    // Shift the asset out of the live region of `assets[]` and replace the
    // trailing slot with u8::MAX (the sentinel used at initialize_lp).
    let live = liquidity_pool.asset_count as usize;
    let mut found_at: Option<usize> = None;
    for i in 0..live {
        if liquidity_pool.assets[i] == asset.index {
            found_at = Some(i);
            break;
        }
    }
    let pos = found_at.ok_or(RlpError::AssetNotWhitelisted)?;
    for j in pos..(live - 1) {
        liquidity_pool.assets[j] = liquidity_pool.assets[j + 1];
    }
    liquidity_pool.assets[live - 1] = u8::MAX;
    liquidity_pool.asset_count -= 1;

    emit!(ForceRemoveAssetEvent {
        admin: ctx.accounts.signer.key(),
        liquidity_pool: liquidity_pool.key(),
        asset: asset.mint,
        asset_index: asset.index,
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(liquidity_pool_id: u8)]
pub struct ForceRemoveAsset<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump = permissions.bump,
        constraint = permissions.can_perform_protocol_action(
            Action::RemovePoolAsset,
            &settings.access_control
        ) @ RlpError::PermissionsTooLow,
    )]
    pub permissions: Account<'info, UserPermissions>,

    #[account(
        seeds = [SETTINGS_SEED.as_bytes()],
        bump = settings.bump,
        constraint = !settings.access_control.killswitch.is_frozen(&Action::RemovePoolAsset) @ RlpError::Frozen,
    )]
    pub settings: Box<Account<'info, Settings>>,

    #[account(
        mut,
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            &liquidity_pool_id.to_le_bytes()
        ],
        bump = liquidity_pool.bump,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    #[account(
        seeds = [
            ASSET_SEED.as_bytes(),
            &asset.mint.to_bytes()
        ],
        bump = asset.bump,
    )]
    pub asset: Account<'info, Asset>,

    #[account(
        address = asset.mint
    )]
    pub asset_mint: Box<Account<'info, Mint>>,

    /// The pool's reserve ATA for the asset being removed. Must be frozen.
    #[account(
        token::mint = asset_mint,
        token::authority = liquidity_pool,
        constraint = pool_token_account.key() == get_associated_token_address(&liquidity_pool.key(), &asset_mint.key()) @ RlpError::InvalidInput,
    )]
    pub pool_token_account: Box<Account<'info, TokenAccount>>,
}
