// Transaction signing module for EVM chains
// Uses k256 for ECDSA signing, sha3 for hashing, rlp for encoding, reqwest for RPC

use k256::ecdsa::signature::Signer;
use k256::ecdsa::SigningKey;
use primitive_types::H160;
use rlp::RlpStream;
use serde::Deserialize;
use sha3::{Digest, Keccak256};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Signing failed: {0}")]
    Signing(String),
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("Wallet not found: {0}")]
    WalletNotFound(String),
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(u64),
    #[error("Encoding error: {0}")]
    Encoding(String),
}

impl From<TransactionError> for String {
    fn from(e: TransactionError) -> Self {
        e.to_string()
    }
}

impl From<TransactionError> for serde_json::Value {
    fn from(e: TransactionError) -> Self {
        serde_json::json!({ "error": e.to_string() })
    }
}

fn get_rpc_url(chain_id: u64) -> Option<String> {
    match chain_id {
        56 => Some("https://bsc-dataseed.binance.org".to_string()),
        1 => Some("https://eth.llamarpc.com".to_string()),
        137 => Some("https://polygon-rpc.com".to_string()),
        42161 => Some("https://arb1.arbitrum.io/rpc".to_string()),
        10 => Some("https://mainnet.optimism.io".to_string()),
        43114 => Some("https://api.avax.network/ext/bc/C/rpc".to_string()),
        _ => None,
    }
}

#[derive(Debug, Deserialize)]
struct RpcResponse<T> {
    result: Option<T>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    message: String,
}

/// Parse hex string (with or without 0x prefix) to Vec<u8>
fn parse_hex(s: &str) -> Result<Vec<u8>, TransactionError> {
    let s = s.trim_start_matches("0x");
    hex::decode(s).map_err(|e| TransactionError::Encoding(format!("Invalid hex: {}", e)))
}

/// Parse hex string to H160 address
fn parse_address(s: &str) -> Result<H160, TransactionError> {
    let s = s.trim_start_matches("0x");
    if s.len() != 40 {
        return Err(TransactionError::InvalidAddress(s.to_string()));
    }
    let bytes = hex::decode(s).map_err(|e| TransactionError::InvalidAddress(e.to_string()))?;
    let arr: [u8; 20] = bytes.try_into().map_err(|_| TransactionError::InvalidAddress("wrong length".to_string()))?;
    Ok(H160::from(arr))
}

/// Encode a u64 as big-endian bytes, stripping leading zeros (but at least 1 byte)
fn encode_u64_be(v: u64) -> Vec<u8> {
    if v == 0 {
        vec![0]
    } else {
        let mut b = vec![];
        let mut n = v;
        while n > 0 {
            b.insert(0, (n & 0xff) as u8);
            n >>= 8;
        }
        b
    }
}

/// Encode a u128 as big-endian bytes, stripping leading zeros
fn encode_u128_be(v: u128) -> Vec<u8> {
    if v == 0 {
        vec![0]
    } else {
        let mut b = vec![];
        let mut n = v;
        while n > 0 {
            b.insert(0, (n & 0xff) as u8);
            n >>= 8;
        }
        b
    }
}

/// Encode a U256 (from primitive-types) as big-endian bytes via to_big_endian
fn encode_u256_be(value: &primitive_types::U256) -> Vec<u8> {
    let mut bytes = [0u8; 32];
    value.to_big_endian(&mut bytes);
    bytes.to_vec()
}

/// Convert U256 to 32-byte little-endian (for ABI encoding)
fn u256_to_le_bytes(value: &primitive_types::U256) -> [u8; 32] {
    let mut bytes = [0u8; 32];
    value.to_little_endian(&mut bytes);
    bytes
}

