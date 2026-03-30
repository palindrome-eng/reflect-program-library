use std::io::Write;
use strum_macros::EnumIter;
use crate::states::*;
use anchor_lang::prelude::{*, borsh::BorshSchema};
use crate::errors::RlpError;

pub const MAX_ACTION_MAPPINGS: usize = 18;
pub const MAX_ROLES: usize = 18;

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
            _ => Err(error!(RlpError::InvalidState)),
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
    pub role_count: u8,
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

    pub fn add_role(&mut self, role: Role) -> Result<()> {
        if self.role_count as usize >= MAX_ROLES {
            return Err(RlpError::NoEntriesLeft.into());
        }

        for i in 0..self.role_count as usize {
            if self.allowed_roles[i] == role {
                return Err(RlpError::ActionHasAssignedRole.into());
            }
        }

        if self.role_count as usize >= MAX_ROLES {
            return Err(RlpError::NoEntriesLeft.into());
        }

        self.allowed_roles[self.role_count as usize] = role;
        self.role_count += 1;

        Ok(())
    }

    pub fn remove_role(&mut self, role: Role) -> Result<()> {
        if self.role_count == 0 {
            return Err(RlpError::NoEntriesLeft.into());
        }

        let mut found = false;

        for i in 0..self.role_count as usize {
            if self.allowed_roles[i] == role {
                for j in i..(self.role_count as usize - 1) {
                    self.allowed_roles[j] = self.allowed_roles[j + 1];
                }
                self.allowed_roles[self.role_count as usize - 1] = Role::UNSET;
                self.role_count -= 1;
                found = true;
                break;
            }
        }

        if found {
            Ok(())
        } else {
            Err(RlpError::RoleNotUnderAction.into())
        }
    }

    pub fn has_role(&self, role: Role) -> bool {
        for i in 0..self.role_count as usize {
            if self.allowed_roles[i] == role {
                return true;
            }
        }
        false
    }
}

#[repr(C)]
#[derive(BorshSchema, AnchorDeserialize, AnchorSerialize, Default, Debug, PartialEq, Eq, Clone, InitSpace, Copy)]
pub struct AccessMap {
    pub action_permissions: [ActionMapping; MAX_ACTION_MAPPINGS],
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
    
    pub fn get_action_allowees(&self, action: Action) -> Option<&[Role]> {
        self.action_permissions
            .iter()
            .find(|mapping| mapping.action == action)
            .map(|mapping| &mapping.allowed_roles[..mapping.role_count as usize])
    }

    pub fn is_public_action(&self, action: Action) -> bool {
        self.action_permissions
            .iter()
            .find(|mapping| mapping.action == action)
            .map_or(false, |mapping| {
                (0..mapping.role_count as usize)
                    .any(|i| mapping.allowed_roles[i].is_public())
            })
    }

    pub fn add_role_to_action(&mut self, action: Action, role: Role) -> Result<()> {
        for i in 0..self.action_permissions.len() {
            if self.action_permissions[i].action == action && self.action_permissions[i].role_count > 0 {
                return self.action_permissions[i].add_role(role);
            }
        }
        
        for i in 0..self.action_permissions.len() {
            if self.action_permissions[i].role_count == 0 {
                self.action_permissions[i].action = action;
                let result = self.action_permissions[i].add_role(role);

                if result.is_ok() {
                    if self.mapping_count + 1 <= MAX_ACTION_MAPPINGS as u8{
                        self.mapping_count += 1;
                    } else{
                        return Err(RlpError::NoEntriesLeft.into())
                    }
                }

                return result;
            }
        }

        Err(RlpError::NoEntriesLeft.into())
    }
    
    pub fn remove_role_from_action(&mut self, action: Action, role: Role) -> Result<()> {
        for i in 0..self.action_permissions.len() {
            if self.action_permissions[i].action == action && self.action_permissions[i].role_count > 0 {
                let result = self.action_permissions[i].remove_role(role);
                
                if result.is_ok() && self.action_permissions[i].role_count == 0 {
                    self.mapping_count = self.mapping_count.saturating_sub(1);
                }
                
                return result;
            }
        }
        
        Err(RlpError::ActionNotFound.into())
    }
}

#[repr(C)]
#[derive(BorshSchema, AnchorDeserialize, AnchorSerialize, Default, Debug, Clone, InitSpace)]
pub struct AccessControl {
    pub access_map: AccessMap,
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
    
    pub fn new_defaults() -> Result<Self> {
        let mut access_control: AccessControl = Self::default();
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

        access_control.add_role_to_action(Action::Slash, Role::CRANK)?;
        access_control.add_role_to_action(Action::Swap, Role::CRANK)?;

        access_control.add_role_to_action(Action::Restake, Role::TESTEE)?;
        access_control.add_role_to_action(Action::Withdraw, Role::TESTEE)?;

        access_control.add_role_to_action(Action::FreezeRestake, Role::FREEZE)?;
        access_control.add_role_to_action(Action::FreezeWithdraw, Role::FREEZE)?;
        access_control.add_role_to_action(Action::FreezeSlash, Role::FREEZE)?;
        access_control.add_role_to_action(Action::FreezeSwap, Role::FREEZE)?;

        Ok(access_control)
    }
      
    pub fn add_role_to_action(&mut self, action: Action, role: Role) -> Result<()> {
        self.access_map.add_role_to_action(action, role)
    }

    pub fn remove_role_from_action(&mut self, action: Action, role: Role) -> Result<()> {
        self.access_map.remove_role_from_action(action, role)
    }

    pub fn is_public_action(&self, action: Action) -> bool {
        self.access_map.is_public_action(action)
    }
}