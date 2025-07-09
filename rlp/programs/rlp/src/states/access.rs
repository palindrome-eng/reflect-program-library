use std::io::Write;
use strum_macros::EnumIter;
use crate::states::*;
use anchor_lang::prelude::{*, borsh::BorshSchema};
use crate::errors::InsuranceFundError;

// Any more than 18 and it will fail spectacularly.
pub const MAX_ACTION_MAPPINGS: usize = 18;
pub const MAX_ROLES: usize = 18;

// ----- Role Enum -----
#[derive(BorshSchema, AnchorSerialize,  Default, AnchorDeserialize, Copy, Clone, Debug, PartialEq, Eq, InitSpace, EnumIter)]
pub enum Role {
    #[default]
    UNSET,
    PUBLIC, 
    TESTEE,
    FREEZE,
    CRANK,
    MANAGER,    
    SUPREMO
}

impl Role {        
    pub fn is_public(&self) -> bool {
        *self == Role::PUBLIC
    }

    pub fn deserialize(buf: &mut &[u8]) -> Result<Self> {
        let variant = u8::deserialize(buf)?;
        match variant {
            0 => Ok(Role::UNSET),
            1 => Ok(Role::PUBLIC),
            2 => Ok(Role::TESTEE),
            3 => Ok(Role::FREEZE),
            4 => Ok(Role::CRANK),
            5 => Ok(Role::MANAGER),
            6 => Ok(Role::SUPREMO),
            _ => Err(error!(InsuranceFundError::InvalidState)),
        }
    }
    
    pub fn try_serialise<W: Write>(&self, writer: &mut W) -> Result<()> {
        let variant = match self {
            Role::UNSET => 0u8,
            Role::PUBLIC => 1u8,
            Role::TESTEE => 2u8,
            Role::FREEZE => 3u8,
            Role::CRANK => 4u8,
            Role::MANAGER => 5u8,
            Role::SUPREMO => 6u8,
        };
        
        variant.serialize(writer)?;
        Ok(())
    }
}

#[repr(C)]
#[derive(BorshSchema, AnchorDeserialize, AnchorSerialize, Default, Debug, PartialEq, Eq, Clone, InitSpace, Copy)]
pub struct ActionMapping {
    pub action: Action,
    pub allowed_roles: [Role; MAX_ROLES],
    pub role_count: u8, // Keep track of the number of roles in the array.
}

impl ActionMapping {
    pub fn deserialize(buf: &mut &[u8]) -> Result<Self> {
        let action = Action::deserialize(buf)?;        
        let mut allowed_roles = [Role::default(); MAX_ROLES];        
        
        for i in 0..MAX_ROLES {
            allowed_roles[i] = Role::deserialize(buf)?;
        }
        
        let role_count = u8::deserialize(buf)?;
        
        Ok(ActionMapping {
            action,
            allowed_roles,
            role_count,
        })
    }

    pub fn try_serialise<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.action.try_serialise(writer)?;        

        for role in &self.allowed_roles {
            role.try_serialise(writer)?;
        }        

