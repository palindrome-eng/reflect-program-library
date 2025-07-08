use anchor_lang::prelude::*;
use crate::errors::InsuranceFundError;

#[account]
#[derive(InitSpace)]
pub struct RewardBoost {
    pub index: u64,
    // Minimum USD value of the deposit to be included in the tier.
    pub min_usd_value: u64,
    // % of the $R rewards in basepoints
    pub boost_bps: u64,
    // Which lockup does this boost apply to.
    pub lockup: u64,
}

impl RewardBoost {
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