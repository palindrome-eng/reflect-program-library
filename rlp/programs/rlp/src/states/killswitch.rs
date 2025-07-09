use std::io::Write;
use anchor_lang::prelude::{*, borsh::BorshSchema};
use crate::states::*;
use crate::errors::InsuranceFundError;

#[repr(C)]
#[derive(BorshSchema, AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace, Default)]
pub struct KillSwitch {
    /** Bool indices:
    - [0] - mint
    - [1] - redeem
    - [2] - rebalance
    - [3] - capture (print and distribute stable)
     */
    pub frozen: u8,
}

impl KillSwitch {
    pub fn deserialize(buf: &mut &[u8]) -> Result<Self> {
        let frozen = u8::deserialize(buf)?;
        Ok(KillSwitch { frozen })
    }
    
    pub fn try_serialise<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.frozen.serialize(writer)?;
        Ok(())
    }
    
    /** Check if an action is frozen. */
    pub fn is_frozen(&self, action: &Action) -> bool {
        let mask = 1u8 << (*action as u8);
        (self.frozen & mask) != 0
    }    

    /** Throw if action can't be performed. */
    pub fn action_unsuspended(&self, action: &Action) -> Result<()> {
        if !self.is_frozen(action) { 
            return Ok(()); 
        }        
        Err(error!(InsuranceFundError::ActionFrozen))
    }

    /** Freeze an action. */
    pub fn freeze(&mut self, action: &Action) {
        let mask = 1u8 << (*action as u8);
        self.frozen |= mask;
    }

    /** Unfreeze an action. */
    pub fn unfreeze(&mut self, action: &Action) {
        let mask = 1u8 << (*action as u8);
        self.frozen &= !mask;
    }
}