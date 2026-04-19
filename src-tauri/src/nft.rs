// NFT module for ERC-721 and ERC-1155 token interactions

use crate::errors::AppError;
use crate::rpc::Provider;
use serde::{Deserialize, Serialize};

/// NFT metadata response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTMetadata {
    pub name: String,
    pub description: Option<String>,
    pub image: Option<String>,
    pub animation_url: Option<String>,
    pub external_url: Option<String>,
    pub attributes: Vec<NFTAttribute>,
}

/// NFT attribute
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTAttribute {
    #[serde(rename = "trait_type")]
    pub trait_type: Option<String>,
    pub value: serde_json::Value,
    #[serde(rename = "display_type")]
    pub display_type: Option<String>,
}

/// NFT info structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTInfo {
    pub token_id: String,
    pub contract_address: String,
    pub contract_type: String,
    pub name: String,
    pub symbol: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub animation_url: Option<String>,
    pub external_url: Option<String>,
    pub attributes: Vec<NFTAttribute>,
    pub balance: Option<String>,
    pub owner: String,
    pub chain_id: u64,
}

/// NFT transfer parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NFTTransferParams {
    pub chain_id: u64,
    pub contract_address: String,
    pub from: String,
    pub to: String,
    pub token_id: String,
    pub amount: Option<String>,
}

/// ERC-165 interface for contract type detection
const ERC165_INTERFACE_ID: &str = "0x01ffc9a7";
const ERC721_INTERFACE_ID: &str = "0x80ac58cd";
const ERC1155_INTERFACE_ID: &str = "0xd9b67a26";

/// Detect contract type by checking supportsInterface
pub fn detect_contract_type(rpc_url: &str, contract_address: &str) -> Result<String, AppError> {
    let client = ureq::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build();
    
    // Try ERC-1155 first as it's more specific
    let erc1155_check = check_interface(&client, rpc_url, contract_address, ERC1155_INTERFACE_ID)?;
    if erc1155_check {
        return Ok("erc1155".to_string());
    }
    
    // Check for ERC-721
    let erc721_check = check_interface(&client, rpc_url, contract_address, ERC721_INTERFACE_ID)?;
    if erc721_check {
        return Ok("erc721".to_string());
    }
    
    Ok("unknown".to_string())
}

fn check_interface(client: &ureq::Agent, rpc_url: &str, contract_address: &str, interface_id: &str) -> Result<bool, AppError> {
    let method = "eth_call";
    let params = serde_json::json!([
        {
            "to": contract_address,
            "data": format!("{}{}", ERC165_INTERFACE_ID, interface_id[2..])
        },
        "latest"
    ]);
    
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    
    let response = client
        .post(rpc_url)
        .send_json(&request)
        .map_err(|e| AppError::Rpc(format!("RPC request failed: {}", e)))?;
    
    let result: serde_json::Value = response.into_json()
        .map_err(|e| AppError::Parse(format!("Failed to parse response: {}", e)))?;
    
    if let Some(error) = result.get("error") {
        return Err(AppError::Rpc(error.to_string()));
    }
    
    let data = result.get("result")
        .and_then(|r| r.as_str())
        .unwrap_or("0x");
    
    Ok(data != "0x0000000000000000000000000000000000000000000000000000000000000000")
}

/// Get ERC-721 token balance for an address
pub fn get_erc721_balance(rpc_url: &str, contract_address: &str, owner: &str) -> Result<u64, AppError> {
    let client = ureq::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build();
    
    // balanceOf selector: 70a08231 + pad32(owner)
    let owner_padded = format!("{:0>64}", &owner[2..]);
    let data = format!("70a08231{}", owner_padded);
    
    let params = serde_json::json!([
        {
            "to": contract_address,
            "data": data
        },
        "latest"
    ]);
    
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": params,
        "id": 1
    });
    
    let response = client
        .post(rpc_url)
        .send_json(&request)
        .map_err(|e| AppError::Rpc(format!("RPC request failed: {}", e)))?;
    
    let result: serde_json::Value = response.into_json()
        .map_err(|e| AppError::Parse(format!("Failed to parse response: {}", e)))?;
    
    if let Some(error) = result.get("error") {
        return Err(AppError::Rpc(error.to_string()));
    }
    
    let hex_balance = result.get("result")
        .and_then(|r| r.as_str())
        .ok_or_else(|| AppError::Rpc("No result in response".to_string()))?;
    
    let balance = u64::from_str_radix(hex_balance.trim_start_matches("0x"), 16)
        .map_err(|e| AppError::Parse(format!("Failed to parse balance: {}", e)))?;
    
    Ok(balance)
}

