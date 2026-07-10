use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;
use crate::errors::RlpError;
use crate::events::UpdateProtectedVaultEvent;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct UpdateProtectedVaultArgs {
    pub liquidity_pool_id: u8,
    pub new_protected_vault: Pubkey,
}

pub fn update_protected_vault(
    ctx: Context<UpdateProtectedVault>,
    args: UpdateProtectedVaultArgs
) -> Result<()> {
    let UpdateProtectedVaultArgs {
        liquidity_pool_id: _,
        new_protected_vault,
    } = args;

    let liquidity_pool = &mut ctx.accounts.liquidity_pool;
    let old_protected_vault = liquidity_pool.protected_vault;
    liquidity_pool.protected_vault = Some(new_protected_vault);

    emit!(UpdateProtectedVaultEvent {
        admin: ctx.accounts.signer.key(),
        liquidity_pool: liquidity_pool.key(),
        old_protected_vault,
        new_protected_vault,
    });

    Ok(())
}

#[event_cpi]
#[derive(Accounts)]
#[instruction(args: UpdateProtectedVaultArgs)]
pub struct UpdateProtectedVault<'info> {
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
        bump = admin.bump,
        constraint = admin.can_perform_protocol_action(Action::UpdateProtectedVault, &settings.access_control) @ RlpError::InvalidSigner,
    )]
    pub admin: Account<'info, UserPermissions>,

    #[account(
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump = settings.bump,
        constraint = !settings.access_control.killswitch.is_frozen(&Action::UpdateProtectedVault) @ RlpError::Frozen,
    )]
    pub settings: Box<Account<'info, Settings>>,

    #[account(
        mut,
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            &liquidity_pool.index.to_le_bytes()
        ],
        bump = liquidity_pool.bump,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
}
