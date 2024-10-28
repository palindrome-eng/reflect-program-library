use anchor_lang::prelude::*;
use anchor_spl::token::{
    TokenAccount,
    Mint,
    Token,
    transfer,
    Transfer
};
use crate::states::*;
use crate::constants::*;
use crate::errors::InsuranceFundError;

// TODO: Intent should be created no matter how big the deposit size is, but how big the requested withdrawal is.
// If the deposit is larger than entire hot wallet share of the insurance fund, but single withdrawal is smaller,
// the withdrawal should be processed as normal.

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct CreateIntentArgs {
    pub amount: u64,
    pub lockup_id: u64,
    pub deposit_id: u64
}

pub fn create_intent(
    ctx: Context<CreateIntent>,
    args: CreateIntentArgs
) -> Result<()> {
    let CreateIntentArgs {
        amount,
        deposit_id: _,
        lockup_id
    } = args;

    // TODO: Cover case when single withdrawal is >hot wallet && >cold wallet, i.e 70% of the insurance fund
    // Transfer entire hot wallet & create intent for the difference.

    let user = &ctx.accounts.user;
    let deposit = &mut ctx.accounts.deposit;
    let lockup = &ctx.accounts.lockup;
    let lockup_asset_vault = &ctx.accounts.lockup_asset_vault;
    let intent = &mut ctx.accounts.intent;
    let settings = &ctx.accounts.settings;
    let user_asset_ata = &ctx.accounts.user_asset_ata;
    let token_program = &ctx.accounts.token_program;

    let SharesConfig { 
        hot_wallet_share_bps, 
        cold_wallet_share_bps 
    } = settings.shares_config;

    let total_estimated_lockup = lockup_asset_vault.amount as f64 / (hot_wallet_share_bps as f64 / 10_000_f64);
    let total_cold_wallet_lockup = (total_estimated_lockup as f64 * cold_wallet_share_bps as f64 / 10_000_f64) as u64;
    let total_hot_wallet_lockup = lockup_asset_vault.amount;

    // TODO: Update `asset` TVL stat.

    if (amount > total_cold_wallet_lockup) {
        // Difference between hot wallet deposit and user's deposit
        let difference = amount - total_hot_wallet_lockup;

        let seeds = &[
            LOCKUP_SEED.as_bytes(),
            &lockup_id.to_le_bytes(),
            &[ctx.bumps.lockup]
        ];

        msg!("total_hot_wallet_lockup: {:?}", total_hot_wallet_lockup);
        msg!("lockup_asset_vault: {:?}", lockup_asset_vault.amount);

        // Transfer ENTIRE hot wallet share of the insurance fund.
        transfer(
            CpiContext::new_with_signer(
                token_program.to_account_info(), 
                Transfer {
                    from: lockup_asset_vault.to_account_info(),
                    to: user_asset_ata.to_account_info(),
                    authority: lockup.to_account_info()
                }, 
                &[seeds]
            ), 
            total_hot_wallet_lockup
        )?;

        deposit.amount = deposit
            .amount
            .checked_sub(total_hot_wallet_lockup)
            .ok_or(InsuranceFundError::MathOverflow)?;

        // Create intent to transfer the difference via company multisig
        intent.amount = amount;
        intent.total_deposit = deposit.amount;
        intent.lockup = lockup.key();
    }

    intent.amount = amount;
    intent.total_deposit = deposit.amount;
    intent.lockup = lockup.key();

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: CreateIntentArgs
)]
pub struct CreateIntent<'info> {
    #[account(
        mut
    )]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds = [
            SETTINGS_SEED.as_bytes()
        ],
        bump,
    )]
    pub settings: Account<'info, Settings>,

    #[account(
        mut,
        seeds = [
            LOCKUP_SEED.as_bytes(),
            &args.lockup_id.to_le_bytes()
        ],
        bump,
    )]
    pub lockup: Account<'info, Lockup>,

    #[account(
        mut,
        address = lockup.asset
    )]
    pub asset_mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = asset_mint
    )]
    pub user_asset_ata: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            VAULT_SEED.as_bytes(),
            lockup.key().as_ref(),
            asset_mint.key().as_ref(),
        ],
        bump,
        token::mint = asset_mint,
        token::authority = lockup,
        // Intents are used exclusively for withdrawals bigger than the hot wallet share.
        constraint = lockup_asset_vault.amount < args.amount
    )]
    pub lockup_asset_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [
            DEPOSIT_SEED.as_bytes(),
            lockup.key().as_ref(),
            &args.deposit_id.to_le_bytes()
        ],
        bump,
        // Cannot create withdrawal intent before lockup ends
        constraint = deposit.unlock_ts <= (clock.unix_timestamp as u64) @ InsuranceFundError::LockupInForce,
        // Cannot request withdrawal for more than in deposit
        constraint = deposit.amount >= args.amount @ InsuranceFundError::NotEnoughFunds,
        // Enforce account ownership
        has_one = user
    )]
    pub deposit: Account<'info, Deposit>,

    #[account(
        init,
        payer = user,
        // Only allow one withdrawal intent per deposit to process atomically.
        seeds = [
            INTENT_SEED.as_bytes(),
            &deposit.key().to_bytes(),
        ],
        bump,
        space = Intent::LEN
    )]
    pub intent: Account<'info, Intent>,

    #[account()]
    pub system_program: Program<'info, System>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub clock: Sysvar<'info, Clock>,
}