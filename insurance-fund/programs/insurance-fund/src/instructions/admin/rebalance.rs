use anchor_lang::prelude::*;
use crate::states::*;
use crate::errors::InsuranceFundError;
use crate::constants::*;
use anchor_spl::token::{
    Mint,
    Token,
    TokenAccount,
    transfer,
    Transfer
};


#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct RebalanceArgs {
    pub lockup_id: u64,
}

pub fn rebalance(
    ctx: Context<Rebalance>,
    args: RebalanceArgs
) -> Result<()> {

    let RebalanceArgs {
        lockup_id
    } = args;

    let settings = &ctx.accounts.settings;
    let token_program = &ctx.accounts.token_program;

    let SharesConfig {
        cold_wallet_share_bps,
        hot_wallet_share_bps
    } = settings.shares_config;

    let lockup = &ctx.accounts.lockup;
    let cold_wallet = &ctx.accounts.cold_wallet;
    let lockup_cold_vault = &ctx.accounts.lockup_cold_vault;
    let lockup_hot_vault = &ctx.accounts.lockup_hot_vault;

    let total_deposit = lockup_cold_vault.amount
        .checked_add(lockup_hot_vault.amount)
        .ok_or(InsuranceFundError::MathOverflow)?;

    let real_cold_wallet_share_bps = lockup_cold_vault.amount
        .checked_mul(10_000)
        .ok_or(InsuranceFundError::MathOverflow)?
        .checked_div(total_deposit)
        .ok_or(InsuranceFundError::MathOverflow)?;

    let real_hot_wallet_share_bps = 10_000_u64
        .checked_sub(real_cold_wallet_share_bps)
        .ok_or(InsuranceFundError::MathOverflow)?;

    if (real_hot_wallet_share_bps > hot_wallet_share_bps) {
        let diff = real_hot_wallet_share_bps
            .checked_sub(hot_wallet_share_bps)
            .ok_or(InsuranceFundError::MathOverflow)?
            .checked_mul(total_deposit)
            .ok_or(InsuranceFundError::MathOverflow)?
            .checked_div(10_000)
            .ok_or(InsuranceFundError::MathOverflow)?;

        let lockup_seeds = &[
            LOCKUP_SEED.as_bytes(),
            &lockup_id.to_le_bytes(),
            &[lockup.bump]
        ];

        transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(), 
                Transfer {
                    from: lockup_hot_vault.to_account_info(),
                    authority: lockup.to_account_info(),
                    to: lockup_cold_vault.to_account_info()
                }, 
                &[lockup_seeds]
            ), 
            diff
        )?;
    } else if (real_cold_wallet_share_bps > real_hot_wallet_share_bps) {
        let diff = real_cold_wallet_share_bps
            .checked_sub(cold_wallet_share_bps)
            .ok_or(InsuranceFundError::MathOverflow)?
            .checked_mul(total_deposit)
            .ok_or(InsuranceFundError::MathOverflow)?
            .checked_div(10_000)
            .ok_or(InsuranceFundError::MathOverflow)?;

            transfer(
                CpiContext::new(
                    token_program.to_account_info(), 
                    Transfer {
                        from: lockup_cold_vault.to_account_info(),
                        authority: cold_wallet.to_account_info(),
                        to: lockup_hot_vault.to_account_info()
                    }
                ), 
                diff
            )?;
    }

    Ok(())
}

#[derive(Accounts)]
#[instruction(args: RebalanceArgs)]
pub struct Rebalance<'info> {
    #[account(
        mut,
    )]
    pub signer: Signer<'info>,

    #[account(
        mut,
        constraint = admin.address == signer.key(),
        constraint = admin.has_permissions(Permissions::AssetsAndLockups) @ InsuranceFundError::InvalidSigner,
    )]
    pub admin: Account<'info, Admin>,

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
            LOCKUP_SEED.as_bytes(),
            &args.lockup_id.to_le_bytes()
        ],
        bump
    )]
    pub lockup: Account<'info, Lockup>,

    #[account(
        mut,
        address = lockup.asset_mint,
    )]
    pub asset_mint: Box<Account<'info, Mint>>,

    #[account(
        mut,
        seeds = [
            HOT_VAULT_SEED.as_bytes(),
            lockup.key().as_ref(),
            asset_mint.key().as_ref(),
        ],
        bump,
        token::mint = asset_mint,
        token::authority = lockup,
    )]
    pub lockup_hot_vault: Account<'info, TokenAccount>,

    #[account(
        address = settings.cold_wallet
    )]
    pub cold_wallet: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [
            COLD_VAULT_SEED.as_bytes(),
            lockup.key().as_ref(),
            asset_mint.key().as_ref(),
        ],
        bump,
        token::mint = asset_mint,
        token::authority = cold_wallet,
    )]
    pub lockup_cold_vault: Account<'info, TokenAccount>,

    #[account()]
    pub token_program: Program<'info, Token>,
}