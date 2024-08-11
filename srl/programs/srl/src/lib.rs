use anchor_lang::prelude::*;

pub mod state;
pub mod errors;

pub mod instructions;
pub use instructions::*;

declare_id!("GWh5R5QiMyTibcsqSKhgyobzJfwS1LVeCW29aq7qFECw");

#[program]
pub mod srl {
    use super::*;

    // Setup
    pub fn create_orderbook(ctx: Context<CreateOrderbook>, args: CreateOrderbookArgs) -> Result<()> {
        instructions::setup::create_orderbook::handler(ctx, args)
    }

    pub fn manage_lock(ctx: Context<ManageLock>,) -> Result<()> {
        instructions::setup::manage_lock::handler(ctx)
    }

    pub fn claim_fees(ctx: Context<ClaimFees>,) -> Result<()> {
        instructions::setup::claim_fees::handler(ctx)
    }

    // Actions
    pub fn place_bid(ctx: Context<PlaceBid>, args: PlaceBidArgs) -> Result<()> {
        instructions::actions::place_bid::handler(ctx, args)
    }

    pub fn retract_bid(ctx: Context<RetractBid>,) -> Result<()> {
        instructions::actions::retract_bid::handler(ctx)
    }

    pub fn place_ask<'info>(ctx: Context<'_, '_, '_, 'info, PlaceAsk<'info>>, args: PlaceAskArgs) -> Result<()> {
        instructions::actions::place_ask::handler(ctx, args)
    }

    pub fn retract_ask(ctx: Context<RetractAsk>,) -> Result<()> {
        instructions::actions::retract_ask::handler(ctx)
    }

    pub fn take<'info>(ctx: Context<'_, '_, '_, 'info, Take<'info>>,) -> Result<()> {
        instructions::actions::take::handler(ctx)
    }

    pub fn repay_loan(ctx: Context<RepayLoan>,) -> Result<()> {
        instructions::actions::repay_loan::handler(ctx)
    }

}
