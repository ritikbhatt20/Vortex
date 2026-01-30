use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount, Transfer, MintTo};

use crate::state::Pool;
use crate::constants::*;
use crate::errors::AmmError;
use crate::events::LiquidityAdded;
use crate::math::{calculate_initial_liquidity, calculate_liquidity_to_mint};

#[derive(Accounts)]
pub struct AddLiquidity<'info> {
    /// Liquidity provider
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

    /// LP mint
    #[account(
        mut,
        seeds = [LP_MINT_SEED, pool.key().as_ref()],
        bump,
        constraint = lp_mint.key() == pool.lp_mint @ AmmError::InvalidVault
    )]
    pub lp_mint: Account<'info, Mint>,

    /// LP mint authority
    /// CHECK: PDA used as mint authority
    #[account(
        seeds = [LP_MINT_AUTHORITY_SEED, pool.key().as_ref()],
        bump = pool.lp_mint_authority_bump
    )]
    pub lp_mint_authority: UncheckedAccount<'info>,

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
    ctx: Context<AddLiquidity>,
    amount_a: u64,
    amount_b: u64,
    min_liquidity: u64,
) -> Result<()> {
    require!(amount_a > 0 && amount_b > 0, AmmError::AmountTooSmall);

    let pool = &ctx.accounts.pool;
    let total_supply = ctx.accounts.lp_mint.supply;

    // Calculate liquidity to mint
    let liquidity = if !pool.is_initialized() {
        // First deposit - use geometric mean
        require!(
            amount_a >= MIN_INITIAL_LIQUIDITY && amount_b >= MIN_INITIAL_LIQUIDITY,
            AmmError::InitialLiquidityTooSmall
        );
        calculate_initial_liquidity(amount_a, amount_b)?
            .checked_sub(MINIMUM_LIQUIDITY)
            .ok_or(AmmError::MathOverflow)?
    } else {
        // Subsequent deposits - proportional
        calculate_liquidity_to_mint(
            amount_a,
            amount_b,
            pool.reserve_a,
            pool.reserve_b,
            total_supply,
        )?
    };

    require!(liquidity >= min_liquidity, AmmError::SlippageExceeded);

    // Transfer token A from user to vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_a.to_account_info(),
                to: ctx.accounts.token_a_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount_a,
    )?;

    // Transfer token B from user to vault
    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_b.to_account_info(),
                to: ctx.accounts.token_b_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount_b,
    )?;

    // Mint LP tokens to user
    let pool_key = ctx.accounts.pool.key();
    let seeds = &[
        LP_MINT_AUTHORITY_SEED,
        pool_key.as_ref(),
        &[ctx.accounts.pool.lp_mint_authority_bump],
    ];
    let signer_seeds = &[&seeds[..]];

    // For first deposit, mint MINIMUM_LIQUIDITY to pool (locked forever)
    if !ctx.accounts.pool.is_initialized() {
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    mint: ctx.accounts.lp_mint.to_account_info(),
                    to: ctx.accounts.token_a_vault.to_account_info(), // Burn to vault
                    authority: ctx.accounts.lp_mint_authority.to_account_info(),
                },
                signer_seeds,
            ),
            MINIMUM_LIQUIDITY,
        )?;
    }

    // Mint LP tokens to user
    token::mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.lp_mint.to_account_info(),
                to: ctx.accounts.user_lp_token.to_account_info(),
                authority: ctx.accounts.lp_mint_authority.to_account_info(),
            },
            signer_seeds,
        ),
        liquidity,
    )?;

    // Update pool reserves
    let pool = &mut ctx.accounts.pool;
    pool.reserve_a = pool.reserve_a.checked_add(amount_a).ok_or(AmmError::MathOverflow)?;
    pool.reserve_b = pool.reserve_b.checked_add(amount_b).ok_or(AmmError::MathOverflow)?;
    pool.last_update_slot = Clock::get()?.slot;

    emit!(LiquidityAdded {
        pool: pool.key(),
        user: ctx.accounts.user.key(),
        amount_a,
        amount_b,
        liquidity_minted: liquidity,
        reserve_a: pool.reserve_a,
        reserve_b: pool.reserve_b,
        timestamp: Clock::get()?.unix_timestamp,
    });

    msg!("Added liquidity: {} A, {} B, minted {} LP", amount_a, amount_b, liquidity);

    Ok(())
}
