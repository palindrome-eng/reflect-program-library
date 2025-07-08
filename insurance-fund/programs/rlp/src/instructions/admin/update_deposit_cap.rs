use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;
use crate::errors::InsuranceFundError;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct UpdateDepositCapArgs {
    pub lockup_id: u64,
    pub new_cap: Option<u64>
}

pub fn update_deposit_cap(
    ctx: Context<UpdateDepositCap>,
    args: UpdateDepositCapArgs
) -> Result<()> {
    let UpdateDepositCapArgs {
        lockup_id: _,
        new_cap
    } = args;

    let lockup = &mut ctx.accounts.lockup;
    lockup.deposit_cap = new_cap;

    Ok(())
}

#[derive(Accounts)]
#[instruction(args: UpdateDepositCapArgs)]
pub struct UpdateDepositCap<'info> {
    #[account(
        mut,
    )]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump,
        constraint = admin.can_perform_protocol_action(Action::UpdateDepositCap, &settings.access_control) @ InsuranceFundError::InvalidSigner,
    )]
    pub admin: Account<'info, UserPermissions>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        constraint = !settings.frozen @ InsuranceFundError::Frozen
    )]
    pub settings: Account<'info, Settings>,

    #[account(
        mut,
        seeds = [
            LOCKUP_SEED.as_bytes(),
            &args.lockup_id.to_le_bytes()
        ],
        bump
    )]
    pub lockup: Account<'info, Lockup>,
}