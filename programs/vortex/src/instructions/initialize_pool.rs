use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};

use crate::state::Pool;
use crate::constants::*;
use crate::errors::AmmError;
use crate::events::PoolCreated;

#[derive(Accounts)]
pub struct InitializePool<'info> {
    /// Pool creator and authority
    #[account(mut)]
    pub authority: Signer<'info>,

    /// Token A mint (must be < token B mint lexicographically)
    pub token_a_mint: Account<'info, Mint>,

    /// Token B mint
    pub token_b_mint: Account<'info, Mint>,

    /// Pool state account
    #[account(
        init,
        payer = authority,
        space = 8 + Pool::INIT_SPACE,
        seeds = [POOL_SEED, token_a_mint.key().as_ref(), token_b_mint.key().as_ref()],
        bump
    )]
    pub pool: Account<'info, Pool>,

    /// Token A vault
    #[account(
        init,
        payer = authority,
        seeds = [VAULT_A_SEED, pool.key().as_ref()],
        bump,
        token::mint = token_a_mint,
        token::authority = pool
    )]
    pub token_a_vault: Account<'info, TokenAccount>,

    /// Token B vault
    #[account(
        init,
        payer = authority,
        seeds = [VAULT_B_SEED, pool.key().as_ref()],
        bump,
        token::mint = token_b_mint,
        token::authority = pool
    )]
    pub token_b_vault: Account<'info, TokenAccount>,

    /// LP token mint
    #[account(
        init,
        payer = authority,
        seeds = [LP_MINT_SEED, pool.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = lp_mint_authority
    )]
    pub lp_mint: Account<'info, Mint>,

    /// LP mint authority PDA
    /// CHECK: PDA used as mint authority
    #[account(
        seeds = [LP_MINT_AUTHORITY_SEED, pool.key().as_ref()],
        bump
    )]
    pub lp_mint_authority: UncheckedAccount<'info>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn handler(
    ctx: Context<InitializePool>,
    fee_numerator: u64,
    fee_denominator: u64,
) -> Result<()> {
    // Validate token mints are different
    require!(
        ctx.accounts.token_a_mint.key() != ctx.accounts.token_b_mint.key(),
        AmmError::IdenticalTokenMints
    );

    // Validate fee parameters
    require!(
        validate_fee(fee_numerator, fee_denominator),
        AmmError::InvalidFeeParameters
    );

    let clock = Clock::get()?;
    let pool = &mut ctx.accounts.pool;

    // Initialize pool state
    pool.version = PROTOCOL_VERSION;
    pool.bump = ctx.bumps.pool;
    pool.lp_mint_authority_bump = ctx.bumps.lp_mint_authority;

    pool.token_a_mint = ctx.accounts.token_a_mint.key();
    pool.token_b_mint = ctx.accounts.token_b_mint.key();
    pool.token_a_vault = ctx.accounts.token_a_vault.key();
    pool.token_b_vault = ctx.accounts.token_b_vault.key();
    pool.lp_mint = ctx.accounts.lp_mint.key();

    pool.reserve_a = 0;
    pool.reserve_b = 0;

    pool.fee_numerator = fee_numerator;
    pool.fee_denominator = fee_denominator;

    pool.authority = ctx.accounts.authority.key();
    pool.paused = false;

    pool.total_swaps = 0;
    pool.cumulative_volume_a = 0;
    pool.cumulative_volume_b = 0;
    pool.cumulative_fees_a = 0;
    pool.cumulative_fees_b = 0;

    pool.created_at = clock.unix_timestamp;
    pool.last_swap_timestamp = 0;
    pool.last_update_slot = clock.slot;

    emit!(PoolCreated {
        pool: pool.key(),
        token_a_mint: pool.token_a_mint,
        token_b_mint: pool.token_b_mint,
        fee_numerator,
        fee_denominator,
        timestamp: clock.unix_timestamp,
    });

    msg!("Pool initialized: {}", pool.key());

    Ok(())
}
