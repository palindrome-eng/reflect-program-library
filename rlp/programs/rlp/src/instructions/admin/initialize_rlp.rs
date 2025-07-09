use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;
use crate::events::InitializeRlp as InitializeRlpEvent;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitializeRlpArgs {
    pub cooldown_duration: u64,
}

pub fn initialize_rlp(
    ctx: Context<InitializeRlp>,
    args: InitializeRlpArgs
) -> Result<()> {

    let InitializeRlpArgs {
        cooldown_duration
    } = args;

    let signer = &ctx.accounts.signer;
    let permissions = &mut ctx.accounts.permissions;
    
    permissions.set_inner(UserPermissions {
        authority: signer.key(),
        bump: ctx.bumps.permissions,
        protocol_roles: LevelRoles::new(Role::SUPREMO)
    });

    let settings = &mut ctx.accounts.settings;
    settings.set_inner(Settings::default());

    emit!(InitializeRlpEvent {
        caller: signer.key()
    });

    Ok(())
}

#[derive(Accounts)]
pub struct InitializeRlp<'info> {
    #[account(
        mut
    )]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump,
        space = 8 + UserPermissions::INIT_SPACE
    )]
    pub permissions: Account<'info, UserPermissions>,

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