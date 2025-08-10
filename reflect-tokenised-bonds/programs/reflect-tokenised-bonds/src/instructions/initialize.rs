use anchor_lang::prelude::*;
use crate::constants::{
    CONFIG_SEED
};
use crate::state::{
    Config,
};

pub fn initialize(
    ctx: Context<Initialize>
) -> Result<()> {
    let Initialize {
        signer,
        config,
        system_program: _
    } = ctx.accounts;

    config.set_inner(Config {
        bump: ctx.bumps.config,
        vaults: 0,
        frozen: false,
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
}