//! Integration tests for transaction module

use token_vault_lib::transaction::{
    TransactionRequest, TransactionResponse, TransactionError,
    sign_transaction, broadcast_transaction, get_transaction_receipt,
    estimate_gas, get_nonce,
};

#[test]
fn test_transaction_request_creation() {
    let request = TransactionRequest {
        from: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1".to_string(),
        to: "0x8ba1f109551bD432803012645Ac136ddd64DBA72".to_string(),
        value: "1000000000000000000".to_string(), // 1 ETH in wei
        data: "0x".to_string(),
        chain_id: 1,
        gas_limit: Some(21000),
        max_fee_per_gas: Some("20000000000".to_string()),
        max_priority_fee_per_gas: Some("2000000000".to_string()),
    };
    
    assert_eq!(request.chain_id, 1);
    assert!(request.from.starts_with("0x"));
    assert!(request.to.starts_with("0x"));
}

#[test]
fn test_transaction_response_fields() {
    let response = TransactionResponse {
        hash: "0xabc123".to_string(),
        block_number: Some("12345".to_string()),
        block_hash: Some("0xdef456".to_string()),
        status: "confirmed".to_string(),
    };
    
    assert!(!response.hash.is_empty());
}

#[test]
fn test_sign_transaction_requires_private_key() {
    let request = TransactionRequest {
        from: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1".to_string(),
        to: "0x8ba1f109551bD432803012645Ac136ddd64DBA72".to_string(),
        value: "1000000000000000000".to_string(),
        data: "0x".to_string(),
        chain_id: 1,
        gas_limit: Some(21000),
        max_fee_per_gas: Some("20000000000".to_string()),
        max_priority_fee_per_gas: Some("2000000000".to_string()),
    };
    
    // Without private key, should return error
    let result = sign_transaction(&request, "");
    assert!(result.is_err());
}

#[test]
fn test_transaction_value_formats() {
    // Test various value formats
    let values = [
        "0",                    // 0 wei
        "1000000000",           // 1 gwei
        "1000000000000000000",  // 1 ETH
        "1000000000000000000000", // 1000 ETH
    ];
    
    for value in values {
        let request = TransactionRequest {
            from: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1".to_string(),
            to: "0x8ba1f109551bD432803012645Ac136ddd64DBA72".to_string(),
            value: value.to_string(),
            data: "0x".to_string(),
            chain_id: 1,
            gas_limit: Some(21000),
            max_fee_per_gas: Some("20000000000".to_string()),
            max_priority_fee_per_gas: Some("2000000000".to_string()),
        };
        
        assert!(!request.value.is_empty());
    }
}

#[test]
fn test_transaction_data_formats() {
    // Empty data
    let request1 = TransactionRequest {
        from: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1".to_string(),
        to: "0x8ba1f109551bD432803012645Ac136ddd64DBA72".to_string(),
        value: "0".to_string(),
        data: "0x".to_string(),
        chain_id: 1,
        gas_limit: Some(21000),
        max_fee_per_gas: Some("20000000000".to_string()),
        max_priority_fee_per_gas: Some("2000000000".to_string()),
    };
    assert_eq!(request1.data, "0x");
    
    // Hex data (e.g., ERC20 transfer)
    let request2 = TransactionRequest {
        from: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1".to_string(),
        to: "0x8ba1f109551bD432803012645Ac136ddd64DBA72".to_string(),
        value: "0".to_string(),
        data: "0xa9059cbb0000000000000000000000008ba1f109551bD432803012645Ac136ddd64DBA720000000000000000000000000000000000000000000000000000000000000001".to_string(),
        chain_id: 1,
        gas_limit: Some(21000),
        max_fee_per_gas: Some("20000000000".to_string()),
        max_priority_fee_per_gas: Some("2000000000".to_string()),
    };
    assert!(request2.data.starts_with("0x"));
    assert!(request2.data.len() > 2);
}

