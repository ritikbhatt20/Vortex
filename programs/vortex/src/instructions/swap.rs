use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

use crate::state::Pool;
use crate::constants::*;
use crate::errors::AmmError;
use crate::events::SwapExecuted;
use crate::math::{calculate_swap_output, verify_invariant};

#[derive(Accounts)]
pub struct Swap<'info> {
    /// User performing swap
    #[account(mut)]
    pub user: Signer<'info>,

    /// Pool state
    #[account(
        mut,
        seeds = [POOL_SEED, pool.token_a_mint.as_ref(), pool.token_b_mint.as_ref()],
        bump = pool.bump,
        constraint = !pool.paused @ AmmError::PoolPaused
    )]
    pub pool: Account<'info, Pool>,

    /// Token A vault
    #[account(
        mut,
        seeds = [VAULT_A_SEED, pool.key().as_ref()],
        bump,
        constraint = token_a_vault.key() == pool.token_a_vault @ AmmError::InvalidVault
    )]
    pub token_a_vault: Account<'info, TokenAccount>,

    /// Token B vault
    #[account(
        mut,
        seeds = [VAULT_B_SEED, pool.key().as_ref()],
        bump,
        constraint = token_b_vault.key() == pool.token_b_vault @ AmmError::InvalidVault
    )]
    pub token_b_vault: Account<'info, TokenAccount>,

    /// User's token A account
    #[account(
        mut,
        constraint = user_token_a.mint == pool.token_a_mint @ AmmError::InvalidTokenMint
    )]
    pub user_token_a: Account<'info, TokenAccount>,

    /// User's token B account
    #[account(
        mut,
        constraint = user_token_b.mint == pool.token_b_mint @ AmmError::InvalidTokenMint
    )]
    pub user_token_b: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<Swap>,
    amount_in: u64,
    min_amount_out: u64,
    a_to_b: bool,
) -> Result<()> {
    require!(amount_in >= MIN_SWAP_AMOUNT, AmmError::AmountTooSmall);

    let pool = &ctx.accounts.pool;
    require!(pool.is_initialized(), AmmError::PoolNotInitialized);

    // Determine reserves based on direction
    let (reserve_in, reserve_out) = if a_to_b {
        (pool.reserve_a, pool.reserve_b)
    } else {
        (pool.reserve_b, pool.reserve_a)
    };

    // Calculate output amount
    let (amount_out, fee_amount) = calculate_swap_output(
        amount_in,
        reserve_in,
        reserve_out,
        pool.fee_numerator,
        pool.fee_denominator,
    )?;

    // Slippage check
    require!(amount_out >= min_amount_out, AmmError::SlippageExceeded);

    // Determine accounts based on direction
    let (user_in, user_out, vault_in, vault_out) = if a_to_b {
        (
            ctx.accounts.user_token_a.to_account_info(),
            ctx.accounts.user_token_b.to_account_info(),
            ctx.accounts.token_a_vault.to_account_info(),
            ctx.accounts.token_b_vault.to_account_info(),
        )
    } else {
        (
            ctx.accounts.user_token_b.to_account_info(),
            ctx.accounts.user_token_a.to_account_info(),
            ctx.accounts.token_b_vault.to_account_info(),
            ctx.accounts.token_a_vault.to_account_info(),
        )
    };

    // Transfer input tokens from user to vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: user_in,
                to: vault_in,
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount_in,
    )?;

    // Transfer output tokens from vault to user
    let token_a_mint = ctx.accounts.pool.token_a_mint;
    let token_b_mint = ctx.accounts.pool.token_b_mint;
    let bump = ctx.accounts.pool.bump;

    let seeds = &[
        POOL_SEED,
        token_a_mint.as_ref(),
        token_b_mint.as_ref(),
        &[bump],
    ];
    let signer_seeds = &[&seeds[..]];

    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: vault_out,
                to: user_out,
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer_seeds,
        ),
        amount_out,
    )?;

    // Calculate new reserves
    let (new_reserve_a, new_reserve_b) = if a_to_b {
        (
            pool.reserve_a.checked_add(amount_in).ok_or(AmmError::MathOverflow)?,
            pool.reserve_b.checked_sub(amount_out).ok_or(AmmError::MathOverflow)?,
        )
    } else {
        (
            pool.reserve_a.checked_sub(amount_out).ok_or(AmmError::MathOverflow)?,
            pool.reserve_b.checked_add(amount_in).ok_or(AmmError::MathOverflow)?,
        )
    };

    // Verify invariant k did not decrease
    verify_invariant(pool.reserve_a, pool.reserve_b, new_reserve_a, new_reserve_b)?;

    // Update pool state
    let clock = Clock::get()?;
    let pool = &mut ctx.accounts.pool;

    pool.update_reserves(new_reserve_a, new_reserve_b);

    // Record stats
    let (volume_a, volume_b, fee_a, fee_b) = if a_to_b {
        (amount_in, amount_out, fee_amount, 0u64)
    } else {
        (amount_out, amount_in, 0u64, fee_amount)
    };

    pool.record_swap(volume_a, volume_b, fee_a, fee_b, clock.unix_timestamp, clock.slot);

    let (token_in, token_out) = if a_to_b {
        (pool.token_a_mint, pool.token_b_mint)
    } else {
        (pool.token_b_mint, pool.token_a_mint)
    };

    emit!(SwapExecuted {
        pool: pool.key(),
        user: ctx.accounts.user.key(),
        token_in,
        token_out,
        amount_in,
        amount_out,
        fee_amount,
        reserve_a: pool.reserve_a,
        reserve_b: pool.reserve_b,
        timestamp: clock.unix_timestamp,
    });

    msg!("Swapped {} for {}, fee: {}", amount_in, amount_out, fee_amount);

    Ok(())
}
