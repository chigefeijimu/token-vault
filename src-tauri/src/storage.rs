// Persistent storage module for wallet data, transactions, and settings
// Uses JSON file storage with optional encryption for sensitive data

use crate::errors::AppError;
use crate::wallet::WalletInfo;
use crate::crypto::{self, CryptoError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Encryption error: {0}")]
    Encryption(String),
    #[error("Data directory not initialized")]
    NotInitialized,
    #[error("Wallet not found: {0}")]
    WalletNotFound(String),
    #[error("Transaction not found: {0}")]
    TransactionNotFound(String),
}

impl From<StorageError> for AppError {
    fn from(err: StorageError) -> Self {
        match err {
            StorageError::Io(e) => AppError::Storage(1000, e.to_string()),
            StorageError::Serialization(e) => AppError::Storage(1001, e.to_string()),
            StorageError::Encryption(e) => AppError::Storage(1002, e),
            StorageError::NotInitialized => AppError::Storage(1003, "Data directory not initialized".to_string()),
            StorageError::WalletNotFound(id) => AppError::Storage(1004, format!("Wallet not found: {}", id)),
            StorageError::TransactionNotFound(hash) => AppError::Storage(1005, format!("Transaction not found: {}", hash)),
        }
    }
}

/// Transaction history record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRecord {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: String,
    pub timestamp: u64,
    pub block_number: String,
    pub block_hash: String,
    pub chain_id: u64,
    pub status: String,
    pub gas_used: Option<String>,
    pub gas_price: Option<String>,
    pub nonce: Option<u64>,
    pub input: Option<String>,
    pub wallet_id: String,
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub theme: String,
    pub currency: String,
    pub language: String,
    pub auto_lock_minutes: u32,
    pub backup_reminder: bool,
    pub rpc_timeout_seconds: u32,
    pub default_chain_id: u64,
    pub hide_balances: bool,
    pub max_gas_price_gwei: Option<f64>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            currency: "USD".to_string(),
            language: "en".to_string(),
            auto_lock_minutes: 5,
            backup_reminder: true,
            rpc_timeout_seconds: 30,
            default_chain_id: 1,
            hide_balances: false,
            max_gas_price_gwei: None,
        }
    }
}

/// Encrypted wallet storage format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedWalletData {
    pub id: String,
    pub name: String,
    pub address: String,
    pub created_at: u64,
    pub encrypted_mnemonic: Option<String>,
    pub encrypted_private_key: Option<String>,
    pub salt: String,
}

/// Wallet metadata (non-sensitive)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletMetadata {
    pub id: String,
    pub name: String,
    pub address: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub last_used: Option<u64>,
    pub chain_ids: Vec<u64>,
}

/// Storage metadata/index
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageIndex {
    pub version: String,
    pub last_updated: u64,
    pub wallets: Vec<WalletMetadata>,
}

impl Default for StorageIndex {
    fn default() -> Self {
        Self {
            version: "1.0.0".to_string(),
            last_updated: chrono_timestamp(),
            wallets: Vec::new(),
        }
    }
}

/// Main storage service
pub struct StorageService {
    data_dir: PathBuf,
    wallets_file: PathBuf,
    transactions_file: PathBuf,
    settings_file: PathBuf,
    index_file: PathBuf,
    master_key: Option<Vec<u8>>,
}

impl StorageService {
    /// Create a new storage service with the specified data directory
    pub fn new(data_dir: PathBuf) -> Result<Self, StorageError> {
        let storage = Self {
            wallets_file: data_dir.join("wallets.json"),
            transactions_file: data_dir.join("transactions.json"),
            settings_file: data_dir.join("settings.json"),
            index_file: data_dir.join("index.json"),
            data_dir,
            master_key: None,
        };
        Ok(storage)
    }

