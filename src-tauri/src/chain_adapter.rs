//! Chain adapter layer for multi-chain support
//! Provides unified interface for different blockchain explorers

use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

// ============== Data Types ==============

/// Chain configuration for multi-chain support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub name: String,
    pub rpc_url: String,
    pub explorer_url: String,
    pub native_currency: String,
    pub symbol: String,
}

/// Unified transaction history item structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplorerTx {
    pub tx_hash: String,
    pub block_number: u64,
    pub block_hash: String,
    pub timestamp: u64,
    pub from: String,
    pub to: String,
    pub value: String,
    pub gas_used: String,
    pub gas_price: String,
    pub status: String,
    pub confirmations: Option<u64>,
    pub nonce: Option<u64>,
}

/// Explorer error types
#[derive(Error, Debug)]
pub enum ExplorerError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("API error: {0}")]
    Api(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),
}

impl From<reqwest::Error> for ExplorerError {
    fn from(e: reqwest::Error) -> Self {
        ExplorerError::Network(e.to_string())
    }
}

// ============== ChainExplorer Trait ==============

/// Async trait for blockchain explorer adapters
#[async_trait]
pub trait ChainExplorer: Send + Sync {
    /// Get chain ID
    fn chain_id(&self) -> u64;
    
    /// Get chain name
    fn chain_name(&self) -> &str;
    
    /// Fetch transaction history for an address
    async fn get_tx_history(
        &self,
        address: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<ExplorerTx>, ExplorerError>;
    
    /// Fetch a single transaction by hash
    async fn get_tx(&self, tx_hash: &str) -> Result<Option<ExplorerTx>, ExplorerError>;
    
    /// Fetch gas price from explorer (optional, returns None if not supported)
    async fn get_gas_price(&self) -> Result<Option<(String, String, String)>, ExplorerError> {
        Ok(None)
    }
}

// ============== BSC Explorer ==============

/// BSCScan API response types
#[derive(Debug, Deserialize)]
struct BscScanResponse<T> {
    status: String,
    message: String,
    result: T,
}

#[derive(Debug, Deserialize)]
struct BscScanTx {
    #[serde(rename = "blockNumber")]
    block_number: String,
    #[serde(rename = "timeStamp")]
    timestamp: String,
    hash: String,
    #[serde(rename = "from")]
    from: String,
    #[serde(rename = "to")]
    to: String,
    #[serde(rename = "value")]
    value: String,
    #[serde(rename = "gasUsed")]
    gas_used: String,
    #[serde(rename = "gasPrice")]
    gas_price: String,
    #[serde(rename = "isError")]
    is_error: String,
    #[serde(rename = "txreceipt_status")]
    tx_receipt_status: String,
    #[serde(rename = "confirmations")]
    confirmations: Option<String>,
    #[serde(rename = "nonce")]
    nonce: Option<String>,
}

/// BSCScan API implementation
pub struct BSCExplorer {
    chain_id: u64,
    name: String,
    api_key: String,
    base_url: String,
    client: Client,
}

impl BSCExplorer {
    pub fn new(chain_id: u64, name: String, api_key: String, base_url: String) -> Self {
        Self {
            chain_id,
            name,
            api_key,
            base_url,
            client: Client::new(),
        }
    }

    fn parse_hex_u64(s: &str) -> Option<u64> {
        let s = s.trim_start_matches("0x");
        u64::from_str_radix(s, 16).ok()
    }

    fn parse_hex_u128(s: &str) -> Option<u128> {
        let s = s.trim_start_matches("0x");
        u128::from_str_radix(s, 16).ok()
    }

    async fn get(
        &self,
        module: &str,
        action: &str,
        params: &[(&str, &str)],
    ) -> Result<serde_json::Value, ExplorerError> {
        let mut url = format!(
            "{}/api?module={}&action={}&apikey={}",
            self.base_url, module, action, self.api_key
        );
        for (key, value) in params {
            url.push_str(&format!("&{}={}", key, value));
        }

        let resp = self.client.get(&url).send().await?;
        let json: serde_json::Value = resp.json().await?;
        Ok(json)
    }
}

#[async_trait::async_trait]
impl ChainExplorer for BSCExplorer {
    fn chain_id(&self) -> u64 {
        self.chain_id
    }