/// Get ERC-1155 token balance for an address
pub fn get_erc1155_balance(rpc_url: &str, contract_address: &str, owner: &str, token_id: &str) -> Result<String, AppError> {
    let client = ureq::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build();
    
    // balanceOf selector: 00fdd58e + pad32(owner) + pad32(token_id)
    let owner_padded = format!("{:0>64}", &owner[2..]);
    let token_padded = format!("{:0>64}", token_id.trim_start_matches("0x"));
    let data = format!("00fdd58e{}{}", owner_padded, token_padded);
    
    let params = serde_json::json!([
        {
            "to": contract_address,
            "data": data
        },
        "latest"
    ]);
    
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": params,
        "id": 1
    });
    
    let response = client
        .post(rpc_url)
        .send_json(&request)
        .map_err(|e| AppError::Rpc(format!("RPC request failed: {}", e)))?;
    
    let result: serde_json::Value = response.into_json()
        .map_err(|e| AppError::Parse(format!("Failed to parse response: {}", e)))?;
    
    if let Some(error) = result.get("error") {
        return Err(AppError::Rpc(error.to_string()));
    }
    
    let hex_balance = result.get("result")
        .and_then(|r| r.as_str())
        .ok_or_else(|| AppError::Rpc("No result in response".to_string()))?;
    
    Ok(hex_balance.to_string())
}

/// Get token URI for NFT metadata
pub fn get_token_uri(rpc_url: &str, contract_address: &str, token_id: &str, contract_type: &str) -> Result<Option<String>, AppError> {
    let client = ureq::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build();
    
    let data = if contract_type == "erc1155" {
        // URI selector for ERC-1155: 0x0e89341c + pad32(token_id)
        let token_padded = format!("{:0>64}", token_id.trim_start_matches("0x"));
        format!("0e89341c{}", token_padded)
    } else {
        // tokenURI selector for ERC-721: 0xc87b56dd + pad32(token_id)
        let token_padded = format!("{:0>64}", token_id.trim_start_matches("0x"));
        format!("c87b56dd{}", token_padded)
    };
    
    let params = serde_json::json!([
        {
            "to": contract_address,
            "data": data
        },
        "latest"
    ]);
    
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": params,
        "id": 1
    });
    
    let response = client
        .post(rpc_url)
        .send_json(&request)
        .map_err(|e| AppError::Rpc(format!("RPC request failed: {}", e)))?;
    
    let result: serde_json::Value = response.into_json()
        .map_err(|e| AppError::Parse(format!("Failed to parse response: {}", e)))?;
    
    if let Some(error) = result.get("error") {
        return Err(AppError::Rpc(error.to_string()));
    }
    
    let hex_uri = result.get("result")
        .and_then(|r| r.as_str())
        .ok_or_else(|| AppError::Rpc("No result in response".to_string()))?;
    
    if hex_uri == "0x" {
        return Ok(None);
    }
    
    // Decode hex to string (remove 0x prefix and trailing zeros)
    let hex_str = hex_uri.trim_start_matches("0x");
    let bytes = hex::decode(hex_str)
        .map_err(|e| AppError::Parse(format!("Failed to decode hex: {}", e)))?;
    
    // Find the actual string start (skip padding) and trim null bytes
    let uri = String::from_utf8(bytes)
        .map_err(|e| AppError::Parse(format!("Failed to decode URI: {}", e)))?
        .trim_start_matches('\0')
        .trim()
        .to_string();
    
    if uri.is_empty() {
        Ok(None)
    } else {
        Ok(Some(uri))
    }
}

/// Fetch NFT metadata from URI
pub fn fetch_metadata(uri: &str) -> Result<NFTMetadata, AppError> {
    let client = ureq::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build();
    
    // Replace ipfs:// with ipfs gateway
    let normalized_uri = if uri.starts_with("ipfs://") {
        format!("https://ipfs.io/ipfs/{}", &uri[7..])
    } else {
        uri.to_string()
    };
    
    let response = client
        .get(&normalized_uri)
        .call()
        .map_err(|e| AppError::Rpc(format!("Failed to fetch metadata: {}", e)))?;
    
    let metadata: NFTMetadata = response.into_json()
        .map_err(|e| AppError::Parse(format!("Failed to parse metadata: {}", e)))?;
    
    Ok(metadata)
}

