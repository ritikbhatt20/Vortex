use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod math;
pub mod state;
pub mod instructions;

use instructions::*;

declare_id!("71kECueXZuecQ7ngyxbThU22XyTM1jfk4SpGk7PSVbGY");

#[program]
pub mod vortex {
    use super::*;

    /// Initialize a new liquidity pool
    pub fn initialize_pool(
        ctx: Context<InitializePool>,
        fee_numerator: u64,
        fee_denominator: u64,
    ) -> Result<()> {
        instructions::initialize_pool::handler(ctx, fee_numerator, fee_denominator)
    }

    /// Add liquidity to pool
    pub fn add_liquidity(
        ctx: Context<AddLiquidity>,
        amount_a: u64,
        amount_b: u64,
        min_liquidity: u64,
    ) -> Result<()> {
        instructions::add_liquidity::handler(ctx, amount_a, amount_b, min_liquidity)
    }

    /// Remove liquidity from pool
    pub fn remove_liquidity(
        ctx: Context<RemoveLiquidity>,
        liquidity_amount: u64,
        min_amount_a: u64,
        min_amount_b: u64,
    ) -> Result<()> {
        instructions::remove_liquidity::handler(ctx, liquidity_amount, min_amount_a, min_amount_b)
    }

    /// Swap tokens
    pub fn swap(
        ctx: Context<Swap>,
        amount_in: u64,
        min_amount_out: u64,
        a_to_b: bool,
    ) -> Result<()> {
        instructions::swap::handler(ctx, amount_in, min_amount_out, a_to_b)
    }
}
