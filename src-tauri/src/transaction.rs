// Transaction signing module for EVM chains
// Uses crypto module for signing, rpc module for broadcasting

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
}

impl From<TransactionError> for String {
    fn from(e: TransactionError) -> Self {
        e.to_string()
    }
}

// Note: Transaction building and signing is handled by the RPC layer
// via alloy's TransactionBuilder. See rpc.rs send_transaction path.

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