/// Get owner of an ERC-721 token
pub fn get_erc721_owner(rpc_url: &str, contract_address: &str, token_id: &str) -> Result<String, AppError> {
    let client = ureq::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build();
    
    // ownerOf selector: 6352211e + pad32(token_id)
    let token_padded = format!("{:0>64}", token_id.trim_start_matches("0x"));
    let data = format!("6352211e{}", token_padded);
    
    let params = serde_json::json!([
        {
            "to": contract_address,
            "data": data
        },
        "latest"
    ]);
    
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": params,
        "id": 1
    });
    
    let response = client
        .post(rpc_url)
        .send_json(&request)
        .map_err(|e| AppError::Rpc(format!("RPC request failed: {}", e)))?;
    
    let result: serde_json::Value = response.into_json()
        .map_err(|e| AppError::Parse(format!("Failed to parse response: {}", e)))?;
    
    if let Some(error) = result.get("error") {
        return Err(AppError::Rpc(error.to_string()));
    }
    
    let owner_hex = result.get("result")
        .and_then(|r| r.as_str())
        .ok_or_else(|| AppError::Rpc("No result in response".to_string()))?;
    
    if owner_hex == "0x0000000000000000000000000000000000000000" {
        return Err(AppError::Nft("NFT has no owner or is burned".to_string()));
    }
    
    Ok(format!("0x{}", &owner_hex[26..]))
}