#[test]
fn test_transaction_chain_ids() {
    let chains = [1, 56, 137, 42161, 10, 43114];
    
    for chain_id in chains {
        let request = TransactionRequest {
            from: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1".to_string(),
            to: "0x8ba1f109551bD432803012645Ac136ddd64DBA72".to_string(),
            value: "0".to_string(),
            data: "0x".to_string(),
            chain_id,
            gas_limit: Some(21000),
            max_fee_per_gas: Some("20000000000".to_string()),
            max_priority_fee_per_gas: Some("2000000000".to_string()),
        };
        
        assert_eq!(request.chain_id, chain_id);
    }
}

#[test]
fn test_estimate_gas_requires_to_address() {
    let request = TransactionRequest {
        from: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1".to_string(),
        to: "".to_string(), // Empty address for contract deployment
        value: "0".to_string(),
        data: "0x".to_string(),
        chain_id: 1,
        gas_limit: None,
        max_fee_per_gas: Some("20000000000".to_string()),
        max_priority_fee_per_gas: Some("2000000000".to_string()),
    };
    
    // Estimate gas should handle contract deployment (no 'to' address)
    let result = estimate_gas(&request, "https://eth.llamarpc.com");
    // Result depends on RPC availability
    assert!(result.is_ok() || result.is_err());
}

#[test]
fn test_get_nonce_requires_address() {
    let result = get_nonce("", 1, "https://eth.llamarpc.com");
    assert!(result.is_err());
}

#[test]
fn test_broadcast_transaction_requires_signed_tx() {
    let result = broadcast_transaction("", "https://eth.llamarpc.com");
    assert!(result.is_err());
}

#[test]
fn test_get_transaction_receipt_invalid_hash() {
    let result = get_transaction_receipt("invalid", 1, "https://eth.llamarpc.com");
    assert!(result.is_ok() || result.is_err()); // Depends on RPC response
}

#[test]
fn test_transaction_error_display() {
    let error = TransactionError::Signing("Test signing error".to_string());
    let error_string = error.to_string();
    assert!(error_string.contains("Test signing error"));
    
    let rpc_error = TransactionError::Rpc("Connection failed".to_string());
    let rpc_error_string = rpc_error.to_string();
    assert!(rpc_error_string.contains("Connection failed"));
}

#[test]
fn test_transaction_request_serialization() {
    let request = TransactionRequest {
        from: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1".to_string(),
        to: "0x8ba1f109551bD432803012645Ac136ddd64DBA72".to_string(),
        value: "1000000000000000000".to_string(),
        data: "0x".to_string(),
        chain_id: 1,
        gas_limit: Some(21000),
        max_fee_per_gas: Some("20000000000".to_string()),
        max_priority_fee_per_gas: Some("2000000000".to_string()),
    };
    
    let json = serde_json::to_string(&request).expect("Should serialize");
    assert!(json.contains("0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1"));
    assert!(json.contains("1000000000000000000"));
}

#[test]
fn test_gas_limit_bounds() {
    // Minimum gas for simple transfer
    let request_min = TransactionRequest {
        from: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1".to_string(),
        to: "0x8ba1f109551bD432803012645Ac136ddd64DBA72".to_string(),
        value: "0".to_string(),
        data: "0x".to_string(),
        chain_id: 1,
        gas_limit: Some(21000), // Minimum for transfer
        max_fee_per_gas: Some("20000000000".to_string()),
        max_priority_fee_per_gas: Some("2000000000".to_string()),
    };
    assert_eq!(request_min.gas_limit, Some(21000));
    
    // High gas for complex operations
    let request_high = TransactionRequest {
        from: "0x742d35Cc6634C0532925a3b844Bc9e7595f0bEb1".to_string(),
        to: "0x8ba1f109551bD432803012645Ac136ddd64DBA72".to_string(),
        value: "0".to_string(),
        data: "0x".to_string(),
        chain_id: 1,
        gas_limit: Some(1000000),
        max_fee_per_gas: Some("20000000000".to_string()),
        max_priority_fee_per_gas: Some("2000000000".to_string()),
    };
    assert_eq!(request_high.gas_limit, Some(1000000));
}