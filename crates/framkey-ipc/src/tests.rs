use super::*;

#[test]
fn round_trips_native_message_payload() {
    let mut wire = Vec::new();
    write_native_message(&mut wire, br#"{"id":"1"}"#).unwrap();

    let payload = read_native_message(&mut wire.as_slice()).unwrap().unwrap();
    assert_eq!(payload, br#"{"id":"1"}"#);
}

#[test]
fn rejects_truncated_native_message_header() {
    let mut wire = [1_u8, 0, 0].as_slice();

    let error = read_native_message(&mut wire).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("truncated native messaging header")
    );
}

#[test]
fn rejects_oversized_native_message_from_header() {
    let header = ((MAX_NATIVE_MESSAGE_BYTES + 1) as u32).to_le_bytes();
    let mut wire = header.as_slice();

    let error = read_native_message(&mut wire).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("native message exceeds 1048576 bytes")
    );
}

#[test]
fn signer_helper_ok_response_wire_format_is_stable() {
    let response = SignerHelperResponse::ok(SignerHelperResult::ValidateRecoveryFiles(
        SignerValidateRecoveryFilesResponse {
            backup_set_id: "backup".to_owned(),
            wallet_id: "wallet".to_owned(),
            generation: 7,
            policy_id: "two-cloud-one-local".to_owned(),
            recovery_share_file_count: 3,
            satisfied_groups: vec!["local".to_owned(), "cloud".to_owned()],
            can_recover: true,
            failure_reason: None,
        },
    ));

    let encoded = serde_json::to_value(&response).unwrap();
    assert_eq!(encoded["status"], "ok");
    assert_eq!(encoded["result"]["kind"], "validate_recovery_files");
    assert_eq!(encoded["result"]["canRecover"], true);

    let decoded: SignerHelperResponse = serde_json::from_value(encoded).unwrap();
    assert!(matches!(
        decoded.into_result().unwrap(),
        SignerHelperResult::ValidateRecoveryFiles(SignerValidateRecoveryFilesResponse {
            can_recover: true,
            recovery_share_file_count: 3,
            ..
        })
    ));
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