/// Get NFT name and symbol
pub fn get_nft_name_symbol(rpc_url: &str, contract_address: &str) -> Result<(String, String), AppError> {
    let client = ureq::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build();
    
    // name selector: 06fdde03
    let name_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [
            {
                "to": contract_address,
                "data": "06fdde03"
            },
            "latest"
        ],
        "id": 1
    });
    
    // symbol selector: 95d89b41
    let symbol_request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_call",
        "params": [
            {
                "to": contract_address,
                "data": "95d89b41"
            },
            "latest"
        ],
        "id": 2
    });
    
    let name_response = client
        .post(rpc_url)
        .send_json(&name_request)
        .map_err(|e| AppError::Rpc(format!("RPC request failed: {}", e)))?;
    
    let name_result: serde_json::Value = name_response.into_json()
        .map_err(|e| AppError::Parse(format!("Failed to parse response: {}", e)))?;
    
    let name = if let Some(result) = name_result.get("result").and_then(|r| r.as_str()) {
        if result != "0x" {
            let bytes = hex::decode(&result[2..]).unwrap_or_default();
            String::from_utf8(bytes).unwrap_or_default().trim_matches('\0').to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    
    let symbol_response = client
        .post(rpc_url)
        .send_json(&symbol_request)
        .map_err(|e| AppError::Rpc(format!("RPC request failed: {}", e)))?;
    
    let symbol_result: serde_json::Value = symbol_response.into_json()
        .map_err(|e| AppError::Parse(format!("Failed to parse response: {}", e)))?;
    
    let symbol = if let Some(result) = symbol_result.get("result").and_then(|r| r.as_str()) {
        if result != "0x" {
            let bytes = hex::decode(&result[2..]).unwrap_or_default();
            String::from_utf8(bytes).unwrap_or_default().trim_matches('\0').to_string()
        } else {
            String::new()
        }
    } else {
        String::new()
    };
    
    Ok((name, symbol))
}

/// Get all NFTs for an address (simplified - iterates through possible token IDs)
/// For production, use an indexer API like Alchemy, Moralis, or OpenSea API
pub fn get_nfts_for_address(
    rpc_url: &str,
    contract_address: &str,
    owner: &str,
    chain_id: u64,
    start_token: u64,
    count: u64,
) -> Result<Vec<NFTInfo>, AppError> {
    let contract_type = detect_contract_type(rpc_url, contract_address)?;
    let (name, symbol) = get_nft_name_symbol(rpc_url, contract_address)?;
    
    let mut nfts = Vec::new();
    
    for i in 0..count {
        let token_id = start_token + i;
        let token_id_hex = format!("0x{:x}", token_id);
        
        let balance = if contract_type == "erc1155" {
            Some(get_erc1155_balance(rpc_url, contract_address, owner, &token_id_hex)?)
        } else {
            // For ERC-721, check if owner matches
            let token_owner = match get_erc721_owner(rpc_url, contract_address, &token_id_hex) {
                Ok(o) => o,
                Err(_) => continue,
            };
            
            if token_owner.to_lowercase() != owner.to_lowercase() {
                continue;
            }
            Some("1".to_string())
        };
        
        let owner = if contract_type == "erc1155" {
            owner.to_string()
        } else {
            get_erc721_owner(rpc_url, contract_address, &token_id_hex)?
        };
        
        // Try to get metadata
        let token_uri = get_token_uri(rpc_url, contract_address, &token_id_hex, &contract_type)?;
        
        let (description, image_url, animation_url, external_url, attributes) = if let Some(uri) = token_uri {
            match fetch_metadata(&uri) {
                Ok(metadata) => (
                    metadata.description,
                    metadata.image,
                    metadata.animation_url,
                    metadata.external_url,
                    metadata.attributes,
                ),
                Err(_) => (None, None, None, None, vec![]),
            }
        } else {
            (None, None, None, None, vec![])
        };
        
        nfts.push(NFTInfo {
            token_id: token_id_hex,
            contract_address: contract_address.to_string(),
            contract_type: contract_type.clone(),
            name: if name.is_empty() { format!("{} #{}", symbol, token_id) } else { name.clone() },
            symbol: symbol.clone(),
            description,
            image_url,
            animation_url,
            external_url,
            attributes,
            balance,
            owner,
            chain_id,
        });
    }
    
    Ok(nfts)
}

/// Build NFT transfer transaction data
pub fn build_nft_transfer_data(
    contract_type: &str,
    to: &str,
    token_id: &str,
    amount: &str,
) -> Result<String, AppError> {
    let to_padded = format!("{:0>64}", &to[2..]);
    let token_padded = format!("{:0>64}", token_id.trim_start_matches("0x"));
    
    if contract_type == "erc1155" {
        // safeTransferFrom selector: f2424328 + pad32(from) + pad32(to) + pad32(token_id) + pad32(amount)
        let amount_padded = format!("{:0>64}", u64::from_str_radix(amount.trim_start_matches("0x"), 16).unwrap_or(1));
        Ok(format!("f2424328{:0>64}{}{}{}", to_padded, token_padded, amount_padded))
    } else {
        // safeTransferFrom selector for ERC-721: 42842e0e + pad32(from) + pad32(to) + pad32(token_id)
        Ok(format!("42842e0e{:0>64}{}{}", to_padded, token_padded))
    }
}

/// Estimate gas for NFT transfer
pub fn estimate_nft_transfer_gas(
    rpc_url: &str,
    contract_address: &str,
    from: &str,
    to: &str,
    token_id: &str,
    amount: &str,
    contract_type: &str,
) -> Result<String, AppError> {
    let client = ureq::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build();
    
    let data = build_nft_transfer_data(contract_type, to, token_id, amount)?;
    
    let params = serde_json::json!([{
        "from": from,
        "to": contract_address,
        "data": data
    }]);
    
    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_estimateGas",
        "params": params,
        "id": 1
    });
    
    let response = client
        .post(rpc_url)
        .send_json(&request)
        .map_err(|e| AppError::Rpc(format!("RPC request failed: {}", e)))?;
    
    let result: serde_json::Value = response.into_json()
        .map_err(|e| AppError::Parse(format!("Failed to parse response: {}", e)))?;
    
    if let Some(error) = result.get("error") {
        // Default gas estimate if estimation fails
        return Ok("100000".to_string());
    }
    
    Ok(result.get("result")
        .and_then(|r| r.as_str())
        .unwrap_or("100000")
        .to_string())
}

// Tauri commands

