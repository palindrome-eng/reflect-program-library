use crate::constants::*;
use crate::states::*;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::token::{mint_to, transfer, Mint, MintTo, Token, TokenAccount, Transfer};
use spl_math::precise_number::PreciseNumber;

#[derive(InitSpace)]
#[account]
pub struct LiquidityPool {
    pub bump: u8,
    pub index: u8,
    pub lp_token: Pubkey,
    pub cooldowns: u64,
    pub cooldown_duration: u64,
    pub deposit_cap: Option<u64>,
}

impl LiquidityPool {
    pub fn deposit<'info>(
        &self,
        signer: &Signer<'info>,
        asset_amount: u64,
        asset_user_account: &Account<'info, TokenAccount>,
        asset_pool: &Account<'info, TokenAccount>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        if asset_amount > 0 {
            transfer(
                CpiContext::new(
                    token_program.to_account_info(),
                    Transfer {
                        from: asset_user_account.to_account_info(),
                        to: asset_pool.to_account_info(),
                        authority: signer.to_account_info(),
                    },
                ),
                asset_amount,
            )?
        }

        Ok(())
    }

    pub fn calculate_total_pool_value(
        &self,
        remaining_accounts: &[AccountInfo],
        liquidity_pool: &Account<LiquidityPool>,
        settings: &Account<Settings>,
        clock: &Clock,
    ) -> Result<PreciseNumber> {
        let expected_len = settings.assets as usize * 4;
        let mut total_pool_value =
            PreciseNumber::new(0).ok_or(crate::errors::RlpError::MathOverflow)?;

        require!(
            remaining_accounts.len() == expected_len,
            crate::errors::RlpError::InvalidInput
        );

        let mut visited_mints: Vec<Pubkey> = Vec::with_capacity(settings.assets as usize);

        let mut i = 0;
        while i < remaining_accounts.len() {
            let token_account_info = &remaining_accounts[i];
            let asset_info = &remaining_accounts[i + 1];
            let oracle_info = &remaining_accounts[i + 2];
            let mint_info = &remaining_accounts[i + 3];

            // Validate token account
            require!(
                token_account_info.owner == &anchor_spl::token::ID,
                crate::errors::RlpError::InvalidInput
            );

            let token_account = TokenAccount::try_deserialize(
                &mut token_account_info.try_borrow_mut_data()?.as_ref(),
            )
            .map_err(|_| crate::errors::RlpError::InvalidInput)?;

            require!(
                token_account.owner == liquidity_pool.key(),
                crate::errors::RlpError::InvalidInput
            );

            // let asset_info = &remaining_accounts[i + 1];

            // Validate asset info
            require!(
                asset_info.owner == &crate::ID,
                crate::errors::RlpError::InvalidInput
            );

            let asset = Asset::try_deserialize(&mut asset_info.try_borrow_mut_data()?.as_ref())
                .map_err(|_| crate::errors::RlpError::InvalidInput)?;

            let asset_mint = asset.mint;

            // Validate asset mint has not already been visited
            require!(
                !visited_mints.contains(&asset_mint),
                crate::errors::RlpError::InvalidInput
            );

            visited_mints.push(asset_mint);

            require!(
                asset.mint == token_account.mint,
                crate::errors::RlpError::InvalidInput
            );

            let (expected_asset_pda, _) = Pubkey::find_program_address(
                &[
                    crate::constants::ASSET_SEED.as_bytes(),
                    &asset.mint.to_bytes(),
                ],
                &crate::ID,
            );

            // Verify this is the correct PDA for asset
            require!(
                asset_info.key() == expected_asset_pda,
                crate::errors::RlpError::InvalidInput
            );

            // Verify this is the correct associated token account for the liquidity pool and asset
            let expected_pool_token_account =
                get_associated_token_address(&liquidity_pool.key(), &asset.mint);

            require!(
                token_account_info.key() == expected_pool_token_account,
                crate::errors::RlpError::InvalidInput
            );

            // Verify oracle key matches assets'
            require!(
                oracle_info.key() == *asset.oracle.key(),
                crate::errors::RlpError::InvalidInput
            );

            let asset_price = asset
                .get_price(oracle_info, clock)
                .map_err(|_| crate::errors::RlpError::InvalidInput)?;

            // Verify token mint owner
            require!(
                mint_info.owner == &anchor_spl::token::ID,
                crate::errors::RlpError::InvalidInput
            );

            // Verify token mint address matches assets'
            require!(
                mint_info.key() == asset.mint,
                crate::errors::RlpError::InvalidInput
            );

            let mint_data = &mut mint_info.try_borrow_mut_data()?;
            let mint_account = Mint::try_deserialize(&mut mint_data.as_ref())
                .map_err(|_| crate::errors::RlpError::InvalidInput)?;

            let token_balance = token_account.amount;
            let token_decimals = mint_account.decimals;
            if token_balance > 0 {
                let token_value_precise =
                    PreciseNumber::new(asset_price.mul(token_balance, token_decimals)?)
                        .ok_or(crate::errors::RlpError::MathOverflow)?;
                total_pool_value = total_pool_value
                    .checked_add(&token_value_precise)
                    .ok_or(crate::errors::RlpError::MathOverflow)?;
            }

            i += 4;
        }

        Ok(total_pool_value)
    }

    pub fn calculate_lp_tokens_on_deposit(
        &self,
        lp_token: &Account<Mint>,
        total_pool_value: PreciseNumber,
        deposit_value: PreciseNumber,
    ) -> Result<u64> {
        let pool_value_is_zero = total_pool_value
            .to_imprecise()
            .map_or(true, |v| v == 0);

        let lp_tokens_to_mint = if lp_token.supply == 0 || pool_value_is_zero {
            // Use initial deposit formula when pool is empty or has only dead shares
            let lp_decimals = lp_token.decimals as u32;
            let scale_down_precise = PreciseNumber::new(10u128.pow(PRECISION - lp_decimals))
                .ok_or(crate::errors::RlpError::MathOverflow)?;

            deposit_value
                .checked_div(&scale_down_precise)
                .ok_or(crate::errors::RlpError::MathOverflow)?
                .to_imprecise()
                .ok_or(crate::errors::RlpError::MathOverflow)?
                .try_into()
                .map_err(|_| crate::errors::RlpError::MathOverflow)?
        } else {
            let lp_supply_precise = PreciseNumber::new(lp_token.supply as u128)
                .ok_or(crate::errors::RlpError::MathOverflow)?;

            let deposit_ratio = deposit_value
                .checked_mul(&lp_supply_precise)
                .ok_or(crate::errors::RlpError::MathOverflow)?
                .checked_div(&total_pool_value)
                .ok_or(crate::errors::RlpError::MathOverflow)?;

            deposit_ratio
                .to_imprecise()
                .ok_or(crate::errors::RlpError::MathOverflow)?
                .try_into()
                .map_err(|_| crate::errors::RlpError::MathOverflow)?
        };

        Ok(lp_tokens_to_mint)
    }

    pub fn mint_lp_token<'info>(
        &self,
        amount: u64,
        liquidity_pool: &Account<'info, LiquidityPool>,
        lp_token: &Account<'info, Mint>,
        lockup_lp_token_vault: &Account<'info, TokenAccount>,
        token_program: &Program<'info, Token>,
    ) -> Result<()> {
        let signer_seeds = &[
            LIQUIDITY_POOL_SEED.as_bytes(),
            &self.index.to_le_bytes(),
            &[self.bump],
        ];

        mint_to(
            CpiContext::new_with_signer(
                token_program.to_account_info(),
                MintTo {
                    mint: lp_token.to_account_info(),
                    to: lockup_lp_token_vault.to_account_info(),
                    authority: liquidity_pool.to_account_info(),
                },
                &[signer_seeds],
            ),
            amount,
        )?;

        Ok(())
    }
}
