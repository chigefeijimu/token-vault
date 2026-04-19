

// Address Book Module - Save and manage commonly used addresses

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AddressBookError {
    #[error("Address not found: {0}")]
    NotFound(String),
    #[error("Address already exists: {0}")]
    AlreadyExists(String),
    #[error("Invalid address format: {0}")]
    InvalidAddress(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedAddress {
    pub address: String,
    pub label: String,
    pub chain_id: Option<u64>,
    pub notes: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AddressBook {
    pub addresses: HashMap<String, SavedAddress>,
}

impl AddressBook {
    pub fn new() -> Self {
        Self {
            addresses: HashMap::new(),
        }
    }

    pub fn add_address(&mut self, address: SavedAddress) -> Result<(), AddressBookError> {
        let addr = address.address.clone();
        if self.addresses.contains_key(&addr) {
            return Err(AddressBookError::AlreadyExists(addr));
        }
        self.addresses.insert(addr, address);
        Ok(())
    }

    pub fn update_address(&mut self, address: SavedAddress) -> Result<(), AddressBookError> {
        let addr = address.address.clone();
        if !self.addresses.contains_key(&addr) {
            return Err(AddressBookError::NotFound(addr));
        }
        self.addresses.insert(addr, address);
        Ok(())
    }

    pub fn remove_address(&mut self, address: &str) -> Result<(), AddressBookError> {
        if self.addresses.remove(address).is_none() {
            return Err(AddressBookError::NotFound(address.to_string()));
        }
        Ok(())
    }

    pub fn get_address(&self, address: &str) -> Option<&SavedAddress> {
        self.addresses.get(address)
    }

    pub fn list_addresses(&self) -> Vec<&SavedAddress> {
        self.addresses.values().collect()
    }

    pub fn search_by_label(&self, query: &str) -> Vec<&SavedAddress> {
        let query_lower = query.to_lowercase();
        self.addresses
            .values()
            .filter(|addr| addr.label.to_lowercase().contains(&query_lower))
            .collect()
    }

    pub fn save_to_file(&self, path: &str) -> Result<(), AddressBookError> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_from_file(path: &str) -> Result<Self, AddressBookError> {
        let content = std::fs::read_to_string(path)?;
        let address_book: AddressBook = serde_json::from_str(&content)?;
        Ok(address_book)
    }
}

impl SavedAddress {
    pub fn new(address: String, label: String) -> Self {
        let now = chrono_lite_now();
        Self {
            address,
            label,
            chain_id: None,
            notes: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }

    pub fn with_chain_id(mut self, chain_id: u64) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    pub fn with_notes(mut self, notes: String) -> Self {
        self.notes = Some(notes);
        self
    }
}

fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}