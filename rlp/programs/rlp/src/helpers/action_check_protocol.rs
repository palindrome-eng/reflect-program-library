use anchor_lang::prelude::*;
use crate::{errors::InsuranceFundError, states::{AccessControl, Action, UserPermissions}};

#[inline(never)]
fn check_action_permission(
    action: Action,
    creds: Option<&UserPermissions>,
    access_controls: &[&AccessControl],
) -> Result<()> {
    
    // Check if core action is frozen at all relevant levels.
    if action.is_core() {
        for ac in access_controls.iter() {
            ac.action_unsuspended(&action).or_else(|e| {
                msg!("Action {:?} is frozen at protocol level", action);
                Err(e)
            })?;
        }
    }

    // msg!("is public {:?}", access_controls[0].is_public_action(Action::PublicSwap));
    // msg!("allowed roles: {:?}", access_controls[0].access_map.get_action_allowees(Action::PublicSwap));
    
    // for ac in access_controls[0].access_map.action_permissions.iter() {
    //     msg!("{:?}: {:?}", ac.action, ac.allowed_roles);
    // }
    
    // Check if action is public at any level.
    if access_controls.iter().any(|ac| ac.is_public_action(action)) {
        return Ok(());
    }
    
    // Check user permissions using inheritance.
    if let Some(creds) = creds {
        let has_permission = creds.can_perform_protocol_action(action, access_controls[0]);
        
        if has_permission {
            return Ok(());
        }
        
        msg!("User {:?} does not have permission for {:?} at protocol level", creds.authority, action);
    } else {
        msg!("No credentials provided for action {:?} at protocol level", action);
    }
    
    Err(InsuranceFundError::IncorrectAdmin.into())
}

#[inline(never)]
pub fn action_check_protocol(
    action: Action,
    creds: Option<&UserPermissions>,
    protocol_access_control: &AccessControl,
) -> Result<()> {
    check_action_permission(
        action,
        creds,
        &[protocol_access_control],
    )
}