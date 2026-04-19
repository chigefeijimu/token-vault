use ethers::middleware::Middleware;
use ethers::types::H256;
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

#[derive(Debug, Clone, Serialize)]
pub struct GasEstimateResult {
    pub gas_limit: String,
}

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
pub async fn get_gas_price(chain_id: u64) -> Result<GasPriceInfo, String> {
    use ethers::providers::{Provider, Http};

    let client = RpcClient::new();
    let config = client.get_chain_config(chain_id)
        .ok_or_else(|| format!("Unsupported chain: {}", chain_id))?;

    let provider = Provider::<Http>::try_from(config.rpc_url.as_str())
        .map_err(|e| format!("Failed to connect to RPC: {}", e))?;

    let gas_price = provider.get_gas_price()
        .await
        .map_err(|e| format!("Failed to get gas price: {}", e))?;

    let gas_hex = format!("0x{:x}", gas_price);

    Ok(GasPriceInfo {
        slow: gas_hex.clone(),
        standard: gas_hex.clone(),
        fast: gas_hex,
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
    use ethers::providers::{Provider, Http};
    use std::str::FromStr;

    let client = RpcClient::new();
    let config = client.get_chain_config(chain_id)
        .ok_or_else(|| format!("Unsupported chain: {}", chain_id))?;

    let provider = Provider::<Http>::try_from(config.rpc_url.as_str())
        .map_err(|e| format!("Failed to connect to RPC: {}", e))?;

    let from_addr = ethers::types::Address::from_str(&from)
        .map_err(|e| format!("Invalid from address: {}", e))?;
    let to_addr = ethers::types::Address::from_str(&to)
        .map_err(|e| format!("Invalid to address: {}", e))?;
    let val = ethers::types::U256::from_str(&value)
        .map_err(|e| format!("Invalid value: {}", e))?;

    let calldata = data.unwrap_or_else(|| "0x".to_string());

    let request = ethers::types::TransactionRequest {
        from: Some(from_addr),
        to: Some(ethers::types::NameOrAddress::Address(to_addr)),
        value: Some(val),
        data: Some(ethers::types::Bytes::from_str(&calldata).unwrap_or_default()),
        ..Default::default()
    };

    let gas = provider.estimate_gas(&request.into(), None)
        .await
        .map_err(|e| format!("Gas estimation failed: {}", e))?;

    Ok(GasEstimateResult {
        gas_limit: format!("0x{:x}", gas),
    })
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
    tx_hash: String,
    chain_id: u64,
) -> Result<Option<TransactionReceipt>, String> {
    use ethers::providers::{Provider, Http};
    use std::str::FromStr;

    let client = RpcClient::new();
    let config = client.get_chain_config(chain_id)
        .ok_or_else(|| format!("Unsupported chain: {}", chain_id))?;

    let provider = Provider::<Http>::try_from(config.rpc_url.as_str())
        .map_err(|e| format!("Failed to connect to RPC: {}", e))?;

    let tx_hashParsed = H256::from_str(&tx_hash)
        .map_err(|e| format!("Invalid tx hash: {}", e))?;

    let receipt = provider.get_transaction_receipt(tx_hashParsed)
        .await
        .map_err(|e| format!("Failed to get receipt: {}", e))?;

    match receipt {
        Some(r) => {
            let status = r.status.map(|s| s.as_u64() == 1).unwrap_or(false);
            let tx = r.transaction_hash;
            let block_hash = r.block_hash.map(|h| format!("{:?}", h)).unwrap_or_default();
            let block_number = r.block_number.map(|b| format!("0x{:x}", b.as_u64())).unwrap_or_default();
            let cumulative_gas_used = format!("0x{:x}", r.cumulative_gas_used);
            let gas_used = r.gas_used
                .map(|g| format!("0x{:x}", g))
                .unwrap_or_else(|| "0x0".to_string());
            let effective_gas_price = r.effective_gas_price
                .map(|p| format!("0x{:x}", p))
                .unwrap_or_else(|| "0x0".to_string());

            Ok(Some(TransactionReceipt {
                transaction_hash: format!("{:?}", tx),
                block_number,
                block_hash,
                from: r.from.to_string(),
                to: r.to.map(|t| t.to_string()),
                cumulative_gas_used,
                gas_used,
                effective_gas_price,
                logs: r.logs.iter().map(|l| TransactionLog {
                    address: l.address.to_string(),
                    topics: l.topics.iter().map(|t| format!("{:?}", t)).collect(),
                    data: format!("0x{}", hex::encode(&l.data.0)),
                    block_number: l.block_number.map(|b| format!("0x{:x}", b.as_u64())).unwrap_or_default(),
                    transaction_hash: l.transaction_hash.map(|h| format!("{:?}", h)).unwrap_or_default(),
                    log_index: l.log_index.map(|i| i.as_u64() as usize).unwrap_or(0),
                    removed: l.removed.unwrap_or(false),
                }).collect(),
                status,
            }))
        }
        None => Ok(None),
    }
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
    pub total_count: usize,
    pub has_more: bool,
}

#[tauri::command]
pub async fn get_transaction_history(
    address: String,
    chain_id: u64,
    page: u32,
    page_size: u32,
) -> Result<TxHistoryResult, String> {
    use ethers::providers::{Provider, Http};
    use std::str::FromStr;

    let client = RpcClient::new();
    let config = client.get_chain_config(chain_id)
        .ok_or_else(|| format!("Unsupported chain: {}", chain_id))?;

    let rpc_url = &config.rpc_url;
    let provider = Provider::<Http>::try_from(rpc_url.as_str())
        .map_err(|e| format!("Failed to connect to RPC: {}", e))?;

    let addr = ethers::types::Address::from_str(&address)
        .map_err(|e| format!("Invalid address: {}", e))?;

    let start_block = ethers::types::U64::from(0);
    let end_block = ethers::types::U64::from(u64::MAX);

    let filter = ethers::types::Filter::new()
        .from_block(start_block)
        .to_block(end_block)
        .address(addr);

    let logs = provider.get_logs(&filter)
        .await
        .map_err(|e| format!("Failed to fetch logs: {}", e))?;

    let mut txs: Vec<TxHistoryItem> = Vec::new();

    for log in logs {
        let tx_hash = log.transaction_hash.map(|h| format!("{:?}", h)).unwrap_or_default();

        // Get receipt for each tx to get gas_used, status, etc.
        let (gas_used, gas_price, status, block) = if !tx_hash.is_empty() {
            match provider.get_transaction_receipt(tx_hash.parse::<H256>().unwrap_or_default()).await {
                Ok(Some(receipt)) => {
                    let gas_used = receipt.gas_used.map(|g| format!("0x{:x}", g)).unwrap_or_default();
                    let gas_price = receipt.effective_gas_price.map(|p| format!("0x{:x}", p)).unwrap_or_else(|| "0x0".to_string());
                    let status = if receipt.status.map(|s| s.as_u64() == 1).unwrap_or(false) {
                        "0x1".to_string()
                    } else {
                        "0x0".to_string()
                    };
                    let block_num = receipt.block_number.map(|b| b.as_u64()).unwrap_or(0);
                    (gas_used, gas_price, status, block_num)
                }
                _ => ("0x0".to_string(), "0x0".to_string(), "0x1".to_string(), 0),
            }
        } else {
            ("0x0".to_string(), "0x0".to_string(), "0x1".to_string(), 0)
        };

        let block_hash = log.block_hash.map(|h| format!("{:?}", h)).unwrap_or_default();
        let _tx_index = log.transaction_index.map(|i| i.as_u64()).unwrap_or(0);

        let timestamp = if block > 0 {
            match provider.get_block(block).await {
                Ok(Some(b)) => b.timestamp.as_u64(),
                _ => 0,
            }
        } else {
            0
        };

        txs.push(TxHistoryItem {
            transaction_hash: tx_hash,
            block_number: block,
            block_hash,
            timestamp,
            from: log.address.to_string(),
            to: String::new(),
            value: "0x0".to_string(),
            gas_used,
            gas_price,
            status,
        });
    }

    // Sort by block number descending (newest first)
    txs.sort_by(|a, b| b.block_number.cmp(&a.block_number));

    let total_count = txs.len();
    let start = (page as usize).saturating_mul(page_size as usize);
    let end = (start + page_size as usize).min(total_count);

    let page_txs: Vec<TxHistoryItem> = if start < total_count {
        txs[start..end].to_vec()
    } else {
        Vec::new()
    };

    Ok(TxHistoryResult {
        transactions: page_txs,
        total_count,
        has_more: end < total_count,
    })
}
