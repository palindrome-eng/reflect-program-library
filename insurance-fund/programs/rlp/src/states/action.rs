use std::io::Write;
use anchor_lang::prelude::{*, borsh::BorshSchema};
use strum_macros::EnumIter;
use crate::errors::InsuranceFundError;

#[derive(BorshSchema, AnchorSerialize,  Default, AnchorDeserialize, Copy, Clone, Debug, PartialEq, Eq, InitSpace, EnumIter)]
pub enum Action {
    #[default]
    // Core actions
    /** Mints stable. */
    Restake = 0,
    /** Redeems input spl. */
    Withdraw = 2,

    // Core actions freeze
    /** Freezes Mint action. */
    FreezeRestake = 1,  
    /** Freezes Redeem action. */
    FreezeWithdraw = 3,

    InitializeLiquidityPool = 4,
    /** Updates how much stable can be minted. */
    AddAsset = 5,
    /** Updates how much stable can be minted. */
    UpdateDepositCap = 6,
    /** Updates how much stable can be minted. */
    Slash = 7,
    /** Allows to swap between assets within RLP. */
    Swap = 8,
    /** Generic management. */
    Management = 10,    
    SuspendDeposits = 11,     
    FreezeProgram = 12,    
    UpdateRole = 13,
    UpdateAction = 14,
}

impl Action {

     /// Custom deserialization for Action enum
     pub fn deserialize(buf: &mut &[u8]) -> Result<Self> {
        let variant = u8::deserialize(buf)?;
        match variant {
            0 => Ok(Action::Restake),
            1 => Ok(Action::FreezeRestake),
            2 => Ok(Action::Withdraw),
            3 => Ok(Action::FreezeWithdraw),
            4 => Ok(Action::InitializeLiquidityPool),
            5 => Ok(Action::AddAsset),
            6 => Ok(Action::UpdateDepositCap),
            7 => Ok(Action::Slash),
            8 => Ok(Action::Swap),
            10 => Ok(Action::Management),
            11 => Ok(Action::SuspendDeposits),
            12 => Ok(Action::FreezeProgram),
            13 => Ok(Action::UpdateRole),
            14 => Ok(Action::UpdateAction),
            _ => Err(error!(InsuranceFundError::InvalidState)),
        }
    }
    
    /// Custom serialization for Action enum
    pub fn try_serialise<W: Write>(&self, writer: &mut W) -> Result<()> {
        let variant = match self {
            Action::Restake => 0u8,
            Action::FreezeRestake => 1u8,
            Action::Withdraw => 2u8,
            Action::FreezeWithdraw => 3u8,
            Action::InitializeLiquidityPool => 4u8,
            Action::AddAsset => 5u8,
            Action::UpdateDepositCap => 6u8,
            Action::Slash => 7u8,
            Action::Swap => 8u8,
            Action::Management => 10u8,
            Action::SuspendDeposits => 11u8,
            Action::FreezeProgram => 12u8,
            Action::UpdateRole => 13u8,
            Action::UpdateAction => 14u8,
        };
        
        variant.serialize(writer)?;
        Ok(())
    }
    
    pub fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Action::Restake),
            1 => Some(Action::FreezeRestake),
            2 => Some(Action::Withdraw),
            3 => Some(Action::FreezeWithdraw),
            4 => Some(Action::InitializeLiquidityPool),
            5 => Some(Action::AddAsset),
            6 => Some(Action::UpdateDepositCap),
            7 => Some(Action::Slash),
            8 => Some(Action::Swap),
            10 => Some(Action::Management),
            11 => Some(Action::SuspendDeposits),
            12 => Some(Action::FreezeProgram),
            13 => Some(Action::UpdateRole),
            14 => Some(Action::UpdateAction),
            _ => None,
        }
    }

    /** Checks if the action is recurrant and can be frozen. */
    pub fn is_core(&self) -> bool {
        match self {
            Action::Restake | Action::Withdraw => true,
            _ => false,
        }
    }

    /** Converts a freeze action to its corresponding regular action. */
    pub fn to_action(&self) -> Result<Self> {
        match self {
            Action::FreezeRestake => Ok(Action::Restake),
            Action::FreezeWithdraw => Ok(Action::Withdraw),
            _ => Err(InsuranceFundError::ActionNotFound.into()),
        }
    }
}

