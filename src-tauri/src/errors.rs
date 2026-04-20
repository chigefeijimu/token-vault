// Unified error handling module for the application

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Unified application error type that wraps all module-specific errors
#[derive(Error, Debug, Serialize, Deserialize, Clone)]
pub enum AppError {
    // Crypto module errors (1000-1999)
    #[error("Crypto error: {message}")]
    Crypto {
        code: u32,
        message: String,
    },
    
    // Wallet module errors (2000-2999)
    #[error("Wallet error: {message}")]
    Wallet {
        code: u32,
        message: String,
    },
    
    // Transaction module errors (3000-3999)
    #[error("Transaction error: {message}")]
    Transaction {
        code: u32,
        message: String,
    },
    
    // RPC module errors (4000-4999)
    #[error("RPC error: {message}")]
    Rpc {
        code: u32,
        message: String,
    },
    
    // ERC20 module errors (5000-5999)
    #[error("ERC20 error: {message}")]
    Erc20 {
        code: u32,
        message: String,
    },
    
    // Storage module errors (6000-6999)
    #[error("Storage error: {message}")]
    Storage {
        code: u32,
        message: String,
    },
    
    // Network errors (7000-7999)
    #[error("Network error: {message}")]
    Network {
        code: u32,
        message: String,
    },
    
    // Validation errors (8000-8999)
    #[error("Validation error: {message}")]
    Validation {
        code: u32,
        message: String,
    },
    
    // Internal errors (9000-9999)
    #[error("Internal error: {message}")]
    Internal {
        code: u32,
        message: String,
    },
}

impl AppError {
    /// Create a new crypto error with code
    pub fn crypto(code: u32, message: impl Into<String>) -> Self {
        Self::Crypto { code, message: message.into() }
    }
    
    /// Create a new wallet error with code
    pub fn wallet(code: u32, message: impl Into<String>) -> Self {
        Self::Wallet { code, message: message.into() }
    }
    
    /// Create a new transaction error with code
    pub fn transaction(code: u32, message: impl Into<String>) -> Self {
        Self::Transaction { code, message: message.into() }
    }
    
    /// Create a new RPC error with code
    pub fn rpc(code: u32, message: impl Into<String>) -> Self {
        Self::Rpc { code, message: message.into() }
    }
    
    /// Create a new ERC20 error with code
    pub fn erc20(code: u32, message: impl Into<String>) -> Self {
        Self::Erc20 { code, message: message.into() }
    }
    
    /// Create a new storage error with code
    pub fn storage(code: u32, message: impl Into<String>) -> Self {
        Self::Storage { code, message: message.into() }
    }
    
    /// Create a new network error with code
    pub fn network(code: u32, message: impl Into<String>) -> Self {
        Self::Network { code, message: message.into() }
    }
    
    /// Create a new validation error with code
    pub fn validation(code: u32, message: impl Into<String>) -> Self {
        Self::Validation { code, message: message.into() }
    }
    
    /// Create a new internal error with code
    pub fn internal(code: u32, message: impl Into<String>) -> Self {
        Self::Internal { code, message: message.into() }
    }
    
    /// Get the error code
    pub fn code(&self) -> u32 {
        match self {
            Self::Crypto { code, .. } => *code,
            Self::Wallet { code, .. } => *code,
            Self::Transaction { code, .. } => *code,
            Self::Rpc { code, .. } => *code,
            Self::Erc20 { code, .. } => *code,
            Self::Storage { code, .. } => *code,
            Self::Network { code, .. } => *code,
            Self::Validation { code, .. } => *code,
            Self::Internal { code, .. } => *code,
        }
    }
    
    /// Get the error category name
    pub fn category(&self) -> &'static str {
        match self {
            Self::Crypto { .. } => "crypto",
            Self::Wallet { .. } => "wallet",
            Self::Transaction { .. } => "transaction",
            Self::Rpc { .. } => "rpc",
            Self::Erc20 { .. } => "erc20",
            Self::Storage { .. } => "storage",
            Self::Network { .. } => "network",
            Self::Validation { .. } => "validation",
            Self::Internal { .. } => "internal",
        }
    }
}

// Crypto error codes
pub mod crypto_codes {
    pub const ENCRYPTION_FAILED: u32 = 1000;
    pub const DECRYPTION_FAILED: u32 = 1001;
    pub const INVALID_KEY: u32 = 1002;
    pub const KEY_DERIVATION_FAILED: u32 = 1003;
    pub const MNEMONIC_GENERATION_FAILED: u32 = 1004;
    pub const INVALID_MNEMONIC: u32 = 1005;
    pub const PRIVATE_KEY_INVALID: u32 = 1006;
    pub const SIGNATURE_FAILED: u32 = 1007;
}

// Wallet error codes
pub mod wallet_codes {
    pub const NOT_FOUND: u32 = 2000;
    pub const ALREADY_EXISTS: u32 = 2001;
    pub const INVALID_ADDRESS: u32 = 2002;
    pub const CHAIN_NOT_SUPPORTED: u32 = 2003;
    pub const BALANCE_FETCH_FAILED: u32 = 2004;
    pub const ENCRYPTION_FAILED: u32 = 2005;
    pub const DECRYPTION_FAILED: u32 = 2006;
    pub const INVALID_PASSWORD: u32 = 2007;
}

