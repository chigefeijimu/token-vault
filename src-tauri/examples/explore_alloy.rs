//! Explore alloy 2.x transaction signing API
//! Run with: cargo build --example explore_alloy -p token-vault

use alloy::{
    primitives::U256,
    signers::local::PrivateKeySigner,
};
use alloy_network::NetworkWallet;
use alloy_provider::ReqwestProvider;
use alloy_rpc_types::TransactionRequest;

#[tokio::main]
async fn main() {
    // 1. Create signer from private key
    let signer: PrivateKeySigner = "0x9676de3bd9a001bd581411e81dfc7b814b7ef3ff1cf5957476196d16c666a4c8"
        .parse()
        .unwrap();
    let sender = signer.address();
    println!("Sender: {sender}");

    // 2. Create provider
    let rpc_url = "https://bsc-dataseed.binance.org".parse().unwrap();
    let provider = ReqwestProvider::new_http(rpc_url);

    // 3. Get nonce and gas price
    let nonce = provider.get_transaction_count(sender).await.unwrap();
    println!("Nonce: {nonce}");
    let gas_price = provider.get_gas_price().await.unwrap();
    println!("Gas price: {gas_price}");

    // 4. Build a simple native transfer
    let to = "0x1234567890123456789012345678901234567890".parse().unwrap();
    let req = TransactionRequest::new()
        .with_from(sender)
        .with_to(to)
        .with_value(U256::from(100_000_000_000_000u64))
        .with_nonce(nonce)
        .with_gas_price(gas_price)
        .with_gas_limit(21_000);

    println!("TransactionRequest built OK");
    println!("req.from = {:?}", req.from());
    println!("req.to = {:?}", req.to());
    println!("req.value = {:?}", req.value());

    // 5. Sign and send using alloy's send_transaction (auto-signs with internal wallet)
    println!("\nSending native transfer...");
    match provider.send_transaction(req).await {
        Ok(result) => println!("TX SENT! Hash: {:?}", result.tx_hash()),
        Err(e) => println!("send_transaction error: {e}"),
    }
}
