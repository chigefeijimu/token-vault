use ethers::providers::Provider;
use ethers::types::{H160, U256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFT {
    pub token_id: String,
    pub contract_address: String,
    pub name: String,
    pub description: String,
    pub image: String,
    pub collection: String,
    pub chain_id: u64,
    pub token_type: String,
    pub attributes: Vec<NFTTrait>,
    pub animation_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTTrait {
    pub trait_type: String,
    pub value: String,
    pub rarity: Option<String>,
}

#[derive(Debug, Deserialize)]
struct TokenURIResponse {
    name: Option<String>,
    description: Option<String>,
    image: Option<String>,
    animation_url: Option<String>,
    attributes: Option<Vec<RawTrait>>,
}

#[derive(Debug, Deserialize)]
struct RawTrait {
    trait_type: Option<String>,
    value: Option<serde_json::Value>,
}

fn get_rpc_url(chain_id: u64) -> Option<String> {
    match chain_id {
        1 => Some("https://eth.llamarpc.com".to_string()),
        56 => Some("https://bsc-dataseed.binance.org".to_string()),
        137 => Some("https://polygon.llamarpc.com".to_string()),
        42161 => Some("https://arb1.arbitrum.io/rpc".to_string()),
        10 => Some("https://mainnet.optimism.io".to_string()),
        43114 => Some("https://api.avax.network/ext/bc/C/rpc".to_string()),
        _ => None,
    }
}

pub async fn fetch_nfts(address: &str, chain_id: u64) -> Result<Vec<NFT>, String> {
    let rpc_url = get_rpc_url(chain_id).ok_or("Unsupported chain")?;
    let address: H160 = address.parse().map_err(|e| format!("Invalid address: {}", e))?;

    let client = reqwest::Client::new();

    // eth_getLogs for Transfer events
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_getLogs",
        "params": [{
            "fromBlock": "0x0",
            "toBlock": "latest",
            "address": "0x0000000000000000000000000000000000000000",
            "topics": [
                "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
                null,
                format!("0x{:040x}", address)
            ]
        }],
        "id": 1
    });

    let resp = client.post(&rpc_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("RPC request failed: {}", e))?
        .json::<RpcResponse>()
        .await
        .map_err(|e| format!("Failed to parse RPC response: {}", e))?;

    if let Some(error) = resp.error {
        return Err(error.message);
    }

    let logs = resp.result.unwrap_or_default();

    // Group by contract address, collect unique token ids
    let mut contract_tokens: HashMap<String, Vec<String>> = HashMap::new();
    for log in logs {
        let contract = log.address.clone();
        if let Some(token_id_hex) = log.topics.get(3) {
            let token_id = hex_to_decimal(token_id_hex);
            contract_tokens.entry(contract).or_default().push(token_id);
        }
    }

    let mut nfts = Vec::new();
    for (contract_addr, token_ids) in contract_tokens {
        for token_id in token_ids {
            match fetch_nft_from_contract(&rpc_url, &contract_addr, &token_id, chain_id).await {
                Ok(nft) => nfts.push(nft),
                Err(_) => continue,
            }
        }
    }

    nfts.sort_by(|a, b| a.collection.cmp(&b.collection));
    Ok(nfts)
}

fn hex_to_decimal(hex: &str) -> String {
    let hex = hex.strip_prefix("0x").unwrap_or(hex);
    let val: u128 = u128::from_str_radix(hex, 16).unwrap_or(0);
    val.to_string()
}

async fn fetch_nft_from_contract(
    rpc_url: &str,
    contract: &str,
    token_id: &str,
    chain_id: u64,
) -> Result<NFT, String> {
    let client = reqwest::Client::new();

    // tokenURI(uint256) selector: 0xc87b56dd
    let token_uri_hex = eth_call(
        &client,
        rpc_url,
        contract,
        &format!("0xc87b56dd{:0>64}", token_id),
    ).await?;

    let uri = decode_string_hex(&token_uri_hex).ok_or("Failed to decode tokenURI")?;

    let metadata = fetch_metadata(&uri).await?;

    let name = metadata.name.unwrap_or_else(|| format!("#{}", token_id));
    let description = metadata.description.unwrap_or_default();
    let image = metadata.image.unwrap_or_default();
    let animation_url = metadata.animation_url;
    let attributes = parse_traits(metadata.attributes);

    Ok(NFT {
        token_id: token_id.to_string(),
        contract_address: contract.to_string(),
        name,
        description,
        image,
        collection: "Unknown Collection".to_string(),
        chain_id,
        token_type: "ERC721".to_string(),
        attributes,
        animation_url,
    })
}

async fn eth_call(client: &reqwest::Client, rpc_url: &str, to: &str, data: &str) -> Result<String, String> {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [{
            "to": to,
            "data": data
        }, "latest"],
        "id": 1
    });

    let resp = client.post(rpc_url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("eth_call failed: {}", e))?
        .json::<EthCallResponse>()
        .await
        .map_err(|e| format!("RPC parse failed: {}", e))?;

    if let Some(error) = resp.error {
        return Err(error.message);
    }

    resp.result.ok_or("No result".to_string())
}

fn decode_string_hex(hex: &str) -> Option<String> {
    let hex = hex.strip_prefix("0x")?;
    let bytes = hex::decode(hex).ok()?;
    if bytes.len() < 64 {
        return None;
    }
    let offset = u128::from_be_bytes(bytes[0..32].try_into().ok()?) as usize;
    let len = u128::from_be_bytes(bytes[32..64].try_into().ok()?) as usize;
    if offset + len > bytes.len() {
        return None;
    }
    String::from_utf8(bytes[offset..offset + len].to_vec()).ok()
}

async fn fetch_metadata(uri: &str) -> Result<TokenURIResponse, String> {
    if uri.starts_with("data:") {
        let json = uri.strip_prefix("data:application/json,")
            .or_else(|| uri.strip_prefix("data:,"));
        if let Some(json) = json {
            return serde_json::from_str(json).map_err(|e| e.to_string());
        }
    }

    let client = reqwest::get(uri).await
        .map_err(|e| format!("HTTP error: {}", e))?
        .text()
        .await
        .map_err(|e| format!("Read error: {}", e))?;

    serde_json::from_str(&client).map_err(|e| format!("Parse error: {}", e))
}

fn parse_traits(raw: Option<Vec<RawTrait>>) -> Vec<NFTTrait> {
    raw.map(|traits| {
        traits.into_iter().filter_map(|t| {
            Some(NFTTrait {
                trait_type: t.trait_type?,
                value: match t.value? {
                    serde_json::Value::String(s) => s,
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => b.to_string(),
                    _ => return None,
                },
                rarity: None,
            })
        }).collect()
    }).unwrap_or_default()
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    #[serde(default)]
    result: Option<Vec<LogEntry>>,
    #[serde(default)]
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct EthCallResponse {
    #[serde(default)]
    result: Option<String>,
    #[serde(default)]
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    message: String,
}

#[derive(Debug, Deserialize)]
struct LogEntry {
    address: String,
    topics: Vec<String>,
}

#[tauri::command]
pub async fn get_nfts(address: String, chain_id: u64) -> Result<Vec<NFT>, String> {
    fetch_nfts(&address, chain_id).await
}

#[tauri::command]
pub async fn get_nft_metadata(
    contract_address: String,
    token_id: String,
    chain_id: u64,
) -> Result<NFT, String> {
    let rpc_url = get_rpc_url(chain_id).ok_or("Unsupported chain")?;
    fetch_nft_from_contract(&rpc_url, &contract_address, &token_id, chain_id).await
}
