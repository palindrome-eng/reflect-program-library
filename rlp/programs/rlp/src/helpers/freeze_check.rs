use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use anchor_spl::token::spl_token::state::AccountState;

use crate::errors::RlpError;

/// Reject if any of the supplied pool reserve token accounts has been frozen
/// by its mint's freeze authority. The freeze blocks SPL transfers, which
/// would otherwise revert the whole withdraw or deposit instruction once it
/// hit the frozen reserve.
///
/// `reserves` is a slice of `(reserve_ata_key, TokenAccount)` tuples; the
/// caller has already loaded these (e.g. via `load_reserves` or
/// `calculate_total_pool_value`).
pub fn assert_no_reserve_frozen(reserves: &[(Pubkey, TokenAccount)]) -> Result<()> {
    for (_, reserve) in reserves.iter() {
        require!(
            reserve.state != AccountState::Frozen,
            RlpError::PoolAssetFrozen
        );
    }
    Ok(())
}
