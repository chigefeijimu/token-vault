use sha2::{Digest, Sha256};
use serde::{Deserialize, Serialize};
use base64::{engine::general_purpose, Engine as _};
use thiserror::Error;
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Invalid data: {0}")]
    InvalidData(String),
    #[error("Mnemonic error: {0}")]
    MnemonicError(String),
}

// Simple in-memory store for encrypted keys (in production, use proper secure storage)
lazy_static::lazy_static! {
    static ref KEY_STORE: Mutex<HashMap<String, Vec<u8>>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
#[allow(dead_code)]
pub struct KeyPair {
    pub private_key: String,
    pub public_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    pub ciphertext: String,
    pub nonce: String,
}

// ============== Mnemonic Functions ==============

const MNEMONIC_WORDS: &[&str] = &[
    "abandon", "ability", "able", "about", "above", "absent", "absorb", "abstract", "absurd", "abuse",
    "access", "accident", "account", "accuse", "achieve", "acid", "acoustic", "acquire", "across", "act",
    "action", "actor", "actress", "actual", "adapt", "add", "addict", "address", "adjust", "admit",
    "adult", "advance", "advice", "aerobic", "affair", "afford", "afraid", "again", "age", "agent",
    "agree", "ahead", "aim", "air", "airport", "aisle", "alarm", "album", "alcohol", "alert",
    "alien", "all", "alley", "allow", "almost", "alone", "alpha", "already", "also", "alter",
    "always", "amateur", "amazing", "among", "amount", "amused", "analyst", "anchor", "ancient", "anger",
    "angle", "angry", "animal", "ankle", "announce", "annual", "another", "answer", "antenna", "antique",
    "anxiety", "any", "apart", "apology", "appear", "apple", "approve", "april", "arch", "arctic",
    "area", "arena", "argue", "arm", "armed", "armor", "army", "around", "arrange", "arrest",
];

pub fn generate_mnemonic() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let words: Vec<&str> = (0..12).map(|_| MNEMONIC_WORDS[rng.gen_range(0..MNEMONIC_WORDS.len())]).collect();
    words.join(" ")
}

pub fn validate_mnemonic(mnemonic: &str) -> bool {
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    if words.len() != 12 {
        return false;
    }
    words.iter().all(|w| MNEMONIC_WORDS.contains(w))
}

pub fn derive_private_key_from_mnemonic(mnemonic: &str, _derivation_path: &str) -> Result<Vec<u8>, CryptoError> {
    if !validate_mnemonic(mnemonic) {
        return Err(CryptoError::MnemonicError("Invalid mnemonic".to_string()));
    }
    // Simple deterministic derivation (in production, use proper BIP39/BIP32)
    let mut hasher = Sha256::new();
    hasher.update(mnemonic.as_bytes());
    let result = hasher.finalize();
    Ok(result.to_vec())
}

pub fn derive_eth_address(private_key_bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(private_key_bytes);
    let hash = hasher.finalize();
    // Take last 20 bytes for Ethereum address
    format!("0x{}", hex::encode(&hash[12..]))
}

// ============== Password Hashing ==============

pub fn generate_salt() -> Vec<u8> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    (0..16).map(|_| rng.gen()).collect()
}

pub fn hash_password(password: &str, salt: &[u8]) -> Vec<u8> {
    use argon2::{Argon2, PasswordHasher, password_hash::Salt};
    use base64::{engine::general_purpose, Engine as _};

    let argon2 = Argon2::default();
    let salt_b64 = general_purpose::STANDARD.encode(salt);
    let salt = Salt::from_b64(&salt_b64).unwrap();
    let hash = argon2.hash_password(password.as_bytes(), salt).unwrap();
    hash.hash.unwrap().as_bytes().to_vec()
}

pub fn verify_password(password: &str, salt: &[u8], expected_hash: &[u8]) -> bool {
    let computed = hash_password(password, salt);
    computed == expected_hash
}

// ============== Encryption Functions ==============

#[tauri::command]
pub fn generate_private_key() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    hex::encode(bytes)
}

#[tauri::command]
pub fn derive_public_key(private_key: &str) -> Result<String, String> {
    let bytes = hex::decode(&private_key[2..])
        .map_err(|e| format!("Invalid private key: {}", e))?;
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let result = hasher.finalize();
    Ok(format!("0x{}", hex::encode(result)))
}

#[tauri::command]
pub fn public_key_to_address(public_key: &str) -> String {
    let key_str = public_key.strip_prefix("0x").unwrap_or(public_key);
    let bytes = match hex::decode(key_str) {
        Ok(b) => b,
        Err(_) => {
            let mut hasher = Sha256::new();
            hasher.update(public_key.as_bytes());
            hasher.finalize().to_vec()
        }
    };
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let hash = hasher.finalize();
    format!("0x{}", hex::encode(&hash[12..]))
}

// NOTE: sign_data moved to transaction.rs (real implementation)

#[tauri::command]
pub fn encrypt_data(plaintext: String, key: String) -> Result<EncryptedData, String> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let nonce: Vec<u8> = (0..12).map(|_| rng.gen()).collect();

    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let key_hash = hasher.finalize();

    // Simple XOR encryption for demo (use proper AES in production)
    let mut ciphertext = Vec::new();
    for (i, byte) in plaintext.bytes().enumerate() {
        let key_byte = key_hash[i % key_hash.len()];
        ciphertext.push(byte ^ key_byte);
    }

    Ok(EncryptedData {
        ciphertext: general_purpose::STANDARD.encode(&ciphertext),
        nonce: general_purpose::STANDARD.encode(&nonce),
    })
}

#[tauri::command]
pub fn decrypt_data(encrypted: EncryptedData, key: String) -> Result<String, String> {
    let ciphertext = general_purpose::STANDARD
        .decode(&encrypted.ciphertext)
        .map_err(|e| format!("Failed to decode ciphertext: {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(key.as_bytes());
    let key_hash = hasher.finalize();

    let mut plaintext = Vec::new();
    for (i, byte) in ciphertext.iter().enumerate() {
        let key_byte = key_hash[i % key_hash.len()];
        plaintext.push(byte ^ key_byte);
    }

    String::from_utf8(plaintext).map_err(|e| format!("Failed to decode plaintext: {}", e))
}

#[tauri::command]
pub fn hash_data(data: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    let result = hasher.finalize();
    format!("0x{}", hex::encode(result))
}

#[tauri::command]
pub fn validate_mnemonic_cmd(mnemonic: String) -> bool {
    validate_mnemonic(&mnemonic)
}

#[tauri::command]
pub fn generate_mnemonic_cmd() -> String {
    generate_mnemonic()
}

// ============== Tests ==============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_mnemonic() {
        let mnemonic = generate_mnemonic();
        assert_eq!(mnemonic.split_whitespace().count(), 12);
        assert!(validate_mnemonic(&mnemonic));
    }

    #[test]
    fn test_validate_mnemonic() {
        // Valid 12-word mnemonic (all words from BIP39 list)
        assert!(validate_mnemonic("abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon"));
        // Invalid: contains words not in the list
        assert!(!validate_mnemonic("notaword notaword notaword notaword notaword notaword notaword notaword notaword notaword notaword notaword"));
        // Invalid: too few words
        assert!(!validate_mnemonic("abandon about"));
    }

    #[test]
    fn test_encrypt_decrypt() {
        let plaintext = "Hello, World!";
        let key = "my_secret_key";
        let encrypted = encrypt_data(plaintext.to_string(), key.to_string()).unwrap();
        let decrypted = decrypt_data(encrypted, key.to_string()).unwrap();
        assert_eq!(decrypted, plaintext);
    }
}

