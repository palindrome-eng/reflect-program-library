use crate::states::*;
use crate::constants::*;
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct RlpAdminMain<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    
    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump = settings.bump
    )]
    pub settings: Account<'info, Settings>,

    pub system_program: Program<'info, System>,

    #[account(  
        mut, 
        seeds = [PERMISSIONS_SEED.as_bytes(), admin.key().as_ref()],
        bump = admin_permissions.bump,        
    )]
    pub admin_permissions: Account<'info, UserPermissions>,
}
