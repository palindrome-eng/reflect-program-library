use anchor_lang::prelude::*;

#[derive(InitSpace)]
#[account]
pub struct LpLockup {
    pub duration: u64,
    pub deposits: u64,
    // pub lp_token: u64, // not essential
}