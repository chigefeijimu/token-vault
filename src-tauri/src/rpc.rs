use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::chain_adapter::{ChainConfig, ChainRegistry};

// ============== Data Types ==============

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub address: String,
    pub balance: String,
    pub symbol: String,
    pub decimals: u8,
    #[serde(rename = "balanceFormatted")]
    pub balance_formatted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GasPriceInfo {
    pub slow: String,
    pub standard: String,
    pub fast: String,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GasEstimateResult {
    pub gas_limit: String,
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize)]
pub struct TransactionReceipt {
    pub transaction_hash: String,
    pub block_number: String,
    pub block_hash: String,
    pub from: String,
    pub to: Option<String>,
    pub cumulative_gas_used: String,
    pub gas_used: String,
    pub effective_gas_price: String,
    pub logs: Vec<TransactionLog>,
    pub status: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, serde::Serialize)]
pub struct TransactionLog {
    pub address: String,
    pub topics: Vec<String>,
    pub data: String,
    pub block_number: String,
    pub transaction_hash: String,
    pub log_index: usize,
    pub removed: bool,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct TxHistoryItem {
    pub transaction_hash: String,
    pub block_number: u64,
    pub block_hash: String,
    pub timestamp: u64,
    pub from: String,
    pub to: String,
    pub value: String,
    pub gas_used: String,
    pub gas_price: String,
    pub status: String,
}

#[derive(Debug, serde::Serialize)]
pub struct TxHistoryResult {
    pub transactions: Vec<TxHistoryItem>,
    #[serde(rename = "totalCount")]
    pub total_count: usize,
    #[serde(rename = "hasMore")]
    pub has_more: bool,
    #[serde(rename = "page")]
    pub page: u32,
    #[serde(rename = "pageSize")]
    pub page_size: u32,
}

// ============== RpcClient ==============

#[derive(Debug, Clone, Serialize)]
pub struct RpcClient {
    pub chains: HashMap<String, ChainConfig>,
}

impl RpcClient {
    pub fn new() -> Self {
        let mut chains = HashMap::new();
        chains.insert("ethereum".to_string(), ChainConfig {
            chain_id: 1,
            name: "Ethereum Mainnet".to_string(),
            rpc_url: "https://eth.llamarpc.com".to_string(),
            native_currency: "ETH".to_string(),
            explorer_url: "https://etherscan.io".to_string(),
            symbol: "ETH".to_string(),
        });
        chains.insert("polygon".to_string(), ChainConfig {
            chain_id: 137,
            name: "Polygon Mainnet".to_string(),
            rpc_url: "https://polygon-rpc.com".to_string(),
            native_currency: "MATIC".to_string(),
            explorer_url: "https://polygonscan.com".to_string(),
            symbol: "MATIC".to_string(),
        });
        chains.insert("bsc".to_string(), ChainConfig {
            chain_id: 56,
            name: "BNB Smart Chain".to_string(),
            rpc_url: "https://bsc-dataseed.binance.org".to_string(),
            native_currency: "BNB".to_string(),
            explorer_url: "https://bscscan.com".to_string(),
            symbol: "BNB".to_string(),
        });
        chains.insert("arbitrum".to_string(), ChainConfig {
            chain_id: 42161,
            name: "Arbitrum One".to_string(),
            rpc_url: "https://arb1.arbitrum.io/rpc".to_string(),
            native_currency: "ETH".to_string(),
            explorer_url: "https://arbiscan.io".to_string(),
            symbol: "ETH".to_string(),
        });
        chains.insert("optimism".to_string(), ChainConfig {
            chain_id: 10,
            name: "Optimism".to_string(),
            rpc_url: "https://mainnet.optimism.io".to_string(),
            native_currency: "ETH".to_string(),
            explorer_url: "https://optimistic.etherscan.io".to_string(),
            symbol: "ETH".to_string(),
        });
        chains.insert("avalanche".to_string(), ChainConfig {
            chain_id: 43114,
            name: "Avalanche C-Chain".to_string(),
            rpc_url: "https://api.avax.network/ext/bc/C/rpc".to_string(),
            native_currency: "AVAX".to_string(),
            explorer_url: "https://snowtrace.io".to_string(),
            symbol: "AVAX".to_string(),
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
    let rpc_url = match chain_id {
        1 => "https://eth.llamarpc.com",
        56 => "https://bsc-dataseed.binance.org",
        137 => "https://polygon-rpc.com",
        42161 => "https://arb1.arbitrum.io/rpc",
        10 => "https://mainnet.optimism.io",
        43114 => "https://api.avax.network/ext/bc/C/rpc",
        _ => return Err(format!("Unsupported chain: {}", chain_id)),
    };

    let client = reqwest::Client::new();
    let params = serde_json::json!([
        {"jsonrpc": "2.0", "method": "eth_getBalance", "params": [&address, "latest"], "id": 1}
    ]);

    let resp = client
        .post(rpc_url)
        .json(&params)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Invalid response: {}", e))?;

    let balance_hex = json["result"]
        .as_str()
        .ok_or("Invalid balance response")?;

    let balance_u128: u128 = u128::from_str_radix(balance_hex.trim_start_matches("0x"), 16)
        .map_err(|_| "Invalid balance hex")?;

    let symbol = match chain_id {
        1 => "ETH",
        56 => "BNB",
        137 => "MATIC",
        42161 => "ETH",
        10 => "ETH",
        43114 => "AVAX",
        _ => "ETH",
    };

    Ok(BalanceInfo {
        address: address.clone(),
        balance: balance_u128.to_string(),
        symbol: symbol.to_string(),
        decimals: 18,
        balance_formatted: format_balance_wei(balance_u128, 18, &symbol),
    })
}

// Format wei to human-readable string
fn format_balance_wei(wei: u128, decimals: u8, symbol: &str) -> String {
    let divisor = 10u128.pow(decimals as u32);
    let whole = wei / divisor;
    let remainder = wei % divisor;
    let decimals_str = format!("{:0>width$}", remainder, width = decimals as usize);
    let decimals_part = &decimals_str[..decimals.min(8) as usize].trim_end_matches('0');
    if decimals_part.is_empty() || *decimals_part == "0" {
        format!("{} {}", whole, symbol)
    } else {
        format!("{}.{} {}", whole, decimals_part, symbol)
    }
}

#[tauri::command]
pub async fn get_gas_price(chain_id: u64) -> Result<GasPriceInfo, String> {
    let rpc_url = match chain_id {
        1 => "https://eth.llamarpc.com",
        56 => "https://bsc-dataseed.binance.org",
        137 => "https://polygon-rpc.com",
        42161 => "https://arb1.arbitrum.io/rpc",
        10 => "https://mainnet.optimism.io",
        43114 => "https://api.avax.network/ext/bc/C/rpc",
        _ => return Err(format!("Unsupported chain: {}", chain_id)),
    };

    let client = reqwest::Client::new();
    let params = serde_json::json!([
        {"jsonrpc": "2.0", "method": "eth_gasPrice", "params": [], "id": 1}
    ]);

    let resp = client
        .post(rpc_url)
        .json(&params)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Invalid response: {}", e))?;

    let gas_price_hex = json["result"]
        .as_str()
        .ok_or("Invalid gas price response")?;

    let gas_price_u128: u128 = u128::from_str_radix(gas_price_hex.trim_start_matches("0x"), 16)
        .map_err(|_| "Invalid gas price hex")?;

    // Calculate slow/standard/fast based on the base gas price
    let slow = gas_price_u128 * 80 / 100;  // 80%
    let standard = gas_price_u128;          // 100%
    let fast = gas_price_u128 * 120 / 100; // 120%

    Ok(GasPriceInfo {
        slow: format!("0x{:x}", slow),
        standard: format!("0x{:x}", standard),
        fast: format!("0x{:x}", fast),
        unit: "wei".to_string(),
    })
}

#[tauri::command]
pub async fn estimate_gas(
    from: String,
    to: String,
    value: String,
    data: Option<String>,
    chain_id: u64,
) -> Result<GasEstimateResult, String> {
    let rpc_url = match chain_id {
        1 => "https://eth.llamarpc.com",
        56 => "https://bsc-dataseed.binance.org",
        137 => "https://polygon-rpc.com",
        42161 => "https://arb1.arbitrum.io/rpc",
        10 => "https://mainnet.optimism.io",
        43114 => "https://api.avax.network/ext/bc/C/rpc",
        _ => return Err(format!("Unsupported chain: {}", chain_id)),
    };

    let client = reqwest::Client::new();
    let params = serde_json::json!([
        {
            "jsonrpc": "2.0",
            "method": "eth_estimateGas",
            "params": [{
                "from": &from,
                "to": &to,
                "value": &value,
                "data": data.as_deref().unwrap_or("0x")
            }],
            "id": 1
        }
    ]);

    let resp = client
        .post(rpc_url)
        .json(&params)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let json: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Invalid response: {}", e))?;

    let gas_hex = json["result"]
        .as_str()
        .ok_or("Invalid gas estimate response")?;

    Ok(GasEstimateResult {
        gas_limit: gas_hex.to_string(),
    })
}

#[tauri::command]
pub async fn send_raw_transaction(
    _from: String,
    _to: String,
    _amount: String,
    _chain_id: u64,
) -> Result<String, String> {
    // Stub: return mock transaction hash
    let tx_hash = format!("0x{:064x}", rand::random::<u128>());
    Ok(tx_hash)
}

// NOTE: get_transaction_receipt moved to transaction.rs (real implementation)
// NOTE: sign_data moved to transaction.rs (real implementation)

#[tauri::command]
pub async fn get_transaction_history(
    address: String,
    chain_id: u64,
    page: u32,
    page_size: u32,
) -> Result<TxHistoryResult, String> {
    // For BSC (chain_id=56), use BSCExplorer if we have an API key
    // BSCScan API provides complete tx history including native BNB transfers
    if chain_id == 56 {
        let registry = ChainRegistry::new();
        if let Some(explorer) = registry.get_explorer(56) {
            match explorer.get_tx_history(&address, page, page_size).await {
                Ok(txs) => {
                    return Ok(TxHistoryResult {
                        transactions: txs.into_iter().map(|tx| TxHistoryItem {
                            transaction_hash: tx.tx_hash,
                            from: tx.from,
                            to: tx.to,
                            value: tx.value,
                            gas_used: tx.gas_used.clone(),
                            gas_price: tx.gas_price.clone(),
                            timestamp: tx.timestamp,
                            block_number: tx.block_number,
                            block_hash: tx.block_hash,
                            status: tx.status,
                        }).collect(),
                            total_count: (page * page_size + 100) as usize,
                            has_more: true,
                            page,
                            page_size,
                    });
                }
                Err(e) => {
                    eprintln!("BSCExplorer failed, falling back to block scan: {}", e);
                }
            }
        }
    }
    
    // Fallback: use eth_getLogs (only finds ERC20 token transfers, not native tx)
    let rpc_url = match chain_id {
        1 => "https://eth.llamarpc.com",
        56 => "https://bsc-dataseed.binance.org",
        137 => "https://polygon-rpc.com",
        42161 => "https://arb1.arbitrum.io/rpc",
        10 => "https://mainnet.optimism.io",
        43114 => "https://api.avax.network/ext/bc/C/rpc",
        _ => return Err(format!("Unsupported chain: {}", chain_id)),
    };

    let client = reqwest::Client::new();

    // Calculate block range for pagination
    // Get latest block first
    let latest_params = serde_json::json!([
        {"jsonrpc": "2.0", "method": "eth_blockNumber", "params": [], "id": 1}
    ]);
    let latest_resp = client
        .post(rpc_url)
        .json(&latest_params)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    let latest_json: serde_json::Value = latest_resp
        .json()
        .await
        .map_err(|e| format!("Invalid response: {}", e))?;
    let latest_block_hex = latest_json["result"].as_str().unwrap_or("0x0");
    let latest_block: u64 = u64::from_str_radix(latest_block_hex.trim_start_matches("0x"), 16)
        .unwrap_or(0);

    // Calculate from block (go back ~10000 blocks per page to get enough tx)
    let blocks_per_page = 10000u64;
    let from_block = if latest_block > page as u64 * blocks_per_page {
        latest_block - (page as u64 * blocks_per_page)
    } else {
        0
    };

    // Use eth_getLogs to get logs involving this address
    let logs_params = serde_json::json!([
        {
            "jsonrpc": "2.0",
            "method": "eth_getLogs",
            "params": [{
                "fromBlock": format!("0x{:x}", from_block),
                "toBlock": latest_block_hex,
                "address": &address,
                "topics": [],
                "offset": (page as u64 * blocks_per_page),
                "limit": page_size
            }],
            "id": 1
        }
    ]);

    let logs_resp = client
        .post(rpc_url)
        .json(&logs_params)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let logs_json: serde_json::Value = logs_resp
        .json()
        .await
        .map_err(|e| format!("Invalid response: {}", e))?;

    let logs = logs_json["result"].as_array().map(|a| a.clone()).unwrap_or_default();

    // Convert logs to TxHistoryItems
    let mut transactions: Vec<TxHistoryItem> = Vec::new();

    for log in logs {
        let tx_hash = log["transactionHash"].as_str().unwrap_or("").to_string();
        let block_number = log["blockNumber"].as_str().unwrap_or("0x0");
        let block_num: u64 = u64::from_str_radix(block_number.trim_start_matches("0x"), 16).unwrap_or(0);
        let _log_index = log["logIndex"].as_u64().unwrap_or(0) as usize;

        // Get block timestamp
        let block_params = serde_json::json!([
            {"jsonrpc": "2.0", "method": "eth_getBlockByNumber", "params": [block_number, false], "id": 1}
        ]);
        let block_resp = client
            .post(rpc_url)
            .json(&block_params)
            .send()
            .await;
        let timestamp = if let Ok(resp) = block_resp {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                json["result"]["timestamp"]
                    .as_str()
                    .map(|s| {
                        u64::from_str_radix(s.trim_start_matches("0x"), 16).unwrap_or(0)
                    })
                    .unwrap_or(0)
            } else {
                0
            }
        } else {
            0
        };

        // Get transaction receipt for more details
        let receipt_params = serde_json::json!([
            {"jsonrpc": "2.0", "method": "eth_getTransactionReceipt", "params": [&tx_hash], "id": 1}
        ]);
        let receipt_resp = client
            .post(rpc_url)
            .json(&receipt_params)
            .send()
            .await;
        let (from, to, value, gas_used, gas_price, status) = if let Ok(resp) = receipt_resp {
            if let Ok(json) = resp.json::<serde_json::Value>().await {
                let result = &json["result"];
                let from = result["from"].as_str().unwrap_or("").to_string();
                let to = result["to"].as_str().unwrap_or("").to_string();
                let value = result["value"].as_str().unwrap_or("0x0").to_string();
                let gas_used = result["gasUsed"].as_str().unwrap_or("0x0").to_string();
                let gas_price = result["effectiveGasPrice"].as_str().unwrap_or("0x0").to_string();
                let status = result["status"].as_str().unwrap_or("0x1").to_string();
                (from, to, value, gas_used, gas_price, status)
            } else {
                (String::new(), String::new(), "0x0".to_string(), "0x0".to_string(), "0x0".to_string(), "0x1".to_string())
            }
        } else {
            (String::new(), String::new(), "0x0".to_string(), "0x0".to_string(), "0x0".to_string(), "0x1".to_string())
        };

        let block_hash = log["blockHash"].as_str().unwrap_or("").to_string();

        transactions.push(TxHistoryItem {
            transaction_hash: tx_hash,
            block_number: block_num,
            block_hash,
            timestamp,
            from,
            to,
            value,
            gas_used,
            gas_price,
            status,
        });
    }

    // Sort by block number descending (newest first)
    transactions.sort_by(|a, b| b.block_number.cmp(&a.block_number));

    // Limit to page_size
    transactions.truncate(page_size as usize);

    let has_more = transactions.len() as u32 == page_size;
    let total_count = transactions.len();

    Ok(TxHistoryResult {
        transactions,
        total_count,
        has_more,
        page,
        page_size,
    })
}