    fn chain_name(&self) -> &str {
        &self.name
    }

    async fn get_tx_history(
        &self,
        address: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Vec<ExplorerTx>, ExplorerError> {
        let page_str = page.to_string();
        let page_size_str = page_size.to_string();
        let params = vec![
            ("address", address),
            ("startblock", "0"),
            ("endblock", "99999999"),
            ("page", &page_str),
            ("offset", &page_size_str),
            ("sort", "desc"),
        ];

        let json = self.get("account", "txlist", &params).await?;
        
        let response: BscScanResponse<Vec<BscScanTx>> = serde_json::from_value(json)
            .map_err(|e| ExplorerError::Parse(e.to_string()))?;

        if response.status != "1" && response.result.is_empty() == false {
            if response.message != "No transactions found" {
                return Err(ExplorerError::Api(response.message));
            }
        }

        let txs = response.result;
        let mut explorer_txs = Vec::with_capacity(txs.len());

        for tx in txs {
            let block_number = Self::parse_hex_u64(&tx.block_number).unwrap_or(0);
            let timestamp = Self::parse_hex_u64(&tx.timestamp).unwrap_or(0);
            let confirmations = tx.confirmations.as_ref().and_then(|s| Self::parse_hex_u64(s));
            let nonce = tx.nonce.as_ref().and_then(|s| Self::parse_hex_u64(s));
            let gas_used = tx.gas_used.clone();
            let gas_price = tx.gas_price.clone();
            
            // Parse value from wei to readable format
            let value = if let Some(v) = Self::parse_hex_u128(&tx.value) {
                // Convert wei to BNB (18 decimals)
                let bnb = v as f64 / 1e18_f64;
                format!("{:.8}", bnb)
            } else {
                tx.value.clone()
            };

            let status = if tx.is_error == "1" || tx.tx_receipt_status == "0" {
                "failed".to_string()
            } else {
                "success".to_string()
            };

            explorer_txs.push(ExplorerTx {
                tx_hash: tx.hash,
                block_number,
                block_hash: String::new(), // BSCScan doesn't provide block hash in txlist
                timestamp,
                from: tx.from,
                to: tx.to,
                value,
                gas_used,
                gas_price,
                status,
                confirmations,
                nonce,
            });
        }

        Ok(explorer_txs)
    }

    async fn get_tx(&self, tx_hash: &str) -> Result<Option<ExplorerTx>, ExplorerError> {
        let params = vec![("txhash", tx_hash)];
        let json = self.get("account", "txlist", &params).await?;

        let response: BscScanResponse<Vec<BscScanTx>> = serde_json::from_value(json)
            .map_err(|e| ExplorerError::Parse(e.to_string()))?;

        if response.result.is_empty() {
            return Ok(None);
        }

        let tx = &response.result[0];
        let block_number = Self::parse_hex_u64(&tx.block_number).unwrap_or(0);
        let timestamp = Self::parse_hex_u64(&tx.timestamp).unwrap_or(0);
        let confirmations = tx.confirmations.as_ref().and_then(|s| Self::parse_hex_u64(s));
        let nonce = tx.nonce.as_ref().and_then(|s| Self::parse_hex_u64(s));

        let value = if let Some(v) = Self::parse_hex_u128(&tx.value) {
            let bnb = v as f64 / 1e18_f64;
            format!("{:.8}", bnb)
        } else {
            tx.value.clone()
        };

        let status = if tx.is_error == "1" || tx.tx_receipt_status == "0" {
            "failed".to_string()
        } else {
            "success".to_string()
        };

        Ok(Some(ExplorerTx {
            tx_hash: tx.hash.clone(),
            block_number,
            block_hash: String::new(),
            timestamp,
            from: tx.from.clone(),
            to: tx.to.clone(),
            value,
            gas_used: tx.gas_used.clone(),
            gas_price: tx.gas_price.clone(),
            status,
            confirmations,
            nonce,
        }))
    }
}

// ============== Chain Registry ==============

/// Registry for managing chain explorer adapters
pub struct ChainRegistry {
    explorers: HashMap<u64, Box<dyn ChainExplorer>>,
    configs: HashMap<u64, ChainConfig>,
}

impl ChainRegistry {
    pub fn new() -> Self {
        Self {
            explorers: HashMap::new(),
            configs: HashMap::new(),
        }
    }

