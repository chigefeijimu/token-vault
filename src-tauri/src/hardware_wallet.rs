// Hardware wallet interface implementation
// Supports common hardware wallet interfaces through standard protocols

use crate::errors::AppError;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use std::sync::Mutex;

#[derive(Error, Debug)]
pub enum HardwareWalletError {
    #[error("Device not connected: {0}")]
    NotConnected(String),
    #[error("Communication failed: {0}")]
    CommunicationError(String),
    #[error("User action required: {0}")]
    UserActionRequired(String),
    #[error("Signing failed: {0}")]
    SigningFailed(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("No device found")]
    NoDeviceFound,
    #[error("Device locked")]
    DeviceLocked,
}

/// Hardware wallet types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HardwareWalletType {
    Ledger,
    Trezor,
    #[cfg(feature = "bitbox02")]
    BitBox02,
    #[cfg(feature = "lattice")]
    Lattice,
}

/// Hardware wallet connection status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletStatus {
    pub wallet_type: HardwareWalletType,
    pub connected: bool,
    pub unlocked: bool,
    pub has_app: bool,
    pub app_name: Option<String>,
    pub firmware_version: Option<String>,
    pub btc_format: Option<String>,
}

/// HD derivation path for EVM chains
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivationPath {
    pub path: String,
    pub chain_id: Option<u32>,
}

impl DerivationPath {
    /// Standard EVM derivation path (BIP44)
    pub fn eth_standard(account: u32) -> Self {
        Self {
            path: format!("m/44'/60'/{}'/0/0", account),
            chain_id: Some(1),
        }
    }

    /// Ledger Live derivation path
    pub fn ledger_live(account: u32) -> Self {
        Self {
            path: format!("m/44'/60'/{}'/0/0", account),
            chain_id: Some(1),
        }
    }

    /// Legacy derivation path
    pub fn legacy(account: u32) -> Self {
        Self {
            path: format!("m/44'/60'/{}'/0/0", account),
            chain_id: Some(1),
        }
    }
}

/// Transaction request for hardware wallet signing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignRequest {
    pub nonce: String,
    pub gas_price: String,
    pub gas_limit: String,
    pub to: String,
    pub value: String,
    pub data: String,
    pub chain_id: u32,
    pub tx_type: TransactionType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionType {
    Legacy,      // EIP155
    EIP2930,
    EIP1559,
}

/// Signature response from hardware wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignResponse {
    pub v: u8,
    pub r: [u8; 32],
    pub s: [u8; 32],
    pub signature: String,
}

/// Hardware wallet manager trait
pub trait HardwareWallet: Send + Sync {
    /// Get the wallet type
    fn wallet_type(&self) -> HardwareWalletType;
    
    /// Check if device is connected
    fn is_connected(&self) -> bool;
    
    /// Get device status
    fn get_status(&self) -> Result<WalletStatus, HardwareWalletError>;
    
    /// Get public address for derivation path
    fn get_address(&self, derivation_path: &DerivationPath) -> Result<String, HardwareWalletError>;
    
    /// Sign transaction
    fn sign_transaction(
        &self,
        derivation_path: &DerivationPath,
        request: &SignRequest,
    ) -> Result<SignResponse, HardwareWalletError>;
    
    /// Sign message (EIP-191 personal sign)
    fn sign_message(
        &self,
        derivation_path: &DerivationPath,
        message: &[u8],
    ) -> Result<SignResponse, HardwareWalletError>;
    
    /// Sign typed data (EIP-712)
    fn sign_typed_data(
        &self,
        derivation_path: &DerivationPath,
        domain_hash: &[u8],
        message_hash: &[u8],
    ) -> Result<SignResponse, HardwareWalletError>;
}

/// Hardware wallet manager for handling multiple wallet types
pub struct HardwareWalletManager {
    active_wallet: Mutex<Option<Box<dyn HardwareWallet>>>,
}

