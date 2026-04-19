// Wallet management and EVM chain interactions

use crate::crypto::{self, CryptoError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Wallet not found: {0}")]
    NotFound(String),
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Wallet already exists")]
    AlreadyExists,
    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletInfo {
    pub id: String,
    pub name: String,
    pub address: String,
    pub created_at: String,
}

#[derive(Debug, Clone)]
struct StoredWallet {
    info: WalletInfo,
    encrypted_private_key: Vec<u8>,
    salt: Vec<u8>,
}

pub struct WalletManager {
    wallets: HashMap<String, StoredWallet>,
}

impl WalletManager {
    pub fn new() -> Self {
        Self {
            wallets: HashMap::new(),
        }
    }

    /// Create a new wallet with generated mnemonic
    pub fn create_wallet(&mut self, name: &str, _password: &str) -> Result<WalletInfo, WalletError> {
        // Generate mnemonic
        let mnemonic = crypto::generate_mnemonic(12)?;

        // Derive private key from mnemonic
        let private_key = crypto::derive_private_key_from_mnemonic(&mnemonic, "m/44'/60'/0'/0/0")?;

        // Derive address
        let address = crypto::derive_eth_address(&private_key)?;

        // Create wallet ID
        let id = crypto::generate_id();

        let info = WalletInfo {
            id: id.clone(),
            name: name.to_string(),
            address,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        // Store wallet (simplified - in production, encrypt private key)
        let wallet = StoredWallet {
            info: info.clone(),
            encrypted_private_key: private_key,
            salt: vec![],
        };

        self.wallets.insert(id, wallet);
        Ok(info)
    }

    /// Import wallet from mnemonic phrase
    pub fn import_from_mnemonic(
        &mut self,
        name: &str,
        mnemonic: &str,
        _password: &str,
    ) -> Result<WalletInfo, WalletError> {
        if !crypto::validate_mnemonic(mnemonic) {
            return Err(WalletError::NotFound("Invalid mnemonic phrase".into()));
        }

        let private_key = crypto::derive_private_key_from_mnemonic(mnemonic, "m/44'/60'/0'/0/0")?;
        let address = crypto::derive_eth_address(&private_key)?;
        let id = crypto::generate_id();

        let info = WalletInfo {
            id: id.clone(),
            name: name.to_string(),
            address,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let wallet = StoredWallet {
            info: info.clone(),
            encrypted_private_key: private_key,
            salt: vec![],
        };

        self.wallets.insert(id, wallet);
        Ok(info)
    }

    /// Import wallet from private key
    pub fn import_from_private_key(
        &mut self,
        name: &str,
        private_key_hex: &str,
        _password: &str,
    ) -> Result<WalletInfo, WalletError> {
        let private_key = hex::decode(private_key_hex.trim_start_matches("0x"))
            .map_err(|_| WalletError::NotFound("Invalid private key format".into()))?;

        let address = crypto::derive_eth_address(&private_key)?;
        let id = crypto::generate_id();

        let info = WalletInfo {
            id: id.clone(),
            name: name.to_string(),
            address,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let wallet = StoredWallet {
            info: info.clone(),
            encrypted_private_key: private_key,
            salt: vec![],
        };

        self.wallets.insert(id, wallet);
        Ok(info)
    }

    /// List all wallets
    pub fn list_wallets(&self) -> Vec<WalletInfo> {
        self.wallets.values().map(|w| w.info.clone()).collect()
    }

    /// Get a specific wallet by ID
    pub fn get_wallet(&self, id: &str) -> Option<&WalletInfo> {
        self.wallets.get(id).map(|w| &w.info)
    }

    /// Export private key for a wallet (simplified - no actual decryption)
    pub fn export_private_key(&self, id: &str, _password: &str) -> Result<String, WalletError> {
        let wallet = self.wallets.get(id)
            .ok_or_else(|| WalletError::NotFound(id.to_string()))?;
        
        Ok(hex::encode(&wallet.encrypted_private_key))
    }

    /// Delete a wallet by ID
    pub fn delete_wallet(&mut self, id: &str) -> Result<(), WalletError> {
        if !self.wallets.contains_key(id) {
            return Err(WalletError::NotFound(format!("Wallet with ID '{}' not found", id)));
        }
        
        self.wallets.remove(id);
        Ok(())
    }
}

impl Default for WalletManager {
    fn default() -> Self {
        Self::new()
    }
}