// Tauri command: send native token
#[tauri::command]
pub async fn send_transaction(
    wallet_id: String,
    to: String,
    amount: String,
    chain_id: u64,
) -> Result<String, String> {
    let rpc_url = get_rpc_url(chain_id)
        .ok_or_else(|| TransactionError::UnsupportedChain(chain_id).to_string())?;

    let manager = crate::wallet::WalletManager::new();
    let wallet = manager.get_wallet(&wallet_id)
        .ok_or_else(|| TransactionError::WalletNotFound(wallet_id.clone()).to_string())?;

    let private_key_hex = manager.export_private_key(&wallet_id, "")
        .map_err(|e| e.to_string())?;
    let private_key_bytes = parse_hex(&private_key_hex)?;
    let signing_key = SigningKey::from_slice(&private_key_bytes)
        .map_err(|e| format!("Invalid private key: {}", e))?;

    let to_address = parse_address(&to)?;
    let from_address = parse_address(&wallet.address)?;

    // Parse amount - supports hex (0x...) or decimal
    let amount_str = amount.trim_start_matches("0x");
    let amount_wei = if amount.starts_with("0x") {
        primitive_types::U256::from_str_radix(amount_str, 16)
            .map_err(|e| format!("Invalid amount: {}", e))?
    } else {
        primitive_types::U256::from_dec_str(amount_str)
            .map_err(|e| format!("Invalid amount: {}", e))?
    };

    let client = reqwest::Client::new();

    // 1. Get nonce
    let nonce_resp: RpcResponse<String> = client.post(&rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionCount",
            "params": [format!("0x{:040x}", from_address), "latest"],
            "id": 1
        }))
        .send().await
        .map_err(|e| format!("RPC failed: {}", e))?
        .json().await
        .map_err(|e| format!("Parse error: {}", e))?;
    if let Some(err) = nonce_resp.error { return Err(TransactionError::Rpc(err.message).to_string()); }
    let nonce = u64::from_str_radix(
        nonce_resp.result.ok_or("No nonce")?.trim_start_matches("0x"), 16
    ).map_err(|e| format!("Bad nonce: {}", e))?;

    // 2. Get gas price
    let gas_resp: RpcResponse<String> = client.post(&rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0", "method": "eth_gasPrice", "params": [], "id": 1
        }))
        .send().await
        .map_err(|e| format!("RPC failed: {}", e))?
        .json().await
        .map_err(|e| format!("Parse error: {}", e))?;
    if let Some(err) = gas_resp.error { return Err(TransactionError::Rpc(err.message).to_string()); }
    let gas_price = u128::from_str_radix(
        gas_resp.result.ok_or("No gas price")?.trim_start_matches("0x"), 16
    ).map_err(|e| format!("Bad gas price: {}", e))?;

    // 3. Build and sign
    let tx = SignedTransactionBuilder {
        nonce,
        gas_price,
        gas_limit: 21_000,
        to: to_address,
        value: amount_wei,
        data: vec![],
        chain_id,
    };
    let signed_bytes = tx.sign(&signing_key)?;

    // 4. Broadcast
    let broadcast_resp: RpcResponse<String> = client.post(&rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_sendRawTransaction",
            "params": [format!("0x{}", hex::encode(&signed_bytes))],
            "id": 1
        }))
        .send().await
        .map_err(|e| format!("RPC failed: {}", e))?
        .json().await
        .map_err(|e| format!("Parse error: {}", e))?;
    if let Some(err) = broadcast_resp.error { return Err(TransactionError::Rpc(err.message).to_string()); }
    Ok(broadcast_resp.result.ok_or("No tx hash")?)
}

/// Builder for signing EIP-155 transactions
struct SignedTransactionBuilder {
    nonce: u64,
    gas_price: u128,
    gas_limit: u64,
    to: H160,
    value: primitive_types::U256,
    data: Vec<u8>,
    chain_id: u64,
}

impl SignedTransactionBuilder {
    /// Encode transaction fields for signing (EIP-155)
    fn encode_for_signing(&self) -> Vec<u8> {
        let mut stream = RlpStream::new();
        stream.begin_list(9);
        stream.append(&encode_u64_be(self.nonce));
        stream.append(&encode_u128_be(self.gas_price));
        stream.append(&encode_u64_be(self.gas_limit));
        stream.append(&self.to_bytes());
        stream.append(&self.value_bytes());
        stream.append(&self.data);
        stream.append(&encode_u64_be(self.chain_id));
        stream.append(&encode_u64_be(0));
        stream.append(&encode_u64_be(0));
        stream.out().to_vec()
    }

    fn to_bytes(&self) -> Vec<u8> {
        self.to.as_bytes().to_vec()
    }

    fn value_bytes(&self) -> Vec<u8> {
        encode_u256_be(&self.value)
    }

