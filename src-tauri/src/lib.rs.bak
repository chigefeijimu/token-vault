mod wallet;
mod rpc;
mod crypto;

use tauri::Builder;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    Builder::default()
        .invoke_handler(tauri::generate_handler![
            wallet::create_wallet,
            wallet::get_balance,
            wallet::send_transaction,
            rpc::call_rpc,
            rpc::get_block_number,
            crypto::encrypt_data,
            crypto::decrypt_data,
            crypto::hash_data,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}