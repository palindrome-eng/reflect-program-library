use anchor_lang::prelude::*;
use crate::{constants::RTB_SEED, state::*};

pub fn initialize_protocol(
    ctx: Context<InitializeProtocol>
) -> Result<()> {
    Ok(())
}

#[derive(Accounts)]
pub struct InitializeProtocol<'info> {
    #[account(
        mut
    )]
    pub payer: Signer<'info>,

    #[account(
        init,
        seeds = [
            RTB_SEED.as_bytes()
        ],
        bump,
        space = 8 + 8,
        payer = payer,
    )]
    pub rtb_protocol: Account<'info, RTBProtocol>,

    #[account()]
    pub system_program: Program<'info, System>,
}