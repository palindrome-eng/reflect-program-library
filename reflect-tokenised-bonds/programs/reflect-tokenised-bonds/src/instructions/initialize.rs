use anchor_lang::prelude::*;
use crate::constants::{
    CONFIG_SEED,
    ADMIN_SEED
};
use crate::program::ReflectTokenisedBonds;
use crate::state::{
    Admin,
    Config,
    Permissions
};
use crate::errors::ReflectError;

pub fn initialize(
    ctx: Context<Initialize>
) -> Result<()> {
    let Initialize {
        signer,
        admin,
        config,
        program: _,
        program_data: __,
        system_program: ___
    } = ctx.accounts;

    config.set_inner(Config {
        bump: ctx.bumps.config,
        vaults: 0,
        frozen: false,
    });

    admin.set_inner(Admin {
        pubkey: signer.key(),
        permissions: vec![
            Permissions::Superadmin
        ]
    });

    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        mut
    )]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        space = Admin::INIT_SPACE,
        seeds = [
            ADMIN_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump
    )]
    pub admin: Account<'info, Admin>,

    #[account(
        init,
        seeds = [
            CONFIG_SEED.as_bytes()
        ],
        bump,
        space = 8 + Config::INIT_SPACE,
        payer = signer,
    )]
    pub config: Account<'info, Config>,

    #[account()]
    pub system_program: Program<'info, System>,

    #[account()]
    pub program: Program<'info, ReflectTokenisedBonds>,

    #[account(
        constraint = program
            .programdata_address()?
            .is_some_and(|result| result.eq(&program_data.key())) @ ReflectError::ProgramAccountsMismatch,
        
        constraint = program_data.upgrade_authority_address.eq(&Some(signer.key())) @ ReflectError::InvalidSigner
    )]
    pub program_data: Account<'info, ProgramData>,
}