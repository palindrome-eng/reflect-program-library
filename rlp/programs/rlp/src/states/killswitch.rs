use std::io::Write;
use anchor_lang::prelude::{*, borsh::BorshSchema};
use crate::states::*;
use crate::errors::RlpError;

#[repr(C)]
#[derive(BorshSchema, AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq, Eq, InitSpace, Default)]
pub struct KillSwitch {
    pub frozen: u16,
}

impl KillSwitch {
    pub fn deserialize(buf: &mut &[u8]) -> Result<Self> {
        let frozen = u16::deserialize(buf)?;
        Ok(KillSwitch { frozen })
    }

    pub fn try_serialise<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.frozen.serialize(writer)?;
        Ok(())
    }

    pub fn is_frozen(&self, action: &Action) -> bool {
        let mask = 1u16 << (*action as u16);
        (self.frozen & mask) != 0
    }

    pub fn action_unsuspended(&self, action: &Action) -> Result<()> {
        if !self.is_frozen(action) {
            return Ok(());
        }
        Err(error!(RlpError::ActionFrozen))
    }

    pub fn freeze(&mut self, action: &Action) {
        let mask = 1u16 << (*action as u16);
        self.frozen |= mask;
    }

    pub fn unfreeze(&mut self, action: &Action) {
        let mask = 1u16 << (*action as u16);
        self.frozen &= !mask;
    }
}