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
    Withdraw = 1,
/** Slashes LP */
    Slash = 2,
    /** Swaps between two assets of access level Public in the LP. */
    PublicSwap = 3,
    /** Swaps between two assets of any access level in the LP. */
    PrivateSwap = 4,

    // Core actions freeze
    /** Freezes Mint action. */
    FreezeRestake = 5,  
    /** Freezes Redeem action. */
    FreezeWithdraw = 6,
    /** Freezes Slash action. */
    FreezeSlash = 7,
    /** Freezes PublicSwap action. */
    FreezePublicSwap = 8,
    /** Freezes PrivateSwap action. */
    FreezePrivateSwap = 9,

    InitializeLiquidityPool = 10,
    /** Adds new asset to the LP. */
    AddAsset = 11,
    /** Updates how much of asset can be deposited in the LP. */
    UpdateDepositCap = 12,
    /** Allows depositing liquidity without increasing supply of LP token. */
    DepositRewards = 13,
    /** Generic management. */
    Management = 14,
    SuspendDeposits = 15, 
    UpdateRole = 16,
    UpdateAction = 17,
}

impl Action {

     /// Custom deserialization for Action enum
     pub fn deserialize(buf: &mut &[u8]) -> Result<Self> {
        let variant = u8::deserialize(buf)?;
        match variant {
            0 => Ok(Action::Restake),
            1 => Ok(Action::Withdraw),
            2 => Ok(Action::Slash),
            3 => Ok(Action::PublicSwap),
            4 => Ok(Action::PrivateSwap),
            5 => Ok(Action::FreezeRestake),
            6 => Ok(Action::FreezeWithdraw),
            7 => Ok(Action::FreezeSlash),
            8 => Ok(Action::FreezePublicSwap),
            9 => Ok(Action::FreezePrivateSwap),
            10 => Ok(Action::InitializeLiquidityPool),
            11 => Ok(Action::AddAsset),
            12 => Ok(Action::UpdateDepositCap),
            13 => Ok(Action::DepositRewards),
            14 => Ok(Action::Management),
            15 => Ok(Action::SuspendDeposits),
            16 => Ok(Action::UpdateRole),
            17 => Ok(Action::UpdateAction),
            _ => Err(error!(InsuranceFundError::InvalidState)),
        }
    }
    
    /// Custom serialization for Action enum
    pub fn try_serialise<W: Write>(&self, writer: &mut W) -> Result<()> {
        let variant = match self {
            Action::Restake => 0u8,
            Action::Withdraw => 1u8,
            Action::Slash => 2u8,
            Action::PublicSwap => 3u8,
            Action::PrivateSwap => 4u8,
            Action::FreezeRestake => 5u8,
            Action::FreezeWithdraw => 6u8,
            Action::FreezeSlash => 7u8,
            Action::FreezePublicSwap => 8u8,
            Action::FreezePrivateSwap => 9u8,
            Action::InitializeLiquidityPool => 10u8,
            Action::AddAsset => 11u8,
            Action::UpdateDepositCap => 12u8,
            Action::DepositRewards => 13u8,
            Action::Management => 14u8,
            Action::SuspendDeposits => 15u8,
            Action::UpdateRole => 16u8,
            Action::UpdateAction => 17u8,
        };
        
        variant.serialize(writer)?;
        Ok(())
    }
    
    pub fn from_u8(byte: u8) -> Option<Self> {
        match byte {
            0 => Some(Action::Restake),
            1 => Some(Action::Withdraw),
            2 => Some(Action::Slash),
            3 => Some(Action::PublicSwap),
            4 => Some(Action::PrivateSwap),
            5 => Some(Action::FreezeRestake),
            6 => Some(Action::FreezeWithdraw),
            7 => Some(Action::FreezeSlash),
            8 => Some(Action::FreezePublicSwap),
            9 => Some(Action::FreezePrivateSwap),
            10 => Some(Action::InitializeLiquidityPool),
            11 => Some(Action::AddAsset),
            12 => Some(Action::UpdateDepositCap),
            13 => Some(Action::DepositRewards),
            14 => Some(Action::Management),
            15 => Some(Action::SuspendDeposits),
            16 => Some(Action::UpdateRole),
            17 => Some(Action::UpdateAction),
            _ => None,
        }
    }

    /** Checks if the action is recurrant and can be frozen. */
    pub fn is_core(&self) -> bool {
        match self {
            Action::Restake | Action::Withdraw | Action::PublicSwap | Action::PrivateSwap | Action::Slash => true,
            _ => false,
        }
    }

    /** Converts a freeze action to its corresponding regular action. */
    pub fn to_action(&self) -> Result<Self> {
        match self {
            Action::FreezeRestake => Ok(Action::Restake),
            Action::FreezeWithdraw => Ok(Action::Withdraw),
            Action::FreezeSlash => Ok(Action::Slash),
            Action::FreezePublicSwap => Ok(Action::PublicSwap),
            Action::FreezePrivateSwap => Ok(Action::PrivateSwap),
            _ => Err(InsuranceFundError::ActionNotFound.into()),
        }
    }
}

