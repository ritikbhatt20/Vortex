use anchor_lang::prelude::*;

#[error_code]
pub enum AmmError {
    // Initialization
    #[msg("Invalid fee parameters")]
    InvalidFeeParameters,

    #[msg("Fee too high")]
    FeeTooHigh,

    #[msg("Pool already initialized")]
    PoolAlreadyInitialized,

    #[msg("Token mints must be different")]
    IdenticalTokenMints,

    // Liquidity
    #[msg("Pool not initialized")]
    PoolNotInitialized,

    #[msg("Initial liquidity too small")]
    InitialLiquidityTooSmall,

    #[msg("Insufficient liquidity minted")]
    InsufficientLiquidityMinted,

    #[msg("Insufficient liquidity burned")]
    InsufficientLiquidityBurned,

    #[msg("Slippage tolerance exceeded")]
    SlippageExceeded,

    #[msg("Amount too small")]
    AmountTooSmall,

    // Swap
    #[msg("Insufficient output amount")]
    InsufficientOutputAmount,

    #[msg("Insufficient liquidity for swap")]
    InsufficientLiquidity,

    #[msg("Invalid swap direction")]
    InvalidSwapDirection,

    #[msg("Output exceeds reserves")]
    OutputExceedsReserves,

    // Math
    #[msg("Math overflow")]
    MathOverflow,

    #[msg("Division by zero")]
    DivisionByZero,

    #[msg("Invariant violated")]
    InvariantViolation,

    // Accounts
    #[msg("Invalid token mint")]
    InvalidTokenMint,

    #[msg("Invalid vault")]
    InvalidVault,

    #[msg("Vault balance mismatch")]
    VaultBalanceMismatch,

    // Permissions
    #[msg("Unauthorized")]
    Unauthorized,

    #[msg("Pool paused")]
    PoolPaused,
}
