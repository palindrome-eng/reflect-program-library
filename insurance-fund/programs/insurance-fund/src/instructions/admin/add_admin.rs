use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;
use crate::errors::InsuranceFundError;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct AddAdminArgs {
    permissions: Permissions
}

pub fn add_admin(
    ctx: Context<AddAdmin>,
    args: AddAdminArgs
) -> Result<()> {

    let new_admin = &mut ctx.accounts.new_admin;
    new_admin.permissions = args.permissions;

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