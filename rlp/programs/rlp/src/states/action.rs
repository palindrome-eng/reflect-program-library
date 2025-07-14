use std::io::Write;
use anchor_lang::prelude::{*, borsh::BorshSchema};
use strum_macros::EnumIter;
use crate::errors::InsuranceFundError;

#[derive(BorshSchema, AnchorSerialize,  Default, AnchorDeserialize, Copy, Clone, Debug, PartialEq, Eq, InitSpace, EnumIter)]
pub enum Action {
    #[default]
    // Core actions
    /** Restakes asset. */
    Restake = 0,
    /** Withdraws asset */
    Withdraw = 2,
/** Slashes LP */
    Slash = 4,
    /** Swaps between two assets in the LP. */
    Swap = 6,

    // Core actions freeze
    /** Freezes Mint action. */
    FreezeRestake = 1,  
    /** Freezes Redeem action. */
    FreezeWithdraw = 3,
    /** Freezes Slash action. */
    FreezeSlash = 5,
    /** Freezes Swap action. */
    FreezeSwap = 7,

    InitializeLiquidityPool = 8,
    /** Adds new asset to the LP. */
    AddAsset = 9,
    /** Updates how much of asset can be deposited in the LP. */
    UpdateDepositCap = 10,
    /** Allows depositing liquidity without increasing supply of LP token. */
    DepositRewards = 11,
    /** Generic management. */
    Management = 12,    
    SuspendDeposits = 13,     
    FreezeProgram = 14,    
    UpdateRole = 15,
    UpdateAction = 16,
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
            4 => Ok(Action::Slash),
            5 => Ok(Action::FreezeSlash),
            6 => Ok(Action::Swap),
            7 => Ok(Action::FreezeSwap),
            8 => Ok(Action::InitializeLiquidityPool),
            9 => Ok(Action::AddAsset),
            10 => Ok(Action::UpdateDepositCap),
            11 => Ok(Action::DepositRewards),
            12 => Ok(Action::Management),
            13 => Ok(Action::SuspendDeposits),
            14 => Ok(Action::FreezeProgram),
            15 => Ok(Action::UpdateRole),
            16 => Ok(Action::UpdateAction),
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
            Action::Slash => 4u8,
            Action::FreezeSlash => 5u8,
            Action::Swap => 6u8,
            Action::FreezeSwap => 7u8,
            Action::InitializeLiquidityPool => 8u8,
            Action::AddAsset => 9u8,
            Action::UpdateDepositCap => 10u8,
            Action::DepositRewards => 11u8,
            Action::Management => 12u8,
            Action::SuspendDeposits => 13u8,
            Action::FreezeProgram => 14u8,
            Action::UpdateRole => 15u8,
            Action::UpdateAction => 16u8,
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
            4 => Some(Action::Slash),
            5 => Some(Action::FreezeSlash),
            6 => Some(Action::Swap),
            7 => Some(Action::FreezeSwap),
            8 => Some(Action::InitializeLiquidityPool),
            9 => Some(Action::AddAsset),
            10 => Some(Action::UpdateDepositCap),
            11 => Some(Action::DepositRewards),
            12 => Some(Action::Management),
            13 => Some(Action::SuspendDeposits),
            14 => Some(Action::FreezeProgram),
            15 => Some(Action::UpdateRole),
            16 => Some(Action::UpdateAction),
            _ => None,
        }
    }

    /** Checks if the action is recurrant and can be frozen. */
    pub fn is_core(&self) -> bool {
        match self {
            Action::Restake | Action::Withdraw | Action::Swap | Action::Slash => true,
            _ => false,
        }
    }

    /** Converts a freeze action to its corresponding regular action. */
    pub fn to_action(&self) -> Result<Self> {
        match self {
            Action::FreezeRestake => Ok(Action::Restake),
            Action::FreezeWithdraw => Ok(Action::Withdraw),
            Action::FreezeSlash => Ok(Action::Slash),
            Action::FreezeSwap => Ok(Action::Swap),
            _ => Err(InsuranceFundError::ActionNotFound.into()),
        }
    }
}

