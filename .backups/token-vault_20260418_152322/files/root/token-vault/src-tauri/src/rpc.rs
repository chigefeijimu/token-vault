// RPC module for EVM chain interactions
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub rpc_url: String,
    pub symbol: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub address: String,
    pub chain_id: u64,
    pub balance: String,      // Raw balance in wei
    pub balance_formatted: String,  // Human readable
    pub symbol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub timestamp: u64,
    pub block_number: u64,
    pub status: String,
}

// Default supported chains
pub fn get_default_chains() -> Vec<ChainConfig> {
    vec![
        ChainConfig {
            chain_id: 1,
            rpc_url: "https://eth.llamarpc.com".to_string(),
            symbol: "ETH".to_string(),
            name: "Ethereum".to_string(),
        },
        ChainConfig {
            chain_id: 56,
            rpc_url: "https://bsc-rpc.publicnode.com".to_string(),
            symbol: "BNB".to_string(),
            name: "BNB Chain".to_string(),
        },
        ChainConfig {
            chain_id: 137,
            rpc_url: "https://polygon-rpc.com".to_string(),
            symbol: "MATIC".to_string(),
            name: "Polygon".to_string(),
        },
        ChainConfig {
            chain_id: 42161,
            rpc_url: "https://arb1.arbitrum.io/rpc".to_string(),
            symbol: "ETH".to_string(),
            name: "Arbitrum".to_string(),
        },
        ChainConfig {
            chain_id: 10,
            rpc_url: "https://mainnet.optimism.io".to_string(),
            symbol: "ETH".to_string(),
            name: "Optimism".to_string(),
        },
        ChainConfig {
            chain_id: 43114,
            rpc_url: "https://api.avax.network/ext/bc/C/rpc".to_string(),
            symbol: "AVAX".to_string(),
            name: "Avalanche".to_string(),
        },
    ]
}

// Get chain config by chain_id
pub fn get_chain_config(chain_id: u64) -> Option<ChainConfig> {
    get_default_chains()
        .into_iter()
        .find(|c| c.chain_id == chain_id)
}

// JSON-RPC request
#[derive(Debug, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    method: String,
    params: Vec<serde_json::Value>,
    id: u64,
}

impl JsonRpcRequest {
    fn new(method: &str, params: Vec<serde_json::Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: 1,
        }
    }
}

// Get native token balance for an address
pub async fn get_native_balance(address: &str, chain_id: u64) -> Result<BalanceInfo, String> {
    let chains = get_default_chains();
    let chain = chains
        .iter()
        .find(|c| c.chain_id == chain_id)
        .ok_or_else(|| format!("Unsupported chain_id: {}", chain_id))?;

    let client = reqwest::Client::new();
    
    let request = JsonRpcRequest::new(
        "eth_getBalance",
        vec![serde_json::Value::String(address.to_string()), serde_json::Value::String("latest".to_string())],
    );

    let response = client
        .post(&chain.rpc_url)
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("RPC request failed: {}", e))?;

    let result: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse RPC response: {}", e))?;

    let balance_hex = result
        .get("result")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "No result in RPC response".to_string())?;

    // Parse hex balance
    let balance_u128 = u128::from_str_radix(balance_hex.trim_start_matches("0x"), 16)
        .unwrap_or(0);

    // Convert to human readable (divide by 10^18)
    let decimals = 18u64;
    let divisor = 10_u128.pow(decimals as u32);
    let whole = balance_u128 / divisor;
    let fraction = balance_u128 % divisor;
    let formatted = format!("{}.{:0>18}", whole, fraction);

    Ok(BalanceInfo {
        address: address.to_string(),
        chain_id,
        balance: balance_hex.to_string(),
        balance_formatted: formatted,
        symbol: chain.symbol.clone(),
    })
}
