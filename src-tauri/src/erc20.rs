// ERC20 Token Functions

use serde::{Deserialize, Serialize};

/// ERC20 token balance response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBalance {
    pub balance: String,
    pub decimals: u8,
    pub symbol: String,
}

/// Token info response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
}

/// Get ERC20 token info (name, symbol, decimals)
#[tauri::command]
pub async fn get_token_info(
    token_address: String,
    chain_id: u64,
) -> Result<TokenInfo, String> {
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
    let name = get_token_name(&client, rpc_url, &token_address).await?;
    let symbol = get_token_symbol(&client, rpc_url, &token_address).await?;
    let decimals = get_token_decimals(&client, rpc_url, &token_address).await?;

    Ok(TokenInfo { name, symbol, decimals })
}

/// Get ERC20 token balance for an address
pub async fn get_erc20_balance(
    rpc_url: &str,
    token_address: &str,
    wallet_address: &str,
) -> Result<TokenBalance, String> {
    let client = reqwest::Client::new();

    // ERC20 balanceOf function selector: 0x70a08231
    let padded_address = format!(
        "0x70a08231000000000000000000000000{}",
        wallet_address.trim_start_matches("0x")
    );

    let params = serde_json::json!([
        {
            "to": token_address,
            "data": padded_address
        },
        "latest"
    ]);

    let response: serde_json::Value = eth_call(&client, rpc_url, params)
        .await?;

    let balance_hex = response["result"]
        .as_str()
        .ok_or("Invalid response: missing result")?;

    // Parse the balance (returns uint256 in hex)
    let balance_str = if balance_hex == "0x0" {
        "0".to_string()
    } else {
        balance_hex.trim_start_matches("0x").to_string()
    };

    // Get token metadata (decimals and symbol)
    let decimals = get_token_decimals(&client, rpc_url, token_address).await?;
    let symbol = get_token_symbol(&client, rpc_url, token_address).await?;

    Ok(TokenBalance {
        balance: balance_str,
        decimals,
        symbol,
    })
}

/// Get token decimals
async fn get_token_decimals(client: &reqwest::Client, rpc_url: &str, token_address: &str) -> Result<u8, String> {
    // decimals() function selector: 0x313ce567
    let data = "0x313ce567".to_string();
    
    let params = serde_json::json!([
        {
            "to": token_address,
            "data": data
        },
        "latest"
    ]);

    let response: serde_json::Value = eth_call(client, rpc_url, params).await?;

    let decimals_hex = response["result"]
        .as_str()
        .ok_or("Invalid response: missing result")?;

    let decimals = u8::from_str_radix(decimals_hex.trim_start_matches("0x"), 16)
        .map_err(|_| "Failed to parse decimals")?;

    Ok(decimals)
}

/// Get token symbol
async fn get_token_symbol(client: &reqwest::Client, rpc_url: &str, token_address: &str) -> Result<String, String> {
    // symbol() function selector: 0x95d89b41
    let data = "0x95d89b41".to_string();
    
    let params = serde_json::json!([
        {
            "to": token_address,
            "data": data
        },
        "latest"
    ]);

    let response: serde_json::Value = eth_call(client, rpc_url, params).await?;

    let result_hex = response["result"]
        .as_str()
        .ok_or("Invalid response: missing result")?;

    // Symbol is returned as bytes32, we need to parse it
    let symbol = parse_bytes32_string(result_hex);
    
    Ok(symbol)
}

/// Get token name
async fn get_token_name(client: &reqwest::Client, rpc_url: &str, token_address: &str) -> Result<String, String> {
    // name() function selector: 0x06fdde03
    let data = "0x06fdde03".to_string();
    
    let params = serde_json::json!([
        {
            "to": token_address,
            "data": data
        },
        "latest"
    ]);

    let response: serde_json::Value = eth_call(client, rpc_url, params).await?;

    let result_hex = response["result"]
        .as_str()
        .ok_or("Invalid response: missing result")?;

    // Handle case where name() returns empty string (no name defined)
    if result_hex == "0x0000000000000000000000000000000000000000000000000000000000000000" {
        return Ok("Unknown".to_string());
    }

    Ok(parse_bytes32_string(result_hex))
}

/// Perform eth_call
async fn eth_call(client: &reqwest::Client, rpc_url: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
    let payload = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": params,
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

    Ok(serde_json::json!({ "result": resp.result }))
}

#[derive(Debug, Deserialize)]
struct EthCallResponse {
    result: Option<String>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    message: String,
}

/// Parse bytes32 string from hex
fn parse_bytes32_string(hex_str: &str) -> String {
    let hex = hex_str.trim_start_matches("0x");
    
    // Take the first 32 bytes and remove trailing zeros
    let bytes: Vec<u8> = hex
        .chars()
        .collect::<Vec<char>>()
        .chunks(2)
        .take(64)
        .filter_map(|chunk| {
            if chunk.len() == 2 {
                let s: String = chunk.iter().collect();
                u8::from_str_radix(&s, 16).ok()
            } else {
                None
            }
        })
        .collect();

    // Find the last non-zero byte and trim
    let end = bytes.iter().rposition(|&b| b != 0).map(|i| i + 1).unwrap_or(0);
    let trimmed = &bytes[..end];

    String::from_utf8_lossy(trimmed).to_string()
}

/// Convert raw balance to human readable format
pub fn format_balance(raw_balance: &str, decimals: u8) -> String {
    if raw_balance == "0" {
        return "0".to_string();
    }

    let decimals = decimals as usize;
    let balance_bytes = raw_balance.as_bytes();
    
    if balance_bytes.len() <= decimals {
        let padding = "0".repeat(decimals - balance_bytes.len());
        format!("0.{}{}", padding, raw_balance)
    } else {
        let split_point = balance_bytes.len() - decimals;
        let whole = &raw_balance[..split_point];
        let fraction = &raw_balance[split_point..];
        let trimmed_fraction = fraction.trim_end_matches('0');
        if trimmed_fraction.is_empty() {
            whole.to_string()
        } else {
            format!("{}.{}", whole, trimmed_fraction)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_balance() {
        // Test with 18 decimals (standard ERC20)
        assert_eq!(format_balance("1000000000000000000", 18), "1");
        assert_eq!(format_balance("1500000000000000000", 18), "1.5");
        assert_eq!(format_balance("1234567890123456789", 18), "1.234567890123456789");
        assert_eq!(format_balance("1000000", 6), "1");
        assert_eq!(format_balance("0", 18), "0");
    }

    #[test]
    fn test_parse_bytes32_string() {
        // Test with a symbol like "USDT" padded with zeros
        // USDT in bytes32: 0x5553445400000000000000000000000000000000000000000000000000000000
        let hex = "0x5553445400000000000000000000000000000000000000000000000000000000";
        assert_eq!(parse_bytes32_string(hex), "USDT");
    }
}
