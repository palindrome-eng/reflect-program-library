use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Token, TokenAccount, Transfer};
use crate::constants::*;
use crate::errors::RlpError;
use crate::states::*;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct DrainPoolReservesArgs {
    pub pool_id: u8,
}

pub fn drain_pool_reserves<'a>(
    ctx: Context<'_, '_, 'a, 'a, DrainPoolReserves<'a>>,
    _args: DrainPoolReservesArgs,
) -> Result<()> {
    let pool_index = ctx.accounts.liquidity_pool.index;
    let pool_bump = ctx.accounts.liquidity_pool.bump;
    let pool_key = ctx.accounts.liquidity_pool.key();
    let token_program_info = ctx.accounts.token_program.to_account_info();
    let pool_info = ctx.accounts.liquidity_pool.to_account_info();

    let index_bytes = pool_index.to_le_bytes();
    let lp_seeds: &[&[u8]] = &[
        LIQUIDITY_POOL_SEED.as_bytes(),
        &index_bytes,
        std::slice::from_ref(&pool_bump),
    ];

    let remaining_accounts = ctx.remaining_accounts;

    require!(
        remaining_accounts.len() % 2 == 0,
        RlpError::InvalidInput
    );

    let mut i = 0;
    while i < remaining_accounts.len() {
        let pool_ata_info = &remaining_accounts[i];
        let destination_info = &remaining_accounts[i + 1];

        let pool_ata = TokenAccount::try_deserialize(
            &mut pool_ata_info.try_borrow_mut_data()?.as_ref(),
        ).map_err(|_| RlpError::InvalidInput)?;

        require!(
            pool_ata.owner == pool_key,
            RlpError::InvalidInput
        );

        if pool_ata.amount > 0 {
            transfer(
                CpiContext::new_with_signer(
                    token_program_info.to_account_info(),
                    Transfer {
                        from: pool_ata_info.to_account_info(),
                        to: destination_info.to_account_info(),
                        authority: pool_info.to_account_info(),
                    },
                    &[lp_seeds],
                ),
                pool_ata.amount,
            )?;
        }

        i += 2;
    }

    Ok(())
}

#[derive(Accounts)]
#[instruction(args: DrainPoolReservesArgs)]
pub struct DrainPoolReserves<'info> {
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

    #[account(
        seeds = [
            LIQUIDITY_POOL_SEED.as_bytes(),
            &args.pool_id.to_le_bytes()
        ],
        bump = liquidity_pool.bump,
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,

    pub token_program: Program<'info, Token>,
}
