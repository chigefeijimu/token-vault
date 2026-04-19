mod crypto;
mod wallet;
mod rpc;
mod transaction;
mod lib_status;
mod version;
mod nft;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
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
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}


mod errors;