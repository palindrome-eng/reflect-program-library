use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;
use crate::constants::*;
use crate::states::*;
use crate::errors::InsuranceFundError;
use anchor_spl::token::{
    Transfer,
    transfer,
    Token,
    Mint,
};

// Due to the nature of cold wallet, it might be a case that superadmin
// might not want to actually cosign the smart contract transaction using cold wallet.
// For this reason, this instruction might be just used to update fields (and invoked by
// superadmin hot wallet). 

// For transparency reasons, it is advised to provide signature of 
// the transfer instruction that will be included in logs (if actual transfer happened in diff transaction).

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct SlashColdWalletArgs {
    lockup_id: u64,
    slash_id: u64,
    transfer_funds: bool,
    transfer_sig: Option<String>,
}

pub fn slash_cold_wallet(
    ctx: Context<SlashColdWallet>,
    args: SlashColdWalletArgs
) -> Result<()> {

    let SlashColdWalletArgs {
        transfer_funds,
        lockup_id,
        slash_id,
        transfer_sig
    } = args;

    let settings = &ctx.accounts.settings;
    let cold_wallet = &ctx.accounts.cold_wallet;
    let slash = &mut ctx.accounts.slash;
    let token_program = &ctx.accounts.token_program;

    let SharesConfig {
        cold_wallet_share_bps,
        hot_wallet_share_bps,
    } = settings.shares_config;

    let amount = slash.target_amount * cold_wallet_share_bps / 10_000;

    if (transfer_funds) {
        let source = &ctx.accounts.source;
        let destination = &ctx.accounts.destination;

        require!(
            cold_wallet.is_signer,
            InsuranceFundError::InvalidSigners
        );

        require!(
            source.is_some() && destination.is_some(),
            InsuranceFundError::InvalidInput
        );

        transfer(
            CpiContext::new(
                token_program.to_account_info(), 
                Transfer {
                    authority: cold_wallet.to_account_info(),
                    from: source.as_ref().unwrap().to_account_info(),
                    to: destination.as_ref().unwrap().to_account_info()
                }
            ),
            amount
        )?;
    } else {
        require!(
            transfer_sig.is_some(),
            InsuranceFundError::TransferSignatureRequired
        );
        
        slash.transfer_sig = transfer_sig;
    }
    
    slash.slashed_cold_wallet = true;

    Ok(())
}

#[derive(Accounts)]
#[instruction(
    args: SlashColdWalletArgs
)]
pub struct SlashColdWallet<'info> {
    #[account(
        mut,
    )]
    pub signer: Signer<'info>,

    #[account(
        mut,
        constraint = admin.address == signer.key() @ InsuranceFundError::InvalidSigner,
        constraint = admin.has_permissions(Permissions::Superadmin) @ InsuranceFundError::PermissionsTooLow,
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

    /// CHECK: Directly checking the address
    #[account(
        mut,
        address = settings.cold_wallet
    )]
    pub cold_wallet: AccountInfo<'info>,

    #[account(
        mut,
        seeds = [
            LOCKUP_SEED.as_bytes(),
            &args.lockup_id.to_le_bytes()
        ],
        bump,
        // Make sure that previous steps are done
        constraint = lockup.locked
    )]
    pub lockup: Account<'info, Lockup>,

    #[account(
        mut,
        seeds = [
            SLASH_SEED.as_bytes(),
            lockup.key().as_ref(),
            &args.slash_id.to_le_bytes()
        ],
        bump,
        // All deposits have to be slashed before slashing the cold wallet.
        constraint = slash.target_accounts == slash.slashed_accounts @ InsuranceFundError::DepositsNotSlashed,
        constraint = slash.index == lockup.slash_state.index @ InsuranceFundError::InvalidInput
    )]
    pub slash: Account<'info, Slash>,

    #[account(
        mut,
        address = lockup.asset
    )]
    pub asset_mint: Option<Account<'info, Mint>>,

    #[account(
        mut,
        token::mint = asset_mint,
        token::authority = cold_wallet 
    )]
    pub source: Option<Account<'info, TokenAccount>>,

    #[account(
        mut,
        token::mint = asset_mint,
    )]
    pub destination: Option<Account<'info, TokenAccount>>,

    #[account()]
    pub token_program: Program<'info, Token>,
}