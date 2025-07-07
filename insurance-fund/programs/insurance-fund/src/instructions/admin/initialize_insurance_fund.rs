use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;
use crate::events::InitializeInsuranceFund as InitializeInsuranceFundEvent;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeInsuranceFundArgs {
    pub cooldown_duration: u64,
}

pub fn initialize_insurance_fund(
    ctx: Context<InitializeInsuranceFund>,
    args: InitializeInsuranceFundArgs
) -> Result<()> {

    let InitializeInsuranceFundArgs {
        cooldown_duration
    } = args;

    let signer = &ctx.accounts.signer;
    let admin = &mut ctx.accounts.admin;
    
    admin.address = signer.key();
    admin.permissions = Permissions::Superadmin;

    let settings = &mut ctx.accounts.settings;

    settings.bump = ctx.bumps.settings;
    settings.lockups = 0;
    settings.cooldown_duration = cooldown_duration;
    settings.reward_config = RewardConfig {
        main: reward_mint
    };
    settings.frozen = false;

    emit!(InitializeInsuranceFundEvent {
        new_admin: signer.key()
    });

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
            signer.key().as_ref()
        ],
        bump,
        space = 8 + Admin::INIT_SPACE
    )]
    pub admin: Account<'info, Admin>,

    #[account(
        init,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        payer = signer,
        space = 8 + Settings::INIT_SPACE,
    )]
    pub settings: Account<'info, Settings>,

    #[account()]
    pub system_program: Program<'info, System>
}