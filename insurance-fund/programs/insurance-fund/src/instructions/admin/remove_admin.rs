use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;
use crate::events::*;
use crate::errors::InsuranceFundError;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct RemoveAdminArgs {
    address: Pubkey,
}

pub fn remove_admin(
    ctx: Context<RemoveAdmin>,
    args: RemoveAdminArgs
) -> Result<()> {
    let signer = &ctx.accounts.signer;

    ctx.accounts.verify_removal()?;

    emit!(ChangeAdminEvent {
        affected_admin: args.address,
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
        seeds = [
            ADMIN_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump,
    )]
    pub admin: Account<'info, Admin>,

    #[account(
        mut,
        seeds = [
            ADMIN_SEED.as_bytes(),
            &args.address.as_ref(),
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

impl RemoveAdmin<'_> {

    pub fn verify_removal(&self) -> Result<()> {
        require!(
            self.admin.key() == self.admin_to_remove.key() || 
            self.admin.has_permissions_over(self.admin_to_remove.permissions),
            InsuranceFundError::PermissionsTooLow
        );

        match self.admin_to_remove.permissions {
            Permissions::Superadmin => {
                require!(
                    self.settings.superadmins > 1,
                    InsuranceFundError::MinimumSuperadminsRequired
                );
            },
            _ => {}
        };

        Ok(())
    }
}