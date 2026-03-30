use anchor_lang::prelude::*;
use crate::errors::RlpError;
use crate::states::{Role, Action, AccessControl};

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, Default, InitSpace)]
pub struct LevelRoles {
    #[max_len(10)]
    pub roles: Vec<Role>,
}

impl LevelRoles {
    pub fn new(role: Role) -> Self{
        let mut new_roles = LevelRoles::default();
        new_roles.roles.push(role);
        new_roles
    }

    pub fn has_role(&self, role: Role) -> bool {
        self.roles.contains(&role)
    }

    pub fn is_supremo(&self) -> bool {
        self.has_role(Role::SUPREMO)
    }

    pub fn add_role(&mut self, role: Role) -> Result<()> {
        if self.has_role(role) {
            return Err(RlpError::InvalidInput.into());
        }

        self.roles.push(role);
        Ok(())
    }

    pub fn remove_role(&mut self, role: Role) -> Result<()> {
        let initial_len: usize = self.roles.len();
        self.roles.retain(|&r| r != role);

        if self.roles.len() < initial_len {
            Ok(())
        } else {
            Err(RlpError::InvalidInput.into())
        }
    }

    pub fn can_perform_action(&self, action: Action, access_control: &AccessControl) -> bool {
        if self.is_supremo() {
            return true;
        }

        if let Some(allowed_roles) = access_control.access_map.get_action_allowees(action) {
            for role in &self.roles {
                if allowed_roles.contains(role) {
                    return true;
                }
            }
        }

        false
    }
}

#[account]
#[derive(Debug, Default, InitSpace)]
pub struct UserPermissions {
    pub bump: u8,
    pub authority: Pubkey,
    pub protocol_roles: LevelRoles,
}

impl UserPermissions {
    pub const DISCRIMINATOR_SIZE: usize = 8;
    pub const PUBKEY_SIZE: usize = 32;
    pub const BUMP_SIZE: usize = 1;
    pub const PROTOCOL_ROLES_SIZE: usize = 4 + (5 * 1);

    pub const TOTAL_SIZE: usize =
        Self::DISCRIMINATOR_SIZE +
        (Self::PUBKEY_SIZE) +
        Self::PROTOCOL_ROLES_SIZE +
        Self::BUMP_SIZE;

    pub fn add_protocol_role(&mut self, role: Role) -> Result<()> {
        self.protocol_roles.add_role(role)
    }

    pub fn remove_protocol_role(&mut self, role: Role) -> Result<()> {
        self.protocol_roles.remove_role(role)
    }

    pub fn has_protocol_role(&self, role: Role) -> bool {
        self.protocol_roles.has_role(role)
    }

    pub fn is_super_admin(&self) -> bool {
        self.protocol_roles.is_supremo()
    }

    pub fn validate_supremo(&self) -> Result<()> {
        match self.is_super_admin() {
            true => Ok(()),
            false => Err(RlpError::InvalidSigner.into()),
        }
    }

    pub fn can_perform_protocol_action(&self, action: Action, access_control: &AccessControl) -> bool {
        self.protocol_roles.can_perform_action(action, access_control)
    }
}