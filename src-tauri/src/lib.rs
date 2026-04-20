pub mod errors;
pub mod chain_adapter;
pub mod storage;
mod crypto;
mod wallet;
mod rpc;
mod transaction;
mod lib_status;
mod version;
mod nft;
pub mod solana;
mod erc20;
mod security;

use std::path::PathBuf;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .manage(security::SecurityManager::new())
        .setup(|app| {
            // Initialize SQLite storage in app data directory
            let app_data_dir = app.path().app_data_dir()
                .unwrap_or_else(|_| PathBuf::from("."));
            let _ = storage::init_storage(app_data_dir.clone());

            // Load existing wallets from SQLite into WalletManager
            let _ = wallet::wallet_manager_from_storage();

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Crypto commands
            crypto::generate_private_key,
            crypto::derive_public_key,
            crypto::public_key_to_address,
            crypto::sign_data,
            crypto::encrypt_data,
            crypto::decrypt_data,
            crypto::hash_data,
            crypto::validate_mnemonic_cmd,
            crypto::generate_mnemonic_cmd,
            // Wallet commands
            wallet::create_wallet,
            wallet::import_wallet,
            wallet::list_wallets,
            wallet::get_wallet_info,
            wallet::delete_wallet,
            wallet::export_private_key,
            wallet::decrypt_wallet,
            // RPC commands
            rpc::get_chain_config,
            rpc::get_balance,
            rpc::get_gas_price,
            rpc::estimate_gas,
            rpc::send_raw_transaction,
            rpc::get_transaction_receipt,
            rpc::get_transaction_history,
            // Transaction commands
            transaction::send_transaction,
            transaction::send_erc20_token,
            // Version command
            version::get_version,
            // NFT commands
            nft::get_nfts,
            nft::get_nft_metadata,
            // Solana commands
            solana::solana_generate_wallet,
            solana::solana_get_balance,
            // ERC20 commands
            erc20::get_token_info,
            erc20::get_erc20_balance,
            // Security commands
            security::setup_pin_code,
            security::verify_pin_code,
            security::verify_biometric,
            security::lock_app,
            security::unlock_app,
            security::get_auth_state,
            security::get_security_settings,
            security::update_security_settings,
            security::enable_app_lock,
            security::disable_app_lock,
            security::enable_biometric,
            security::disable_biometric,
            security::disable_pin_code,
            security::update_activity,
            security::check_auto_lock,
            security::get_remaining_attempts,
            security::reset_failed_attempts,
            // Storage commands
            storage::storage_init,
            storage::storage_save_wallet,
            storage::storage_load_wallet,
            storage::storage_get_all_wallets,
            storage::storage_delete_wallet,
            storage::storage_save_transaction,
            storage::storage_get_wallet_transactions,
            storage::storage_get_all_transactions,
            storage::storage_update_transaction_status,
            storage::storage_load_settings,
            storage::storage_save_settings,
            storage::storage_update_setting,
            storage::storage_export_json,
            storage::storage_clear_all_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
