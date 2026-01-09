use anchor_lang::prelude::*;
use crate::errors::AmmError;
use crate::constants::*;

/// Calculate square root using Babylonian method
pub fn sqrt(y: u64) -> Result<u64> {
    if y == 0 {
        return Ok(0);
    }

    let mut z = (y + 1) / 2;
    let mut x = y;

    while z < x {
        x = z;
        z = (y / z + z) / 2;
    }

    Ok(x)
}

/// Calculate output amount for a swap
///
/// Formula: amount_out = (amount_in_with_fee * reserve_out) / (reserve_in + amount_in_with_fee)
/// where amount_in_with_fee = amount_in * (1 - fee)
pub fn calculate_swap_output(
    amount_in: u64,
    reserve_in: u64,
    reserve_out: u64,
    fee_numerator: u64,
    fee_denominator: u64,
) -> Result<(u64, u64)> {
    require!(amount_in > 0, AmmError::AmountTooSmall);
    require!(reserve_in > 0, AmmError::PoolNotInitialized);
    require!(reserve_out > 0, AmmError::PoolNotInitialized);

    // Calculate fee
    let fee_amount = (amount_in as u128)
        .checked_mul(fee_numerator as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(fee_denominator as u128)
        .ok_or(AmmError::DivisionByZero)? as u64;

    // Amount after fee
    let amount_in_with_fee = amount_in
        .checked_sub(fee_amount)
        .ok_or(AmmError::MathOverflow)?;

    // Calculate output
    let numerator = (amount_in_with_fee as u128)
        .checked_mul(reserve_out as u128)
        .ok_or(AmmError::MathOverflow)?;

    let denominator = (reserve_in as u128)
        .checked_add(amount_in_with_fee as u128)
        .ok_or(AmmError::MathOverflow)?;

    let amount_out = numerator
        .checked_div(denominator)
        .ok_or(AmmError::DivisionByZero)? as u64;

    require!(amount_out > 0, AmmError::InsufficientOutputAmount);
    require!(amount_out < reserve_out, AmmError::InsufficientLiquidity);

    Ok((amount_out, fee_amount))
}

/// Calculate liquidity tokens to mint for initial deposit
///
/// Formula: sqrt(amount_a * amount_b)
pub fn calculate_initial_liquidity(amount_a: u64, amount_b: u64) -> Result<u64> {
    let product = (amount_a as u128)
        .checked_mul(amount_b as u128)
        .ok_or(AmmError::MathOverflow)?;

    require!(product <= u64::MAX as u128, AmmError::MathOverflow);

    let liquidity = sqrt(product as u64)?;

    require!(
        liquidity >= MINIMUM_LIQUIDITY,
        AmmError::InitialLiquidityTooSmall
    );

    Ok(liquidity)
}

/// Calculate liquidity tokens to mint for subsequent deposits
///
/// Formula: min(
///   (amount_a / reserve_a) * total_supply,
///   (amount_b / reserve_b) * total_supply
/// )
pub fn calculate_liquidity_to_mint(
    amount_a: u64,
    amount_b: u64,
    reserve_a: u64,
    reserve_b: u64,
    total_supply: u64,
) -> Result<u64> {
    require!(reserve_a > 0 && reserve_b > 0, AmmError::PoolNotInitialized);
    require!(total_supply > 0, AmmError::PoolNotInitialized);

    // Liquidity based on token A
    let liquidity_a = (amount_a as u128)
        .checked_mul(total_supply as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(reserve_a as u128)
        .ok_or(AmmError::DivisionByZero)? as u64;

    // Liquidity based on token B
    let liquidity_b = (amount_b as u128)
        .checked_mul(total_supply as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(reserve_b as u128)
        .ok_or(AmmError::DivisionByZero)? as u64;

    // Return minimum to prevent dilution
    let liquidity = std::cmp::min(liquidity_a, liquidity_b);

    require!(liquidity > 0, AmmError::InsufficientLiquidityMinted);

    Ok(liquidity)
}

/// Calculate token amounts to return when burning liquidity
///
/// Formula:
///   amount_a = (liquidity / total_supply) * reserve_a
///   amount_b = (liquidity / total_supply) * reserve_b
pub fn calculate_amounts_for_liquidity(
    liquidity: u64,
    reserve_a: u64,
    reserve_b: u64,
    total_supply: u64,
) -> Result<(u64, u64)> {
    require!(liquidity > 0, AmmError::InsufficientLiquidityBurned);
    require!(total_supply > 0, AmmError::PoolNotInitialized);
    require!(liquidity <= total_supply, AmmError::InsufficientLiquidityBurned);

    let amount_a = (reserve_a as u128)
        .checked_mul(liquidity as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(total_supply as u128)
        .ok_or(AmmError::DivisionByZero)? as u64;

    let amount_b = (reserve_b as u128)
        .checked_mul(liquidity as u128)
        .ok_or(AmmError::MathOverflow)?
        .checked_div(total_supply as u128)
        .ok_or(AmmError::DivisionByZero)? as u64;

    require!(amount_a > 0 && amount_b > 0, AmmError::InsufficientOutputAmount);

    Ok((amount_a, amount_b))
}

/// Verify invariant k does not decrease after swap
pub fn verify_invariant(
    old_reserve_a: u64,
    old_reserve_b: u64,
    new_reserve_a: u64,
    new_reserve_b: u64,
) -> Result<()> {
    let k_old = (old_reserve_a as u128)
        .checked_mul(old_reserve_b as u128)
        .ok_or(AmmError::MathOverflow)?;

    let k_new = (new_reserve_a as u128)
        .checked_mul(new_reserve_b as u128)
        .ok_or(AmmError::MathOverflow)?;

    require!(k_new >= k_old, AmmError::InvariantViolation);

    Ok(())
}
