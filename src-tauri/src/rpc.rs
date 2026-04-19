use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub chain_name: String,
    pub rpc_url: String,
    pub native_currency: String,
    pub explorer_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub address: String,
    pub balance: String,
    pub symbol: String,
    pub decimals: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub gas_used: String,
    pub gas_price: String,
    pub timestamp: u64,
    pub status: String,
    pub block_number: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasPriceInfo {
    pub slow: String,
    pub standard: String,
    pub fast: String,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RpcClient {
    pub chains: HashMap<String, ChainConfig>,
}

impl RpcClient {
    pub fn new() -> Self {
        let mut chains = HashMap::new();
        chains.insert("ethereum".to_string(), ChainConfig {
            chain_id: 1,
            chain_name: "Ethereum Mainnet".to_string(),
            rpc_url: "https://eth.llamarpc.com".to_string(),
            native_currency: "ETH".to_string(),
            explorer_url: "https://etherscan.io".to_string(),
        });
        chains.insert("polygon".to_string(), ChainConfig {
            chain_id: 137,
            chain_name: "Polygon Mainnet".to_string(),
            rpc_url: "https://polygon-rpc.com".to_string(),
            native_currency: "MATIC".to_string(),
            explorer_url: "https://polygonscan.com".to_string(),
        });
        chains.insert("bsc".to_string(), ChainConfig {
            chain_id: 56,
            chain_name: "BNB Smart Chain".to_string(),
            rpc_url: "https://bsc-dataseed.binance.org".to_string(),
            native_currency: "BNB".to_string(),
            explorer_url: "https://bscscan.com".to_string(),
        });
        chains.insert("arbitrum".to_string(), ChainConfig {
            chain_id: 42161,
            chain_name: "Arbitrum One".to_string(),
            rpc_url: "https://arb1.arbitrum.io/rpc".to_string(),
            native_currency: "ETH".to_string(),
            explorer_url: "https://arbiscan.io".to_string(),
        });
        chains.insert("optimism".to_string(), ChainConfig {
            chain_id: 10,
            chain_name: "Optimism".to_string(),
            rpc_url: "https://mainnet.optimism.io".to_string(),
            native_currency: "ETH".to_string(),
            explorer_url: "https://optimistic.etherscan.io".to_string(),
        });
        chains.insert("avalanche".to_string(), ChainConfig {
            chain_id: 43114,
            chain_name: "Avalanche C-Chain".to_string(),
            rpc_url: "https://api.avax.network/ext/bc/C/rpc".to_string(),
            native_currency: "AVAX".to_string(),
            explorer_url: "https://snowtrace.io".to_string(),
        });
        Self { chains }
    }

    pub fn get_chain_config(&self, chain_id: u64) -> Option<ChainConfig> {
        let chain_name = match chain_id {
            1 => "ethereum",
            56 => "bsc",
            137 => "polygon",
            42161 => "arbitrum",
            10 => "optimism",
            43114 => "avalanche",
            _ => return None,
        };
        self.chains.get(chain_name).cloned()
    }

    pub fn get_native_balance(&self, address: &str, chain_id: u64) -> Option<BalanceInfo> {
        let config = self.get_chain_config(chain_id)?;
        Some(BalanceInfo {
            address: address.to_string(),
            balance: "0".to_string(),
            symbol: config.native_currency,
            decimals: 18,
        })
    }
}

impl Default for RpcClient {
    fn default() -> Self {
        Self::new()
    }
}

// ============== Tauri Commands ==============

#[tauri::command]
pub async fn get_chain_config(chain_id: u64) -> Result<ChainConfig, String> {
    RpcClient::new()
        .get_chain_config(chain_id)
        .ok_or_else(|| format!("Unsupported chain: {}", chain_id))
}

#[tauri::command]
pub async fn get_balance(address: String, chain_id: u64) -> Result<BalanceInfo, String> {
    RpcClient::new()
        .get_native_balance(&address, chain_id)
        .ok_or_else(|| format!("Failed to get balance for chain {}", chain_id))
}

#[tauri::command]
pub async fn estimate_gas(
    _to_address: String,
    _amount: String,
    _chain_id: u64,
) -> Result<String, String> {
    Ok("21000".to_string())
}

#[tauri::command]
pub async fn send_raw_transaction(
    _from: String,
    _to: String,
    _amount: String,
    _chain_id: u64,
) -> Result<String, String> {
    // Mock transaction hash - real implementation would sign and broadcast
    let tx_hash = format!("0x{:064x}", rand::random::<u128>());
    Ok(tx_hash)
}

#[tauri::command]
pub async fn get_transaction_receipt(
    _tx_hash: String,
) -> Result<Option<TransactionInfo>, String> {
    Ok(None)
}
