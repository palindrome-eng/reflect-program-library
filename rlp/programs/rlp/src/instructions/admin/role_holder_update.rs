use crate::errors::InsuranceFundError;
use crate::states::*;
use crate::constants::*;
use anchor_lang::prelude::*;
use crate::helpers::action_check_protocol;

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

    // Verify caller's credentials.
    let creds: &mut Account<UserPermissions> = &mut accounts.admin_permissions;          
    action_check_protocol(Action::UpdateRole, Some(&creds), &settings.access_control)?;

    // If role being assigned is supremo, the caller must be a protocol supremo.
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
    Ok(())
}

#[derive(Accounts)]
pub struct RlpAdminRoleUpdate<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(mut, seeds = [SETTINGS_SEED.as_bytes()], bump = settings.bump)]
    pub settings: Account<'info, Settings>,

    #[account(mut, seeds = [PERMISSIONS_SEED.as_bytes(), admin.key().as_ref()], bump = admin_permissions.bump)]
    pub admin_permissions: Account<'info, UserPermissions>,

    #[account(mut, constraint = update_admin_permissions.key() != admin_permissions.key() @ InsuranceFundError::SameAdmin)]
    pub update_admin_permissions: Account<'info, UserPermissions>,
    
    /// CHECK: Strategy controller verification is done in the instruction.
    #[account(mut)]
    pub strategy: AccountInfo<'info>,
    pub system_program: Program<'info, System>,
}