        self.role_count.serialize(writer)?;        
        Ok(())
    }

    /// Add a role to this action mapping.
    pub fn add_role(&mut self, role: Role) -> Result<()> {
        if self.role_count as usize >= MAX_ROLES {
            msg!("Action {:?} already has maximum allowed roles ({})", self.action, MAX_ACTION_MAPPINGS);
            msg!("Cannot add role {:?} to this action", role);
            return Err(InsuranceFundError::NoEntriesLeft.into());
        }

        // Check if the action is valid.
        // Check if the role is already in the allowed_roles.
        for i in 0..self.role_count as usize {
            if self.allowed_roles[i] == role {
                msg!("Role {:?} is already assigned to action {:?}", role, self.action);
                return Err(InsuranceFundError::ActionHasAssignedRole.into());
            }
        }
        
        // Check if there's room for another role.
        if self.role_count as usize >= MAX_ROLES {
            msg!("Action {:?} already has maximum allowed roles ({})", self.action, MAX_ROLES);
            msg!("Cannot add role {:?} to this action", role);
            return Err(InsuranceFundError::NoEntriesLeft.into());
        }        
        
        self.allowed_roles[self.role_count as usize] = role;
        self.role_count += 1;
        msg!("Added role {:?} to action {:?}", role, self.action);
        
        Ok(())
    }
    
    /// Remove a role from this action mapping.
    pub fn remove_role(&mut self, role: Role) -> Result<()> {
        if self.role_count == 0 {
            msg!("Action {:?} has no roles assigned", self.action);
            msg!("Cannot remove role {:?} from this action", role);            
            return Err(InsuranceFundError::NoEntriesLeft.into());
        }

        let mut found = false;
        
        // Find and remove the role.
        for i in 0..self.role_count as usize {
            if self.allowed_roles[i] == role {
                // Shift roles to fill the gap.
                for j in i..(self.role_count as usize - 1) {
                    self.allowed_roles[j] = self.allowed_roles[j + 1];
                }
                self.allowed_roles[self.role_count as usize - 1] = Role::UNSET;
                self.role_count -= 1;
                found = true;
                msg!("Removed role {:?} from action {:?}", role, self.action);
                break;
            }
        }
        
        if found {
            Ok(())
        } else {
            msg!("{:?} can not be done by {:?}", self.action, role);
            Err(InsuranceFundError::RoleNotUnderAction.into())
        }
    }    

    /// Check if a specific role is assigned to this action.
    pub fn has_role(&self, role: Role) -> bool {
        for i in 0..self.role_count as usize {
            if self.allowed_roles[i] == role {
                return true;
            }
        }
        false
    }
}


// ----- Access Map -----

#[repr(C)]
#[derive(BorshSchema, AnchorDeserialize, AnchorSerialize, Default, Debug, PartialEq, Eq, Clone, InitSpace, Copy)]
pub struct AccessMap {
    /** Maps each regular action to its permissions. */    
    pub action_permissions: [ActionMapping; MAX_ACTION_MAPPINGS],
    /** Counter to track how many mappings are in use. */
    pub mapping_count: u8,
}

impl AccessMap {
    pub fn deserialize(buf: &mut &[u8]) -> Result<Self> {
        let mut action_permissions = [ActionMapping::default(); MAX_ACTION_MAPPINGS];        
        for i in 0..MAX_ACTION_MAPPINGS {
            action_permissions[i] = ActionMapping::deserialize(buf)?;
        }
        
        let mapping_count = u8::deserialize(buf)?;
        
        Ok(AccessMap {
            action_permissions,
            mapping_count,
        })
    }
    
    pub fn try_serialise<W: Write>(&self, writer: &mut W) -> Result<()> {
        for mapping in &self.action_permissions {
            mapping.try_serialise(writer)?;
        }
        
        self.mapping_count.serialize(writer)?;
        
        Ok(())
    }
    
    /** Get roles that can perform a specific action. */
    pub fn get_action_allowees(&self, action: Action) -> Option<&[Role]> {
        self.action_permissions
            .iter()  // Search the full array.
            .find(|mapping| mapping.action == action)  // Find mapping regardless of role_count
            .map(|mapping| &mapping.allowed_roles[..mapping.role_count as usize])  // Return slice (empty if role_count=0)
    }
    
    /** Check if an action is public (anyone can perform it). */
    pub fn is_public_action(&self, action: Action) -> bool {
        self.action_permissions
            .iter()
            .find(|mapping| mapping.action == action)
            .map_or(false, |mapping| {
                (0..mapping.role_count as usize)
                    .any(|i| mapping.allowed_roles[i].is_public())
            })
    }

    /** Add a role to the permit list for a given action. */
    pub fn add_role_to_action(&mut self, action: Action, role: Role) -> Result<()> {
        
        // First try to find existing mapping for this action.
        for i in 0..self.action_permissions.len() {
            if self.action_permissions[i].action == action && self.action_permissions[i].role_count > 0 {
                return self.action_permissions[i].add_role(role);
            }
        }
        
        // No existing mapping found, look for empty slot.
        for i in 0..self.action_permissions.len() {
            if self.action_permissions[i].role_count == 0 {
                // Set the action and add the role
                self.action_permissions[i].action = action;
                let result = self.action_permissions[i].add_role(role);
                
                // Increment the count if successful.
                if result.is_ok() {
                    if self.mapping_count + 1 <= MAX_ACTION_MAPPINGS as u8{
                        self.mapping_count += 1;
                    } else{
                        return Err(InsuranceFundError::NoEntriesLeft.into())
                    }                 
                }
                
                return result;
            }
        }       
        
        msg!("No slots available");
        Err(InsuranceFundError::NoEntriesLeft.into())
    }
    
