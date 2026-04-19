// Transaction signing module for EVM chains
// Uses crypto module for signing, rpc module for broadcasting

use crate::crypto;
use thiserror::Error;
use ethers::types::U256;

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
}

impl From<TransactionError> for String {
    fn from(e: TransactionError) -> Self {
        e.to_string()
    }
}

// Build transaction data for a simple ETH transfer
// Returns the unsigned transaction data ready for signing
pub fn build_eth_transaction(
    to: &str,
    amount_wei: &str,
    nonce: u64,
    gas_price: u64,
    chain_id: u64,
) -> Result<String, TransactionError> {
    // Simple EIP-155 transaction encoding
    // RLP encode: [nonce, gasPrice, gasLimit, to, value, data, chainId, 0, 0]
    let to_addr = to.trim_start_matches("0x");
    if to_addr.len() != 40 {
        return Err(TransactionError::InvalidAddress(to.to_string()));
    }

    let value = U256::from_dec_str(amount_wei)
        .map_err(|_| TransactionError::Signing("Invalid amount".to_string()))?;

    let gas_price_u256 = U256::from(gas_price);
    let nonce_u256 = U256::from(nonce);

    // Build RLP-like encoding for signing
    // For EIP-155: keccak256(RLP([nonce, gasPrice, gasLimit, to, value, data, chainId, 0, 0]))
    // We'll encode the transaction fields for signing
    let mut encoded = Vec::new();
    encode_u256(&mut encoded, &nonce_u256);
    encode_u256(&mut encoded, &gas_price_u256);
    encode_u256(&mut encoded, &U256::from(21000u64)); // gas limit
    encode_bytes(&mut encoded, &hex::decode(to_addr).map_err(|_| TransactionError::InvalidAddress(to.to_string()))?);
    encode_u256(&mut encoded, &value);
    encode_bytes(&mut encoded, &[]); // data
    encode_u256(&mut encoded, &U256::from(chain_id));
    encode_bytes(&mut encoded, &[]); // 0
    encode_bytes(&mut encoded, &[]); // 0

    Ok(hex::encode(encoded))
}

// Helper: encode U256 as RLP
fn encode_u256(output: &mut Vec<u8>, value: &U256) {
    let mut bytes = Vec::new();
    if value.is_zero() {
        bytes.push(0x80); // empty byte
    } else {
        let v = *value;
        let mut buf = [0u8; 32];
        v.to_big_endian(&mut buf);
        let mut start = 0;
        while start < 32 && buf[start] == 0 { start += 1; }
        bytes.extend_from_slice(&buf[start..]);
    }
    encode_bytes(output, &bytes);
}

// Helper: encode bytes as RLP
fn encode_bytes(output: &mut Vec<u8>, bytes: &[u8]) {
    if bytes.len() == 1 && bytes[0] < 0x80 {
        output.push(bytes[0]);
    } else if bytes.len() <= 55 {
        output.push(0x80 + bytes.len() as u8);
        output.extend_from_slice(bytes);
    } else {
        let len_bytes = bytes.len().to_be_bytes();
        let non_zero = len_bytes.iter().position(|&b| b != 0).unwrap_or(0);
        output.push(0xb7 + (len_bytes.len() - non_zero) as u8);
        output.extend_from_slice(&len_bytes[non_zero..]);
        output.extend_from_slice(bytes);
    }
}

// Sign a transaction with a private key
pub fn sign_transaction_data(
    tx_data: &str,
    private_key: &str,
) -> Result<String, TransactionError> {
    let pk_hex = private_key.trim_start_matches("0x");
    let tx_hex = tx_data.trim_start_matches("0x");

    let tx_bytes = hex::decode(tx_hex)
        .map_err(|e| TransactionError::Signing(format!("Invalid tx data: {}", e)))?;
    let pk_bytes = hex::decode(pk_hex)
        .map_err(|e| TransactionError::Signing(format!("Invalid private key: {}", e)))?;

    let signature = crypto::sign_data(
        &hex::encode(&tx_bytes),
        &hex::encode(&pk_bytes),
    ).map_err(|e| TransactionError::Signing(format!("Signing failed: {}", e)))?;

    Ok(signature)
}

// Encode transaction for broadcast (post-signing)
pub fn encode_signed_transaction(
    _tx_data: &str,
    _signature: &str,
    _chain_id: u64,
) -> Result<String, TransactionError> {
    // For now, return mock encoded transaction
    // Real implementation would RLP encode [tx_data, signature, chain_id]
    let encoded = format!("0x{}", _tx_data);
    Ok(encoded)
}

// Tauri command: send ETH transaction
#[tauri::command]
pub async fn send_transaction(
    wallet_id: String,
    to: String,
    _amount: String,
    _chain_id: u64,
) -> Result<String, String> {
    // Get wallet and export private key
    let manager = crate::wallet::WalletManager::new();
    let _wallet = manager.get_wallet(&wallet_id)
        .ok_or_else(|| "Wallet not found".to_string())?;

    let _private_key = manager.export_private_key(&wallet_id, "")
        .map_err(|e| e.to_string())?;

    let to_addr = to.trim_start_matches("0x");
    if to_addr.len() != 40 {
        return Err("Invalid recipient address".to_string());
    }

    // For now, return mock tx hash - real implementation would:
    // 1. Get nonce from RPC
    // 2. Get gas price from RPC
    // 3. Build transaction data
    // 4. Sign with private key
    // 5. Broadcast via RPC
    let tx_hash = format!("0x{:064x}", rand::random::<u128>());
    Ok(tx_hash)
}

// Tauri command: send ERC20 token
#[tauri::command]
pub async fn send_erc20_token(
    _wallet_id: String,
    _token_contract: String,
    _to: String,
    _amount: String,
    _chain_id: u64,
) -> Result<String, String> {
    // Mock tx hash
    let tx_hash = format!("0x{:064x}", rand::random::<u128>());
    Ok(tx_hash)
}
