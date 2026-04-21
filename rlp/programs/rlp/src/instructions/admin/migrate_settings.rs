use anchor_lang::prelude::*;
use crate::constants::*;
use crate::errors::RlpError;
use crate::states::UserPermissions;

/// Old Settings layout has KillSwitch.frozen as u16 (2 bytes).
/// New layout uses u32 (4 bytes). This adds 2 bytes to the account,
/// shifting swap_fee_bps forward.
///
/// Byte layout (after 8-byte discriminator):
///   [0..3]     bump, liquidity_pools, assets
///   [3..364]   access_map (361 bytes: 18 * ActionMapping(20) + mapping_count(1))
///   [364..366] OLD frozen (u16)
///   [366..368] OLD swap_fee_bps (u16)
///
/// After migration (368 → 370 data bytes, 376 → 378 with discriminator):
///   [364..368] NEW frozen (u32)
///   [368..370] NEW swap_fee_bps (u16)
const KILLSWITCH_OFFSET: usize = 8 + 3 + 361; // 372 from account start
const OLD_ACCOUNT_SIZE: usize = 376;
const NEW_ACCOUNT_SIZE: usize = 378;

pub fn migrate_settings(ctx: Context<MigrateSettings>) -> Result<()> {
    let settings_info = &ctx.accounts.settings;

    let data = settings_info.try_borrow_data()?;
    let len = data.len();

    require!(len == OLD_ACCOUNT_SIZE, RlpError::InvalidState);

    let old_frozen = u16::from_le_bytes(
        data[KILLSWITCH_OFFSET..KILLSWITCH_OFFSET + 2]
            .try_into()
            .unwrap(),
    );
    let swap_fee_bps = u16::from_le_bytes(
        data[KILLSWITCH_OFFSET + 2..KILLSWITCH_OFFSET + 4]
            .try_into()
            .unwrap(),
    );

    drop(data);

    settings_info.realloc(NEW_ACCOUNT_SIZE, false)?;

    let payer = &ctx.accounts.signer;
    let rent = Rent::get()?;
    let new_min = rent.minimum_balance(NEW_ACCOUNT_SIZE);
    let current_lamports = settings_info.lamports();
    if current_lamports < new_min {
        let diff = new_min - current_lamports;
        anchor_lang::system_program::transfer(
            CpiContext::new(
                ctx.accounts.system_program.to_account_info(),
                anchor_lang::system_program::Transfer {
                    from: payer.to_account_info(),
                    to: settings_info.to_account_info(),
                },
            ),
            diff,
        )?;
    }

    let mut data = settings_info.try_borrow_mut_data()?;

    let new_frozen = old_frozen as u32;
    data[KILLSWITCH_OFFSET..KILLSWITCH_OFFSET + 4]
        .copy_from_slice(&new_frozen.to_le_bytes());
    data[KILLSWITCH_OFFSET + 4..KILLSWITCH_OFFSET + 6]
        .copy_from_slice(&swap_fee_bps.to_le_bytes());

    Ok(())
}

#[derive(Accounts)]
pub struct MigrateSettings<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump = admin.bump,
        constraint = admin.is_super_admin() @ RlpError::PermissionsTooLow,
    )]
    pub admin: Account<'info, UserPermissions>,

    /// CHECK: Cannot deserialize as Settings (old layout). Validated by seed.
    #[account(
        mut,
        seeds = [SETTINGS_SEED.as_bytes()],
        bump,
    )]
    pub settings: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}
