use anchor_lang::prelude::*;

use crate::constants::PROXY_PROGRAM_ID;
use crate::errors::RlpError;

/// On-disk layout of the proxy program's `ProxyState` account (no Anchor
/// discriminator; the proxy program is pinocchio-based and uses a fixed
/// `#[repr(C)]` byte layout).
///
/// Field offsets (matches reflect-proxy-program/program/src/state.rs):
///
/// ```text
///   0..32    branded_mint
///   32..64   stablecoin_mint
///   64..66   fee (u16 LE)
///   66..74   principal (u64 LE, USDC-denominated)
///   74..82   integrators_commission (u64 LE, USDC-denominated)
///   82..114  authority
///   114      bump
///   115      frozen (0 = active, 1 = frozen)
///   116..124 deposit_cap (u64 LE)
/// ```
const PROXY_STATE_LEN: usize = 124;

const OFFSET_BRANDED_MINT: usize = 0;
const OFFSET_STABLECOIN_MINT: usize = 32;
const OFFSET_PRINCIPAL: usize = 66;
const OFFSET_INTEGRATORS_COMMISSION: usize = 74;
const OFFSET_FROZEN: usize = 115;

#[derive(Debug, Clone, Copy)]
pub struct ProxyStateView {
    pub branded_mint: Pubkey,
    pub stablecoin_mint: Pubkey,
    pub principal: u64,
    pub integrators_commission: u64,
    pub frozen: bool,
}

impl ProxyStateView {
    /// Read a `ProxyStateView` from a raw `AccountInfo`. Verifies that the
    /// account is owned by the proxy program and has the expected byte length.
    pub fn from_account_info(account: &AccountInfo) -> Result<Self> {
        require!(
            account.owner == &PROXY_PROGRAM_ID,
            RlpError::InvalidInput
        );

        let data = account.try_borrow_data()?;
        require!(
            data.len() == PROXY_STATE_LEN,
            RlpError::InvalidInput
        );

        let branded_mint = Pubkey::new_from_array(
            data[OFFSET_BRANDED_MINT..OFFSET_BRANDED_MINT + 32]
                .try_into()
                .unwrap(),
        );
        let stablecoin_mint = Pubkey::new_from_array(
            data[OFFSET_STABLECOIN_MINT..OFFSET_STABLECOIN_MINT + 32]
                .try_into()
                .unwrap(),
        );
        let principal = u64::from_le_bytes(
            data[OFFSET_PRINCIPAL..OFFSET_PRINCIPAL + 8]
                .try_into()
                .unwrap(),
        );
        let integrators_commission = u64::from_le_bytes(
            data[OFFSET_INTEGRATORS_COMMISSION..OFFSET_INTEGRATORS_COMMISSION + 8]
                .try_into()
                .unwrap(),
        );
        let frozen = data[OFFSET_FROZEN] == 1;

        Ok(Self {
            branded_mint,
            stablecoin_mint,
            principal,
            integrators_commission,
            frozen,
        })
    }

    /// Booked senior claims = `principal + integrators_commission`.
    pub fn total_booked_claims(&self) -> Result<u64> {
        self.principal
            .checked_add(self.integrators_commission)
            .ok_or_else(|| RlpError::MathOverflow.into())
    }
}
