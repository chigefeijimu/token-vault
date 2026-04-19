//! Integration tests for wallet module

use token_vault_lib::wallet::{
    create_wallet, list_wallets, get_wallet, delete_wallet,
    WalletInfo, WalletError, validate_address,
};
use token_vault_lib::storage::{self, StorageError};
use std::path::PathBuf;

fn get_test_storage_path() -> PathBuf {
    let temp_dir = std::env::temp_dir();
    let test_id = uuid::Uuid::new_v4().to_string();
    temp_dir.join(format!("token_vault_test_{}", test_id))
}

fn cleanup_test_storage(path: &PathBuf) {
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn test_validate_address_valid() {
    // Valid checksummed address
    let valid_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1";
    assert!(validate_address(valid_address).is_ok());
}

#[test]
fn test_validate_address_invalid_prefix() {
    let invalid_address = "742d35Cc6634C0532925a3b844Bc9e7595f0bEb1";
    assert!(validate_address(invalid_address).is_err());
}

#[test]
fn test_validate_address_invalid_length() {
    let short_address = "0x742d35Cc6634";
    assert!(validate_address(short_address).is_err());
    
    let long_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb172d35Cc6634C0532925a3b844Bc9e7595f0bEb1";
    assert!(validate_address(long_address).is_err());
}

#[test]
fn test_validate_address_invalid_characters() {
    let invalid_address = "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEzz";
    assert!(validate_address(invalid_address).is_err());
}

#[test]
fn test_create_wallet_returns_valid_info() {
    let storage_path = get_test_storage_path();
    let wallet = create_wallet("Test Wallet", &storage_path).expect("Failed to create wallet");
    
    assert!(!wallet.id.is_empty());
    assert_eq!(wallet.name, "Test Wallet");
    assert!(wallet.address.starts_with("0x"));
    assert!(wallet.address.len() == 42);
    
    cleanup_test_storage(&storage_path);
}

#[test]
fn test_create_wallet_unique_ids() {
    let storage_path1 = get_test_storage_path();
    let storage_path2 = get_test_storage_path();
    
    let wallet1 = create_wallet("Wallet 1", &storage_path1).expect("Failed to create wallet1");
    let wallet2 = create_wallet("Wallet 2", &storage_path2).expect("Failed to create wallet2");
    
    assert_ne!(wallet1.id, wallet2.id);
    
    cleanup_test_storage(&storage_path1);
    cleanup_test_storage(&storage_path2);
}

#[test]
fn test_list_wallets_empty() {
    let storage_path = get_test_storage_path();
    let wallets = list_wallets(&storage_path).expect("Failed to list wallets");
    
    assert!(wallets.is_empty());
    
    cleanup_test_storage(&storage_path);
}

#[test]
fn test_create_and_list_wallets() {
    let storage_path = get_test_storage_path();
    
    let wallet1 = create_wallet("Wallet 1", &storage_path).expect("Failed to create wallet1");
    let wallet2 = create_wallet("Wallet 2", &storage_path).expect("Failed to create wallet2");
    
    let wallets = list_wallets(&storage_path).expect("Failed to list wallets");
    
    assert_eq!(wallets.len(), 2);
    assert!(wallets.iter().any(|w| w.id == wallet1.id));
    assert!(wallets.iter().any(|w| w.id == wallet2.id));
    
    cleanup_test_storage(&storage_path);
}

#[test]
fn test_get_wallet_by_id() {
    let storage_path = get_test_storage_path();
    
    let created = create_wallet("My Wallet", &storage_path).expect("Failed to create wallet");
    let retrieved = get_wallet(&created.id, &storage_path).expect("Failed to get wallet");
    
    assert_eq!(retrieved.id, created.id);
    assert_eq!(retrieved.name, created.name);
    assert_eq!(retrieved.address, created.address);
    
    cleanup_test_storage(&storage_path);
}

#[test]
fn test_get_wallet_not_found() {
    let storage_path = get_test_storage_path();
    
    let result = get_wallet("non-existent-id", &storage_path);
    assert!(result.is_err());
    
    cleanup_test_storage(&storage_path);
}

#[test]
fn test_delete_wallet() {
    let storage_path = get_test_storage_path();
    
    let wallet = create_wallet("To Delete", &storage_path).expect("Failed to create wallet");
    let wallet_id = wallet.id.clone();
    
    delete_wallet(&wallet_id, &storage_path).expect("Failed to delete wallet");
    
    let result = get_wallet(&wallet_id, &storage_path);
    assert!(result.is_err());
    
    cleanup_test_storage(&storage_path);
}

#[test]
fn test_delete_wallet_not_found() {
    let storage_path = get_test_storage_path();
    
    let result = delete_wallet("non-existent-id", &storage_path);
    assert!(result.is_err());
    
    cleanup_test_storage(&storage_path);
}

#[test]
fn test_wallet_persistence() {
    let storage_path = get_test_storage_path();
    
    let wallet = create_wallet("Persistent Wallet", &storage_path).expect("Failed to create wallet");
    let wallet_id = wallet.id.clone();
    
    // Drop the wallet variable
    drop(wallet);
    
    // Retrieve the wallet again - should still exist
    let retrieved = get_wallet(&wallet_id, &storage_path).expect("Wallet should persist");
    assert_eq!(retrieved.name, "Persistent Wallet");
    
    cleanup_test_storage(&storage_path);
}

#[test]
fn test_multiple_wallets_different_chains() {
    let storage_path = get_test_storage_path();
    
    // Create multiple wallets
    let wallet1 = create_wallet("ETH Wallet", &storage_path).expect("Failed to create ETH wallet");
    let wallet2 = create_wallet("BNB Wallet", &storage_path).expect("Failed to create BNB wallet");
    
    // Both should have valid addresses
    assert!(wallet1.address.starts_with("0x"));
    assert!(wallet2.address.starts_with("0x"));
    
    // Addresses should be different (statistically improbable to be the same)
    assert_ne!(wallet1.address, wallet2.address);
    
    cleanup_test_storage(&storage_path);
}

#[test]
fn test_wallet_info_fields() {
    let storage_path = get_test_storage_path();
    let wallet = create_wallet("Full Info Test", &storage_path).expect("Failed to create wallet");
    
    assert!(!wallet.id.is_empty());
    assert!(!wallet.name.is_empty());
    assert!(!wallet.address.is_empty());
    assert!(wallet.created_at > 0);
    assert!(wallet.encrypted_mnemonic.is_some() || wallet.encrypted_private_key.is_some());
    
    cleanup_test_storage(&storage_path);
}