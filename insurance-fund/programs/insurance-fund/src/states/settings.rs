use anchor_lang::prelude::*;

#[account]
pub struct Settings {
    pub bump: u8,
    pub superadmin: Pubkey,
    pub tvl: u64,
    pub lockups: u64,
    pub whitelisted_assets: Vec<Pubkey>,
    pub deposits_locked: bool,
}

impl Settings {
    pub const SIZE: usize = 8 + 1 + 2 * 8 + 32 + 4 + 1;
}