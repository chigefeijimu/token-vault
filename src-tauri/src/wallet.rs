// Wallet management and EVM chain interactions

use crate::crypto::{self, CryptoError};
use crate::storage;
use crate::errors::AppError;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

lazy_static! {
    static ref WALLET_MANAGER: Mutex<WalletManager> = Mutex::new(WalletManager::new());
}

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
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateWalletResult {
    pub id: String,
    pub name: String,
    pub address: String,
    pub created_at: String,
    pub mnemonic: String,
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

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    fn generate_id() -> String {
        use argon2::password_hash::rand_core::OsRng;
        use argon2::password_hash::rand_core::RngCore;
        let mut bytes = [0u8; 16];
        OsRng.fill_bytes(&mut bytes);
        hex::encode(bytes)
    }

    /// Persist a wallet to SQLite storage
    fn persist_wallet(&self, wallet: &StoredWallet, encrypted_mnemonic: Option<&str>) {
        let wallet_info = WalletInfo {
            id: wallet.info.id.clone(),
            name: wallet.info.name.clone(),
            address: wallet.info.address.clone(),
            created_at: wallet.info.created_at,
        };
        let encrypted_pk = format!("0x{}", hex::encode(&wallet.encrypted_private_key));
        let salt_hex = hex::encode(&wallet.salt);

        if let Ok(guard) = storage::get_storage() {
            if let Some(storage) = guard.as_ref() {
                let _ = storage.save_wallet(
                    &wallet_info,
                    encrypted_mnemonic,
                    Some(&encrypted_pk),
                    &salt_hex,
                );
            }
        }
    }

    pub fn create_wallet(&mut self, name: &str, password: &str) -> Result<WalletInfo, WalletError> {
        let mnemonic = crypto::generate_mnemonic();
        let private_key =
            crypto::derive_private_key_from_mnemonic(&mnemonic, "m/44'/60'/0'/0/0")?;
        let address = crypto::derive_eth_address(&private_key);
        let id = Self::generate_id();
        let created_at = Self::now();

        let salt = crypto::generate_salt();
        let password_hash = crypto::hash_password(password, &salt);

        let info = WalletInfo {
            id: id.clone(),
            name: name.to_string(),
            address: address.clone(),
            created_at,
        };

        let wallet = StoredWallet {
            info: info.clone(),
            encrypted_private_key: private_key,
            salt: salt.to_vec(),
            password_hash: Some(hex::encode(&password_hash)),
            mnemonic: Some(mnemonic.clone()),
        };

        self.wallets.insert(id.clone(), wallet);

        // Persist to SQLite
        if let Some(stored) = self.wallets.get(&id) {
            self.persist_wallet(stored, Some(&mnemonic));
        }

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
        let private_key =
            crypto::derive_private_key_from_mnemonic(mnemonic, "m/44'/60'/0'/0/0")?;
        let address = crypto::derive_eth_address(&private_key);
        let id = Self::generate_id();
        let created_at = Self::now();

        let salt = crypto::generate_salt();
        let password_hash = crypto::hash_password(password, &salt);

        let info = WalletInfo {
            id: id.clone(),
            name: name.to_string(),
            address,
            created_at,
        };

        let wallet = StoredWallet {
            info: info.clone(),
            encrypted_private_key: private_key,
            salt: salt.to_vec(),
            password_hash: Some(hex::encode(&password_hash)),
            mnemonic: Some(mnemonic.to_string()),
        };

        self.wallets.insert(id.clone(), wallet);

        if let Some(stored) = self.wallets.get(&id) {
            self.persist_wallet(stored, Some(mnemonic));
        }

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
        let created_at = Self::now();

        let salt = crypto::generate_salt();
        let password_hash = crypto::hash_password(password, &salt);

        let info = WalletInfo {
            id: id.clone(),
            name: name.to_string(),
            address,
            created_at,
        };

        let wallet = StoredWallet {
            info: info.clone(),
            encrypted_private_key: private_key,
            salt: salt.to_vec(),
            password_hash: Some(hex::encode(&password_hash)),
            mnemonic: None,
        };

        self.wallets.insert(id.clone(), wallet);

        if let Some(stored) = self.wallets.get(&id) {
            self.persist_wallet(stored, None);
        }

        Ok(info)
    }

    pub fn list_wallets(&self) -> Vec<WalletInfo> {
        self.wallets
            .values()
            .map(|w| w.info.clone())
            .collect()
    }

    pub fn get_wallet(&self, id: &str) -> Option<WalletInfo> {
        self.wallets.get(id).map(|w| w.info.clone())
    }

    pub fn delete_wallet(&mut self, id: &str) -> Result<(), WalletError> {
        if self.wallets.remove(id).is_some() {
            // Remove from SQLite
            if let Ok(guard) = storage::get_storage() {
                if let Some(s) = guard.as_ref() {
                    let _ = s.delete_wallet(id);
                }
            }
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
        let wallet = self
            .wallets
            .get(wallet_id)
            .ok_or_else(|| WalletError::NotFound(format!("Wallet not found: {}", wallet_id)))?;

        if let Some(ref hash) = wallet.password_hash {
            if !crypto::verify_password(
                password,
                &wallet.salt,
                &hex::decode(hash).unwrap_or_default(),
            ) {
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
        let wallet = self
            .wallets
            .get(wallet_id)
            .ok_or_else(|| WalletError::NotFound(format!("Wallet not found: {}", wallet_id)))?;

        if let Some(ref hash) = wallet.password_hash {
            if !crypto::verify_password(
                password,
                &wallet.salt,
                &hex::decode(hash).unwrap_or_default(),
            ) {
                return Err(WalletError::InvalidPassword);
            }
        }

        let mnemonic = wallet
            .mnemonic
            .clone()
            .ok_or_else(|| WalletError::NotFound("Mnemonic not available for this wallet".into()))?;

        Ok(WalletData {
            mnemonic,
            private_key: format!("0x{}", hex::encode(&wallet.encrypted_private_key)),
            address: wallet.info.address.clone(),
        })
    }

    /// Load all wallets from SQLite into memory at startup
    pub fn load_from_storage(&mut self) {
        if let Ok(guard) = storage::get_storage() {
            if let Some(s) = guard.as_ref() {
                if let Ok(metas) = s.get_all_wallets() {
                    for meta in metas {
                        if let Ok(encrypted) = s.load_wallet(&meta.id) {
                            // Reconstruct in-memory wallet from encrypted storage
                            // We need to re-derive the private key from stored data
                            // For now, just restore wallet metadata; private key material
                            // stays in memory after first unlock
                            let info = WalletInfo {
                                id: meta.id.clone(),
                                name: meta.name,
                                address: meta.address,
                                created_at: meta.created_at,
                            };
                            let stored = StoredWallet {
                                info,
                                encrypted_private_key: Vec::new(), // placeholder until unlock
                                salt: hex::decode(&encrypted.salt).unwrap_or_default(),
                                password_hash: None,
                                mnemonic: None,
                            };
                            self.wallets.insert(meta.id, stored);
                        }
                    }
                }
            }
        }
    }
}

impl Default for WalletManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============== Tauri Commands ==============

#[tauri::command]
pub fn create_wallet(name: String, password: String) -> Result<CreateWalletResult, String> {
    let mut manager = WALLET_MANAGER.lock().unwrap();
    let info = manager
        .create_wallet(&name, &password)
        .map_err(|e| e.to_string())?;

    // Retrieve the mnemonic from the stored wallet
    let wallet = manager
        .wallets
        .get(&info.id)
        .ok_or_else(|| "Wallet not found after creation".to_string())?;
    let mnemonic = wallet
        .mnemonic
        .clone()
        .ok_or_else(|| "Mnemonic not available".to_string())?;

    // Format created_at as RFC3339 for frontend compatibility
    let created_at = chrono::DateTime::from_timestamp(info.created_at as i64, 0)
        .map(|dt| dt.to_rfc3339())
        .unwrap_or_default();

    Ok(CreateWalletResult {
        id: info.id,
        name: info.name,
        address: info.address,
        created_at,
        mnemonic,
    })
}

#[tauri::command]
pub fn import_wallet(
    name: String,
    password: String,
    mnemonic: Option<String>,
    private_key: Option<String>,
) -> Result<WalletInfo, String> {
    let mut manager = WALLET_MANAGER.lock().unwrap();
    if let Some(mnemonic) = mnemonic {
        manager
            .import_from_mnemonic(&name, &mnemonic, &password)
            .map_err(|e| e.to_string())
    } else if let Some(pk) = private_key {
        manager
            .import_from_private_key(&name, &pk, &password)
            .map_err(|e| e.to_string())
    } else {
        Err("Either mnemonic or private_key must be provided".to_string())
    }
}

#[tauri::command]
pub fn list_wallets() -> Result<Vec<WalletInfo>, String> {
    let manager = WALLET_MANAGER.lock().unwrap();
    Ok(manager.list_wallets())
}

#[tauri::command]
pub fn get_wallet_info(id: String) -> Result<WalletInfo, String> {
    let manager = WALLET_MANAGER.lock().unwrap();
    manager
        .get_wallet(&id)
        .ok_or_else(|| format!("Wallet not found: {}", id))
}

#[tauri::command]
pub fn delete_wallet(id: String) -> Result<(), String> {
    let mut manager = WALLET_MANAGER.lock().unwrap();
    manager.delete_wallet(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_private_key(id: String, password: String) -> Result<String, String> {
    let manager = WALLET_MANAGER.lock().unwrap();
    manager
        .export_private_key(&id, &password)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn decrypt_wallet(id: String, password: String) -> Result<WalletData, String> {
    let manager = WALLET_MANAGER.lock().unwrap();
    manager
        .decrypt_wallet(&id, &password)
        .map_err(|e| e.to_string())
}
