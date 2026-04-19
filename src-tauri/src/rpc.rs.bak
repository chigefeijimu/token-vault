use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub chain_id: u64,
    pub chain_name: String,
    pub rpc_url: String,
    pub native_currency: String,
    pub explorer_url: String,
    pub enabled: bool,
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
pub struct JsonRpcRequest {
    pub method: String,
    pub params: Vec<serde_json::Value>,
    pub id: u64,
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
    pub default_chain: String,
}

impl RpcClient {
    pub fn new() -> Self {
        let mut chains = HashMap::new();
        
        chains.insert(
            "ethereum".to_string(),
            ChainConfig {
                chain_id: 1,
                chain_name: "Ethereum Mainnet".to_string(),
                rpc_url: "https://eth.llamarpc.com".to_string(),
                native_currency: "ETH".to_string(),
                explorer_url: "https://etherscan.io".to_string(),
                enabled: true,
            },
        );
        
        chains.insert(
            "polygon".to_string(),
            ChainConfig {
                chain_id: 137,
                chain_name: "Polygon Mainnet".to_string(),
                rpc_url: "https://polygon-rpc.com".to_string(),
                native_currency: "MATIC".to_string(),
                explorer_url: "https://polygonscan.com".to_string(),
                enabled: true,
            },
        );
        
        chains.insert(
            "bsc".to_string(),
            ChainConfig {
                chain_id: 56,
                chain_name: "BNB Smart Chain".to_string(),
                rpc_url: "https://bsc-dataseed.binance.org".to_string(),
                native_currency: "BNB".to_string(),
                explorer_url: "https://bscscan.com".to_string(),
                enabled: true,
            },
        );
        
        chains.insert(
            "arbitrum".to_string(),
            ChainConfig {
                chain_id: 42161,
                chain_name: "Arbitrum One".to_string(),
                rpc_url: "https://arb1.arbitrum.io/rpc".to_string(),
                native_currency: "ETH".to_string(),
                explorer_url: "https://arbiscan.io".to_string(),
                enabled: true,
            },
        );
        
        chains.insert(
            "optimism".to_string(),
            ChainConfig {
                chain_id: 10,
                chain_name: "Optimism".to_string(),
                rpc_url: "https://mainnet.optimism.io".to_string(),
                native_currency: "ETH".to_string(),
                explorer_url: "https://optimistic.etherscan.io".to_string(),
                enabled: true,
            },
        );

        RpcClient {
            chains,
            default_chain: "ethereum".to_string(),
        }
    }

    pub fn get_default_chains(&self) -> Vec<ChainConfig> {
        self.chains.values().cloned().collect()
    }

    pub fn get_chain_config(&self, chain: &str) -> Option<ChainConfig> {
        self.chains.get(chain).cloned()
    }

    pub fn get_native_balance(&self, address: &str, chain: &str) -> Result<BalanceInfo, String> {
        let chain_config = self.chains.get(chain).ok_or("Chain not found")?;
        
        Ok(BalanceInfo {
            address: address.to_string(),
            balance: "0".to_string(),
            symbol: chain_config.native_currency.clone(),
            decimals: 18,
        })
    }

    pub fn get_gas_price(&self, chain: &str) -> Result<GasPriceInfo, String> {
        let _chain_config = self.chains.get(chain).ok_or("Chain not found")?;
        
        Ok(GasPriceInfo {
            slow: "1000000000".to_string(),
            standard: "2000000000".to_string(),
            fast: "5000000000".to_string(),
            unit: "wei".to_string(),
        })
    }

    pub fn get_transaction_history(&self, address: &str, chain: &str) -> Result<Vec<TransactionInfo>, String> {
        let chain_config = self.chains.get(chain).ok_or("Chain not found")?;
        
        if !chain_config.enabled {
            return Err("Chain is not enabled".to_string());
        }
        
        Ok(vec![])
    }

    pub fn get_erc20_balance(&self, address: &str, token_address: &str, chain: &str) -> Result<BalanceInfo, String> {
        let chain_config = self.chains.get(chain).ok_or("Chain not found")?;
        
        Ok(BalanceInfo {
            address: address.to_string(),
            balance: "0".to_string(),
            symbol: "TOKEN".to_string(),
            decimals: 18,
        })
    }
}

impl Default for RpcClient {
    fn default() -> Self {
        Self::new()
    }
}