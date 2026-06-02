use super::*;

#[test]
fn round_trips_native_message_payload() {
    let mut wire = Vec::new();
    write_native_message(&mut wire, br#"{"id":"1"}"#).unwrap();

    let payload = read_native_message(&mut wire.as_slice()).unwrap().unwrap();
    assert_eq!(payload, br#"{"id":"1"}"#);
}

#[test]
fn serializes_sign_transaction_request_kind() {
    let request = SignerHelperRequest::SignTransaction(SignerSignTransactionRequest {
        save_image: vec![0_u8; MIN_SIGNER_HELPER_SAVE_IMAGE_BYTES],
        keychain_service: "io.framkey.kek".to_owned(),
        keychain_account: "default".to_owned(),
        expected_address: Some("0x0000000000000000000000000000000000000001".to_owned()),
        transaction: SignerEvmTransaction {
            chain_id: 1,
            nonce: "0x0".to_owned(),
            gas_limit: "0x5208".to_owned(),
            to: Some("0x0000000000000000000000000000000000000002".to_owned()),
            value: "0x0".to_owned(),
            data: "0x".to_owned(),
            gas_price: Some("0x1".to_owned()),
            max_fee_per_gas: None,
            max_priority_fee_per_gas: None,
        },
    });

    let encoded = serde_json::to_value(&request).unwrap();
    assert_eq!(encoded["method"], "sign_transaction");
    assert_eq!(encoded["transaction"]["chainId"], 1);
}

#[test]
fn serializes_sign_typed_data_request_kind() {
    let request = SignerHelperRequest::SignTypedData(SignerSignTypedDataRequest {
        save_image: vec![0_u8; MIN_SIGNER_HELPER_SAVE_IMAGE_BYTES],
        keychain_service: "io.framkey.kek".to_owned(),
        keychain_account: "default".to_owned(),
        expected_address: Some("0x0000000000000000000000000000000000000001".to_owned()),
        typed_data: serde_json::json!({
            "types": {"EIP712Domain": [], "Permit": []},
            "primaryType": "Permit",
            "domain": {},
            "message": {}
        }),
    });

    let encoded = serde_json::to_value(&request).unwrap();
    assert_eq!(encoded["method"], "sign_typed_data");
    assert_eq!(encoded["typed_data"]["primaryType"], "Permit");
}

#[test]
fn serializes_recover_keychain_vault_request_kind() {
    let request = SignerHelperRequest::RecoverKeychainVault(SignerRecoverKeychainVaultRequest {
        save_image: vec![0_u8; MIN_SIGNER_HELPER_SAVE_IMAGE_BYTES],
        keychain_service: "io.framkey.kek".to_owned(),
        keychain_account: "default".to_owned(),
        recovery_files: Vec::new(),
    });

    let encoded = serde_json::to_value(&request).unwrap();
    assert_eq!(encoded["method"], "recover_keychain_vault");
    assert_eq!(encoded["recovery_files"], serde_json::json!([]));
}

#[test]
fn serializes_validate_recovery_files_request_kind() {
    let request = SignerHelperRequest::ValidateRecoveryFiles(SignerValidateRecoveryFilesRequest {
        recovery_files: Vec::new(),
    });

    let encoded = serde_json::to_value(&request).unwrap();
    assert_eq!(encoded["method"], "validate_recovery_files");
    assert_eq!(encoded["recovery_files"], serde_json::json!([]));
}
