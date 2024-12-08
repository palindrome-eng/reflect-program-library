use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;
use crate::events::*;
use crate::errors::InsuranceFundError;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RemoveAdminArgs {
    admin_id: u8,
}

pub fn remove_admin(
    ctx: Context<RemoveAdmin>,
    _: RemoveAdminArgs
) -> Result<()> {

    let admin_to_remove = &ctx.accounts.admin_to_remove;
    let signer = &ctx.accounts.signer;

    emit!(ChangeAdminEvent {
        affected_admin: admin_to_remove.address,
        signer: signer.key(),
        permission_change: None
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(args: RemoveAdminArgs)]
pub struct RemoveAdmin<'info> {
    #[account(
        mut
    )]
    pub signer: Signer<'info>,

    #[account(
        mut,
        constraint = signer.key() == admin.address @ InsuranceFundError::InvalidSigner,
        // Either self-remove, or only remove if the other admin has smaller permissions (equal is not enough, unless superadmin).
        constraint = admin.key() == admin_to_remove.key() || admin.has_permissions_over(admin_to_remove.permissions) @ InsuranceFundError::PermissionsTooLow
    )]
    pub admin: Account<'info, Admin>,

    #[account(
        mut,
        seeds = [
            ADMIN_SEED.as_bytes(),
            &args.admin_id.to_le_bytes(),
        ],
        bump,
        close = signer,
    )]
    pub admin_to_remove: Account<'info, Admin>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        constraint = !settings.frozen @ InsuranceFundError::Frozen
    )]
    pub settings: Account<'info, Settings>,

    #[account()]
    pub system_program: Program<'info, System>,
}