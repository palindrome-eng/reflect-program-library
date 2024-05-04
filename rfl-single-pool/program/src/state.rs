//! State transition types

use {
    crate::{error::ReflectPoolError, find_pool_address},
    borsh::{BorshDeserialize, BorshSchema, BorshSerialize},
    solana_program::{
        account_info::AccountInfo, borsh1::try_from_slice_unchecked, program_error::ProgramError,
        pubkey::Pubkey,
    },
};

/// Single-Validator Stake Pool account type
#[derive(Clone, Debug, Default, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub enum ReflectPoolAccountType {
    /// Uninitialized account
    #[default]
    Uninitialized,
    /// Main pool account
    Pool,
}

/// Single-Validator Stake Pool account, used to derive all PDAs
#[derive(Clone, Debug, Default, PartialEq, BorshDeserialize, BorshSerialize, BorshSchema)]
pub struct ReflectPool {
    /// Pool account type, reserved for future compat
    pub account_type: ReflectPoolAccountType,
    /// The vote account this pool is mapped to
    pub vote_account_address: Pubkey,
}
impl ReflectPool {
    /// Create a ReflectPool struct from its account info
    pub fn from_account_info(
        account_info: &AccountInfo,
        program_id: &Pubkey,
    ) -> Result<Self, ProgramError> {
        // pool is allocated and owned by this program
        if account_info.data_len() == 0 || account_info.owner != program_id {
            return Err(ReflectPoolError::InvalidPoolAccount.into());
        }

        let pool = try_from_slice_unchecked::<ReflectPool>(&account_info.data.borrow())?;

        // pool is well-typed
        if pool.account_type != ReflectPoolAccountType::Pool {
            return Err(ReflectPoolError::InvalidPoolAccount.into());
        }

        // pool vote account address is properly configured. in practice this is
        // irrefutable because the pool is initialized from the address that
        // derives it, and never modified
        if *account_info.key != find_pool_address(program_id, &pool.vote_account_address) {
            return Err(ReflectPoolError::InvalidPoolAccount.into());
        }

        Ok(pool)
    }
}