    /// Sign the transaction and return RLP-encoded signed tx
    fn sign(self, signing_key: &SigningKey) -> Result<Vec<u8>, TransactionError> {
        let encoded = self.encode_for_signing();
        let hash = Keccak256::digest(&encoded);
        let signature: k256::ecdsa::Signature = signing_key.sign(&hash);
        let sig_bytes = signature.to_bytes();
        let r: [u8; 32] = sig_bytes[0..32].try_into().unwrap();
        let s: [u8; 32] = sig_bytes[32..64].try_into().unwrap();
        let v = (sig_bytes[32] & 1) as u64 + self.chain_id * 2 + 35;

        let mut stream = RlpStream::new();
        stream.begin_list(9);
        stream.append(&encode_u64_be(self.nonce));
        stream.append(&encode_u128_be(self.gas_price));
        stream.append(&encode_u64_be(self.gas_limit));
        stream.append(&self.to_bytes());
        stream.append(&self.value_bytes());
        stream.append(&self.data);
        stream.append(&encode_u64_be(v));
        stream.append(&r.as_slice());
        stream.append(&s.as_slice());
        Ok(stream.out().to_vec())
    }
}

// Tauri command: send ERC20 token
#[tauri::command]
pub async fn send_erc20_token(
    wallet_id: String,
    token_contract: String,
    to: String,
    amount: String,
    chain_id: u64,
) -> Result<String, String> {
    let rpc_url = get_rpc_url(chain_id)
        .ok_or_else(|| TransactionError::UnsupportedChain(chain_id).to_string())?;

    let manager = crate::wallet::WalletManager::new();
    let wallet = manager.get_wallet(&wallet_id)
        .ok_or_else(|| TransactionError::WalletNotFound(wallet_id.clone()).to_string())?;

    let private_key_hex = manager.export_private_key(&wallet_id, "")
        .map_err(|e| e.to_string())?;
    let private_key_bytes = parse_hex(&private_key_hex)?;
    let signing_key = SigningKey::from_slice(&private_key_bytes)
        .map_err(|e| format!("Invalid private key: {}", e))?;

    let token_address = parse_address(&token_contract)?;
    let to_address = parse_address(&to)?;
    let from_address = parse_address(&wallet.address)?;

    // Parse amount
    let amount_str = amount.trim_start_matches("0x");
    let amount_parsed: u128 = if amount.starts_with("0x") {
        u128::from_str_radix(amount_str, 16).map_err(|e| format!("Invalid amount: {}", e))?
    } else {
        amount_str.parse::<u128>().map_err(|e| format!("Invalid amount: {}", e))?
    };
    let amount_wei = primitive_types::U256::from(amount_parsed);

    // Build ERC20 transfer calldata: selector 0xa9059cbb + address (padded) + amount (uint256)
    let mut data = vec![0xa9, 0x05, 0x9c, 0xbb];
    data.extend_from_slice(&[0u8; 12]); // padding
    data.extend_from_slice(to_address.as_bytes());
    let amount_bytes = u256_to_le_bytes(&amount_wei);
    data.extend_from_slice(&amount_bytes);

    let client = reqwest::Client::new();

    // 1. Get nonce
    let nonce_resp: RpcResponse<String> = client.post(&rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionCount",
            "params": [format!("0x{:040x}", from_address), "latest"],
            "id": 1
        }))
        .send().await
        .map_err(|e| format!("RPC failed: {}", e))?
        .json().await
        .map_err(|e| format!("Parse error: {}", e))?;
    if let Some(err) = nonce_resp.error { return Err(TransactionError::Rpc(err.message).to_string()); }
    let nonce = u64::from_str_radix(
        nonce_resp.result.ok_or("No nonce")?.trim_start_matches("0x"), 16
    ).map_err(|e| format!("Bad nonce: {}", e))?;

    // 2. Get gas price
    let gas_resp: RpcResponse<String> = client.post(&rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0", "method": "eth_gasPrice", "params": [], "id": 1
        }))
        .send().await
        .map_err(|e| format!("RPC failed: {}", e))?
        .json().await
        .map_err(|e| format!("Parse error: {}", e))?;
    if let Some(err) = gas_resp.error { return Err(TransactionError::Rpc(err.message).to_string()); }
    let gas_price = u128::from_str_radix(
        gas_resp.result.ok_or("No gas price")?.trim_start_matches("0x"), 16
    ).map_err(|e| format!("Bad gas price: {}", e))?;

    // 3. Estimate gas
    let gas_est_resp: RpcResponse<String> = client.post(&rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_estimateGas",
            "params": [{
                "from": format!("0x{:040x}", from_address),
                "to": format!("0x{:040x}", token_address),
                "data": format!("0x{}", hex::encode(&data)),
            }],
            "id": 1
        }))
        .send().await
        .map_err(|e| format!("RPC failed: {}", e))?
        .json().await
        .map_err(|e| format!("Parse error: {}", e))?;
    let gas_limit = gas_est_resp.result
        .and_then(|h| u64::from_str_radix(h.trim_start_matches("0x"), 16).ok())
        .unwrap_or(100_000);

    // 4. Build and sign
    let tx = SignedTransactionBuilder {
        nonce,
        gas_price,
        gas_limit,
        to: token_address,
        value: primitive_types::U256::zero(),
        data,
        chain_id,
    };
    let signed_bytes = tx.sign(&signing_key)?;

    // 5. Broadcast
    let broadcast_resp: RpcResponse<String> = client.post(&rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_sendRawTransaction",
            "params": [format!("0x{}", hex::encode(&signed_bytes))],
            "id": 1
        }))
        .send().await
        .map_err(|e| format!("RPC failed: {}", e))?
        .json().await
        .map_err(|e| format!("Parse error: {}", e))?;
    if let Some(err) = broadcast_resp.error { return Err(TransactionError::Rpc(err.message).to_string()); }
    Ok(broadcast_resp.result.ok_or("No tx hash")?)
}

