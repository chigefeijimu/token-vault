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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletData {
    pub mnemonic: String,
    pub private_key: String,
    pub address: String,
}

#[derive(Debug, Clone)]
struct StoredWallet {
    info: WalletInfo,
    encrypted_private_key: Vec<u8>,
    salt: Vec<u8>,
    password_hash: Option<String>,
    mnemonic: Option<String>,
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

    fn generate_id() -> String {
        use argon2::password_hash::rand_core::OsRng;
        use argon2::password_hash::rand_core::RngCore;
        let mut bytes = [0u8; 16];
        OsRng.fill_bytes(&mut bytes);
        hex::encode(bytes)
    }

    pub fn create_wallet(&mut self, name: &str, password: &str) -> Result<WalletInfo, WalletError> {
        let mnemonic = crypto::generate_mnemonic();
        let private_key = crypto::derive_private_key_from_mnemonic(&mnemonic, "m/44'/60'/0'/0/0")?;
        let address = crypto::derive_eth_address(&private_key);
        let id = Self::generate_id();

        let salt = crypto::generate_salt();
        let password_hash = crypto::hash_password(password, &salt);

        let info = WalletInfo {
            id: id.clone(),
            name: name.to_string(),
            address: address.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
        };

        let wallet = StoredWallet {
            info: info.clone(),
            encrypted_private_key: private_key,
            salt: salt.to_vec(),
            password_hash: Some(hex::encode(&password_hash)),
            mnemonic: Some(mnemonic),
        };

        self.wallets.insert(id, wallet);
        Ok(info)
    }

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
        let address = crypto::derive_eth_address(&private_key);
        let id = Self::generate_id();

        let salt = crypto::generate_salt();
        let password_hash = crypto::hash_password(password, &salt);

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
            password_hash: Some(hex::encode(&password_hash)),
            mnemonic: Some(mnemonic.to_string()),
        };

        self.wallets.insert(id, wallet);
        Ok(info)
    }

    pub fn import_from_private_key(
        &mut self,
        name: &str,
        private_key_hex: &str,
        password: &str,
    ) -> Result<WalletInfo, WalletError> {
        let private_key = hex::decode(private_key_hex.trim_start_matches("0x"))
            .map_err(|_| WalletError::NotFound("Invalid private key format".into()))?;

        let address = crypto::derive_eth_address(&private_key);
        let id = Self::generate_id();

        let salt = crypto::generate_salt();
        let password_hash = crypto::hash_password(password, &salt);

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
            password_hash: Some(hex::encode(&password_hash)),
            mnemonic: None,
        };

        self.wallets.insert(id, wallet);
        Ok(info)
    }

    pub fn list_wallets(&self) -> Vec<WalletInfo> {
        self.wallets.values().map(|w| w.info.clone()).collect()
    }

    pub fn get_wallet(&self, id: &str) -> Option<WalletInfo> {
        self.wallets.get(id).map(|w| w.info.clone())
    }

    pub fn delete_wallet(&mut self, id: &str) -> Result<(), WalletError> {
        if self.wallets.remove(id).is_some() {
            Ok(())
        } else {
            Err(WalletError::NotFound(format!("Wallet not found: {}", id)))
        }
    }

    pub fn export_private_key(
        &self,
        wallet_id: &str,
        password: &str,
    ) -> Result<String, WalletError> {
        let wallet = self.wallets.get(wallet_id)
            .ok_or_else(|| WalletError::NotFound(format!("Wallet not found: {}", wallet_id)))?;

        if let Some(ref hash) = wallet.password_hash {
            if !crypto::verify_password(password, &wallet.salt, &hex::decode(hash).unwrap_or_default()) {
                return Err(WalletError::InvalidPassword);
            }
        }

        Ok(format!("0x{}", hex::encode(&wallet.encrypted_private_key)))
    }

    pub fn decrypt_wallet(
        &self,
        wallet_id: &str,
        password: &str,
    ) -> Result<WalletData, WalletError> {
        let wallet = self.wallets.get(wallet_id)
            .ok_or_else(|| WalletError::NotFound(format!("Wallet not found: {}", wallet_id)))?;

        if let Some(ref hash) = wallet.password_hash {
            if !crypto::verify_password(password, &wallet.salt, &hex::decode(hash).unwrap_or_default()) {
                return Err(WalletError::InvalidPassword);
            }
        }

        let mnemonic = wallet.mnemonic.clone()
            .ok_or_else(|| WalletError::NotFound("Mnemonic not available for this wallet".into()))?;

        Ok(WalletData {
            mnemonic,
            private_key: format!("0x{}", hex::encode(&wallet.encrypted_private_key)),
            address: wallet.info.address.clone(),
        })
    }
}

impl Default for WalletManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============== Tauri Commands ==============

#[tauri::command]
pub fn create_wallet(name: String, password: String) -> Result<WalletInfo, String> {
    let mut manager = WalletManager::new();
    manager.create_wallet(&name, &password).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn import_wallet(
    name: String,
    password: String,
    mnemonic: Option<String>,
    private_key: Option<String>,
) -> Result<WalletInfo, String> {
    let mut manager = WalletManager::new();
    if let Some(mnemonic) = mnemonic {
        manager.import_from_mnemonic(&name, &mnemonic, &password).map_err(|e| e.to_string())
    } else if let Some(pk) = private_key {
        manager.import_from_private_key(&name, &pk, &password).map_err(|e| e.to_string())
    } else {
        Err("Either mnemonic or private_key must be provided".to_string())
    }
}

#[tauri::command]
pub fn list_wallets() -> Result<Vec<WalletInfo>, String> {
    let manager = WalletManager::new();
    Ok(manager.list_wallets())
}

#[tauri::command]
pub fn get_wallet_info(id: String) -> Result<WalletInfo, String> {
    let manager = WalletManager::new();
    manager.get_wallet(&id).ok_or_else(|| format!("Wallet not found: {}", id))
}

#[tauri::command]
pub fn delete_wallet(id: String) -> Result<(), String> {
    let mut manager = WalletManager::new();
    manager.delete_wallet(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_private_key(id: String, password: String) -> Result<String, String> {
    let manager = WalletManager::new();
    manager.export_private_key(&id, &password).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn decrypt_wallet(id: String, password: String) -> Result<WalletData, String> {
    let manager = WalletManager::new();
    manager.decrypt_wallet(&id, &password).map_err(|e| e.to_string())
}