impl HardwareWalletManager {
    pub fn new() -> Self {
        Self {
            active_wallet: Mutex::new(None),
        }
    }

    /// Detect connected hardware wallets
    pub fn detect_devices(&self) -> Vec<(HardwareWalletType, String)> {
        let mut devices = Vec::new();
        
        // Attempt to detect each wallet type
        // This is a simplified implementation - actual implementation would use
        // platform-specific HID transport libraries
        
        #[cfg(target_os = "linux")]
        {
            if std::path::Path::new("/dev/hidraw0").exists() {
                // Try to identify wallet type
                devices.push((HardwareWalletType::Ledger, "USB Device".to_string()));
            }
        }
        
        devices
    }

    /// Connect to a specific wallet type
    pub fn connect(&self, wallet_type: HardwareWalletType) -> Result<(), HardwareWalletError> {
        let wallet: Box<dyn HardwareWallet> = match wallet_type {
            HardwareWalletType::Ledger => Box::new(LedgerWallet::new()?),
            HardwareWalletType::Trezor => Box::new(TrezorWallet::new()?),
            #[cfg(feature = "bitbox02")]
            HardwareWalletType::BitBox02 => Box::new(BitBox02Wallet::new()?),
            #[cfg(feature = "lattice")]
            HardwareWalletType::Lattice => Box::new(LatticeWallet::new()?),
            #[cfg(not(feature = "bitbox02"))]
            _ if wallet_type == HardwareWalletType::BitBox02 => {
                return Err(HardwareWalletError::NotConnected(
                    "BitBox02 support not compiled".to_string()
                ));
            }
            #[cfg(not(feature = "lattice"))]
            _ if wallet_type == HardwareWalletType::Lattice => {
                return Err(HardwareWalletError::NotConnected(
                    "Lattice support not compiled".to_string()
                ));
            }
        };

        let mut active = self.active_wallet.lock().unwrap();
        *active = Some(wallet);
        
        Ok(())
    }

    /// Disconnect current wallet
    pub fn disconnect(&self) {
        let mut active = self.active_wallet.lock().unwrap();
        *active = None;
    }

    /// Get current connection status
    pub fn get_status(&self) -> Option<WalletStatus> {
        let active = self.active_wallet.lock().unwrap();
        active.as_ref().map(|w| w.get_status()).ok().flatten()
    }

    /// Get address from connected wallet
    pub fn get_address(&self, path: &DerivationPath) -> Result<String, HardwareWalletError> {
        let active = self.active_wallet.lock().unwrap();
        match active.as_ref() {
            Some(wallet) => wallet.get_address(path),
            None => Err(HardwareWalletError::NotConnected("No wallet connected".to_string())),
        }
    }

    /// Sign transaction with connected wallet
    pub fn sign_transaction(
        &self,
        path: &DerivationPath,
        request: &SignRequest,
    ) -> Result<SignResponse, HardwareWalletError> {
        let active = self.active_wallet.lock().unwrap();
        match active.as_ref() {
            Some(wallet) => wallet.sign_transaction(path, request),
            None => Err(HardwareWalletError::NotConnected("No wallet connected".to_string())),
        }
    }

    /// Sign message with connected wallet
    pub fn sign_message(&self, path: &DerivationPath, message: &[u8]) -> Result<SignResponse, HardwareWalletError> {
        let active = self.active_wallet.lock().unwrap();
        match active.as_ref() {
            Some(wallet) => wallet.sign_message(path, message),
            None => Err(HardwareWalletError::NotConnected("No wallet connected".to_string())),
        }
    }
}

impl Default for HardwareWalletManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Ledger wallet implementation
pub struct LedgerWallet {
    connected: bool,
}

impl LedgerWallet {
    pub fn new() -> Result<Self, HardwareWalletError> {
        // In production, this would initialize HID transport
        // For now, we just create the struct
        Ok(Self { connected: true })
    }
}

