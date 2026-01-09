use anchor_lang::prelude::*;

/// Emitted when a new pool is created
#[event]
pub struct PoolCreated {
    pub pool: Pubkey,
    pub token_a_mint: Pubkey,
    pub token_b_mint: Pubkey,
    pub fee_numerator: u64,
    pub fee_denominator: u64,
    pub timestamp: i64,
}

/// Emitted when liquidity is added
#[event]
pub struct LiquidityAdded {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub amount_a: u64,
    pub amount_b: u64,
    pub liquidity_minted: u64,
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub timestamp: i64,
}

/// Emitted when liquidity is removed
#[event]
pub struct LiquidityRemoved {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub liquidity_burned: u64,
    pub amount_a: u64,
    pub amount_b: u64,
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub timestamp: i64,
}

/// Emitted when a swap occurs
#[event]
pub struct SwapExecuted {
    pub pool: Pubkey,
    pub user: Pubkey,
    pub token_in: Pubkey,
    pub token_out: Pubkey,
    pub amount_in: u64,
    pub amount_out: u64,
    pub fee_amount: u64,
    pub reserve_a: u64,
    pub reserve_b: u64,
    pub timestamp: i64,
}