#[tauri::command]
pub fn get_nfts(
    rpc_url: String,
    contract_address: String,
    owner: String,
    chain_id: u64,
    start_token: u64,
    count: u64,
) -> Result<Vec<NFTInfo>, String> {
    get_nfts_for_address(&rpc_url, &contract_address, &owner, chain_id, start_token, count)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_nft_metadata(
    rpc_url: String,
    contract_address: String,
    token_id: String,
    contract_type: String,
) -> Result<NFTMetadata, String> {
    let uri = get_token_uri(&rpc_url, &contract_address, &token_id, &contract_type)
        .map_err(|e| e.to_string())?;
    
    match uri {
        Some(uri) => fetch_metadata(&uri).map_err(|e| e.to_string()),
        None => Err("No token URI found".to_string()),
    }
}

#[tauri::command]
pub fn detect_nft_type(
    rpc_url: String,
    contract_address: String,
) -> Result<String, String> {
    detect_contract_type(&rpc_url, &contract_address).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_nft_info(
    rpc_url: String,
    contract_address: String,
    token_id: String,
    owner: String,
    chain_id: u64,
) -> Result<NFTInfo, String> {
    let contract_type = detect_contract_type(&rpc_url, &contract_address)
        .map_err(|e| e.to_string())?;
    
    let (name, symbol) = get_nft_name_symbol(&rpc_url, &contract_address)
        .map_err(|e| e.to_string())?;
    
    let balance = if contract_type == "erc1155" {
        get_erc1155_balance(&rpc_url, &contract_address, &owner, &token_id)
            .map_err(|e| e.to_string())?
    } else {
        "1".to_string()
    };
    
    let actual_owner = if contract_type == "erc1155" {
        owner.clone()
    } else {
        get_erc721_owner(&rpc_url, &contract_address, &token_id)
            .map_err(|e| e.to_string())?
    };
    
    let token_uri = get_token_uri(&rpc_url, &contract_address, &token_id, &contract_type)
        .map_err(|e| e.to_string())?;
    
    let (description, image_url, animation_url, external_url, attributes) = if let Some(uri) = token_uri {
        match fetch_metadata(&uri) {
            Ok(metadata) => (
                metadata.description,
                metadata.image,
                metadata.animation_url,
                metadata.external_url,
                metadata.attributes,
            ),
            Err(_) => (None, None, None, None, vec![]),
        }
    } else {
        (None, None, None, None, vec![])
    };
    
    Ok(NFTInfo {
        token_id,
        contract_address,
        contract_type,
        name: if name.is_empty() { "Unknown NFT".to_string() } else { name },
        symbol,
        description,
        image_url,
        animation_url,
        external_url,
        attributes,
        balance: Some(balance),
        owner: actual_owner,
        chain_id,
    })
}

#[tauri::command]
pub fn build_nft_transfer(
    from: String,
    to: String,
    contract_address: String,
    token_id: String,
    contract_type: String,
    amount: Option<String>,
) -> Result<String, String> {
    let amount = amount.unwrap_or_else(|| "1".to_string());
    build_nft_transfer_data(&contract_type, &to, &token_id, &amount)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn estimate_nft_gas(
    rpc_url: String,
    contract_address: String,
    from: String,
    to: String,
    token_id: String,
    contract_type: String,
    amount: Option<String>,
) -> Result<String, String> {
    let amount = amount.unwrap_or_else(|| "1".to_string());
    estimate_nft_transfer_gas(&rpc_url, &contract_address, &from, &to, &token_id, &amount, &contract_type)
        .map_err(|e| e.to_string())
}

// NFT (ERC721) module for querying and transferring NFTs

use serde::{Deserialize, Serialize;
use std::collections::HashMap;

/// NFT metadata response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftInfo {
    pub token_id: String,
    pub owner: String,
    pub contract_address: String,
    pub token_uri: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
}

/// NFT transfer request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftTransferRequest {
    pub contract_address: String,
    pub from: String,
    pub to: String,
    pub token_id: String,
    pub private_key: String,
}

/// Result of an NFT operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NftOperationResult {
    pub success: bool,
    pub transaction_hash: Option<String>,
    pub error: Option<String>,
}

/// Get NFT balance (number of NFTs owned by an address)
/// 
/// # Arguments
/// * `rpc_url` - The RPC endpoint URL
/// * `contract_address` - The NFT contract address
/// * `owner_address` - The wallet address to query
#[tauri::command]
pub async fn get_nft_balance(
    rpc_url: String,
    contract_address: String,
    owner_address: String,
) -> Result<u64, String> {
    // ERC721 balanceOf function selector: 0x70a08231
    let data = format!(
        "0x70a08231000000000000000000000000{}",
        owner_address.trim_start_matches("0x")
    );
    
    let client = reqwest::Client::new();
    let params = serde_json::json!([
        {
            "to": contract_address,
            "data": data
        },
        "latest"
    ]);
    
    let response: serde_json::Value = client
        .post(&rpc_url)
        .json(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    
    if let Some(result) = response.get("result") {
        let hex_balance = result.as_str().ok_or("Invalid result")?;
        let balance = u64::from_str_radix(hex_balance.trim_start_matches("0x"), 16)
            .map_err(|e| e.to_string())?;
        Ok(balance)
    } else {
        Err("Failed to get NFT balance".to_string())
    }
}

/// Get NFT owner of a specific token
/// 
/// # Arguments
/// * `rpc_url` - The RPC endpoint URL
/// * `contract_address` - The NFT contract address
/// * `token_id` - The token ID to query
#[tauri::command]
pub async fn get_nft_owner(
    rpc_url: String,
    contract_address: String,
    token_id: String,
) -> Result<String, String> {
    // ERC721 ownerOf function selector: 0x6352211e
    let token_id_hex = format!("{:064x}", token_id.parse::<u256>().unwrap_or_default());
    let data = format!("0x6352211e{}", token_id_hex);
    
    let client = reqwest::Client::new();
    let params = serde_json::json!([
        {
            "to": contract_address,
            "data": data
        },
        "latest"
    ]);
    
    let response: serde_json::Value = client
        .post(&rpc_url)
        .json(&params)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    
    if let Some(result) = response.get("result") {
        let hex_owner = result.as_str().ok_or("Invalid result")?;
        let owner = format!("0x{}", &hex_owner.trim_start_matches("0x")[24..64]);
        Ok(owner)
    } else {
        Err("Failed to get NFT owner".to_string())
    }
}

/// Transfer an NFT from one address to another
/// 
/// # Arguments
/// * `request` - The NFT transfer request containing contract, from, to, token_id, and private_key
#[tauri::command]
pub async fn transfer_nft(request: NftTransferRequest) -> Result<NftOperationResult, String> {
    use crate::transaction;
    
    // Build the transfer calldata
    // safeTransferFrom(address from, address to, uint256 tokenId)
    // Function selector: 0xb88d4fde
    let token_id_hex = format!("{:064x}", request.token_id.parse::<u256>().unwrap_or_default());
    let from_address = request.from.trim_start_matches("0x");
    let to_address = request.to.trim_start_matches("0x");
    
    let calldata = format!(
        "0xb88d4fde000000000000000000000000{}000000000000000000000000{}",
        from_address, to_address
    );
    let calldata = format!("{}{}", calldata, token_id_hex);
    
    let tx = transaction::build_transaction(
        request.contract_address,
        "0x0".to_string(), // No value sent
        calldata,
        request.private_key,
    ).await?;
    
    match transaction::send_transaction(tx).await {
        Ok(tx_hash) => Ok(NftOperationResult {
            success: true,
            transaction_hash: Some(tx_hash),
            error: None,
        }),
        Err(e) => Ok(NftOperationResult {
            success: false,
            transaction_hash: None,
            error: Some(e),
        }),
    }
}

/// Get all NFTs owned by an address (requires index iteration)
/// 
/// # Arguments
/// * `rpc_url` - The RPC endpoint URL
/// * `contract_address` - The NFT contract address
/// * `owner_address` - The wallet address to query
/// * `max_count` - Maximum number of NFTs to check
#[tauri::command]
pub async fn get_owned_nfts(
    rpc_url: String,
    contract_address: String,
    owner_address: String,
    max_count: u64,
) -> Result<Vec<NftInfo>, String> {
    let balance = get_nft_balance(rpc_url.clone(), contract_address.clone(), owner_address.clone()).await?;
    
    let mut nfts = Vec::new();
    let check_count = balance.min(max_count);
    
    for i in 0..check_count {
        // Try to get token at index i
        // tokenOfOwnerByIndex(address owner, uint256 index)
        // Function selector: 0x2f745c59
        let owner_hex = format!("{:064}", owner_address.trim_start_matches("0x"));
        let index_hex = format!("{:064x}", i);
        let data = format!("0x2f745c59{}{}", owner_hex, index_hex);
        
        let client = reqwest::Client::new();
        let params = serde_json::json!([
            {
                "to": contract_address,
                "data": data
            },
            "latest"
        ]);
        
        let response: serde_json::Value = client
            .post(&rpc_url)
            .json(&params)
            .send()
            .await
            .map_err(|e| e.to_string())?
            .json()
            .await
            .map_err(|e| e.to_string())?;
        
        if let Some(result) = response.get("result") {
            if let Some(token_id) = result.as_str() {
                if !token_id.eq("0x0000000000000000000000000000000000000000000000000000000000000000") {
                    nfts.push(NftInfo {
                        token_id: format!("0x{}", &token_id[2..]),
                        owner: owner_address.clone(),
                        contract_address: contract_address.clone(),
                        token_uri: None,
                        name: None,
                        description: None,
                    });
                }
            }
        }
    }
    
    Ok(nfts)
}