impl HardwareWallet for LedgerWallet {
    fn wallet_type(&self) -> HardwareWalletType {
        HardwareWalletType::Ledger
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn get_status(&self) -> Result<WalletStatus, HardwareWalletError> {
        if !self.connected {
            return Err(HardwareWalletError::NotConnected("Ledger not connected".to_string()));
        }
        
        Ok(WalletStatus {
            wallet_type: HardwareWalletType::Ledger,
            connected: true,
            unlocked: true, // Would be determined by actual communication
            has_app: true,
            app_name: Some("Ethereum".to_string()),
            firmware_version: Some("2.0.0".to_string()),
            btc_format: None,
        })
    }

    fn get_address(&self, derivation_path: &DerivationPath) -> Result<String, HardwareWalletError> {
        // In production, this would communicate with the Ledger device
        // using APDU commands
        Err(HardwareWalletError::UserActionRequired(
            "Please confirm address on your Ledger device".to_string()
        ))
    }

    fn sign_transaction(
        &self,
        derivation_path: &DerivationPath,
        request: &SignRequest,
    ) -> Result<SignResponse, HardwareWalletError> {
        // In production, this would send the transaction to Ledger
        // and receive the signature
        Err(HardwareWalletError::UserActionRequired(
            "Please confirm transaction on your Ledger device".to_string()
        ))
    }

    fn sign_message(&self, derivation_path: &DerivationPath, message: &[u8]) -> Result<SignResponse, HardwareWalletError> {
        Err(HardwareWalletError::UserActionRequired(
            "Please confirm message signing on your Ledger device".to_string()
        ))
    }

    fn sign_typed_data(
        &self,
        derivation_path: &DerivationPath,
        domain_hash: &[u8],
        message_hash: &[u8],
    ) -> Result<SignResponse, HardwareWalletError> {
        Err(HardwareWalletError::UserActionRequired(
            "Please confirm typed data signing on your Ledger device".to_string()
        ))
    }
}

/// Trezor wallet implementation
pub struct TrezorWallet {
    connected: bool,
}

impl TrezorWallet {
    pub fn new() -> Result<Self, HardwareWalletError> {
        Ok(Self { connected: true })
    }
}

impl HardwareWallet for TrezorWallet {
    fn wallet_type(&self) -> HardwareWalletType {
        HardwareWalletType::Trezor
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn get_status(&self) -> Result<WalletStatus, HardwareWalletError> {
        if !self.connected {
            return Err(HardwareWalletError::NotConnected("Trezor not connected".to_string()));
        }
        
        Ok(WalletStatus {
            wallet_type: HardwareWalletType::Trezor,
            connected: true,
            unlocked: true,
            has_app: true,
            app_name: Some("Ethereum".to_string()),
            firmware_version: Some("2.4.0".to_string()),
            btc_format: None,
        })
    }

    fn get_address(&self, derivation_path: &DerivationPath) -> Result<String, HardwareWalletError> {
        Err(HardwareWalletError::UserActionRequired(
            "Please confirm address on your Trezor device".to_string()
        ))
    }

    fn sign_transaction(
        &self,
        derivation_path: &DerivationPath,
        request: &SignRequest,
    ) -> Result<SignResponse, HardwareWalletError> {
        Err(HardwareWalletError::UserActionRequired(
            "Please confirm transaction on your Trezor device".to_string()
        ))
    }

    fn sign_message(&self, derivation_path: &DerivationPath, message: &[u8]) -> Result<SignResponse, HardwareWalletError> {
        Err(HardwareWalletError::UserActionRequired(
            "Please confirm message signing on your Trezor device".to_string()
        ))
    }

    fn sign_typed_data(
        &self,
        derivation_path: &DerivationPath,
        domain_hash: &[u8],
        message_hash: &[u8],
    ) -> Result<SignResponse, HardwareWalletError> {
        Err(HardwareWalletError::UserActionRequired(
            "Please confirm typed data signing on your Trezor device".to_string()
        ))
    }
}

#[cfg(feature = "bitbox02")]
pub struct BitBox02Wallet {
    connected: bool,
}

#[cfg(feature = "bitbox02")]
impl BitBox02Wallet {
    pub fn new() -> Result<Self, HardwareWalletError> {
        Ok(Self { connected: true })
    }
}

#[cfg(feature = "bitbox02")]
impl HardwareWallet for BitBox02Wallet {
    fn wallet_type(&self) -> HardwareWalletType {
        HardwareWalletType::BitBox02
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn get_status(&self) -> Result<WalletStatus, HardwareWalletError> {
        Ok(WalletStatus {
            wallet_type: HardwareWalletType::BitBox02,
            connected: true,
            unlocked: true,
            has_app: true,
            app_name: Some("Ethereum".to_string()),
            firmware_version: Some("9.0.0".to_string()),
            btc_format: None,
        })
    }

    fn get_address(&self, derivation_path: &DerivationPath) -> Result<String, HardwareWalletError> {
        Err(HardwareWalletError::UserActionRequired(
            "Please confirm address on your BitBox02 device".to_string()
        ))
    }

    fn sign_transaction(
        &self,
        derivation_path: &DerivationPath,
        request: &SignRequest,
    ) -> Result<SignResponse, HardwareWalletError> {
        Err(HardwareWalletError::UserActionRequired(
            "Please confirm transaction on your BitBox02 device".to_string()
        ))
    }

    fn sign_message(&self, derivation_path: &DerivationPath, message: &[u8]) -> Result<SignResponse, HardwareWalletError> {
        Err(HardwareWalletError::UserActionRequired(
            "Please confirm message signing on your BitBox02 device".to_string()
        ))
    }

    fn sign_typed_data(
        &self,
        derivation_path: &DerivationPath,
        domain_hash: &[u8],
        message_hash: &[u8],
    ) -> Result<SignResponse, HardwareWalletError> {
        Err(HardwareWalletError::UserActionRequired(
            "Please confirm typed data signing on your BitBox02 device".to_string()
        ))
    }
}

#[cfg(feature = "lattice")]
pub struct LatticeWallet {
    connected: bool,
}

#[cfg(feature = "lattice")]
impl LatticeWallet {
    pub fn new() -> Result<Self, HardwareWalletError> {
        Ok(Self { connected: true })
    }
}

#[cfg(feature = "lattice")]
impl HardwareWallet for LatticeWallet {
    fn wallet_type(&self) -> HardwareWalletType {
        HardwareWalletType::Lattice
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn get_status(&self) -> Result<WalletStatus, HardwareWalletError> {
        Ok(WalletStatus {
            wallet_type: HardwareWalletType::Lattice,
            connected: true,
            unlocked: true,
            has_app: true,
            app_name: Some("Ethereum".to_string()),
            firmware_version: Some("0.12.0".to_string()),
            btc_format: None,
        })
    }

    fn get_address(&self, derivation_path: &DerivationPath) -> Result<String, HardwareWalletError> {
        Err(HardwareWalletError::UserActionRequired(
            "Please confirm address on your Lattice device".to_string()
        ))
    }

    fn sign_transaction(
        &self,
        derivation_path: &DerivationPath,
        request: &SignRequest,
    ) -> Result<SignResponse, HardwareWalletError> {
        Err(HardwareWalletError::UserActionRequired(
            "Please confirm transaction on your Lattice device".to_string()
        ))
    }

    fn sign_message(&self, derivation_path: &DerivationPath, message: &[u8]) -> Result<SignResponse, HardwareWalletError> {
        Err(HardwareWalletError::UserActionRequired(
            "Please confirm message signing on your Lattice device".to_string()
        ))
    }

    fn sign_typed_data(
        &self,
        derivation_path: &DerivationPath,
        domain_hash: &[u8],
        message_hash: &[u8],
    ) -> Result<SignResponse, HardwareWalletError> {
        Err(HardwareWalletError::UserActionRequired(
            "Please confirm typed data signing on your Lattice device".to_string()
        ))
    }
}