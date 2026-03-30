use crate::{errors::RlpError, states::*};
use anchor_lang::prelude::*;
use super::RlpAdminMain;
use crate::helpers::action_check_protocol;
use crate::events::FreezeProtocolActionEvent;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct FreezeProtocolActionArgs {
    pub action: Action,
    pub freeze: bool
}

pub fn freeze_protocol_action(
    ctx: Context<RlpAdminMain>, 
    args: FreezeProtocolActionArgs
) -> Result<()> {
    let settings = &mut ctx.accounts.settings;

    let FreezeProtocolActionArgs {
        action,
        freeze
    } = args;

    action_check_protocol(
        action,
        Some(&ctx.accounts.admin_permissions),
        &settings.access_control
    )?;

    let action: Action = action.to_action()?;    

    match freeze {
        true => {
            let functionality_frozen = settings.access_control.killswitch.is_frozen(&action);
            require!(!functionality_frozen, RlpError::AlreadyFrozen);
            settings.access_control.killswitch.freeze(&action);
        }
        false => {
            let functionality_frozen = settings.access_control.killswitch.is_frozen(&action);
            require!(functionality_frozen, RlpError::AlreadyUnfrozen);
            settings.access_control.killswitch.unfreeze(&action);
        }
    }

    emit!(FreezeProtocolActionEvent {
        action,
        freeze
    });

    Ok(())
}