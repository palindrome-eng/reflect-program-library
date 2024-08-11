use anchor_lang::prelude::*;

use crate::state::OrderBook;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct CreateOrderbookArgs {
    pub fee_basis_points: u16,
    pub authority: Pubkey,
}

#[derive(Accounts)]
pub struct CreateOrderbook<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,
    #[account(
        init,
        payer = signer,
        space = OrderBook::INIT_SPACE,
        seeds = [b"order_book", OrderBook::CURRENT_VERSION.to_le_bytes().as_ref()],
        bump
    )]
    pub order_book: Account<'info, OrderBook>,

    pub system_program: Program<'info, System>,
}

impl<'info> CreateOrderbook<'info> {
    pub fn initialize_order_book(&mut self, args: CreateOrderbookArgs, bump: u8) -> Result<()> {
        
        // Add the data to the account
        self.order_book.set_inner(
            OrderBook {
                version: OrderBook::CURRENT_VERSION,
                locked: false,
                tvl: 0,
                fee_basis_points: args.fee_basis_points,
                authority: args.authority,
                bump,
            }
        );

        Ok(())
    }
}

pub fn handler(ctx: Context<CreateOrderbook>, args: CreateOrderbookArgs) -> Result<()> {

    // Get the bumps
    let bump = ctx.bumps.order_book;

    // Initialize the order book
    ctx.accounts.initialize_order_book(args, bump)?;

    Ok(())
}