    /// Register a chain with its explorer
    pub fn register<E: ChainExplorer + 'static>(&mut self, explorer: E) {
        let chain_id = explorer.chain_id();
        let name = explorer.chain_name().to_string();
        self.explorers.insert(chain_id, Box::new(explorer));
        
        // Also register a default config if not already present
        if !self.configs.contains_key(&chain_id) {
            self.configs.insert(chain_id, ChainConfig {
                chain_id,
                name: name.clone(),
                rpc_url: String::new(),
                explorer_url: String::new(),
                native_currency: String::new(),
                symbol: String::new(),
            });
        }
    }

    /// Register a chain with configuration
    pub fn register_with_config(&mut self, config: ChainConfig, explorer: Box<dyn ChainExplorer>) {
        let chain_id = config.chain_id;
        self.configs.insert(chain_id, config);
        self.explorers.insert(chain_id, explorer);
    }

    /// Get explorer for a chain
    pub fn get_explorer(&self, chain_id: u64) -> Option<&dyn ChainExplorer> {
        self.explorers.get(&chain_id).map(|e| e.as_ref())
    }

    /// Get chain configuration
    pub fn get_config(&self, chain_id: u64) -> Option<&ChainConfig> {
        self.configs.get(&chain_id)
    }

    /// Get all supported chain IDs
    pub fn supported_chains(&self) -> Vec<u64> {
        self.explorers.keys().cloned().collect()
    }

    /// Create default registry with BSC explorer
    pub fn default_with_bsc(api_key: String) -> Self {
        let mut registry = Self::new();
        
        // BSC Mainnet
        let bsc_mainnet = BSCExplorer::new(
            56,
            "BNB Smart Chain".to_string(),
            api_key,
            "https://api.bscscan.com".to_string(),
        );
        registry.register(bsc_mainnet);

        // Update BSC config with proper values
        if let Some(config) = registry.configs.get_mut(&56) {
            *config = ChainConfig {
                chain_id: 56,
                name: "BNB Smart Chain".to_string(),
                rpc_url: "https://bsc-dataseed.binance.org".to_string(),
                explorer_url: "https://bscscan.com".to_string(),
                native_currency: "BNB".to_string(),
                symbol: "BNB".to_string(),
            };
        }

        registry
    }
}

impl Default for ChainRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chain_config_serialization() {
        let config = ChainConfig {
            chain_id: 56,
            name: "BNB Smart Chain".to_string(),
            rpc_url: "https://bsc-dataseed.binance.org".to_string(),
            explorer_url: "https://bscscan.com".to_string(),
            native_currency: "BNB".to_string(),
            symbol: "BNB".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: ChainConfig = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.chain_id, 56);
        assert_eq!(deserialized.name, "BNB Smart Chain");
    }

    #[test]
    fn test_explorer_tx_serialization() {
        let tx = ExplorerTx {
            tx_hash: "0x123".to_string(),
            block_number: 100,
            block_hash: "0xabc".to_string(),
            timestamp: 1234567890,
            from: "0xfrom".to_string(),
            to: "0xto".to_string(),
            value: "1.5".to_string(),
            gas_used: "21000".to_string(),
            gas_price: "1000000000".to_string(),
            status: "success".to_string(),
            confirmations: Some(100),
            nonce: Some(1),
        };

        let json = serde_json::to_string(&tx).unwrap();
        let deserialized: ExplorerTx = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.tx_hash, "0x123");
        assert_eq!(deserialized.status, "success");
    }

    #[test]
    fn test_chain_registry_default() {
        let registry = ChainRegistry::default_with_bsc("test_api_key".to_string());
        
        assert!(registry.supported_chains().contains(&56));
        assert_eq!(registry.get_config(56).unwrap().chain_id, 56);
    }
}
