use anchor_lang::prelude::*;

pub mod states;
pub mod constants;
pub mod errors;
pub mod instructions;
use crate::instructions::*;

declare_id!("CPW6gyeGhh7Kt3LYwjF7yXTYgbcNfT7dYBSRDz7TH5YB");

#[program]
pub mod insurance_fund {
    use super::*;

    pub fn initialize_insurance_fund(
        ctx: Context<InitializeInsuranceFund>, 
    ) -> Result<()> {
        instructions::initialize_insurance_fund(ctx)
    }

    pub fn initialize_lockup(
        ctx: Context<InitializeLockup>,
        args: InitializeLockupArgs
    ) -> Result<()> {
        instructions::initialize_lockup(ctx, args)
    }
}