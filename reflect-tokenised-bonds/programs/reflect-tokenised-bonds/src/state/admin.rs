use anchor_lang::prelude::*;

#[derive(AnchorDeserialize, AnchorSerialize, Clone, InitSpace)]
pub enum Permissions {
    InitializeVaults,
    Freeze,
    Superadmin
}

#[account]
#[derive(InitSpace)]
pub struct Admin {
    pub pubkey: Pubkey,
    #[max_len(5)]
    pub permissions: Vec<Permissions>
}