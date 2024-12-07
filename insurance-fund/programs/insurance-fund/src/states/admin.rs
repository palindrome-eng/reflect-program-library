use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, PartialOrd)]
pub enum Permissions {
    // Only gives ability to freeze the protocol.
    // This should be used for team members' wallets who can easily execute freeze of the protocol in case of emergency.
    Freeze = 0,
    // Gives ability to whitelist new assets for restaking.
    AddAsset = 1,
    // Gives full authority over the program management, including removing and adding new admins of a lower scopes.
    Superadmin = 2
}

#[account]
pub struct Admin {
    pub index: u8,
    pub address: Pubkey,
    pub permissions: Permissions
}

impl Admin {
    pub const SIZE: usize = 8 + 8 + 32 + (1 + 1);

    pub fn has_permissions(&self, activity: Permissions) -> bool {
        self.permissions >= activity
    }

    pub fn has_permissions_over(&self, activity: Permissions) -> bool {
        self.permissions > activity
    }
}