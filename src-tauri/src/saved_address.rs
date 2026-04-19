// Saved addresses management module for storing and managing frequently used addresses

use crate::errors::AppError;
use crate::storage;
use crate::wallet::WalletInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use uuid::Uuid;
use web3::types::Address;

lazy_static::lazy_static! {
    static ref SAVED_ADDRESSES: Mutex<HashMap<String, SavedAddressEntry>> = Mutex::new(HashMap::new());
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedAddressEntry {
    pub id: String,
    pub name: String,
    pub address: String,
    pub chain_id: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_favorite: bool,
    pub tags: Vec<String>,
    pub memo: Option<String>,
}

impl SavedAddressEntry {
    pub fn new(name: String, address: String, chain_id: u64, tags: Vec<String>, memo: Option<String>) -> Self {
        let now = chrono::Utc::now().timestamp_millis() as u64;
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            address: address.to_lowercase(),
            chain_id,
            created_at: now,
            updated_at: now,
            is_favorite: false,
            tags,
            memo,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedAddressResponse {
    pub id: String,
    pub name: String,
    pub address: String,
    pub chain_id: u64,
    pub created_at: u64,
    pub updated_at: u64,
    pub is_favorite: bool,
    pub tags: Vec<String>,
    pub memo: Option<String>,
}

impl From<SavedAddressEntry> for SavedAddressResponse {
    fn from(entry: SavedAddressEntry) -> Self {
        Self {
            id: entry.id,
            name: entry.name,
            address: entry.address,
            chain_id: entry.chain_id,
            created_at: entry.created_at,
            updated_at: entry.updated_at,
            is_favorite: entry.is_favorite,
            tags: entry.tags,
            memo: entry.memo,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressValidationResult {
    pub is_valid: bool,
    pub address: String,
    pub checksum_address: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedAddressInput {
    pub name: String,
    pub address: String,
    pub chain_id: u64,
    pub tags: Option<Vec<String>>,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedAddressUpdate {
    pub id: String,
    pub name: Option<String>,
    pub address: Option<String>,
    pub chain_id: Option<u64>,
    pub is_favorite: Option<bool>,
    pub tags: Option<Vec<String>>,
    pub memo: Option<String>,
}

/// Validate an EVM address
pub fn validate_address(address: &str) -> AddressValidationResult {
    let address = address.trim();
    
    // Check basic format
    if !address.starts_with("0x") {
        return AddressValidationResult {
            is_valid: false,
            address: address.to_string(),
            checksum_address: None,
            error: Some("Address must start with 0x".to_string()),
        };
    }
    
    let hex_part = &address[2..];
    if hex_part.len() != 40 {
        return AddressValidationResult {
            is_valid: false,
            address: address.to_string(),
            checksum_address: None,
            error: Some("Address must be 40 hex characters after 0x".to_string()),
        };
    }
    
    // Check if all characters are valid hex
    if !hex_part.chars().all(|c| c.is_ascii_hexdigit()) {
        return AddressValidationResult {
            is_valid: false,
            address: address.to_string(),
            checksum_address: None,
            error: Some("Address contains invalid hex characters".to_string()),
        };
    }
    
    // Try to parse as web3 Address (validates checksum)
    match address.parse::<Address>() {
        Ok(addr) => AddressValidationResult {
            is_valid: true,
            address: address.to_lowercase(),
            checksum_address: Some(format!("0x{:x}", addr)),
            error: None,
        },
        Err(e) => AddressValidationResult {
            is_valid: false,
            address: address.to_string(),
            checksum_address: None,
            error: Some(format!("Invalid address: {}", e)),
        },
    }
}

/// Check if an address already exists in saved addresses
pub fn address_exists(address: &str, chain_id: u64) -> bool {
    let addresses = SAVED_ADDRESSES.lock().unwrap();
    addresses.values().any(|a| a.address == address.to_lowercase() && a.chain_id == chain_id)
}

/// Get all saved addresses
#[tauri::command]
pub fn get_saved_addresses() -> Result<Vec<SavedAddressResponse>, String> {
    let addresses = SAVED_ADDRESSES.lock().map_err(|e| e.to_string())?;
    let result: Vec<SavedAddressResponse> = addresses
        .values()
        .cloned()
        .map(SavedAddressResponse::from)
        .collect();
    Ok(result)
}

/// Get saved addresses by chain ID
#[tauri::command]
pub fn get_saved_addresses_by_chain(chain_id: u64) -> Result<Vec<SavedAddressResponse>, String> {
    let addresses = SAVED_ADDRESSES.lock().map_err(|e| e.to_string())?;
    let result: Vec<SavedAddressResponse> = addresses
        .values()
        .filter(|a| a.chain_id == chain_id)
        .cloned()
        .map(SavedAddressResponse::from)
        .collect();
    Ok(result)
}

/// Get favorite saved addresses
#[tauri::command]
pub fn get_favorite_saved_addresses() -> Result<Vec<SavedAddressResponse>, String> {
    let addresses = SAVED_ADDRESSES.lock().map_err(|e| e.to_string())?;
    let result: Vec<SavedAddressResponse> = addresses
        .values()
        .filter(|a| a.is_favorite)
        .cloned()
        .map(SavedAddressResponse::from)
        .collect();
    Ok(result)
}

/// Get a single saved address by ID
#[tauri::command]
pub fn get_saved_address(id: String) -> Result<SavedAddressResponse, String> {
    let addresses = SAVED_ADDRESSES.lock().map_err(|e| e.to_string())?;
    addresses
        .get(&id)
        .cloned()
        .map(SavedAddressResponse::from)
        .ok_or_else(|| "Address not found".to_string())
}

/// Search saved addresses by name or address
#[tauri::command]
pub fn search_saved_addresses(query: String) -> Result<Vec<SavedAddressResponse>, String> {
    let addresses = SAVED_ADDRESSES.lock().map_err(|e| e.to_string())?;
    let query_lower = query.to_lowercase();
    let result: Vec<SavedAddressResponse> = addresses
        .values()
        .filter(|a| {
            a.name.to_lowercase().contains(&query_lower)
                || a.address.contains(&query_lower)
                || a.tags.iter().any(|t| t.to_lowercase().contains(&query_lower))
        })
        .cloned()
        .map(SavedAddressResponse::from)
        .collect();
    Ok(result)
}

/// Add a new saved address
#[tauri::command]
pub fn add_saved_address(input: SavedAddressInput) -> Result<SavedAddressResponse, String> {
    // Validate address format
    let validation = validate_address(&input.address);
    if !validation.is_valid {
        return Err(validation.error.unwrap_or_else(|| "Invalid address".to_string()));
    }
    
    let address = validation.checksum_address.unwrap_or(input.address.to_lowercase());
    
    // Check for duplicate
    if address_exists(&address, input.chain_id) {
        return Err("Address already saved on this chain".to_string());
    }
    
    let entry = SavedAddressEntry::new(
        input.name,
        address,
        input.chain_id,
        input.tags.unwrap_or_default(),
        input.memo,
    );
    
    let id = entry.id.clone();
    let response = SavedAddressResponse::from(entry.clone());
    
    let mut addresses = SAVED_ADDRESSES.lock().map_err(|e| e.to_string())?;
    addresses.insert(id, entry);
    
    Ok(response)
}

/// Update a saved address
#[tauri::command]
pub fn update_saved_address(update: SavedAddressUpdate) -> Result<SavedAddressResponse, String> {
    let mut addresses = SAVED_ADDRESSES.lock().map_err(|e| e.to_string())?;
    
    let entry = addresses
        .get_mut(&update.id)
        .ok_or_else(|| "Address not found".to_string())?;
    
    // Validate new address if provided
    if let Some(ref new_address) = update.address {
        let validation = validate_address(new_address);
        if !validation.is_valid {
            return Err(validation.error.unwrap_or_else(|| "Invalid address".to_string()));
        }
        
        let address = validation.checksum_address.unwrap_or(new_address.to_lowercase());
        
        // Check for duplicate (excluding current entry)
        let duplicate_exists = addresses.values().any(|a| {
            a.address == address && a.chain_id == entry.chain_id && a.id != update.id
        });
        
        if duplicate_exists {
            return Err("Address already saved on this chain".to_string());
        }
        
        entry.address = address;
    }
    
    // Update fields
    if let Some(name) = update.name {
        entry.name = name;
    }
    if let Some(chain_id) = update.chain_id {
        entry.chain_id = chain_id;
    }
    if let Some(is_favorite) = update.is_favorite {
        entry.is_favorite = is_favorite;
    }
    if let Some(tags) = update.tags {
        entry.tags = tags;
    }
    if update.memo.is_some() {
        entry.memo = update.memo;
    }
    
    entry.updated_at = chrono::Utc::now().timestamp_millis() as u64;
    
    Ok(SavedAddressResponse::from(entry.clone()))
}

/// Delete a saved address
#[tauri::command]
pub fn delete_saved_address(id: String) -> Result<(), String> {
    let mut addresses = SAVED_ADDRESSES.lock().map_err(|e| e.to_string())?;
    addresses.remove(&id).ok_or_else(|| "Address not found".to_string())?;
    Ok(())
}

/// Toggle favorite status
#[tauri::command]
pub fn toggle_saved_address_favorite(id: String) -> Result<SavedAddressResponse, String> {
    let mut addresses = SAVED_ADDRESSES.lock().map_err(|e| e.to_string())?;
    
    let entry = addresses
        .get_mut(&id)
        .ok_or_else(|| "Address not found".to_string())?;
    
    entry.is_favorite = !entry.is_favorite;
    entry.updated_at = chrono::Utc::now().timestamp_millis() as u64;
    
    Ok(SavedAddressResponse::from(entry.clone()))
}

/// Export all saved addresses
#[tauri::command]
pub fn export_saved_addresses() -> Result<String, String> {
    let addresses = SAVED_ADDRESSES.lock().map_err(|e| e.to_string())?;
    let list: Vec<SavedAddressResponse> = addresses
        .values()
        .cloned()
        .map(SavedAddressResponse::from)
        .collect();
    serde_json::to_string_pretty(&list).map_err(|e| e.to_string())
}

/// Import saved addresses from JSON
#[tauri::command]
pub fn import_saved_addresses(json_data: String, merge: bool) -> Result<Vec<SavedAddressResponse>, String> {
    let imported: Vec<SavedAddressEntry> = serde_json::from_str(&json_data)
        .map_err(|e| format!("Invalid JSON format: {}", e))?;
    
    let mut addresses = SAVED_ADDRESSES.lock().map_err(|e| e.to_string())?;
    
    if !merge {
        addresses.clear();
    }
    
    let mut imported_list = Vec::new();
    for entry in imported {
        // Validate each address
        let validation = validate_address(&entry.address);
        if !validation.is_valid {
            continue;
        }
        
        let address = validation.checksum_address.unwrap_or(entry.address.to_lowercase());
        
        // Skip duplicates
        let duplicate_exists = addresses.values().any(|a| a.address == address && a.chain_id == entry.chain_id);
        if duplicate_exists {
            continue;
        }
        
        let id = entry.id.clone();
        addresses.insert(id.clone(), entry.clone());
        imported_list.push(SavedAddressResponse::from(entry));
    }
    
    Ok(imported_list)
}

/// Get saved addresses count
#[tauri::command]
pub fn get_saved_addresses_count() -> Result<usize, String> {
    let addresses = SAVED_ADDRESSES.lock().map_err(|e| e.to_string())?;
    Ok(addresses.len())
}

###APPEND###
src-tauri/src/lib.rs