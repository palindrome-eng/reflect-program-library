use anchor_lang::prelude::*;
use crate::errors::InsuranceFundError;
use crate::states::{Role, Action, AccessControl};

/** Handles role management at any hierarchy level (protocol or strategy) with methods for:
- Checking if a specific role exists
- Adding and removing roles
- Checking if the SUPREMO role exists
- Validating if roles can perform specific actions
*/
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
            return Err(InsuranceFundError::InvalidInput.into());
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
            Err(InsuranceFundError::InvalidInput.into())
        }
    }

    pub fn can_perform_action(&self, action: Action, access_control: &AccessControl) -> bool {        
        if self.is_supremo() {         
            return true;
        }
        
        // Check if any of the roles allows fit this action.
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
    /** Bump for PDA derivation. */
    pub bump: u8,
    /** Account authority - the entity that can modify this permission set. */
    pub authority: Pubkey,    
    /** Protocol-level roles - permissions that apply across the entire protocol. */
    pub protocol_roles: LevelRoles,
}

impl UserPermissions {
    pub const DISCRIMINATOR_SIZE: usize = 8;
    pub const PUBKEY_SIZE: usize = 32;
    pub const BUMP_SIZE: usize = 1;
    // Adjust vector size estimates based on your expected usage
    pub const PROTOCOL_ROLES_SIZE: usize = 4 + (5 * 1); // Vec length + max 5 roles
    pub const STRATEGY_PERMISSIONS_SIZE: usize = 4 + (20 * (1 + 4 + (5 * 1))); // Vec + max 20 strategies
    
    pub const TOTAL_SIZE: usize = 
        Self::DISCRIMINATOR_SIZE +
        (Self::PUBKEY_SIZE) + // authority + user
        Self::PROTOCOL_ROLES_SIZE +
        Self::STRATEGY_PERMISSIONS_SIZE +
        Self::BUMP_SIZE;   
    
    
    pub fn add_protocol_role(&mut self, role: Role) -> Result<()> {
        self.protocol_roles.add_role(role)
    }
    
    pub fn remove_protocol_role(&mut self, role: Role) -> Result<()> {
        self.protocol_roles.remove_role(role)
    }
    
    /// Check if user has a protocol role. 
    pub fn has_protocol_role(&self, role: Role) -> bool {
        self.protocol_roles.has_role(role)
    }
    
    /// Check if provided address is a protocol supremo.
    pub fn is_super_admin(&self) -> bool {
        self.protocol_roles.is_supremo()
    }

    /// Validate if user is supreme.
    pub fn validate_supremo(&self) -> Result<()> {
        match self.is_super_admin() {
            true => Ok(()),
            false => Err(InsuranceFundError::InvalidSigner.into()),
        }
    }   
    
    /// Check if user can perform a given action at protocol level.
    pub fn can_perform_protocol_action(&self, action: Action, access_control: &AccessControl) -> bool {
        self.protocol_roles.can_perform_action(action, access_control) 
    }
}