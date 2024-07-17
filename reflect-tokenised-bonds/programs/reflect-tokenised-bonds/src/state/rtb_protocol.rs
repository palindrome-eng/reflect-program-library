use anchor_lang::prelude::*;

#[account]
pub struct RTBProtocol {
    pub next_vault_seed: u64,
}