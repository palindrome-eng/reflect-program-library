use anchor_lang::prelude::*;
use anchor_spl::token::Mint;
use anchor_spl::token::{
    TokenAccount,
    Token,
    transfer,
    Transfer
};
use crate::states::*;
use crate::constants::*;
use crate::errors::InsuranceFundError;

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct ProcessIntentArgs {
    pub deposit_id: u64,
    pub lockup_id: u64
}

pub fn process_intent(
    ctx: Context<ProcessIntent>
) -> Result<()> {
    let token_program = &ctx.accounts.token_program;
    let source = &ctx.accounts.source;
    let source_vault = &ctx.accounts.source_vault;
    let intent = &ctx.accounts.intent;
    let intent_owner_ata = &ctx.accounts.intent_owner_ata;

    transfer(
        CpiContext::new(
            token_program.to_account_info(), 
            Transfer {
                from: source_vault.to_account_info(),
                authority: source.to_account_info(),
                to: intent_owner_ata.to_account_info()
            }
        ), 
        intent.amount
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(args: ProcessIntentArgs)]
pub struct ProcessIntent<'info> {
    #[account(
        mut
    )]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            PERMISSIONS_SEED.as_bytes(),
            signer.key().as_ref()
        ],
        bump,
        constraint = admin.can_perform_protocol_action(Action::Management, &settings.access_control)
    )]
    pub admin: Account<'info, UserPermissions>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
        constraint = !settings.frozen @ InsuranceFundError::Frozen
    )]
    pub settings: Account<'info, Settings>,

    /// CHECK: This can be any wallet that 1) has enough tokens, 2) will sign tx to authorize transfer
    #[account()]
    pub source: AccountInfo<'info>,

    #[account(
        mut,
        token::authority = source,
        token::mint = asset_mint
    )]
    pub source_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            LOCKUP_SEED.as_bytes(),
            &args.lockup_id.to_le_bytes()
        ],
        bump
    )]
    pub lockup: Account<'info, Lockup>,

    #[account(
        mut,
        address = lockup.asset_mint
    )]
    pub asset_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            DEPOSIT_SEED.as_bytes(),
            lockup.key().as_ref(),
            &args.deposit_id.to_le_bytes()
        ],
        bump
    )]
    pub deposit: Account<'info, Deposit>,

    /// CHECK: Directly checking address.
    #[account(
        mut,
        address = deposit.user
    )]
    pub intent_owner: AccountInfo<'info>,

    #[account(
        mut,
        associated_token::authority = intent_owner,
        associated_token::mint = asset_mint
    )]
    pub intent_owner_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            INTENT_SEED.as_bytes(),
            deposit.key().as_ref()
        ],
        bump,
        close = intent_owner,
    )]
    pub intent: Account<'info, Intent>,

    #[account()]
    pub token_program: Program<'info, Token>
}