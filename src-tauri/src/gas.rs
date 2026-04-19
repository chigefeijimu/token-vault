// Gas estimation and optimization module for EVM chains
// Provides gas price estimation, fee calculation, and optimization suggestions

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GasError {
    #[error("Failed to fetch gas price: {0}")]
    FetchFailed(String),
    #[error("Failed to estimate gas: {0}")]
    EstimateFailed(String),
    #[error("Chain not supported: {0}")]
    UnsupportedChain(u64),
    #[error("RPC error: {0}")]
    Rpc(String),
}

/// Gas price information for different priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasPrice {
    /// Low priority gas price in wei (slowest, cheapest)
    pub low: String,
    /// Medium priority gas price in wei (standard)
    pub medium: String,
    /// High priority gas price in wei (fastest, most expensive)
    pub high: String,
    /// Estimated time for low priority (in seconds)
    pub low_time: u32,
    /// Estimated time for medium priority (in seconds)
    pub medium_time: u32,
    /// Estimated time for high priority (in seconds)
    pub high_time: u32,
    /// Current base fee (for EIP-1559 chains)
    pub base_fee: Option<String>,
    /// Network congestion level (0-100)
    pub congestion: u8,
}

/// Gas estimation result for a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasEstimate {
    /// Estimated gas limit
    pub gas_limit: String,
    /// Estimated gas price (legacy) or max fee per gas (EIP-1559)
    pub gas_price: String,
    /// Maximum priority fee per gas (EIP-1559 only)
    pub max_priority_fee: Option<String>,
    /// Maximum fee per gas (EIP-1559 only)
    pub max_fee: Option<String>,
    /// Estimated total fee in native token
    pub total_fee: String,
    /// Estimated total fee in USD
    pub total_fee_usd: Option<f64>,
    /// EIP-1559 enabled flag
    pub is_eip1559: bool,
}

/// Gas optimization suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasOptimization {
    /// Current gas price
    pub current_gas_price: String,
    /// Suggested optimal gas price
    pub suggested_gas_price: String,
    /// Potential savings percentage
    pub savings_percent: f64,
    /// Potential savings in native token
    pub savings_amount: String,
    /// Reason for the suggestion
    pub reason: String,
    /// Recommended timing (immediate, wait_seconds)
    pub recommended_timing: GasTiming,
    /// Alternative chains with lower fees (chain_id, gas_price)
    pub alternative_chains: Vec<AlternativeChain>,
}

/// Timing recommendation for gas optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasTiming {
    /// Whether to execute immediately
    pub immediate: bool,
    /// Seconds to wait for better price (if not immediate)
    pub wait_seconds: Option<u32>,
    /// Best time window start (unix timestamp)
    pub best_window_start: Option<u64>,
    /// Best time window end (unix timestamp)
    pub best_window_end: Option<u64>,
}

/// Alternative chain with lower fees
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeChain {
    pub chain_id: u64,
    pub chain_name: String,
    pub gas_price: String,
    pub savings_percent: f64,
}

/// Complete gas info including prices, estimates, and optimizations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasInfo {
    pub chain_id: u64,
    pub gas_prices: GasPrice,
    pub estimates: Option<GasEstimate>,
    pub optimization: Option<GasOptimization>,
    pub timestamp: u64,
    /// Gas price source (e.g., "etherscan", "gas tracker", "rpc")
    pub source: String,
}

/// Fee breakdown for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeBreakdown {
    pub base_fee: String,
    pub priority_fee: String,
    pub gas_limit: String,
    pub gas_price: String,
    pub total_native: String,
    pub total_usd: Option<f64>,
    pub currency_symbol: String,
}