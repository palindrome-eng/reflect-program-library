use anchor_lang::prelude::*;
use crate::states::*;
use crate::constants::*;
use crate::errors::RlpError;
use crate::events::UpdateDepositCapEvent;

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

    let liquidity_pool = &mut ctx.accounts.liquidity_pool;
    liquidity_pool.deposit_cap = new_cap;

    emit!(UpdateDepositCapEvent {
        admin: ctx.accounts.signer.key(),
        liquidity_pool: liquidity_pool.key(),
        new_cap
    });

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
        bump = admin.bump,
        constraint = admin.can_perform_protocol_action(Action::UpdateDepositCap, &settings.access_control) @ RlpError::InvalidSigner,
    )]
    pub admin: Account<'info, UserPermissions>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump = settings.bump,
        // Security Fix: Changed from is_frozen to !is_frozen
        // The constraint should FAIL when frozen, not when unfrozen
        constraint = !settings.access_control.killswitch.is_frozen(&Action::UpdateDepositCap) @ RlpError::Frozen,
    )]
    pub settings: Account<'info, Settings>,

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