use crate::{errors::InsuranceFundError, states::*};
use anchor_lang::prelude::*;
use super::RlpAdminMain;
use crate::helpers::action_check_protocol;

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

    msg!("Caller: {:?} wants to {:?} PROTOCOL", ctx.accounts.admin.key(), action); 
    action_check_protocol(
        action,
        Some(&ctx.accounts.admin_permissions),
        &settings.access_control
    )?;

    let action: Action = action.to_action()?;    

    match freeze {
        true => {
            let functionality_frozen = settings.access_control.killswitch.is_frozen(&action);
            require!(!functionality_frozen, InsuranceFundError::AlreadyFrozen);
            settings.access_control.killswitch.freeze(&action);
        }
        false => {
            let functionality_frozen = settings.access_control.killswitch.is_frozen(&action);
            require!(functionality_frozen, InsuranceFundError::AlreadyUnfrozen);
            settings.access_control.killswitch.unfreeze(&action);
        }
    }

    Ok(())
}