    /** Remove a role from the permit list for a given action. */
    pub fn remove_role_from_action(&mut self, action: Action, role: Role) -> Result<()> {
        // Try to find a mapping with this action
        for i in 0..self.action_permissions.len() {
            if self.action_permissions[i].action == action && self.action_permissions[i].role_count > 0 {
                let result = self.action_permissions[i].remove_role(role);
                
                // If the removal was successful and now there are no roles left.
                if result.is_ok() && self.action_permissions[i].role_count == 0 {
                    self.mapping_count = self.mapping_count.saturating_sub(1);                   
                }
                
                return result;
            }
        }
        
        // Action mapping not found.
        msg!("Action {:?} not found", action);
        Err(InsuranceFundError::ActionNotFound.into())
    }
}


// ----- Access Control -----

#[repr(C)]
#[derive(BorshSchema, AnchorDeserialize, AnchorSerialize, Default, Debug, Clone, InitSpace)]
pub struct AccessControl {    
    /** Access map for this component. */
    pub access_map: AccessMap,
    /** State for freezing particular functionality. */
    pub killswitch: KillSwitch,
}

impl AccessControl {

    pub fn deserialize(buf: &mut &[u8]) -> Result<Self> {        
        Ok(AccessControl {
            access_map: AccessMap::deserialize(buf)?,
            killswitch: KillSwitch::deserialize(buf)?,
        })
    }

    pub fn action_unsuspended(&self, action: &Action) -> Result<()> {
        self.killswitch.action_unsuspended(action)
    }

    pub fn try_serialise<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.access_map.try_serialise(writer)?;
        self.killswitch.try_serialise(writer)?;
        Ok(())
    }
    
    /** Initialise with default permissions. */
    pub fn new_defaults() -> Result<Self> {
        let mut access_control: AccessControl = Self::default();                
                
        // Manager actions (everything except deploying new strategies/accounts).
        access_control.add_role_to_action(Action::UpdateDepositCap, Role::MANAGER)?;
        access_control.add_role_to_action(Action::UpdateRole, Role::MANAGER)?;
        access_control.add_role_to_action(Action::AddAsset, Role::MANAGER)?;
        access_control.add_role_to_action(Action::Management, Role::MANAGER)?;
        access_control.add_role_to_action(Action::Restake, Role::MANAGER)?;
        access_control.add_role_to_action(Action::Withdraw, Role::MANAGER)?;
        access_control.add_role_to_action(Action::FreezeRestake, Role::MANAGER)?;
        access_control.add_role_to_action(Action::FreezeWithdraw, Role::MANAGER)?;
        access_control.add_role_to_action(Action::SuspendDeposits, Role::MANAGER)?;
        access_control.add_role_to_action(Action::UpdateAction, Role::MANAGER)?;

        // Crank actions.  
        access_control.add_role_to_action(Action::Slash, Role::CRANK)?;
        access_control.add_role_to_action(Action::Swap, Role::CRANK)?;

        // User actions - public (test for now).    
        access_control.add_role_to_action(Action::Restake, Role::TESTEE)?;
        access_control.add_role_to_action(Action::Withdraw, Role::TESTEE)?;

        // Freeze actions.
        access_control.add_role_to_action(Action::FreezeRestake, Role::FREEZE)?;
        access_control.add_role_to_action(Action::FreezeWithdraw, Role::FREEZE)?;            
        
        Ok(access_control)
    }
      
    /** Allows a given role to perform an action. */    
    pub fn add_role_to_action(&mut self, action: Action, role: Role) -> Result<()> {        
        self.access_map.add_role_to_action(action, role)
    }

    /** Removes a given role the right perform an action. */    
    pub fn remove_role_from_action(&mut self, action: Action, role: Role) -> Result<()> {        
        self.access_map.remove_role_from_action(action, role)
    }
    
    /** Check if an action is public (anyone can perform it). */
    pub fn is_public_action(&self, action: Action) -> bool {
        self.access_map.is_public_action(action)
    }
}