// Tauri command: get transaction receipt
#[tauri::command]
pub async fn get_transaction_receipt(
    tx_hash: String,
    chain_id: u64,
) -> Result<String, String> {
    let rpc_url = get_rpc_url(chain_id)
        .ok_or_else(|| TransactionError::UnsupportedChain(chain_id).to_string())?;

    let client = reqwest::Client::new();
    let resp: RpcResponse<serde_json::Value> = client.post(&rpc_url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionReceipt",
            "params": [tx_hash],
            "id": 1
        }))
        .send().await
        .map_err(|e| format!("RPC failed: {}", e))?
        .json().await
        .map_err(|e| format!("Parse error: {}", e))?;

    if let Some(err) = resp.error { return Err(TransactionError::Rpc(err.message).to_string()); }
    match resp.result {
        Some(r) => serde_json::to_string(&r).map_err(|e| format!("Serialize error: {}", e)),
        None => Err("Transaction not found".to_string()),
    }
}

// Tauri command: sign data (personal sign / eth_sign)
#[tauri::command]
pub fn sign_data(
    wallet_id: String,
    data: String,
) -> Result<String, String> {
    let manager = crate::wallet::WalletManager::new();
    let private_key_hex = manager.export_private_key(&wallet_id, "")
        .map_err(|e| e.to_string())?;
    let private_key_bytes = parse_hex(&private_key_hex)?;
    let signing_key = SigningKey::from_slice(&private_key_bytes)
        .map_err(|e| format!("Invalid private key: {}", e))?;
    let data_bytes = parse_hex(&data)?;

    // Personal sign: keccak256("\x19Ethereum Signed Message:\n" + len + data)
    let msg_len = data_bytes.len().to_string();
    let prefix = format!("\x19Ethereum Signed Message:\n{}", msg_len);
    let mut msg = prefix.into_bytes();
    msg.extend_from_slice(&data_bytes);
    let hash = Keccak256::digest(&msg);

    let signer = SigningKey::from(signing_key);
    let signature: k256::ecdsa::Signature = signer.sign(&hash);
    let sig_bytes = signature.to_bytes();
    let r: [u8; 32] = sig_bytes[0..32].try_into().unwrap();
    let s: [u8; 32] = sig_bytes[32..64].try_into().unwrap();
    let v = (sig_bytes[32] & 1) as u64 + 27;

    // Encode as rlp list: [v, r, s]
    let mut stream = RlpStream::new();
    stream.begin_list(3);
    stream.append(&encode_u64_be(v));
    stream.append(&r.as_slice());
    stream.append(&s.as_slice());
    let encoded = stream.out().to_vec();

    Ok(format!("0x{}", hex::encode(encoded)))
}
