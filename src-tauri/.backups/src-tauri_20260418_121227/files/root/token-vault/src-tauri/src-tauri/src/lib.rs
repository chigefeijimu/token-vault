// TokenVault - Tauri Backend Library
// Core functionality: wallet management, crypto, chain interactions

pub mod crypto;
pub mod wallet;

use wallet::{WalletManager, WalletInfo};
use tauri::State;
use std::sync::Arc;
use parking_lot::RwLock;

// App state shared across commands
pub struct AppState {
    pub wallet_manager: RwLock<WalletManager>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            wallet_manager: RwLock::new(WalletManager::new()),
        }
    }
}

// ============== Wallet Commands ==============

#[tauri::command]
async fn create_wallet(
    name: String,
    password: String,
    state: State<'_, Arc<AppState>>,
) -> Result<WalletInfo, String> {
    let mut wallet_manager = state.wallet_manager.write();
    wallet_manager.create_wallet(&name, &password)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn import_wallet_from_mnemonic(
    name: String,
    mnemonic: String,
    password: String,
    state: State<'_, Arc<AppState>>,
) -> Result<WalletInfo, String> {
    let mut wallet_manager = state.wallet_manager.write();
    wallet_manager.import_from_mnemonic(&name, &mnemonic, &password)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn import_wallet_from_private_key(
    name: String,
    private_key: String,
    password: String,
    state: State<'_, Arc<AppState>>,
) -> Result<WalletInfo, String> {
    let mut wallet_manager = state.wallet_manager.write();
    wallet_manager.import_from_private_key(&name, &private_key, &password)
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_wallets(
    state: State<'_, Arc<AppState>>,
) -> Result<Vec<WalletInfo>, String> {
    let wallet_manager = state.wallet_manager.read();
    Ok(wallet_manager.list_wallets())
}

#[tauri::command]
async fn export_keystore(
    wallet_id: String,
    password: String,
    state: State<'_, Arc<AppState>>,
) -> Result<String, String> {
    let wallet_manager = state.wallet_manager.read();
    wallet_manager.export_keystore(&wallet_id, &password)
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(Arc::new(AppState::default()))
        .invoke_handler(tauri::generate_handler![
            create_wallet,
            import_wallet_from_mnemonic,
            import_wallet_from_private_key,
            get_wallets,
            export_keystore,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
