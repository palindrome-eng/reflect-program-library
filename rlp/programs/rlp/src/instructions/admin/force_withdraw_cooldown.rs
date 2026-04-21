use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{burn, close_account, transfer, Burn, CloseAccount, Mint, Token, TokenAccount, Transfer};
use spl_math::precise_number::PreciseNumber;
use crate::constants::*;
use crate::errors::RlpError;
use crate::helpers::{load_assets, load_reserves};
use crate::states::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ForceWithdrawCooldownArgs {
    pub pool_id: u8,
    pub cooldown_id: u64,
}

/// Remaining accounts layout (per asset):
///   [asset_pda, pool_reserve_ata, destination_ata]
pub fn force_withdraw_cooldown<'a>(
    ctx: Context<'_, '_, 'a, 'a, ForceWithdrawCooldown<'a>>,
    args: ForceWithdrawCooldownArgs,
) -> Result<()> {
    let liquidity_pool = &ctx.accounts.liquidity_pool;
    let cooldown = &ctx.accounts.cooldown;
    let cooldown_lp_token_account = &ctx.accounts.cooldown_lp_token_account;
    let lp_token_mint = &ctx.accounts.lp_token_mint;
    let token_program = &ctx.accounts.token_program;

    let lp_token_amount = cooldown_lp_token_account.amount;
    let lp_token_supply = lp_token_mint.supply;

    require!(
        lp_token_amount > 0 && lp_token_supply > 0,
        RlpError::InvalidInput
    );

    let remaining_accounts = &ctx.remaining_accounts;
    require!(
        remaining_accounts.len() == liquidity_pool.asset_count as usize * 3,
        RlpError::InvalidInput
    );

    let lp_seeds = &[
        LIQUIDITY_POOL_SEED.as_bytes(),
        &liquidity_pool.index.to_le_bytes(),
        &[liquidity_pool.bump],
    ];

    let assets: Vec<(Pubkey, Asset)> = load_assets(liquidity_pool, remaining_accounts)?;
    let asset_datas = assets.iter().map(|(_, asset)| asset).collect::<Vec<&Asset>>();
    let reserves = load_reserves(liquidity_pool, &asset_datas, remaining_accounts)?;

    for i in 0..assets.len() {
        let (reserve_key, reserve) = &reserves[i];

        let user_pool_share_amount = PreciseNumber::new(reserve.amount as u128)
            .ok_or(RlpError::MathOverflow)?
            .checked_mul(
                &PreciseNumber::new(lp_token_amount as u128)
                    .ok_or(RlpError::MathOverflow)?,
            )
            .ok_or(RlpError::MathOverflow)?
            .checked_div(
                &PreciseNumber::new(lp_token_supply as u128)
                    .ok_or(RlpError::MathOverflow)?,
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

            // Destination ATA is at offset: asset_count (asset PDAs) + asset_count (reserves) + i
            let dest_ata = &remaining_accounts[liquidity_pool.asset_count as usize * 2 + i];

            transfer(
                CpiContext::new_with_signer(
                    token_program.to_account_info(),
                    Transfer {
                        from: reserve_account.to_account_info(),
                        to: dest_ata.to_account_info(),
                        authority: liquidity_pool.to_account_info(),
                    },
                    &[lp_seeds],
                ),
                user_pool_share_amount,
            )?;
        }
    }

    let cooldown_seeds = &[
        COOLDOWN_SEED.as_bytes(),
        &liquidity_pool.index.to_le_bytes(),
        &args.cooldown_id.to_le_bytes(),
        &[cooldown.bump],
    ];

    burn(
        CpiContext::new_with_signer(
            token_program.to_account_info(),
            Burn {
                authority: cooldown.to_account_info(),
                from: cooldown_lp_token_account.to_account_info(),
                mint: lp_token_mint.to_account_info(),
            },
            &[cooldown_seeds],
        ),
        lp_token_amount,
    )?;

    close_account(CpiContext::new_with_signer(
        token_program.to_account_info(),
        CloseAccount {
            account: cooldown_lp_token_account.to_account_info(),
            destination: ctx.accounts.signer.to_account_info(),
            authority: cooldown.to_account_info(),
        },
        &[cooldown_seeds],
    ))?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(args: ForceWithdrawCooldownArgs)]
pub struct ForceWithdrawCooldown<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump = admin.bump,
        constraint = admin.is_super_admin() @ RlpError::PermissionsTooLow,
    )]
    pub admin: Account<'info, UserPermissions>,

    #[account(
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            &args.pool_id.to_le_bytes()
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
        seeds = [
            COOLDOWN_SEED.as_bytes(),
            &liquidity_pool.index.to_le_bytes(),
            &args.cooldown_id.to_le_bytes()
        ],
        bump = cooldown.bump,
        close = signer,
    )]
    pub cooldown: Account<'info, Cooldown>,

    #[account(
        mut,
        associated_token::mint = lp_token_mint,
        associated_token::authority = cooldown,
    )]
    pub cooldown_lp_token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub system_program: Program<'info, System>,
}
