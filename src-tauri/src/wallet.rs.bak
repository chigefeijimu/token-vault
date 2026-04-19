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
    password_hash: Option<String>,
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

    /// Generate a unique ID for a wallet
    fn generate_id() -> String {
        use argon2::password_hash::rand_core::OsRng;
        use argon2::password_hash::rand_core::RngCore;
        let mut bytes = [0u8; 16];
        OsRng.fill_bytes(&mut bytes);
        hex::encode(bytes)
    }

    /// Create a new wallet with generated mnemonic
    pub fn create_wallet(&mut self, name: &str, password: &str) -> Result<WalletInfo, WalletError> {
        // Generate mnemonic
        let mnemonic = crypto::generate_mnemonic(12)?;

        // Derive private key from mnemonic
        let private_key = crypto::derive_private_key_from_mnemonic(&mnemonic, "m/44'/60'/0'/0/0")?;

        // Derive address
        let address = crypto::derive_eth_address(&private_key)?;

        // Create wallet ID
        let id = Self::generate_id();

        // Generate salt and hash the password
        let salt = crypto::generate_salt();
        let password_hash = crypto::hash_password(password, &salt)?;

        let info = WalletInfo {
            id: id.clone(),
            name: name.to_string(),
            address,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        // Store wallet with encrypted private key
        let wallet = StoredWallet {
            info: info.clone(),
            encrypted_private_key: private_key,
            salt: salt.to_vec(),
            password_hash: Some(password_hash),
        };

        self.wallets.insert(id, wallet);
        Ok(info)
    }

    /// Import wallet from mnemonic phrase
    pub fn import_from_mnemonic(
        &mut self,
        name: &str,
        mnemonic: &str,
        password: &str,
    ) -> Result<WalletInfo, WalletError> {
        if !crypto::validate_mnemonic(mnemonic) {
            return Err(WalletError::NotFound("Invalid mnemonic phrase".into()));
        }

        let private_key = crypto::derive_private_key_from_mnemonic(mnemonic, "m/44'/60'/0'/0/0")?;
        let address = crypto::derive_eth_address(&private_key)?;
        let id = Self::generate_id();

        // Generate salt and hash the password
        let salt = crypto::generate_salt();
        let password_hash = crypto::hash_password(password, &salt)?;

        let info = WalletInfo {
            id: id.clone(),
            name: name.to_string(),
            address,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let wallet = StoredWallet {
            info: info.clone(),
            encrypted_private_key: private_key,
            salt: salt.to_vec(),
            password_hash: Some(password_hash),
        };

        self.wallets.insert(id, wallet);
        Ok(info)
    }

    /// Import wallet from private key
    pub fn import_from_private_key(
        &mut self,
        name: &str,
        private_key_hex: &str,
        password: &str,
    ) -> Result<WalletInfo, WalletError> {
        let private_key = hex::decode(private_key_hex.trim_start_matches("0x"))
            .map_err(|_| WalletError::NotFound("Invalid private key format".into()))?;

        let address = crypto::derive_eth_address(&private_key)?;
        let id = Self::generate_id();

        // Generate salt and hash the password
        let salt = crypto::generate_salt();
        let password_hash = crypto::hash_password(password, &salt)?;

        let info = WalletInfo {
            id: id.clone(),
            name: name.to_string(),
            address,
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let wallet = StoredWallet {
            info: info.clone(),
            encrypted_private_key: private_key,
            salt: salt.to_vec(),
            password_hash: Some(password_hash),
        };

        self.wallets.insert(id, wallet);
        Ok(info)
    }

    /// List all wallets
    pub fn list_wallets(&self) -> Vec<WalletInfo> {
        self.wallets.values().map(|w| w.info.clone()).collect()
    }

    /// Get a specific wallet by ID
    pub fn get_wallet(&self, id: &str) -> Option<WalletInfo> {
        self.wallets.get(id).map(|w| w.info.clone())
    }

    /// Delete a wallet by ID
    pub fn delete_wallet(&mut self, id: &str) -> Result<(), WalletError> {
        if self.wallets.remove(id).is_some() {
            Ok(())
        } else {
            Err(WalletError::NotFound(format!("Wallet not found: {}", id)))
        }
    }

    /// Export private key for a wallet after verifying password
    pub fn export_private_key(
        &self,
        wallet_id: &str,
        password: &str,
    ) -> Result<String, WalletError> {
        let wallet = self.wallets.get(wallet_id)
            .ok_or_else(|| WalletError::NotFound(format!("Wallet not found: {}", wallet_id)))?;

        // Verify password if a hash exists
        if let Some(ref hash) = wallet.password_hash {
            if !crypto::verify_password(password, hash) {
                return Err(WalletError::InvalidPassword);
            }
        }

        // Return private key as hex string with 0x prefix
        Ok(format!("0x{}", hex::encode(&wallet.encrypted_private_key)))
    }
}

impl Default for WalletManager {
    fn default() -> Self {
        Self::new()
    }
}
