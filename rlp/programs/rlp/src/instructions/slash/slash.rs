use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::token::{transfer, Mint, Token, TokenAccount, Transfer};

use crate::constants::*;
use crate::errors::RlpError;
use crate::events::SlashEvent;
use crate::helpers::{div_oracle_raw, mul_oracle_raw, ProxyStateView};
use crate::states::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SlashArgs {
    pub liquidity_pool_id: u8,
    /// Requested amount of the protected vault's stablecoin_mint tokens to
    /// transfer from this pool's reserve into the proxy's vault. The
    /// instruction caps this at the proxy's current mark-to-market loss.
    pub amount: u64,
}

/// Junior-tranche slash — covers the senior (proxy) tranche's NAV loss.
///
/// RLP is the junior tranche to a senior proxy (USDC+ vault). When the
/// proxy's vault is worth less than booked claims
/// (`principal + integrators_commission`), there is a mark-to-market loss
/// equal to the gap. This instruction lets an authorized caller pull from
/// this pool's reserve of the proxy's `stablecoin_mint` directly into the
/// proxy's vault token account, capped at the current gap.
///
/// The next time the proxy's `crank` runs (on the next `wrap`, `unwrap`, or
/// `claim_fees`), it reads the higher vault balance and the gap shrinks or
/// closes. No CPI back into the proxy is required — coverage is self-healing
/// via the proxy's existing accounting.
///
/// Slashing is enabled only when the pool has `protected_vault: Some(_)`
/// pinning the ProxyState it covers. Pools without a protected vault are
/// un-slashable by design — there is no operator-discretion exit path from
/// the pool other than user-side LP redemption.
pub fn slash(ctx: Context<Slash>, args: SlashArgs) -> Result<()> {
    let SlashArgs {
        liquidity_pool_id,
        amount,
    } = args;

    require!(amount > 0, RlpError::InvalidInput);

    let liquidity_pool = &ctx.accounts.liquidity_pool;
    let asset = &ctx.accounts.asset;
    let stablecoin_mint = &ctx.accounts.stablecoin_mint;
    let liquidity_pool_token_account = &ctx.accounts.liquidity_pool_token_account;
    let proxy_state_account = &ctx.accounts.proxy_state;
    let protected_vault_token_account = &ctx.accounts.protected_vault_token_account;
    let oracle_account = &ctx.accounts.oracle;
    let token_program = &ctx.accounts.token_program;

    // 1. Pool must have a protected vault wired up.
    let protected_vault = liquidity_pool
        .protected_vault
        .ok_or(RlpError::PoolHasNoProtectedVault)?;

    require!(
        proxy_state_account.key() == protected_vault,
        RlpError::ProtectedVaultMismatch
    );

    // 2. Asset must be the proxy's stablecoin_mint and must belong to this pool.
    require!(
        liquidity_pool.has_asset(asset.index),
        RlpError::AssetNotWhitelisted
    );

    let proxy_state = ProxyStateView::from_account_info(proxy_state_account)?;
    require!(
        proxy_state.stablecoin_mint == stablecoin_mint.key()
            && asset.mint == stablecoin_mint.key(),
        RlpError::ProtectedVaultMintMismatch
    );

    // 3. Verify the destination is the canonical proxy vault ATA.
    let expected_vault_ata =
        get_associated_token_address(&proxy_state_account.key(), &stablecoin_mint.key());
    require!(
        protected_vault_token_account.key() == expected_vault_ata,
        RlpError::ProtectedVaultMismatch
    );

    // 4. Read oracle via RLP's asset binding (audit-3 enforces feed-id pinning).
    let clock = Clock::get()?;
    let oracle_price = asset.get_price(oracle_account, &clock)?;
    require!(oracle_price.price > 0, RlpError::PriceError);

    // 5. Compute mark-to-market gap = booked_claims - vault_usdc_value.
    let booked_claims = proxy_state.total_booked_claims()?;
    let vault_balance = protected_vault_token_account.amount;
    let vault_usdc_value = mul_oracle_raw(
        vault_balance as u128,
        oracle_price.price as u128,
        oracle_price.exponent,
    )?;

    let loss_usdc = (booked_claims as u128).saturating_sub(vault_usdc_value);
    require!(loss_usdc > 0, RlpError::NoNavLossToCover);

    // 6. Convert the loss (USDC raw) to stablecoin_mint tokens (USDC+ raw).
    //    Floor division — caller may need to overshoot by one unit to fully
    //    close the gap, which crank() will absorb correctly.
    let max_cover_tokens = div_oracle_raw(
        loss_usdc,
        oracle_price.price as u128,
        oracle_price.exponent,
    )?;
    let max_cover_tokens_u64: u64 = max_cover_tokens
        .try_into()
        .map_err(|_| RlpError::MathOverflow)?;

    require!(
        amount <= max_cover_tokens_u64,
        RlpError::NavCoverageExceedsLoss
    );

    require!(
        liquidity_pool_token_account.amount >= amount,
        RlpError::NotEnoughFunds
    );

    // 7. Transfer from the pool's reserve into the proxy vault.
    let seeds = &[
        LIQUIDITY_POOL_SEED.as_bytes(),
        &liquidity_pool_id.to_le_bytes(),
        &[liquidity_pool.bump],
    ];

    transfer(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Transfer {
                authority: liquidity_pool.to_account_info(),
                from: liquidity_pool_token_account.to_account_info(),
                to: protected_vault_token_account.to_account_info(),
            },
            &[seeds],
        ),
        amount,
    )?;

    emit_cpi!(SlashEvent {
        admin: ctx.accounts.signer.key(),
        liquidity_pool: liquidity_pool.key(),
        protected_vault,
        stablecoin_mint: stablecoin_mint.key(),
        loss_usdc: u64::try_from(loss_usdc).unwrap_or(u64::MAX),
        amount_tokens: amount,
    });

    Ok(())
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(args: SlashArgs)]
pub struct Slash<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump = permissions.bump,
        constraint = permissions.can_perform_protocol_action(Action::Slash, &settings.access_control) @ RlpError::PermissionsTooLow,
    )]
    pub permissions: Account<'info, UserPermissions>,

    #[account(
        seeds = [SETTINGS_SEED.as_bytes()],
        bump = settings.bump,
        constraint = !settings.access_control.killswitch.is_frozen(&Action::Slash) @ RlpError::Frozen,
    )]
    pub settings: Box<Account<'info, Settings>>,

    #[account(
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            &args.liquidity_pool_id.to_le_bytes()
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

    #[account()]
    pub stablecoin_mint: Box<Account<'info, Mint>>,

    /// Pool's reserve of the proxy's stablecoin_mint (typically USDC+).
    /// Source of the transfer.
    #[account(
        mut,
        token::mint = stablecoin_mint,
        token::authority = liquidity_pool,
    )]
    pub liquidity_pool_token_account: Box<Account<'info, TokenAccount>>,

    /// The proxy program's ProxyState account. Validated against
    /// `liquidity_pool.protected_vault` and read for principal/commission.
    /// CHECK: byte-validated by ProxyStateView::from_account_info (owner +
    /// length check). Not deserialized via Anchor because the proxy program
    /// is pinocchio-based and has no Anchor discriminator.
    pub proxy_state: UncheckedAccount<'info>,

    /// The proxy's stablecoin vault: ATA(proxy_state, stablecoin_mint).
    /// Destination of the transfer.
    #[account(
        mut,
        token::mint = stablecoin_mint,
    )]
    pub protected_vault_token_account: Box<Account<'info, TokenAccount>>,

    /// CHECK: address bound by asset.oracle (audit-3 enforces feed-id pinning).
    #[account(
        constraint = oracle.key() == *asset.oracle.key() @ RlpError::InvalidOracle
    )]
    pub oracle: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
}
