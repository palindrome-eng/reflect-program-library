use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;
use crate::events::*;
use crate::errors::InsuranceFundError;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddAdminArgs {
    address: Pubkey,
    permissions: Permissions
}

pub fn add_admin(
    ctx: Context<AddAdmin>,
    args: AddAdminArgs
) -> Result<()> {

    let AddAdminArgs {
        address,
        permissions
    } = args;

    let settings = &mut ctx.accounts.settings;
    let new_admin = &mut ctx.accounts.new_admin;
    let signer = &ctx.accounts.signer;

    new_admin.address = address;
    new_admin.permissions = permissions;

    emit!(ChangeAdminEvent {
        affected_admin: args.address,
        signer: signer.key(),
        permission_change: Some(args.permissions)
    });

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: AddAdminArgs
)]
pub struct AddAdmin<'info> {
    #[account(
        mut
    )]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            ADMIN_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump,
        constraint = existing_admin.has_permissions_over(args.permissions) @ InsuranceFundError::PermissionsTooLow
    )]
    pub existing_admin: Account<'info, Admin>,

    #[account(
        // If exists, change, grant or remove one's permissions
        init_if_needed,
        payer = signer,
        seeds = [
            ADMIN_SEED.as_bytes(),
            args.address.as_ref(),
        ],
        bump,
        space = 8 + Admin::INIT_SPACE,
    )]
    pub new_admin: Account<'info, Admin>,

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