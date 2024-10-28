use anchor_lang::prelude::*;
use crate::constants::*;
use crate::states::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct ManageFreezeArgs {
    pub freeze: bool
}

pub fn manage_freeze(
    ctx: Context<ManageFreeze>,
    args: ManageFreezeArgs
) -> Result<()> {

    let settings = &mut ctx.accounts.settings;
    
    match args.freeze {
        true => settings.freeze(),
        false => settings.unfreeze(),
    }

    Ok(())
}

#[derive(Accounts)]
pub struct ManageFreeze<'info> {
    #[account(
        mut,
        address = settings.superadmin
    )]
    pub superadmin: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        has_one = superadmin,
    )]
    pub settings: Account<'info, Settings>,
}