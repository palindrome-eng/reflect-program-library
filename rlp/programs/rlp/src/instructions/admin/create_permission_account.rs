use crate::constants::*;
use crate::states::*;
use anchor_lang::prelude::*;

pub fn create_permission_account(
    ctx: Context<RlpUserPermissionsInit>,
    new_admin: Pubkey,
) -> Result<()> {
    let new_creds: &mut Account<UserPermissions> = &mut ctx.accounts.new_creds;    
    new_creds.bump = ctx.bumps.new_creds;         
    new_creds.authority = new_admin;              

    Ok(())
}

#[derive(Accounts)]
#[instruction(new_admin: Pubkey)]
pub struct RlpUserPermissionsInit<'info> {
    #[account(seeds = [SETTINGS_SEED.as_bytes()], bump = settings.bump)]
    pub settings: Account<'info, Settings>,
    
    #[account(
        init,
        payer = caller,
        space = 8 + UserPermissions::INIT_SPACE,
        seeds = [PERMISSIONS_SEED.as_bytes(), new_admin.key().as_ref()],
        bump
    )]
    pub new_creds: Account<'info, UserPermissions>,

    #[account(mut)]
    pub caller: Signer<'info>,
    pub system_program: Program<'info, System>,
}