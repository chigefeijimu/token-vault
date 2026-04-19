use solana_sdk::{signature::Keypair, pubkey::Pubkey};
use solana_client::rpc_client::RpcClient;
use std::str::FromStr;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolanaError {
    #[error("RPC error: {0}")]
    Rpc(String),
    #[error("Signing error: {0}")]
    Signing(String),
    #[error("Invalid address: {0}")]
    InvalidAddress(String),
    #[error("Unsupported network: {0}")]
    UnsupportedNetwork(String),
}

impl From<SolanaError> for String {
    fn from(e: SolanaError) -> Self { e.to_string() }
}

fn get_rpc_url(network: &str) -> Result<String, SolanaError> {
    match network {
        "mainnet" => Ok("https://api.mainnet-beta.solana.com".to_string()),
        "testnet" => Ok("https://api.testnet.solana.com".to_string()),
        "devnet" => Ok("https://api.devnet.solana.com".to_string()),
        _ => Err(SolanaError::UnsupportedNetwork(network.to_string())),
    }
}

/// Generate a new Solana wallet and return the base58-encoded secret key
pub fn generate_solana_wallet() -> String {
    let keypair = Keypair::new();
    bs58::encode(keypair.to_bytes()).into_string()
}

/// Get the balance of a Solana address in SOL
pub fn get_solana_balance(address: &str, network: &str) -> Result<f64, String> {
    let rpc_url = get_rpc_url(network).map_err(|e| e.to_string())?;
    let client = RpcClient::new(rpc_url);
    
    let pubkey = Pubkey::from_str(address)
        .map_err(|e| SolanaError::InvalidAddress(e.to_string()))?;
    
    let lamports = client.get_balance(&pubkey)
        .map_err(|e| SolanaError::Rpc(e.to_string()))?;
    
    Ok(lamports as f64 / 1_000_000_000.0)
}

#[tauri::command]
pub async fn solana_generate_wallet() -> Result<String, String> {
    Ok(generate_solana_wallet())
}

#[tauri::command]
pub async fn solana_get_balance(address: String, network: String) -> Result<f64, String> {
    get_solana_balance(&address, &network)
}