// Transaction error codes
pub mod transaction_codes {
    pub const SIGNING_FAILED: u32 = 3000;
    pub const BROADCAST_FAILED: u32 = 3001;
    pub const INVALID_TX_DATA: u32 = 3002;
    pub const UNDERPRICED_GAS: u32 = 3003;
    pub const NONCE_TOO_LOW: u32 = 3004;
    pub const REPLACEMENT_UNDERPRICED: u32 = 3005;
    pub const TX_REJECTED: u32 = 3006;
    pub const TX_TIMEOUT: u32 = 3007;
}

// RPC error codes
pub mod rpc_codes {
    pub const CONNECTION_FAILED: u32 = 4000;
    pub const REQUEST_FAILED: u32 = 4001;
    pub const INVALID_RESPONSE: u32 = 4002;
    pub const CHAIN_ID_MISMATCH: u32 = 4003;
    pub const BLOCK_NOT_FOUND: u32 = 4004;
    pub const GAS_ESTIMATION_FAILED: u32 = 4005;
    pub const METHOD_NOT_FOUND: u32 = 4006;
}

// ERC20 error codes
pub mod erc20_codes {
    pub const TRANSFER_FAILED: u32 = 5000;
    pub const BALANCE_FETCH_FAILED: u32 = 5001;
    pub const APPROVAL_FAILED: u32 = 5002;
    pub const INVALID_CONTRACT: u32 = 5003;
    pub const TRANSFER_FROM_FAILED: u32 = 5004;
}

// Storage error codes
pub mod storage_codes {
    pub const SAVE_FAILED: u32 = 6000;
    pub const LOAD_FAILED: u32 = 6001;
    pub const DELETE_FAILED: u32 = 6002;
    pub const NOT_FOUND: u32 = 6003;
    pub const SERIALIZATION_FAILED: u32 = 6004;
}

// Network error codes
pub mod network_codes {
    pub const TIMEOUT: u32 = 7000;
    pub const CONNECTION_REFUSED: u32 = 7001;
    pub const DNS_LOOKUP_FAILED: u32 = 7002;
    pub const SSL_ERROR: u32 = 7003;
}

// Validation error codes
pub mod validation_codes {
    pub const INVALID_INPUT: u32 = 8000;
    pub const ADDRESS_MISMATCH: u32 = 8001;
    pub const AMOUNT_INVALID: u32 = 8002;
    pub const CHAIN_ID_INVALID: u32 = 8003;
}

// Convert from CryptoError to AppError
impl From<crate::crypto::CryptoError> for AppError {
    fn from(err: crate::crypto::CryptoError) -> Self {
        use crate::crypto::CryptoError;
        let (code, message) = match err {
            CryptoError::EncryptionFailed(msg) => (crypto_codes::ENCRYPTION_FAILED, msg),
            CryptoError::DecryptionFailed(msg) => (crypto_codes::DECRYPTION_FAILED, msg),
            CryptoError::InvalidData(msg) => (crypto_codes::INVALID_KEY, msg),
            CryptoError::MnemonicError(msg) => (crypto_codes::INVALID_MNEMONIC, msg),
        };
        Self::Crypto { code, message }
    }
}

// Convert from WalletError to AppError
impl From<crate::wallet::WalletError> for AppError {
    fn from(err: crate::wallet::WalletError) -> Self {
        use crate::wallet::WalletError;
        let (code, message) = match err {
            WalletError::NotFound(msg) => (wallet_codes::NOT_FOUND, msg.to_string()),
            WalletError::InvalidPassword => (wallet_codes::INVALID_PASSWORD, "Invalid password".to_string()),
            WalletError::AlreadyExists => (wallet_codes::ALREADY_EXISTS, "Wallet already exists".to_string()),
            WalletError::Crypto(msg) => {
                return Self::Crypto {
                    code: crypto_codes::ENCRYPTION_FAILED,
                    message: msg.to_string(),
                };
            }
        };
        Self::Wallet { code, message }
    }
}

// Convert from TransactionError to AppError
impl From<crate::transaction::TransactionError> for AppError {
    fn from(err: crate::transaction::TransactionError) -> Self {
        use crate::transaction::TransactionError;
        let (code, message) = match err {
            TransactionError::Signing(msg) => (transaction_codes::SIGNING_FAILED, msg),
            TransactionError::Rpc(msg) => (transaction_codes::BROADCAST_FAILED, msg),
            TransactionError::WalletNotFound(msg) => (wallet_codes::NOT_FOUND, msg),
            TransactionError::InvalidAddress(msg) => (wallet_codes::INVALID_ADDRESS, msg),
            TransactionError::UnsupportedChain(msg) => (transaction_codes::BROADCAST_FAILED, msg.to_string()),
            TransactionError::Encoding(msg) => (transaction_codes::BROADCAST_FAILED, msg),
        };
        Self::Transaction { code, message }
    }
}

// Result type alias for AppError
pub type AppResult<T> = Result<T, AppError>;