// Persistent storage module using SQLite
// Replaces JSON file storage with a proper relational database

use crate::errors::AppError;
use crate::wallet::WalletInfo;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Mutex;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
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
            StorageError::Io(e) => AppError::Storage {
                code: 1006,
                message: format!("IO error: {}", e),
            },
            StorageError::Database(e) => AppError::Storage {
                code: 1000,
                message: e.to_string(),
            },
            StorageError::Serialization(e) => AppError::Storage { code: 1001, message: e.to_string() },
            StorageError::Encryption(e) => AppError::Storage { code: 1002, message: e },
            StorageError::NotInitialized => AppError::Storage { code: 1003, message: "Data directory not initialized".to_string() },
            StorageError::WalletNotFound(id) => AppError::Storage { code: 1004, message: format!("Wallet not found: {}", id) },
            StorageError::TransactionNotFound(hash) => AppError::Storage { code: 1005, message: format!("Transaction not found: {}", hash) },
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

/// Main storage service backed by SQLite
pub struct StorageService {
    db: Mutex<Connection>,
}

impl StorageService {
    /// Create a new storage service — opens (or creates) the SQLite database
    pub fn new(db_path: PathBuf) -> Result<Self, StorageError> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&db_path)?;

        // Enable WAL mode for better concurrency
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;

        let storage = Self { db: Mutex::new(conn) };
        Ok(storage)
    }

    /// Initialize tables
    pub fn initialize(&self) -> Result<(), StorageError> {
        let conn = self.db.lock().unwrap();

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS wallets (
                id          TEXT PRIMARY KEY,
                name        TEXT NOT NULL,
                address     TEXT NOT NULL,
                created_at  INTEGER NOT NULL,
                encrypted_mnemonic   TEXT,
                encrypted_private_key TEXT,
                salt        TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_wallets_address ON wallets(address);

            CREATE TABLE IF NOT EXISTS wallet_meta (
                id              TEXT PRIMARY KEY,
                name            TEXT NOT NULL,
                address         TEXT NOT NULL,
                created_at      INTEGER NOT NULL,
                updated_at      INTEGER NOT NULL,
                last_used       INTEGER,
                chain_ids       TEXT NOT NULL DEFAULT '[]'
            );

            CREATE TABLE IF NOT EXISTS transactions (
                hash           TEXT PRIMARY KEY,
                tx_from        TEXT NOT NULL,
                tx_to          TEXT NOT NULL,
                value          TEXT NOT NULL,
                timestamp      INTEGER NOT NULL,
                block_number   TEXT NOT NULL,
                block_hash     TEXT NOT NULL,
                chain_id       INTEGER NOT NULL,
                status         TEXT NOT NULL,
                gas_used       TEXT,
                gas_price      TEXT,
                nonce          INTEGER,
                tx_input       TEXT,
                wallet_id      TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_tx_wallet ON transactions(wallet_id);
            CREATE INDEX IF NOT EXISTS idx_tx_chain  ON transactions(chain_id);
            CREATE INDEX IF NOT EXISTS idx_tx_status ON transactions(status);

            CREATE TABLE IF NOT EXISTS settings (
                key   TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            ",
        )?;

        Ok(())
    }

    /// Get current timestamp in seconds
    fn now() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
    }

    // ==================== Wallet Operations ====================

    /// Save wallet data (encrypted)
    pub fn save_wallet(
        &self,
        wallet: &WalletInfo,
        encrypted_mnemonic: Option<&str>,
        encrypted_pk: Option<&str>,
        salt: &str,
    ) -> Result<(), StorageError> {
        let conn = self.db.lock().unwrap();

        conn.execute(
            "INSERT OR REPLACE INTO wallets
             (id, name, address, created_at, encrypted_mnemonic, encrypted_private_key, salt)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                wallet.id,
                wallet.name,
                wallet.address,
                wallet.created_at as i64,
                encrypted_mnemonic,
                encrypted_pk,
                salt,
            ],
        )?;

        // Update wallet_meta index
        let now = Self::now();
        conn.execute(
            "INSERT OR REPLACE INTO wallet_meta
             (id, name, address, created_at, updated_at, last_used, chain_ids)
             VALUES (?1, ?2, ?3, ?4, ?5, NULL, '[]')",
            params![wallet.id, wallet.name, wallet.address, wallet.created_at as i64, now as i64],
        )?;

        Ok(())
    }

    /// Load wallet data
    pub fn load_wallet(&self, wallet_id: &str) -> Result<EncryptedWalletData, StorageError> {
        let conn = self.db.lock().unwrap();

        let w = conn.query_row(
            "SELECT id, name, address, created_at, encrypted_mnemonic,
                    encrypted_private_key, salt
             FROM wallets WHERE id = ?1",
            params![wallet_id],
            |row| {
                Ok(EncryptedWalletData {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    address: row.get(2)?,
                    created_at: row.get::<_, i64>(3)? as u64,
                    encrypted_mnemonic: row.get(4)?,
                    encrypted_private_key: row.get(5)?,
                    salt: row.get(6)?,
                })
            },
        )
        .map_err(|_| StorageError::WalletNotFound(wallet_id.to_string()))?;

        Ok(w)
    }

    /// Get all wallet metadata (non-sensitive)
    pub fn get_all_wallets(&self) -> Result<Vec<WalletMetadata>, StorageError> {
        let conn = self.db.lock().unwrap();

        let mut stmt = conn.prepare(
            "SELECT id, name, address, created_at, updated_at, last_used, chain_ids
             FROM wallet_meta ORDER BY created_at DESC",
        )?;

        let rows = stmt.query_map([], |row| {
            let chain_ids_str: String = row.get(6)?;
            let chain_ids: Vec<u64> = serde_json::from_str(&chain_ids_str).unwrap_or_default();
            Ok(WalletMetadata {
                id: row.get(0)?,
                name: row.get(1)?,
                address: row.get(2)?,
                created_at: row.get::<_, i64>(3)? as u64,
                updated_at: row.get::<_, i64>(4)? as u64,
                last_used: row.get::<_, Option<i64>>(5)?.map(|v| v as u64),
                chain_ids,
            })
        })?;

        let mut metas = Vec::new();
        for row in rows {
            metas.push(row?);
        }
        Ok(metas)
    }

    /// Delete a wallet
    pub fn delete_wallet(&self, wallet_id: &str) -> Result<(), StorageError> {
        let conn = self.db.lock().unwrap();
        conn.execute("DELETE FROM wallets WHERE id = ?1", params![wallet_id])?;
        conn.execute("DELETE FROM wallet_meta WHERE id = ?1", params![wallet_id])?;
        // Remove associated transactions
        conn.execute("DELETE FROM transactions WHERE wallet_id = ?1", params![wallet_id])?;
        Ok(())
    }

    /// Update wallet last used timestamp
    pub fn update_wallet_last_used(&self, wallet_id: &str) -> Result<(), StorageError> {
        let conn = self.db.lock().unwrap();
        let now = Self::now() as i64;
        conn.execute(
            "UPDATE wallet_meta SET last_used = ?1 WHERE id = ?2",
            params![now, wallet_id],
        )?;
        Ok(())
    }

    // ==================== Transaction Operations ====================

    /// Save a transaction
    pub fn save_transaction(&self, tx: &TransactionRecord) -> Result<(), StorageError> {
        let conn = self.db.lock().unwrap();
        conn.execute(
            "INSERT OR REPLACE INTO transactions
             (hash, tx_from, tx_to, value, timestamp, block_number, block_hash,
              chain_id, status, gas_used, gas_price, nonce, tx_input, wallet_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
            params![
                tx.hash,
                tx.from,
                tx.to,
                tx.value,
                tx.timestamp as i64,
                tx.block_number,
                tx.block_hash,
                tx.chain_id as i64,
                tx.status,
                tx.gas_used,
                tx.gas_price,
                tx.nonce.map(|n| n as i64),
                tx.input,
                tx.wallet_id,
            ],
        )?;
        Ok(())
    }

    /// Get transactions for a wallet
    pub fn get_wallet_transactions(&self, wallet_id: &str) -> Result<Vec<TransactionRecord>, StorageError> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT hash, tx_from, tx_to, value, timestamp, block_number, block_hash,
                    chain_id, status, gas_used, gas_price, nonce, tx_input, wallet_id
             FROM transactions WHERE wallet_id = ?1 ORDER BY timestamp DESC",
        )?;

        let txs: Vec<TransactionRecord> = stmt
            .query_map(params![wallet_id], Self::map_tx_row)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(txs)
    }

    /// Get transactions for a specific chain
    pub fn get_chain_transactions(&self, chain_id: u64) -> Result<Vec<TransactionRecord>, StorageError> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT hash, tx_from, tx_to, value, timestamp, block_number, block_hash,
                    chain_id, status, gas_used, gas_price, nonce, tx_input, wallet_id
             FROM transactions WHERE chain_id = ?1 ORDER BY timestamp DESC",
        )?;

        let txs: Vec<TransactionRecord> = stmt
            .query_map(params![chain_id as i64], Self::map_tx_row)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(txs)
    }

    /// Get all transactions
    pub fn get_all_transactions(&self) -> Result<Vec<TransactionRecord>, StorageError> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT hash, tx_from, tx_to, value, timestamp, block_number, block_hash,
                    chain_id, status, gas_used, gas_price, nonce, tx_input, wallet_id
             FROM transactions ORDER BY timestamp DESC",
        )?;

        let txs: Vec<TransactionRecord> = stmt
            .query_map([], Self::map_tx_row)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(txs)
    }

    /// Update transaction status
    pub fn update_transaction_status(
        &self,
        hash: &str,
        status: &str,
        block_number: Option<&str>,
        block_hash: Option<&str>,
    ) -> Result<(), StorageError> {
        let conn = self.db.lock().unwrap();

        let rows_affected = conn.execute(
            "UPDATE transactions
             SET status = ?1,
                 block_number = COALESCE(?2, block_number),
                 block_hash   = COALESCE(?3, block_hash)
             WHERE hash = ?4",
            params![status, block_number, block_hash, hash],
        )?;

        if rows_affected == 0 {
            return Err(StorageError::TransactionNotFound(hash.to_string()));
        }
        Ok(())
    }

    fn map_tx_row(row: &rusqlite::Row) -> rusqlite::Result<TransactionRecord> {
        Ok(TransactionRecord {
            hash: row.get(0)?,
            from: row.get(1)?,
            to: row.get(2)?,
            value: row.get(3)?,
            timestamp: row.get::<_, i64>(4)? as u64,
            block_number: row.get(5)?,
            block_hash: row.get(6)?,
            chain_id: row.get::<_, i64>(7)? as u64,
            status: row.get(8)?,
            gas_used: row.get(9)?,
            gas_price: row.get(10)?,
            nonce: row.get::<_, Option<i64>>(11)?.map(|n| n as u64),
            input: row.get(12)?,
            wallet_id: row.get(13)?,
        })
    }

    // ==================== Settings Operations ====================

    /// Load application settings
    pub fn load_settings(&self) -> Result<AppSettings, StorageError> {
        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
        let rows: Vec<(String, String)> = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?
            .filter_map(|r| r.ok())
            .collect();

        if rows.is_empty() {
            return Ok(AppSettings::default());
        }

        let mut settings = AppSettings::default();
        for (key, value) in rows {
            match key.as_str() {
                "theme" => settings.theme = value,
                "currency" => settings.currency = value,
                "language" => settings.language = value,
                "auto_lock_minutes" => {
                    settings.auto_lock_minutes = value.parse().unwrap_or(5)
                }
                "backup_reminder" => {
                    settings.backup_reminder = value == "true"
                }
                "rpc_timeout_seconds" => {
                    settings.rpc_timeout_seconds = value.parse().unwrap_or(30)
                }
                "default_chain_id" => {
                    settings.default_chain_id = value.parse().unwrap_or(1)
                }
                "hide_balances" => settings.hide_balances = value == "true",
                "max_gas_price_gwei" => {
                    settings.max_gas_price_gwei = value.parse().ok()
                }
                _ => {}
            }
        }
        Ok(settings)
    }

    /// Save application settings
    pub fn save_settings(&self, settings: &AppSettings) -> Result<(), StorageError> {
        let conn = self.db.lock().unwrap();

        let pairs = [
            ("theme", settings.theme.clone()),
            ("currency", settings.currency.clone()),
            ("language", settings.language.clone()),
            ("auto_lock_minutes", settings.auto_lock_minutes.to_string()),
            ("backup_reminder", settings.backup_reminder.to_string()),
            ("rpc_timeout_seconds", settings.rpc_timeout_seconds.to_string()),
            ("default_chain_id", settings.default_chain_id.to_string()),
            ("hide_balances", settings.hide_balances.to_string()),
            (
                "max_gas_price_gwei",
                settings
                    .max_gas_price_gwei
                    .map(|v| v.to_string())
                    .unwrap_or_default(),
            ),
        ];

        for (key, value) in pairs {
            conn.execute(
                "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
                params![key, value],
            )?;
        }
        Ok(())
    }

    /// Update a single setting
    pub fn update_setting(&self, key: &str, value: serde_json::Value) -> Result<(), StorageError> {
        let conn = self.db.lock().unwrap();
        let value_str = match &value {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => b.to_string(),
            _ => return Err(StorageError::Encryption(format!("Unsupported setting type for key: {}", key))),
        };
        conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
            params![key, value_str],
        )?;
        Ok(())
    }

    // ==================== Backup/Export Operations ====================

    /// Export all data as a JSON blob (encrypted backup would be done by caller)
    pub fn export_all_json(&self) -> Result<Vec<u8>, StorageError> {
        #[derive(Serialize)]
        struct BackupData {
            wallets: Vec<EncryptedWalletData>,
            transactions: Vec<TransactionRecord>,
            settings: AppSettings,
            wallet_meta: Vec<WalletMetadata>,
        }

        let conn = self.db.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, name, address, created_at, encrypted_mnemonic,
                    encrypted_private_key, salt FROM wallets",
        )?;
        let wallets: Vec<EncryptedWalletData> = stmt
            .query_map([], |row| {
                Ok(EncryptedWalletData {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    address: row.get(2)?,
                    created_at: row.get::<_, i64>(3)? as u64,
                    encrypted_mnemonic: row.get(4)?,
                    encrypted_private_key: row.get(5)?,
                    salt: row.get(6)?,
                })
            })?
            .filter_map(|r| r.ok())
            .collect();
        drop(stmt);
        drop(conn);

        let transactions = self.get_all_transactions()?;
        let settings = self.load_settings()?;
        let wallet_meta = self.get_all_wallets()?;

        let backup = BackupData { wallets, transactions, settings, wallet_meta };
        serde_json::to_vec(&backup).map_err(StorageError::Serialization)
    }

    /// Clear all data (factory reset)
    pub fn clear_all_data(&self) -> Result<(), StorageError> {
        let conn = self.db.lock().unwrap();
        conn.execute_batch(
            "DELETE FROM wallets; DELETE FROM wallet_meta;
             DELETE FROM transactions; DELETE FROM settings;",
        )?;
        Ok(())
    }
}