    /// Initialize the storage directories and files
    pub fn initialize(&self) -> Result<(), StorageError> {
        fs::create_dir_all(&self.data_dir)?;
        
        // Initialize index if not exists
        if !self.index_file.exists() {
            let index = StorageIndex::default();
            fs::write(&self.index_file, serde_json::to_string_pretty(&index)?)?;
        }
        
        // Initialize wallets file if not exists
        if !self.wallets_file.exists() {
            fs::write(&self.wallets_file, "[]")?;
        }
        
        // Initialize transactions file if not exists
        if !self.transactions_file.exists() {
            fs::write(&self.transactions_file, "[]")?;
        }
        
        // Initialize settings file if not exists
        if !self.settings_file.exists() {
            let settings = AppSettings::default();
            fs::write(&self.settings_file, serde_json::to_string_pretty(&settings)?)?;
        }
        
        Ok(())
    }

    /// Set the master encryption key for sensitive data
    pub fn set_master_key(&mut self, key: Vec<u8>) {
        self.master_key = Some(key);
    }

    /// Get the data directory path
    pub fn get_data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    // ==================== Wallet Operations ====================

    /// Save wallet data (encrypted)
    pub fn save_wallet(&self, wallet: &WalletInfo, encrypted_mnemonic: Option<&str>, encrypted_pk: Option<&str>, salt: &str) -> Result<(), StorageError> {
        // Load existing wallets
        let mut wallets: Vec<EncryptedWalletData> = if self.wallets_file.exists() {
            let content = fs::read_to_string(&self.wallets_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        // Check if wallet already exists
        let existing_idx = wallets.iter().position(|w| w.id == wallet.id);

        let encrypted_wallet = EncryptedWalletData {
            id: wallet.id.clone(),
            name: wallet.name.clone(),
            address: wallet.address.clone(),
            created_at: wallet.created_at,
            encrypted_mnemonic: encrypted_mnemonic.map(String::from),
            encrypted_private_key: encrypted_pk.map(String::from),
            salt: salt.to_string(),
        };

        if let Some(idx) = existing_idx {
            wallets[idx] = encrypted_wallet;
        } else {
            wallets.push(encrypted_wallet);
        }

        // Save wallets
        fs::write(&self.wallets_file, serde_json::to_string_pretty(&wallets)?)?;

        // Update index
        self.update_wallet_index(wallet)?;

        Ok(())
    }

    /// Load wallet data
    pub fn load_wallet(&self, wallet_id: &str) -> Result<EncryptedWalletData, StorageError> {
        let content = fs::read_to_string(&self.wallets_file)?;
        let wallets: Vec<EncryptedWalletData> = serde_json::from_str(&content)?;

        wallets
            .into_iter()
            .find(|w| w.id == wallet_id)
            .ok_or_else(|| StorageError::WalletNotFound(wallet_id.to_string()))
    }

    /// Get all wallet metadata (non-sensitive)
    pub fn get_all_wallets(&self) -> Result<Vec<WalletMetadata>, StorageError> {
        let index: StorageIndex = if self.index_file.exists() {
            let content = fs::read_to_string(&self.index_file)?;
            serde_json::from_str(&content)?
        } else {
            StorageIndex::default()
        };
        Ok(index.wallets)
    }

    /// Delete a wallet
    pub fn delete_wallet(&self, wallet_id: &str) -> Result<(), StorageError> {
        // Remove from wallets file
        let content = fs::read_to_string(&self.wallets_file)?;
        let mut wallets: Vec<EncryptedWalletData> = serde_json::from_str(&content)?;
        wallets.retain(|w| w.id != wallet_id);
        fs::write(&self.wallets_file, serde_json::to_string_pretty(&wallets)?)?;

        // Remove associated transactions
        self.delete_wallet_transactions(wallet_id)?;

        // Update index
        let mut index: StorageIndex = if self.index_file.exists() {
            let content = fs::read_to_string(&self.index_file)?;
            serde_json::from_str(&content)?
        } else {
            StorageIndex::default()
        };
        index.wallets.retain(|w| w.id != wallet_id);
        index.last_updated = chrono_timestamp();
        fs::write(&self.index_file, serde_json::to_string_pretty(&index)?)?;

        Ok(())
    }

    fn update_wallet_index(&self, wallet: &WalletInfo) -> Result<(), StorageError> {
        let mut index: StorageIndex = if self.index_file.exists() {
            let content = fs::read_to_string(&self.index_file)?;
            serde_json::from_str(&content)?
        } else {
            StorageIndex::default()
        };

        let now = chrono_timestamp();
        
        if let Some(existing) = index.wallets.iter_mut().find(|w| w.id == wallet.id) {
            existing.name = wallet.name.clone();
            existing.updated_at = now;
        } else {
            index.wallets.push(WalletMetadata {
                id: wallet.id.clone(),
                name: wallet.name.clone(),
                address: wallet.address.clone(),
                created_at: wallet.created_at,
                updated_at: now,
                last_used: None,
                chain_ids: vec![],
            });
        }

        index.last_updated = now;
        fs::write(&self.index_file, serde_json::to_string_pretty(&index)?)?;
        Ok(())
    }

    /// Update wallet last used timestamp
    pub fn update_wallet_last_used(&self, wallet_id: &str) -> Result<(), StorageError> {
        let mut index: StorageIndex = if self.index_file.exists() {
            let content = fs::read_to_string(&self.index_file)?;
            serde_json::from_str(&content)?
        } else {
            StorageIndex::default()
        };

        if let Some(wallet) = index.wallets.iter_mut().find(|w| w.id == wallet_id) {
            wallet.last_used = Some(chrono_timestamp());
        }

        fs::write(&self.index_file, serde_json::to_string_pretty(&index)?)?;
        Ok(())
    }

    // ==================== Transaction Operations ====================

    /// Save a transaction
    pub fn save_transaction(&self, tx: &TransactionRecord) -> Result<(), StorageError> {
        let mut transactions: Vec<TransactionRecord> = if self.transactions_file.exists() {
            let content = fs::read_to_string(&self.transactions_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        // Check if transaction already exists (update) or is new
        if let Some(existing_idx) = transactions.iter().position(|t| t.hash == tx.hash) {
            transactions[existing_idx] = tx.clone();
        } else {
            transactions.push(tx.clone());
        }

        // Sort by timestamp descending
        transactions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        fs::write(&self.transactions_file, serde_json::to_string_pretty(&transactions)?)?;
        Ok(())
    }

    /// Get transactions for a wallet
    pub fn get_wallet_transactions(&self, wallet_id: &str) -> Result<Vec<TransactionRecord>, StorageError> {
        let transactions: Vec<TransactionRecord> = if self.transactions_file.exists() {
            let content = fs::read_to_string(&self.transactions_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(transactions.into_iter().filter(|t| t.wallet_id == wallet_id).collect())
    }

    /// Get transactions for a specific chain
    pub fn get_chain_transactions(&self, chain_id: u64) -> Result<Vec<TransactionRecord>, StorageError> {
        let transactions: Vec<TransactionRecord> = if self.transactions_file.exists() {
            let content = fs::read_to_string(&self.transactions_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(transactions.into_iter().filter(|t| t.chain_id == chain_id).collect())
    }

    /// Get all transactions
    pub fn get_all_transactions(&self) -> Result<Vec<TransactionRecord>, StorageError> {
        let transactions: Vec<TransactionRecord> = if self.transactions_file.exists() {
            let content = fs::read_to_string(&self.transactions_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        Ok(transactions)
    }

    /// Update transaction status
    pub fn update_transaction_status(&self, hash: &str, status: &str, block_number: Option<&str>, block_hash: Option<&str>) -> Result<(), StorageError> {
        let mut transactions: Vec<TransactionRecord> = if self.transactions_file.exists() {
            let content = fs::read_to_string(&self.transactions_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            return Err(StorageError::TransactionNotFound(hash.to_string()));
        };

        if let Some(tx) = transactions.iter_mut().find(|t| t.hash == hash) {
            tx.status = status.to_string();
            if let Some(bn) = block_number {
                tx.block_number = bn.to_string();
            }
            if let Some(bh) = block_hash {
                tx.block_hash = bh.to_string();
            }
        } else {
            return Err(StorageError::TransactionNotFound(hash.to_string()));
        }

        fs::write(&self.transactions_file, serde_json::to_string_pretty(&transactions)?)?;
        Ok(())
    }

    /// Delete transactions for a wallet
    fn delete_wallet_transactions(&self, wallet_id: &str) -> Result<(), StorageError> {
        let transactions: Vec<TransactionRecord> = if self.transactions_file.exists() {
            let content = fs::read_to_string(&self.transactions_file)?;
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            Vec::new()
        };

        let filtered: Vec<TransactionRecord> = transactions.into_iter().filter(|t| t.wallet_id != wallet_id).collect();
        fs::write(&self.transactions_file, serde_json::to_string_pretty(&filtered)?)?;
        Ok(())
    }

    // ==================== Settings Operations ====================

    /// Load application settings
    pub fn load_settings(&self) -> Result<AppSettings, StorageError> {
        if !self.settings_file.exists() {
            return Ok(AppSettings::default());
        }
        
        let content = fs::read_to_string(&self.settings_file)?;
        let settings: AppSettings = serde_json::from_str(&content)?;
        Ok(settings)
    }

    /// Save application settings
    pub fn save_settings(&self, settings: &AppSettings) -> Result<(), StorageError> {
        fs::write(&self.settings_file, serde_json::to_string_pretty(&settings)?)?;
        Ok(())
    }

    /// Update a single setting
    pub fn update_setting(&self, key: &str, value: serde_json::Value) -> Result<(), StorageError> {
        let mut settings = self.load_settings()?;
        
        match key {
            "theme" => if let Some(v) = value.as_str() { settings.theme = v.to_string(); },
            "currency" => if let Some(v) = value.as_str() { settings.currency = v.to_string(); },
            "language" => if let Some(v) = value.as_str() { settings.language = v.to_string(); },
            "auto_lock_minutes" => if let Some(v) = value.as_u64() { settings.auto_lock_minutes = v as u32; },
            "backup_reminder" => if let Some(v) = value.as_bool() { settings.backup_reminder = v; },
            "rpc_timeout_seconds" => if let Some(v) = value.as_u64() { settings.rpc_timeout_seconds = v as u32; },
            "default_chain_id" => if let Some(v) = value.as_u64() { settings.default_chain_id = v as u64; },
            "hide_balances" => if let Some(v) = value.as_bool() { settings.hide_balances = v; },
            "max_gas_price_gwei" => settings.max_gas_price_gwei = value.as_f64(),
            _ => return Err(StorageError::Encryption(format!("Unknown setting: {}", key))),
        }

        self.save_settings(&settings)?;
        Ok(())
    }

    // ==================== Backup/Export Operations ====================

    /// Export all data as encrypted backup
    pub fn create_backup(&self, encryption_key: &[u8]) -> Result<Vec<u8>, StorageError> {
        #[derive(Serialize)]
        struct BackupData {
            version: String,
            timestamp: u64,
            index: StorageIndex,
            wallets: Vec<EncryptedWalletData>,
            transactions: Vec<TransactionRecord>,
            settings: AppSettings,
        }

        let index: StorageIndex = if self.index_file.exists() {
            serde_json::from_str(&fs::read_to_string(&self.index_file)?)?
        } else {
            StorageIndex::default()
        };

        let wallets: Vec<EncryptedWalletData> = if self.wallets_file.exists() {
            serde_json::from_str(&fs::read_to_string(&self.wallets_file)?)?
        } else {
            Vec::new()
        };

        let transactions: Vec<TransactionRecord> = if self.transactions_file.exists() {
            serde_json::from_str(&fs::read_to_string(&self.transactions_file)?)?
        } else {
            Vec::new()
        };

        let settings = self.load_settings()?;

        let backup = BackupData {
            version: "1.0.0".to_string(),
            timestamp: chrono_timestamp(),
            index,
            wallets,
            transactions,
            settings,
        };

        let json = serde_json::to_vec(&backup)
            .map_err(|e| StorageError::Serialization(e))?;

        // Encrypt the backup
        let encrypted = crypto::encrypt_data(&json, encryption_key)
            .map_err(|e| StorageError::Encryption(e.to_string()))?;

        Ok(encrypted)
    }

    /// Restore from encrypted backup
    pub fn restore_backup(&self, encrypted_data: &[u8], encryption_key: &[u8]) -> Result<(), StorageError> {
        let decrypted = crypto::decrypt_data(encrypted_data, encryption_key)
            .map_err(|e| StorageError::Encryption(e.to_string()))?;

        #[derive(Deserialize)]
        struct BackupData {
            version: String,
            index: StorageIndex,
            wallets: Vec<EncryptedWalletData>,
            transactions: Vec<TransactionRecord>,
            settings: AppSettings,
        }

        let backup: BackupData = serde_json::from_slice(&decrypted)
            .map_err(|e| StorageError::Serialization(e))?;

        // Restore all data
        fs::write(&self.index_file, serde_json::to_string_pretty(&backup.index)?)?;
        fs::write(&self.wallets_file, serde_json::to_string_pretty(&backup.wallets)?)?;
        fs::write(&self.transactions_file, serde_json::to_string_pretty(&backup.transactions)?)?;
        fs::write(&self.settings_file, serde_json::to_string_pretty(&backup.settings)?)?;

        Ok(())
    }

    /// Clear all data (factory reset)
    pub fn clear_all_data(&self) -> Result<(), StorageError> {
        if self.wallets_file.exists() {
            fs::remove_file(&self.wallets_file)?;
        }
        if self.transactions_file.exists() {
            fs::remove_file(&self.transactions_file)?;
        }
        if self.settings_file.exists() {
            fs::remove_file(&self.settings_file)?;
        }
        if self.index_file.exists() {
            fs::remove_file(&self.index_file)?;
        }

        // Re-initialize with defaults
        self.initialize()?;
        Ok(())
    }
}

/// Get current timestamp in seconds
fn chrono_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// Global storage instance with mutex for thread safety
lazy_static::lazy_static! {
    pub static ref STORAGE: Mutex<Option<StorageService>> = Mutex::new(None);
}

/// Initialize the global storage service
pub fn init_storage(data_dir: PathBuf) -> Result<(), StorageError> {
    let storage = StorageService::new(data_dir)?;
    storage.initialize()?;
    
    let mut global = STORAGE.lock().unwrap();
    *global = Some(storage);
    Ok(())
}

/// Get the global storage instance
pub fn get_storage() -> Result<std::sync::MutexGuard<'static, Option<StorageService>>, StorageError> {
    let guard = STORAGE.lock().map_err(|_| StorageError::NotInitialized)?;
    if guard.is_none() {
        return Err(StorageError::NotInitialized);
    }
    Ok(guard)
}

// ==================== Tauri Commands ====================

#[tauri::command]
pub fn storage_init(data_dir: String) -> Result<(), AppError> {
    let path = PathBuf::from(data_dir);
    init_storage(path).map_err(|e| e.into())
}

#[tauri::command]
pub fn storage_save_wallet(
    id: String,
    name: String,
    address: String,
    created_at: u64,
    encrypted_mnemonic: Option<String>,
    encrypted_private_key: Option<String>,
    salt: String,
) -> Result<(), AppError> {
    let guard = get_storage()?;
    let storage = guard.as_ref().ok_or(StorageError::NotInitialized)?;
    
    let wallet = WalletInfo {
        id,
        name,
        address,
        created_at,
    };
    
    storage.save_wallet(
        &wallet,
        encrypted_mnemonic.as_deref(),
        encrypted_private_key.as_deref(),
        &salt,
    ).map_err(|e| e.into())
}

#[tauri::command]
pub fn storage_load_wallet(wallet_id: String) -> Result<EncryptedWalletData, AppError> {
    let guard = get_storage()?;
    let storage = guard.as_ref().ok_or(StorageError::NotInitialized)?;
    storage.load_wallet(&wallet_id).map_err(|e| e.into())
}

#[tauri::command]
pub fn storage_get_all_wallets() -> Result<Vec<WalletMetadata>, AppError> {
    let guard = get_storage()?;
    let storage = guard.as_ref().ok_or(StorageError::NotInitialized)?;
    storage.get_all_wallets().map_err(|e| e.into())
}

#[tauri::command]
pub fn storage_delete_wallet(wallet_id: String) -> Result<(), AppError> {
    let guard = get_storage()?;
    let storage = guard.as_ref().ok_or(StorageError::NotInitialized)?;
    storage.delete_wallet(&wallet_id).map_err(|e| e.into())
}

#[tauri::command]
pub fn storage_save_transaction(
    hash: String,
    from: String,
    to: String,
    value: String,
    timestamp: u64,
    block_number: String,
    block_hash: String,
    chain_id: u64,
    status: String,
    gas_used: Option<String>,
    gas_price: Option<String>,
    nonce: Option<u64>,
    input: Option<String>,
    wallet_id: String,
) -> Result<(), AppError> {
    let guard = get_storage()?;
    let storage = guard.as_ref().ok_or(StorageError::NotInitialized)?;
    
    let tx = TransactionRecord {
        hash,
        from,
        to,
        value,
        timestamp,
        block_number,
        block_hash,
        chain_id,
        status,
        gas_used,
        gas_price,
        nonce,
        input,
        wallet_id,
    };
    
    storage.save_transaction(&tx).map_err(|e| e.into())
}

#[tauri::command]
pub fn storage_get_wallet_transactions(wallet_id: String) -> Result<Vec<TransactionRecord>, AppError> {
    let guard = get_storage()?;
    let storage = guard.as_ref().ok_or(StorageError::NotInitialized)?;
    storage.get_wallet_transactions(&wallet_id).map_err(|e| e.into())
}

#[tauri::command]
pub fn storage_get_all_transactions() -> Result<Vec<TransactionRecord>, AppError> {
    let guard = get_storage()?;
    let storage = guard.as_ref().ok_or(StorageError::NotInitialized)?;
    storage.get_all_transactions().map_err(|e| e.into())
}

#[tauri::command]
pub fn storage_update_transaction_status(
    hash: String,
    status: String,
    block_number: Option<String>,
    block_hash: Option<String>,
) -> Result<(), AppError> {
    let guard = get_storage()?;
    let storage = guard.as_ref().ok_or(StorageError::NotInitialized)?;
    storage.update_transaction_status(&hash, &status, block_number.as_deref(), block_hash.as_deref())
        .map_err(|e| e.into())
}

#[tauri::command]
pub fn storage_load_settings() -> Result<AppSettings, AppError> {
    let guard = get_storage()?;
    let storage = guard.as_ref().ok_or(StorageError::NotInitialized)?;
    storage.load_settings().map_err(|e| e.into())
}

#[tauri::command]
pub fn storage_save_settings(settings: AppSettings) -> Result<(), AppError> {
    let guard = get_storage()?;
    let storage = guard.as_ref().ok_or(StorageError::NotInitialized)?;
    storage.save_settings(&settings).map_err(|e| e.into())
}

#[tauri::command]
pub fn storage_update_setting(key: String, value: serde_json::Value) -> Result<(), AppError> {
    let guard = get_storage()?;
    let storage = guard.as_ref().ok_or(StorageError::NotInitialized)?;
    storage.update_setting(&key, value).map_err(|e| e.into())
}

#[tauri::command]
pub fn storage_create_backup(encryption_key: String) -> Result<Vec<u8>, AppError> {
    let guard = get_storage()?;
    let storage = guard.as_ref().ok_or(StorageError::NotInitialized)?;
    storage.create_backup(encryption_key.as_bytes()).map_err(|e| e.into())
}

#[tauri::command]
pub fn storage_restore_backup(encrypted_data: Vec<u8>, encryption_key: String) -> Result<(), AppError> {
    let guard = get_storage()?;
    let storage = guard.as_ref().ok_or(StorageError::NotInitialized)?;
    storage.restore_backup(&encrypted_data, encryption_key.as_bytes()).map_err(|e| e.into())
}

#[tauri::command]
pub fn storage_clear_all_data() -> Result<(), AppError> {
    let guard = get_storage()?;
    let storage = guard.as_ref().ok_or(StorageError::NotInitialized)?;
    storage.clear_all_data().map_err(|e| e.into())
}

#[tauri::command]
pub fn storage_get_data_dir() -> Result<String, AppError> {
    let guard = get_storage()?;
    let storage = guard.as_ref().ok_or(StorageError::NotInitialized)?;
    Ok(storage.get_data_dir().to_string_lossy().to_string())
}