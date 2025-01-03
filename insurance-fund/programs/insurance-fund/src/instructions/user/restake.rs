use anchor_lang::prelude::*;
use crate::errors::InsuranceFundError;
use crate::events::RestakeEvent;
use crate::helpers::calculate_receipts_on_mint;
use crate::helpers::calculate_total_deposits;
use crate::states::*;
use crate::constants::*;
use anchor_lang::system_program::{
    create_account,
    CreateAccount
};
use anchor_spl::token::{
    Mint,
    Token,
    TokenAccount,
    transfer,
    Transfer,
    mint_to,
    MintTo,
    initialize_account3,
    InitializeAccount3
};

#[derive(AnchorDeserialize, AnchorSerialize)]
pub struct RestakeArgs {
    pub lockup_id: u64,
    pub amount: u64,
}

#[inline(never)]
pub fn restake(
    ctx: Context<Restake>,
    args: RestakeArgs
) -> anchor_lang::prelude::Result<()> {

    let RestakeArgs {
        amount,
        lockup_id: _
    } = args;

    let token_program = &ctx.accounts.token_program;
    let user = &ctx.accounts.user;
    let asset = &mut ctx.accounts.asset;
    let asset_mint = &ctx.accounts.asset_mint;
    let oracle = &ctx.accounts.oracle;
    let lockup = &mut ctx.accounts.lockup;
    let deposit = &mut ctx.accounts.deposit;
    let lockup_cold_vault = &ctx.accounts.lockup_cold_vault;
    let lockup_hot_vault = &ctx.accounts.lockup_hot_vault;
    let receipt_token_mint = &ctx.accounts.receipt_token_mint;
    let user_asset_ata = &ctx.accounts.user_asset_ata;
    let settings = &ctx.accounts.settings;
    let system_program = &ctx.accounts.system_program;

    let clock = Clock::get()?;
    let unix_ts = clock.unix_timestamp as u64;

    let total_deposits = calculate_total_deposits(
        &lockup_cold_vault,
        &lockup_hot_vault
    )?;

    let deposit_receipt_token_account = &ctx.accounts.deposit_receipt_token_account;
    
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(165);

    create_account(
        CpiContext::new_with_signer(
            system_program.to_account_info(), 
            CreateAccount {
                from: user.to_account_info(),
                to: deposit_receipt_token_account.to_account_info()
            },
            &[
                &[
                    DEPOSIT_RECEIPT_VAULT_SEED.as_bytes(),
                    deposit.key().as_ref(),
                    receipt_token_mint.key().as_ref(),
                    &[ctx.bumps.deposit_receipt_token_account]
                ]
            ]
        ),
        lamports,
        165,
        &token_program.key()
    )?;

    initialize_account3(
        CpiContext::new_with_signer(
            token_program.to_account_info(), 
            InitializeAccount3 {
                account: deposit_receipt_token_account.to_account_info(),
                mint: receipt_token_mint.to_account_info(),
                authority: deposit.to_account_info()
            },
            &[
                &[
                    DEPOSIT_RECEIPT_VAULT_SEED.as_bytes(),
                    deposit.key().as_ref(),
                    receipt_token_mint.key().as_ref(),
                    &[ctx.bumps.deposit_receipt_token_account]
                ]
            ]
        )
    )?;

    match lockup.deposit_cap {
        Some(cap) => {
            require!(
                total_deposits + amount <= cap,
                InsuranceFundError::DepositCapOverflow
            );
        },
        None => {}
    }

    let total_receipts_to_mint: u64 = calculate_receipts_on_mint(
        &receipt_token_mint,
        &amount, 
        &total_deposits
    )?;

    msg!("calculated total receipts to mint");

    lockup.mint_receipts(
        &total_receipts_to_mint, 
        &lockup, 
        &receipt_token_mint, 
        &deposit_receipt_token_account, 
        &token_program
    )?;

    let deserialized = TokenAccount::try_deserialize(
        &mut deposit_receipt_token_account.try_borrow_mut_data()?.as_ref()
    )?;

    msg!("minted receipts");
    msg!("receipt_balance: {:?}", deserialized.amount);

    deposit.lockup = lockup.key();
    deposit.user = user.key();
    deposit.unlock_ts = unix_ts + lockup.duration;
    deposit.last_slashed = None;
    deposit.amount_slashed = 0;

    let receipt_exchange_rate_bps = lockup.receipt_to_reward_exchange_rate_bps_accumulator;
    deposit.initial_receipt_exchange_rate_bps = receipt_exchange_rate_bps;
    
    let oracle_price = asset
        .get_price(oracle, &clock)?;

    msg!("calculated price");

    // Multiply oracle price by amount of tokens, 
    // then divide by token decimals to get value of 1 full token instead of `lamports`.
    let usd_value_with_decimals = oracle_price
        .mul(amount)?;

    let usd_value: u64 = usd_value_with_decimals
        .checked_div(
            10_u128
            .checked_pow(asset_mint.decimals as u32)
            .ok_or(InsuranceFundError::MathOverflow)?
        )
        .ok_or(InsuranceFundError::MathOverflow)?
        .try_into()
        .map_err(|_| InsuranceFundError::MathOverflow)?;

    msg!("calculated usd value");

    deposit.initial_usd_value = usd_value;

    lockup.deposit(
        amount, 
        user, 
        user_asset_ata, 
        settings, 
        lockup_hot_vault, 
        lockup_cold_vault, 
        token_program
    )?;

    lockup.deposits += 1;

    asset.increase_tvl(amount)?;
    asset.add_deposit()?;

    emit!(RestakeEvent {
        from: user.key(),
        asset: asset.key(),
        amount,
        lockup_ts: deposit.unlock_ts
    });

    panic!();

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: RestakeArgs
)]
pub struct Restake<'info> {
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
        constraint = !settings.frozen @ InsuranceFundError::Frozen
    )]
    pub settings: Box<Account<'info, Settings>>,

    #[account(
        mut,
        seeds = [
            LOCKUP_SEED.as_bytes(),
            &args.lockup_id.to_le_bytes()
        ],
        bump,
        constraint = lockup.min_deposit <= args.amount @ InsuranceFundError::DepositTooLow,
    )]
    pub lockup: Box<Account<'info, Lockup>>,

    #[account(
        mut,
        address = lockup.receipt_mint
    )]
    pub receipt_token_mint: Account<'info, Mint>,

    #[account(
        init,
        payer = user,
        seeds = [
            DEPOSIT_SEED.as_bytes(),
            lockup.key().as_ref(),
            &lockup.deposits.to_le_bytes()
        ],
        bump,
        space = Deposit::LEN
    )]
    pub deposit: Account<'info, Deposit>,

    /// CHECK: initializing this manually
    #[account(
        mut,
        seeds = [
            DEPOSIT_RECEIPT_VAULT_SEED.as_bytes(),
            deposit.key().as_ref(),
            receipt_token_mint.key().as_ref(),
        ],
        bump
    )]
    pub deposit_receipt_token_account: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [
            ASSET_SEED.as_bytes(),
            &asset_mint.key().to_bytes()
        ],
        bump,
        constraint = asset.mint == asset_mint.key()
    )]
    pub asset: Account<'info, Asset>,

    #[account(
        mut,
        address = lockup.asset_mint,
    )]
    pub asset_mint: Account<'info, Mint>,

    #[account(
        mut,
        seeds = [
            HOT_VAULT_SEED.as_bytes(),
            lockup.key().as_ref(),
            asset_mint.key().as_ref(),
        ],
        bump,
    )]
    pub lockup_hot_vault: Account<'info, anchor_spl::token::TokenAccount>,

    /// CHECK: DEBUG
    #[account(
        mut,
        seeds = [
            COLD_VAULT_SEED.as_bytes(),
            lockup.key().as_ref(),
            asset_mint.key().as_ref(),
        ],
        bump,
    )]
    pub lockup_cold_vault: Account<'info, anchor_spl::token::TokenAccount>,

    #[account(
        mut,
        associated_token::authority = user,
        associated_token::mint = asset_mint,
        constraint = user_asset_ata.amount >= args.amount @ InsuranceFundError::NotEnoughFunds,
    )]
    pub user_asset_ata: Account<'info, TokenAccount>,

    /// CHECK: Directly checking the address + checking type later.
    #[account(
        mut,
        constraint = asset.oracle.key().eq(&oracle.key())
    )]
    pub oracle: AccountInfo<'info>,

    #[account()]
    pub token_program: Program<'info, Token>,

    #[account()]
    pub system_program: Program<'info, System>,
}