use anchor_lang::prelude::*;
use crate::states::*;
use anchor_spl::token::{
    Mint,
    TokenAccount,
    Token
};
use crate::constants::*;
use crate::errors::InsuranceFundError;

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct BorrowArgs {
    pub amount: u64,
    pub from_lockup_id: u64
}

pub fn borrow(
    ctx: Context<Borrow>,
    args: BorrowArgs,
) -> Result<()> {
    let BorrowArgs {
        amount,
        from_lockup_id: _
    } = args;

    let settings = &mut ctx.accounts.settings;
    let debt_record = &mut ctx.accounts.debt_record;
    let from_lockup = &ctx.accounts.from_lockup;
    let from_asset = &ctx.accounts.from_asset;
    let from_hot_vault = &ctx.accounts.from_hot_vault;
    let reflect_from_token_account = &ctx.accounts.reflect_from_token_account;
    let token_program = &ctx.accounts.token_program;

    let clock = &Clock::get()?;

    from_lockup.withdraw_hot_vault(
        amount, 
        from_hot_vault, 
        from_lockup, 
        reflect_from_token_account, 
        token_program
    )?;

    debt_record.set_inner(DebtRecord { 
        amount,
        asset: from_asset.key(), 
        lockup: from_lockup.index, 
        timestamp: clock
            .unix_timestamp
            .try_into()
            .map_err(|_| InsuranceFundError::MathOverflow)?
    });

    settings.debt_records += 1;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: BorrowArgs
)]
pub struct Borrow<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        constraint = admin.address == signer.key(),
        constraint = admin.has_permissions(Permissions::Superadmin),
    )]
    pub admin: Account<'info, Admin>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        constraint = !settings.frozen @ InsuranceFundError::Frozen
    )]
    pub settings: Account<'info, Settings>,

    // For the incoming transfer
    #[account(
        constraint = from_lockup.asset_mint == from_token.key()
    )]
    pub from_token: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            LOCKUP_SEED.as_bytes(),
            &args.from_lockup_id.to_le_bytes()
        ],
        bump
    )]
    pub from_lockup: Account<'info, Lockup>,

    #[account(
        constraint = from_asset.mint == from_token.key()
    )]
    pub from_asset: Account<'info, Asset>,

    /// CHECK: Directly checking the address
    #[account(
        constraint = from_oracle.key().eq(from_asset.oracle.key())
    )]
    pub from_oracle: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [
            HOT_VAULT_SEED.as_bytes(),
            from_lockup.key().as_ref(),
            from_token.key().as_ref(),
        ],
        bump,
    )]
    pub from_hot_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        token::authority = signer,
        token::mint = from_token
    )]
    pub reflect_from_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = signer,
        space = DebtRecord::INIT_SPACE,
        seeds = [
            DEBT_RECORD_SEED.as_bytes(),
            &settings.debt_records.to_le_bytes()
        ],
        bump
    )]
    pub debt_record: Account<'info, DebtRecord>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub system_program: Program<'info, System>,
}