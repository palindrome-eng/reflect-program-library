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

    let settings = &mut ctx.accounts.settings;
    let new_admin = &mut ctx.accounts.new_admin;
    let signer = &ctx.accounts.signer;

    new_admin.permissions = args.permissions;
    new_admin.address = args.address;
    new_admin.index = settings.admins;

    settings.admins += 1;

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
        constraint = signer.key() == existing_admin.address @ InsuranceFundError::InvalidSigner,
        constraint = existing_admin.has_permissions_over(args.permissions) @ InsuranceFundError::PermissionsTooLow
    )]
    pub existing_admin: Account<'info, Admin>,

    #[account(
        init,
        payer = signer,
        seeds = [
            ADMIN_SEED.as_bytes(),
            &settings.admins.to_le_bytes(),
        ],
        bump,
        space = Admin::SIZE,
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