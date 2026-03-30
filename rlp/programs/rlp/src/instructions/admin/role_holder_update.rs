use crate::errors::RlpError;
use crate::states::*;
use crate::constants::*;
use anchor_lang::prelude::*;
use crate::helpers::action_check_protocol;
use crate::events::UpdateRoleHolderEvent;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct UpdateRoleHolderArgs {
    pub address: Pubkey,
    pub role: Role,
    pub update: Update
}

pub fn update_role_holder_protocol(
    ctx: Context<RlpAdminRoleUpdate>,
    args: UpdateRoleHolderArgs
) -> Result<()> {
    let accounts = ctx.accounts;
    let settings = &mut accounts.settings;
    let update_admin_permissions = &mut accounts.update_admin_permissions;

    let UpdateRoleHolderArgs {
        address,
        role,
        update
    } = args;

    let creds: &mut Account<UserPermissions> = &mut accounts.admin_permissions;          
    action_check_protocol(Action::UpdateRole, Some(&creds), &settings.access_control)?;

    if role == Role::SUPREMO {
        creds.validate_supremo()?;
    }

    match update{
        Update::Add => {
            update_admin_permissions.add_protocol_role(role)?;            
        },
        Update::Remove => {
            update_admin_permissions.remove_protocol_role(role)?;
        }
    }

    emit!(UpdateRoleHolderEvent {
        address,
        role,
        update
    });

    Ok(())
}

#[derive(Accounts)]
pub struct RlpAdminRoleUpdate<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut, seeds = [SETTINGS_SEED.as_bytes()], bump = settings.bump)]
    pub settings: Box<Account<'info, Settings>>,

    #[account(mut, seeds = [PERMISSIONS_SEED.as_bytes(), admin.key().as_ref()], bump = admin_permissions.bump)]
    pub admin_permissions: Account<'info, UserPermissions>,

    #[account(mut, constraint = update_admin_permissions.key() != admin_permissions.key() @ RlpError::SameAdmin)]
    pub update_admin_permissions: Account<'info, UserPermissions>,
    
    pub system_program: Program<'info, System>,
}