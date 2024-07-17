use anchor_lang::prelude::*;

// UserAccount keeps track of user's lockups.
// Without a per-user counter of lockups, it's hard to easily 
// derive `lockup` PDA used in lockup and withdrawal.

#[account]
pub struct UserAccount {
    pub lockups: u64,
}