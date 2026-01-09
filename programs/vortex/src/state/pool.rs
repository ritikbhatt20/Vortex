use anchor_lang::prelude::*;
use crate::constants::*;

/// Liquidity pool state
/// PDA Seeds: ["pool", token_a_mint, token_b_mint]
#[account]
#[derive(InitSpace)]
pub struct Pool {
    /// Version for future upgrades
    pub version: u8,

    /// Bump seed for PDA
    pub bump: u8,

    /// LP mint authority bump
    pub lp_mint_authority_bump: u8,

    /// Token A mint
    pub token_a_mint: Pubkey,

    /// Token B mint
    pub token_b_mint: Pubkey,

    /// Token A vault (PDA-owned)
    pub token_a_vault: Pubkey,

    /// Token B vault (PDA-owned)
    pub token_b_vault: Pubkey,

    /// LP token mint (PDA)
    pub lp_mint: Pubkey,

    /// Reserve of token A
    pub reserve_a: u64,

    /// Reserve of token B
    pub reserve_b: u64,

    /// Fee numerator (e.g., 3 for 0.3%)
    pub fee_numerator: u64,

    /// Fee denominator (e.g., 1000 for 0.3%)
    pub fee_denominator: u64,

    /// Pool authority
    pub authority: Pubkey,

    /// Emergency pause flag
    pub paused: bool,

    /// Total number of swaps
    pub total_swaps: u64,

    /// Cumulative volume in token A
    pub cumulative_volume_a: u64,

    /// Cumulative volume in token B
    pub cumulative_volume_b: u64,

    /// Cumulative fees in token A
    pub cumulative_fees_a: u64,

    /// Cumulative fees in token B
    pub cumulative_fees_b: u64,

    /// Pool creation timestamp
    pub created_at: i64,

    /// Last swap timestamp
    pub last_swap_timestamp: i64,

    /// Last update slot
    pub last_update_slot: u64,

    /// Reserved for future upgrades (128 bytes)
    pub _reserved: [u8; 128],
}

impl Pool {
    pub const SEED_PREFIX: &'static [u8] = POOL_SEED;

    /// Check if pool is initialized
    pub fn is_initialized(&self) -> bool {
        self.reserve_a > 0 && self.reserve_b > 0
    }

    /// Get current price of token B per token A (Q64 format)
    pub fn price_a(&self) -> u128 {
        if self.reserve_a == 0 {
            return 0;
        }
        (self.reserve_b as u128)
            .saturating_mul(Q64)
            .saturating_div(self.reserve_a as u128)
    }

    /// Get current price of token A per token B (Q64 format)
    pub fn price_b(&self) -> u128 {
        if self.reserve_b == 0 {
            return 0;
        }
        (self.reserve_a as u128)
            .saturating_mul(Q64)
            .saturating_div(self.reserve_b as u128)
    }

    /// Calculate invariant k = reserve_a * reserve_b
    pub fn k(&self) -> u128 {
        (self.reserve_a as u128).saturating_mul(self.reserve_b as u128)
    }

    /// Get fee in basis points
    pub fn fee_bps(&self) -> u64 {
        if self.fee_denominator == 0 {
            return 0;
        }
        (self.fee_numerator * BPS_DENOMINATOR) / self.fee_denominator
    }

    /// Validate reserves match vault balances
    pub fn validate_reserves(&self, vault_a_balance: u64, vault_b_balance: u64) -> bool {
        self.reserve_a == vault_a_balance && self.reserve_b == vault_b_balance
    }

    /// Update reserves
    pub fn update_reserves(&mut self, new_reserve_a: u64, new_reserve_b: u64) {
        self.reserve_a = new_reserve_a;
        self.reserve_b = new_reserve_b;
    }

    /// Record swap statistics
    pub fn record_swap(
        &mut self,
        volume_a: u64,
        volume_b: u64,
        fee_a: u64,
        fee_b: u64,
        timestamp: i64,
        slot: u64,
    ) {
        self.total_swaps = self.total_swaps.saturating_add(1);
        self.cumulative_volume_a = self.cumulative_volume_a.saturating_add(volume_a);
        self.cumulative_volume_b = self.cumulative_volume_b.saturating_add(volume_b);
        self.cumulative_fees_a = self.cumulative_fees_a.saturating_add(fee_a);
        self.cumulative_fees_b = self.cumulative_fees_b.saturating_add(fee_b);
        self.last_swap_timestamp = timestamp;
        self.last_update_slot = slot;
    }
}
