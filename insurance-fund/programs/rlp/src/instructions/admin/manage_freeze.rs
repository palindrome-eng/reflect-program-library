use anchor_lang::prelude::*;
use crate::constants::*;
use crate::states::*;
use crate::events::ManageFreezeEvent;
use crate::errors::InsuranceFundError;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ManageFreezeArgs {
    pub freeze: bool
}

pub fn manage_freeze(
    ctx: Context<ManageFreeze>,
    args: ManageFreezeArgs
) -> Result<()> {

    let settings = &mut ctx.accounts.settings;
    let signer = &ctx.accounts.signer;
    
    match args.freeze {
        true => settings.freeze(),
        false => settings.unfreeze(),
    }

    emit!(ManageFreezeEvent {
        admin: signer.key(),
        frozen: args.freeze
    });

    Ok(())
}

#[derive(Accounts)]
pub struct ManageFreeze<'info> {
    #[account(
        mut,
    )]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref(),
        ],
        bump,
        constraint = admin.can_perform_protocol_action(Action::FreezeRestake, &settings.access_control) @ InsuranceFundError::InvalidSigner,
    )]
    pub admin: Account<'info, UserPermissions>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
    )]
    pub settings: Account<'info, Settings>,
}