// ==================== Global Storage ====================

use lazy_static::lazy_static;

lazy_static! {
    pub static ref STORAGE: Mutex<Option<StorageService>> = Mutex::new(None);
}

/// Initialize the global storage service
pub fn init_storage(data_dir: PathBuf) -> Result<(), StorageError> {
    let db_path = data_dir.join("tokenvault.db");
    let storage = StorageService::new(db_path)?;
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

    storage
        .save_wallet(
            &wallet,
            encrypted_mnemonic.as_deref(),
            encrypted_private_key.as_deref(),
            &salt,
        )
        .map_err(|e| e.into())
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
    storage
        .update_transaction_status(&hash, &status, block_number.as_deref(), block_hash.as_deref())
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
pub fn storage_export_json() -> Result<String, AppError> {
    let guard = get_storage()?;
    let storage = guard.as_ref().ok_or(StorageError::NotInitialized)?;
    let bytes: Vec<u8> = storage.export_all_json().map_err(|e| -> AppError { e.into() })?;
    Ok(String::from_utf8_lossy(&bytes).to_string())
}

#[tauri::command]
pub fn storage_clear_all_data() -> Result<(), AppError> {
    let guard = get_storage()?;
    let storage = guard.as_ref().ok_or(StorageError::NotInitialized)?;
    storage.clear_all_data().map_err(|e| e.into())
}
