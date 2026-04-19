// Ethereum UTC/JSON Keystore format implementation
// Compatible with standard Ethereum keystore files (e.g., from geth, parity, metamask)

use crate::errors::AppError;
use crate::crypto;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sha3::Keccak256;
use aes::Aes256Ctr;
use ctr::CtrCore;
use hmac::{Hmac, Mac};
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use std::fs;
use std::path::Path;
use thiserror::Error;

type Aes256CtrBE = CtrCore<Aes256Ctr, u32>;
type HmacSha256 = Hmac<Sha256>;

#[derive(Error, Debug)]
pub enum KeystoreError {
    #[error("Failed to create keystore: {0}")]
    CreateFailed(String),
    #[error("Failed to read keystore: {0}")]
    ReadFailed(String),
    #[error("Invalid keystore format: {0}")]
    InvalidFormat(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Invalid password")]
    InvalidPassword,
    #[error("Path error: {0}")]
    PathError(String),
}

// Standard Ethereum keystore file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystoreFile {
    pub crypto: KeystoreCrypto,
    pub id: String,
    pub version: u32,
    #[serde(rename = "address")]
    pub address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystoreCrypto {
    pub cipher: String,
    pub cipherparams: CipherParams,
    pub ciphertext: String,
    pub kdf: String,
    pub kdfparams: KdfParams,
    pub mac: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CipherParams {
    pub iv: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KdfParams {
    pub dklen: u32,
    pub salt: String,
    pub p: u32,
    pub r: u32,
    pub n: u32,
    pub lin: Option<u32>,
    pub olim: Option<u32>,
}

impl KeystoreFile {
    const CURRENT_VERSION: u32 = 3;
    const CIPHER: &'static str = "aes-128-ctr";
    const KDF: &'static str = "pbkdf2";
    const MAC_CHECK_BYTES: usize = 16;

    /// Create a new keystore file from private key
    pub fn new(
        private_key: &[u8],
        password: &str,
        address: &str,
    ) -> Result<Self, KeystoreError> {
        let mut rng = rand::thread_rng();
        
        // Generate random bytes for salt and IV
        let mut salt = [0u8; 32];
        let mut iv = [0u8; 16];
        rng.fill_bytes(&mut salt);
        rng.fill_bytes(&mut iv);

        // Generate encryption key using PBKDF2
        let derived_key = Self::derive_key(password, &salt)?;
        
        // Split derived key: first 16 bytes for AES key, next 16 bytes for MAC
        let (aes_key, mac_key) = derived_key.split_at(16);
        
        // Encrypt private key
        let ciphertext = Self::encrypt_aes_ctr(private_key, aes_key, &iv)?;
        
        // Calculate MAC
        let mac = Self::calculate_mac(mac_key, &ciphertext)?;
        
        // Generate UUID
        let id = uuid::Uuid::new_v4().to_string();

        let crypto = KeystoreCrypto {
            cipher: Self::CIPHER.to_string(),
            cipherparams: CipherParams {
                iv: hex::encode(iv),
            },
            ciphertext: hex::encode(&ciphertext),
            kdf: Self::KDF.to_string(),
            kdfparams: KdfParams {
                dklen: 32,
                salt: hex::encode(salt),
                p: 1,
                r: 8,
                n: 262144,
                lin: Some(1),
                olim: None,
            },
            mac: hex::encode(mac),
        };

        Ok(Self {
            crypto,
            id,
            version: Self::CURRENT_VERSION,
            address: address.to_lowercase().trim_start_matches("0x").to_string(),
        })
    }

    /// Decrypt keystore file to recover private key
    pub fn decrypt(&self, password: &str) -> Result<Vec<u8>, KeystoreError> {
        // Validate format
        if self.crypto.cipher != Self::CIPHER {
            return Err(KeystoreError::InvalidFormat(format!(
                "Unsupported cipher: {}",
                self.crypto.cipher
            )));
        }

        if self.crypto.kdf != Self::KDF {
            return Err(KeystoreError::InvalidFormat(format!(
                "Unsupported KDF: {}",
                self.crypto.kdf
            )));
        }

        // Parse hex values
        let salt = hex::decode(&self.crypto.kdfparams.salt)
            .map_err(|e| KeystoreError::InvalidFormat(format!("Invalid salt hex: {}", e)))?;
        
        let iv = hex::decode(&self.crypto.cipherparams.iv)
            .map_err(|e| KeystoreError::InvalidFormat(format!("Invalid IV hex: {}", e)))?;
        
        let ciphertext = hex::decode(&self.crypto.ciphertext)
            .map_err(|e| KeystoreError::InvalidFormat(format!("Invalid ciphertext hex: {}", e)))?;
        
        let expected_mac = hex::decode(&self.crypto.mac)
            .map_err(|e| KeystoreError::InvalidFormat(format!("Invalid MAC hex: {}", e)))?;

        // Derive key from password
        let derived_key = Self::derive_key_with_params(password, &salt, &self.crypto.kdfparams)?;
        let (aes_key, mac_key) = derived_key.split_at(16);

        // Verify MAC
        let computed_mac = Self::calculate_mac(mac_key, &ciphertext)?;
        if computed_mac[..Self::MAC_CHECK_BYTES] != expected_mac[..Self::MAC_CHECK_BYTES] {
            return Err(KeystoreError::InvalidPassword);
        }

        // Decrypt private key
        Self::decrypt_aes_ctr(&ciphertext, aes_key, &iv)
    }

    /// Derive key using PBKDF2 with default parameters
    fn derive_key(password: &str, salt: &[u8]) -> Result<[u8; 32], KeystoreError> {
        let mut derived_key = [0u8; 32];
        pbkdf2_hmac::<Sha256>(
            password.as_bytes(),
            salt,
            262144,
            &mut derived_key,
        );
        Ok(derived_key)
    }

    /// Derive key using PBKDF2 with custom parameters
    fn derive_key_with_params(
        password: &str,
        salt: &[u8],
        params: &KdfParams,
    ) -> Result<[u8; 32], KeystoreError> {
        let mut derived_key = [0u8; 32];
        let iterations = params.n as u32;
        pbkdf2_hmac::<Sha256>(
            password.as_bytes(),
            salt,
            iterations,
            &mut derived_key,
        );
        Ok(derived_key)
    }

    /// Encrypt data using AES-128-CTR
    fn encrypt_aes_ctr(data: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>, KeystoreError> {
        use aes::Aes128;
        use ctr::Ctr;
        use ctr::cipher::{StreamCipher, KeyInit};
        
        let cipher = Aes128::new_from_slice(key)
            .map_err(|e| KeystoreError::EncryptionFailed(e.to_string()))?;
        let mut ctr = Ctr::<Aes128, u32>::new(cipher, iv.into())
            .map_err(|e| KeystoreError::EncryptionFailed(e.to_string()))?;
        
        let mut result = data.to_vec();
        ctr.apply_keystream(&mut result);
        Ok(result)
    }

    /// Decrypt data using AES-128-CTR
    fn decrypt_aes_ctr(ciphertext: &[u8], key: &[u8], iv: &[u8]) -> Result<Vec<u8>, KeystoreError> {
        use aes::Aes128;
        use ctr::Ctr;
        use ctr::cipher::{StreamCipher, KeyInit};
        
        let cipher = Aes128::new_from_slice(key)
            .map_err(|e| KeystoreError::DecryptionFailed(e.to_string()))?;
        let mut ctr = Ctr::<Aes128, u32>::new(cipher, iv.into())
            .map_err(|e| KeystoreError::DecryptionFailed(e.to_string()))?;
        
        let mut result = ciphertext.to_vec();
        ctr.apply_keystream(&mut result);
        Ok(result)
    }

    /// Calculate MAC for verification
    fn calculate_mac(mac_key: &[u8], ciphertext: &[u8]) -> Result<[u8; 32], KeystoreError> {
        let mut mac = HmacSha256::new_from_slice(mac_key)
            .map_err(|e| KeystoreError::EncryptionFailed(e.to_string()))?;
        mac.update(&ciphertext);
        let result = mac.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result.into_bytes()[..32]);
        Ok(hash)
    }

    /// Get the address from keystore
    pub fn get_address(&self) -> String {
        format!("0x{}", self.address)
    }
}

/// Save keystore file to disk
pub fn save_keystore(keystore: &KeystoreFile, path: &Path) -> Result<(), KeystoreError> {
    let json = serde_json::to_string_pretty(keystore)
        .map_err(|e| KeystoreError::CreateFailed(e.to_string()))?;
    
    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| KeystoreError::PathError(e.to_string()))?;
    }
    
    fs::write(path, json)
        .map_err(|e| KeystoreError::CreateFailed(e.to_string()))?;
    
    Ok(())
}

/// Load keystore file from disk
pub fn load_keystore(path: &Path) -> Result<KeystoreFile, KeystoreError> {
    let json = fs::read_to_string(path)
        .map_err(|e| KeystoreError::ReadFailed(e.to_string()))?;
    
    serde_json::from_str(&json)
        .map_err(|e| KeystoreError::InvalidFormat(e.to_string()))
}

/// List all keystore files in a directory
pub fn list_keystores(dir: &Path) -> Result<Vec<String>, KeystoreError> {
    let mut addresses = Vec::new();
    
    if !dir.exists() {
        return Ok(addresses);
    }
    
    for entry in fs::read_dir(dir)
        .map_err(|e| KeystoreError::ReadFailed(e.to_string()))? 
    {
        let entry = entry.map_err(|e| KeystoreError::ReadFailed(e.to_string()))?;
        let path = entry.path();
        
        if path.is_file() {
            if let Ok(keystore) = load_keystore(&path) {
                addresses.push(keystore.get_address());
            }
        }
    }
    
    Ok(addresses)
}

/// Import keystore file (copy to app keystore directory)
pub fn import_keystore(
    source_path: &Path,
    dest_dir: &Path,
    password: &str,
    new_password: Option<&str>,
) -> Result<String, KeystoreError> {
    let keystore = load_keystore(source_path)?;
    
    // Decrypt with source password
    let private_key = keystore.decrypt(password)?;
    
    // Re-encrypt with new password if provided
    let final_keystore = if let Some(new_pwd) = new_password {
        let address = keystore.get_address();
        KeystoreFile::new(&private_key, new_pwd, &address)?
    } else {
        keystore
    };
    
    // Save to destination
    let filename = format!("UTC--{}--{}", 
        chrono::Utc::now().format("%Y-%m-%dT%H-%M-%S.%f"),
        final_keystore.address
    );
    let dest_path = dest_dir.join(&filename);
    save_keystore(&final_keystore, &dest_path)?;
    
    Ok(final_keystore.get_address())
}