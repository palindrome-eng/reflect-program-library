use crate::instructions::*;
use crate::states::*;
use crate::helpers::action_check_protocol;
use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct UpdateActionRoleArgs {
    pub action: Action,
    pub role: Role,
    pub update: Update
}

pub fn update_action_role_protocol(
    ctx: Context<RlpAdminMain>,
    args: UpdateActionRoleArgs
) -> Result<()> {
    let accounts: &mut RlpAdminMain = ctx.accounts;
    let settings = &mut accounts.settings;    
    let creds: &mut Account<UserPermissions> = &mut accounts.admin_permissions; 

    let UpdateActionRoleArgs {
        action,
        role,
        update
    } = args;
        
    action_check_protocol(
        action, 
        Some(&creds), 
        &settings.access_control
    )?;
  
    msg!("[PROTOCOL] Caller: {:?} wants to set {:?}  for {:?}", accounts.admin.key(), role, action);

    match update {
        Update::Add => {    
            settings.access_control.add_role_to_action(action, role)?;
        },
        Update::Remove => {
            settings.access_control.remove_role_from_action(action, role)?;
        }
    }
    Ok(())
}