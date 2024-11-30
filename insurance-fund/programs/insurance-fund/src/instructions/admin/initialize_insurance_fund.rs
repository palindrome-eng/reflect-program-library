use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;
use crate::errors::InsuranceFundError;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeInsuranceFundArgs {
    pub cold_wallet: Pubkey,
    pub hot_wallet_share_bps: u64,
    pub cold_wallet_share_bps: u64,
    pub reward_mint: Pubkey,
    pub cooldown_duration: u64,
}

pub fn initialize_insurance_fund(
    ctx: Context<InitializeInsuranceFund>,
    args: InitializeInsuranceFundArgs
) -> Result<()> {

    let InitializeInsuranceFundArgs {
        cold_wallet,
        cold_wallet_share_bps,
        hot_wallet_share_bps,
        reward_mint,
        cooldown_duration
    } = args;

    require!(
        cold_wallet_share_bps + hot_wallet_share_bps == 10_000,
        InsuranceFundError::ShareConfigOverflow
    );

    let signer = &ctx.accounts.signer;
    let admin = &ctx.accounts.admin;
    
    admin.permissions = Permissions::Superadmin;
    admin.address = signer.key();
    admin.index += 1;

    let settings = &mut ctx.accounts.settings;

    settings.bump = ctx.bumps.settings;
    settings.lockups = 0;
    settings.cooldown_duration = cooldown_duration;
    settings.cold_wallet = cold_wallet;
    settings.shares_config = SharesConfig {
        cold_wallet_share_bps,
        hot_wallet_share_bps
    };
    settings.reward_config = RewardConfig {
        main: reward_mint
    };
    settings.frozen = false;

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeInsuranceFund<'info> {
    #[account(
        mut
    )]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        seeds = [
            ADMIN_SEED.as_bytes(),
            // Zero index.
            &(0_u8).to_le_bytes(),
        ],
        bump,
        space = Admin::SIZE
    )]
    pub admin: Account<'info, Admin>,

    #[account(
        init,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        payer = signer,
        space = Settings::SIZE,
    )]
    pub settings: Account<'info, Settings>,

    #[account()]
    pub system_program: Program<'info, System>
}