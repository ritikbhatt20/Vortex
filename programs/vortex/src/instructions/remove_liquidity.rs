use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer, Burn};

use crate::state::Pool;
use crate::constants::*;
use crate::errors::AmmError;
use crate::events::LiquidityRemoved;
use crate::math::calculate_amounts_for_liquidity;

#[derive(Accounts)]
pub struct RemoveLiquidity<'info> {
    /// Liquidity provider
    #[account(mut)]
    pub user: Signer<'info>,

    /// Pool state
    #[account(
        mut,
        seeds = [POOL_SEED, pool.token_a_mint.as_ref(), pool.token_b_mint.as_ref()],
        bump = pool.bump
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

    /// LP mint
    #[account(
        mut,
        seeds = [LP_MINT_SEED, pool.key().as_ref()],
        bump,
        constraint = lp_mint.key() == pool.lp_mint @ AmmError::InvalidVault
    )]
    pub lp_mint: Account<'info, Mint>,

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

    /// User's LP token account
    #[account(
        mut,
        constraint = user_lp_token.mint == pool.lp_mint @ AmmError::InvalidTokenMint
    )]
    pub user_lp_token: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn handler(
    ctx: Context<RemoveLiquidity>,
    liquidity_amount: u64,
    min_amount_a: u64,
    min_amount_b: u64,
) -> Result<()> {
    require!(liquidity_amount > 0, AmmError::AmountTooSmall);

    let pool = &ctx.accounts.pool;
    let total_supply = ctx.accounts.lp_mint.supply;

    require!(pool.is_initialized(), AmmError::PoolNotInitialized);

    // Calculate amounts to return
    let (amount_a, amount_b) = calculate_amounts_for_liquidity(
        liquidity_amount,
        pool.reserve_a,
        pool.reserve_b,
        total_supply,
    )?;

    // Slippage check
    require!(amount_a >= min_amount_a, AmmError::SlippageExceeded);
    require!(amount_b >= min_amount_b, AmmError::SlippageExceeded);

    // Burn LP tokens from user
    token::burn(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Burn {
                mint: ctx.accounts.lp_mint.to_account_info(),
                from: ctx.accounts.user_lp_token.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        liquidity_amount,
    )?;

    // Transfer token A from vault to user
    let pool_key = ctx.accounts.pool.key();
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
                from: ctx.accounts.token_a_vault.to_account_info(),
                to: ctx.accounts.user_token_a.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer_seeds,
        ),
        amount_a,
    )?;

    // Transfer token B from vault to user
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.token_b_vault.to_account_info(),
                to: ctx.accounts.user_token_b.to_account_info(),
                authority: ctx.accounts.pool.to_account_info(),
            },
            signer_seeds,
        ),
        amount_b,
    )?;

    // Update pool reserves
    let pool = &mut ctx.accounts.pool;
    pool.reserve_a = pool.reserve_a.checked_sub(amount_a).ok_or(AmmError::MathOverflow)?;
    pool.reserve_b = pool.reserve_b.checked_sub(amount_b).ok_or(AmmError::MathOverflow)?;
    pool.last_update_slot = Clock::get()?.slot;

    emit!(LiquidityRemoved {
        pool: pool.key(),
        user: ctx.accounts.user.key(),
        liquidity_burned: liquidity_amount,
        amount_a,
        amount_b,
        reserve_a: pool.reserve_a,
        reserve_b: pool.reserve_b,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Removed liquidity: burned {} LP, got {} A, {} B", liquidity_amount, amount_a, amount_b);

    Ok(())
}
