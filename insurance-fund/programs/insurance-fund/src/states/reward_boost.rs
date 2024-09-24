use anchor_lang::prelude::*;
use crate::errors::InsuranceFundError;

#[account]
pub struct RewardBoost {
    // Minimum USD value of the deposit to be included in the tier.
    pub min_usd_value: u64,
    // % of the $R rewards in basepoints
    pub boost_bps: u64,
}

impl RewardBoost {
    pub const SIZE: usize = 2 * 8;

    pub fn validate(
        &self,
        amount: u64
    ) -> Result<()> {
        require!(
            amount >= self.min_usd_value,
            InsuranceFundError::BoostNotApplied
        );

        Ok(())
    }
}