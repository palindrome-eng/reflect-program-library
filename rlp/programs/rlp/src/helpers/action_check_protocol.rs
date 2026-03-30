use anchor_lang::prelude::*;
use crate::{errors::RlpError, states::{AccessControl, Action, UserPermissions}};

#[inline(never)]
fn check_action_permission(
    action: Action,
    creds: Option<&UserPermissions>,
    access_controls: &[&AccessControl],
) -> Result<()> {

    if action.is_core() {
        for ac in access_controls.iter() {
            ac.action_unsuspended(&action).or_else(|e| {
                Err(e)
            })?;
        }
    }

    if access_controls.iter().any(|ac| ac.is_public_action(action)) {
        return Ok(());
    }

    if let Some(creds) = creds {
        let has_permission = creds.can_perform_protocol_action(action, access_controls[0]);

        if has_permission {
            return Ok(());
        }
    }

    Err(RlpError::IncorrectAdmin.into())
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