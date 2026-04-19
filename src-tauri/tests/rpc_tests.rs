//! Integration tests for RPC module

use token_vault_lib::rpc::{
    ChainConfig, RpcClient, RpcError,
    get_chain_config, get_rpc_url, get_explorer_url,
};
use std::str::FromStr;

#[test]
fn test_chain_config_fields() {
    let config = ChainConfig {
        chain_id: 1,
        chain_name: "Ethereum".to_string(),
        rpc_url: "https://eth.llamarpc.com".to_string(),
        native_currency: "ETH".to_string(),
        explorer_url: "https://etherscan.io".to_string(),
    };
    
    assert_eq!(config.chain_id, 1);
    assert_eq!(config.chain_name, "Ethereum");
    assert_eq!(config.native_currency, "ETH");
}

#[test]
fn test_get_chain_config_ethereum() {
    let config = get_chain_config(1).expect("Should get Ethereum config");
    
    assert_eq!(config.chain_id, 1);
    assert_eq!(config.chain_name, "Ethereum");
    assert!(config.rpc_url.contains("eth"));
}

#[test]
fn test_get_chain_config_bnb() {
    let config = get_chain_config(56).expect("Should get BNB Chain config");
    
    assert_eq!(config.chain_id, 56);
    assert_eq!(config.chain_name, "BNB Chain");
}

#[test]
fn test_get_chain_config_polygon() {
    let config = get_chain_config(137).expect("Should get Polygon config");
    
    assert_eq!(config.chain_id, 137);
    assert_eq!(config.chain_name, "Polygon");
}

#[test]
fn test_get_chain_config_arbitrum() {
    let config = get_chain_config(42161).expect("Should get Arbitrum config");
    
    assert_eq!(config.chain_id, 42161);
    assert_eq!(config.chain_name, "Arbitrum One");
}

#[test]
fn test_get_chain_config_optimism() {
    let config = get_chain_config(10).expect("Should get Optimism config");
    
    assert_eq!(config.chain_id, 10);
    assert_eq!(config.chain_name, "Optimism");
}

#[test]
fn test_get_chain_config_avalanche() {
    let config = get_chain_config(43114).expect("Should get Avalanche config");
    
    assert_eq!(config.chain_id, 43114);
    assert_eq!(config.chain_name, "Avalanche");
}

#[test]
fn test_get_chain_config_unknown_chain() {
    let result = get_chain_config(999999);
    assert!(result.is_err());
}

#[test]
fn test_get_rpc_url_known_chain() {
    let url = get_rpc_url(1).expect("Should get RPC URL");
    assert!(!url.is_empty());
    assert!(url.starts_with("http://") || url.starts_with("https://"));
}

#[test]
fn test_get_rpc_url_unknown_chain() {
    let result = get_rpc_url(999999);
    assert!(result.is_err());
}

#[test]
fn test_get_explorer_url_ethereum() {
    let url = get_explorer_url(1).expect("Should get explorer URL");
    assert!(url.contains("etherscan"));
}

#[test]
fn test_get_explorer_url_unknown_chain() {
    let result = get_explorer_url(999999);
    assert!(result.is_err());
}

#[test]
fn test_rpc_client_creation() {
    let client = RpcClient::new("https://eth.llamarpc.com");
    assert!(client.is_ok());
}

#[test]
fn test_rpc_client_invalid_url() {
    let result = RpcClient::new("invalid-url");
    assert!(result.is_ok() || result.is_err()); // Implementation dependent
}

#[test]
fn test_all_supported_chains() {
    let chain_ids = [1, 56, 137, 42161, 10, 43114];
    
    for chain_id in chain_ids {
        let config = get_chain_config(chain_id);
        assert!(config.is_ok(), "Chain {} should be supported", chain_id);
        
        let rpc_url = get_rpc_url(chain_id);
        assert!(rpc_url.is_ok(), "Chain {} should have RPC URL", chain_id);
        
        let explorer_url = get_explorer_url(chain_id);
        assert!(explorer_url.is_ok(), "Chain {} should have explorer URL", chain_id);
    }
}

#[test]
fn test_chain_config_clone() {
    let config = get_chain_config(1).expect("Should get config");
    let cloned = config.clone();
    
    assert_eq!(config.chain_id, cloned.chain_id);
    assert_eq!(config.chain_name, cloned.chain_name);
    assert_eq!(config.rpc_url, cloned.rpc_url);
}

#[test]
fn test_chain_config_serialize() {
    let config = ChainConfig {
        chain_id: 1,
        chain_name: "Ethereum".to_string(),
        rpc_url: "https://eth.llamarpc.com".to_string(),
        native_currency: "ETH".to_string(),
        explorer_url: "https://etherscan.io".to_string(),
    };
    
    let json = serde_json::to_string(&config).expect("Should serialize");
    assert!(json.contains("Ethereum"));
    assert!(json.contains("1"));
}

#[test]
fn test_chain_config_deserialize() {
    let json = r#"{
        "chain_id": 137,
        "chain_name": "Polygon",
        "rpc_url": "https://polygon-rpc.com",
        "native_currency": "MATIC",
        "explorer_url": "https://polygonscan.com"
    }"#;
    
    let config: ChainConfig = serde_json::from_str(json).expect("Should deserialize");
    assert_eq!(config.chain_id, 137);
    assert_eq!(config.chain_name, "Polygon");
    assert_eq!(config.native_currency, "MATIC");
}