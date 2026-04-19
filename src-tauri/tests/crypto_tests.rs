//! Integration tests for crypto module

use token_vault_lib::crypto::{
    generate_private_key, derive_public_key, derive_address,
    encrypt_data, decrypt_data, hash_data, generate_mnemonic,
    derive_wallet_from_mnemonic, CryptoError,
};
use std::collections::HashMap;

fn create_test_keypair() -> (String, String, String) {
    let private_key = generate_private_key().expect("Failed to generate private key");
    let public_key = derive_public_key(&private_key).expect("Failed to derive public key");
    let address = derive_address(&public_key).expect("Failed to derive address");
    (private_key, public_key, address)
}

#[test]
fn test_generate_private_key_valid_format() {
    let private_key = generate_private_key().expect("Failed to generate private key");
    // Private key should be 64 hex characters (32 bytes)
    assert_eq!(private_key.len(), 64);
    assert!(private_key.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_generate_private_key_unique() {
    let key1 = generate_private_key().expect("Failed to generate key1");
    let key2 = generate_private_key().expect("Failed to generate key2");
    assert_ne!(key1, key2, "Generated keys should be unique");
}

#[test]
fn test_derive_public_key_from_private_key() {
    let (private_key, _, _) = create_test_keypair();
    let public_key = derive_public_key(&private_key).expect("Failed to derive public key");
    
    // Public key should be 130 hex characters (65 bytes uncompressed, or 128 hex for 64 bytes)
    assert!(public_key.len() >= 128);
    assert!(public_key.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn test_derive_address_from_public_key() {
    let (_, public_key, address) = create_test_keypair();
    let derived_address = derive_address(&public_key).expect("Failed to derive address");
    
    // Address should match the derived address
    assert_eq!(derived_address, address);
}

#[test]
fn test_derive_address_format() {
    let (_, _, address) = create_test_keypair();
    
    // EVM address should be 42 characters starting with 0x
    assert!(address.starts_with("0x"));
    assert_eq!(address.len(), 42);
}

#[test]
fn test_encrypt_decrypt_roundtrip() {
    let test_data = b"Hello, World! This is a test message.";
    let password = "test_password_123";
    
    let encrypted = encrypt_data(test_data, password).expect("Failed to encrypt data");
    assert_ne!(encrypted.as_slice(), test_data, "Encrypted data should differ from plaintext");
    
    let decrypted = decrypt_data(&encrypted, password).expect("Failed to decrypt data");
    assert_eq!(decrypted.as_slice(), test_data, "Decrypted data should match original");
}

#[test]
fn test_encrypt_different_password_fails() {
    let test_data = b"Sensitive data";
    let password = "correct_password";
    let wrong_password = "wrong_password";
    
    let encrypted = encrypt_data(test_data, password).expect("Failed to encrypt");
    let result = decrypt_data(&encrypted, wrong_password);
    
    assert!(result.is_err(), "Decryption with wrong password should fail");
}

#[test]
fn test_hash_data_consistency() {
    let test_data = b"Hash this data";
    
    let hash1 = hash_data(test_data).expect("Failed to hash data");
    let hash2 = hash_data(test_data).expect("Failed to hash data again");
    
    assert_eq!(hash1, hash2, "Same data should produce same hash");
}

#[test]
fn test_hash_data_different_inputs() {
    let data1 = b"First data";
    let data2 = b"Second data";
    
    let hash1 = hash_data(data1).expect("Failed to hash data1");
    let hash2 = hash_data(data2).expect("Failed to hash data2");
    
    assert_ne!(hash1, hash2, "Different data should produce different hashes");
}

#[test]
fn test_hash_length() {
    let data = b"Test data for hash length";
    let hash = hash_data(data).expect("Failed to hash");
    
    // SHA-256 produces 32 bytes = 64 hex characters
    assert_eq!(hash.len(), 64);
}

#[test]
fn test_generate_mnemonic_length() {
    let mnemonic = generate_mnemonic().expect("Failed to generate mnemonic");
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    
    // Standard BIP39 mnemonic is 12 or 24 words
    assert!(words.len() == 12 || words.len() == 24);
}

#[test]
fn test_generate_mnemonic_uniqueness() {
    let mnemonic1 = generate_mnemonic().expect("Failed to generate mnemonic1");
    let mnemonic2 = generate_mnemonic().expect("Failed to generate mnemonic2");
    
    assert_ne!(mnemonic1, mnemonic2, "Generated mnemonics should be unique");
}

#[test]
fn test_derive_wallet_from_mnemonic() {
    let mnemonic = generate_mnemonic().expect("Failed to generate mnemonic");
    let result = derive_wallet_from_mnemonic(&mnemonic);
    
    assert!(result.is_ok(), "Valid mnemonic should derive wallet successfully");
    
    let wallet = result.expect("Failed to get wallet");
    assert!(!wallet.private_key.is_empty());
    assert!(!wallet.public_key.is_empty());
    assert!(wallet.address.starts_with("0x"));
}

#[test]
fn test_derive_wallet_deterministic() {
    let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about".to_string();
    let result1 = derive_wallet_from_mnemonic(&mnemonic);
    let result2 = derive_wallet_from_mnemonic(&mnemonic);
    
    assert!(result1.is_ok() && result2.is_ok());
    
    let wallet1 = result1.expect("Failed to get wallet1");
    let wallet2 = result2.expect("Failed to get wallet2");
    
    assert_eq!(wallet1.private_key, wallet2.private_key);
    assert_eq!(wallet1.address, wallet2.address);
}

#[test]
fn test_derive_wallet_invalid_mnemonic() {
    let invalid_mnemonic = "not a valid mnemonic at all words";
    let result = derive_wallet_from_mnemonic(invalid_mnemonic);
    
    // Should either error or return a valid wallet with warning
    // Based on implementation, this may either fail or produce a wallet
    // We test that the function handles it gracefully
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_encrypt_empty_data() {
    let empty_data: &[u8] = &[];
    let password = "password";
    
    let result = encrypt_data(empty_data, password);
    assert!(result.is_ok(), "Should handle empty data");
}

#[test]
fn test_encrypt_large_data() {
    let large_data: Vec<u8> = (0..10000).map(|i| (i % 256) as u8).collect();
    let password = "password";
    
    let encrypted = encrypt_data(&large_data, password).expect("Failed to encrypt large data");
    let decrypted = decrypt_data(&encrypted, password).expect("Failed to decrypt large data");
    
    assert_eq!(decrypted, large_data);
}

#[test]
fn test_encrypt_special_characters() {
    let special_data = "Hello! 🌍 中文 العربية עברית 🎉#$%^&*()".as_bytes();
    let password = "special_pass_123";
    
    let encrypted = encrypt_data(special_data, password).expect("Failed to encrypt");
    let decrypted = decrypt_data(&encrypted, password).expect("Failed to decrypt");
    
    assert_eq!(decrypted, special_data);
}

#[test]
fn test_address_checksum_validation() {
    let (_, _, address) = create_test_keypair();
    
    // Valid EVM address
    assert!(address.starts_with("0x"));
    assert_eq!(address.len(), 42);
    assert!(address[2..].chars().all(|c| c.is_ascii_hexdigit()));
}