/// Vortex AMM Constants

// ============================================================================
// SCALING CONSTANTS
// ============================================================================

/// Basis points denominator (100% = 10000 BPS)
pub const BPS_DENOMINATOR: u64 = 10_000;

/// Precision for price calculations (2^64)
pub const Q64: u128 = 1u128 << 64;

/// Minimum liquidity locked forever on first deposit
pub const MINIMUM_LIQUIDITY: u64 = 1_000;

// ============================================================================
// FEE TIERS
// ============================================================================

/// Standard fee: 0.3% (3/1000)
pub const STANDARD_FEE_NUMERATOR: u64 = 3;
pub const STANDARD_FEE_DENOMINATOR: u64 = 1_000;

/// Low fee: 0.05% (5/10000) for stablecoins
pub const LOW_FEE_NUMERATOR: u64 = 5;
pub const LOW_FEE_DENOMINATOR: u64 = 10_000;

/// High fee: 1% (1/100) for exotic pairs
pub const HIGH_FEE_NUMERATOR: u64 = 1;
pub const HIGH_FEE_DENOMINATOR: u64 = 100;

// ============================================================================
// LIMITS
// ============================================================================

/// Maximum fee allowed (10% = 1000 BPS)
pub const MAX_FEE_BPS: u64 = 1_000;

/// Minimum fee allowed (0.01% = 1 BPS)
pub const MIN_FEE_BPS: u64 = 1;

/// Minimum swap amount (prevents dust attacks)
pub const MIN_SWAP_AMOUNT: u64 = 100;

/// Minimum initial liquidity
pub const MIN_INITIAL_LIQUIDITY: u64 = 1_000;

// ============================================================================
// PDA SEEDS
// ============================================================================

/// Seed for pool PDA
pub const POOL_SEED: &[u8] = b"pool";

/// Seed for token A vault PDA
pub const VAULT_A_SEED: &[u8] = b"vault_a";

/// Seed for token B vault PDA
pub const VAULT_B_SEED: &[u8] = b"vault_b";

/// Seed for LP token mint PDA
pub const LP_MINT_SEED: &[u8] = b"lp_mint";

/// Seed for LP mint authority PDA
pub const LP_MINT_AUTHORITY_SEED: &[u8] = b"lp_mint_authority";

// ============================================================================
// PROTOCOL
// ============================================================================

/// Current protocol version
pub const PROTOCOL_VERSION: u8 = 1;

// ============================================================================
// HELPERS
// ============================================================================

/// Validate fee parameters
pub fn validate_fee(numerator: u64, denominator: u64) -> bool {
    if denominator == 0 {
        return false;
    }
    let fee_bps = (numerator * BPS_DENOMINATOR) / denominator;
    fee_bps >= MIN_FEE_BPS && fee_bps <= MAX_FEE_BPS
}
