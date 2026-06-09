use crate::{
    handler::chain_accounts_from_secret,
    io::classify_error,
    recovery::validate_recovery_files_drill,
    validation::{
        parse_expected_address, validate_expected_address, validate_personal_sign_message,
        validate_recovery_files, validate_save_image_size, validate_sign_transaction_request,
        validate_typed_data_request,
    },
};
use framkey_core::WalletType;
use framkey_crypto::SecretBytes;
use framkey_evm::EvmAddress;
use framkey_ipc::{
    IpcErrorCode, MAX_SIGNER_HELPER_PERSONAL_SIGN_MESSAGE_BYTES,
    MAX_SIGNER_HELPER_SAVE_IMAGE_BYTES, MAX_SIGNER_HELPER_TRANSACTION_DATA_BYTES,
    MAX_SIGNER_HELPER_TYPED_DATA_BYTES, MIN_SIGNER_HELPER_SAVE_IMAGE_BYTES, SignerEvmTransaction,
};

#[test]
fn accepts_supported_save_image_sizes() {
    validate_save_image_size(MIN_SIGNER_HELPER_SAVE_IMAGE_BYTES).unwrap();
    validate_save_image_size(MAX_SIGNER_HELPER_SAVE_IMAGE_BYTES).unwrap();
}

#[test]
fn rejects_unsupported_save_image_sizes() {
    assert!(validate_save_image_size(MIN_SIGNER_HELPER_SAVE_IMAGE_BYTES - 1).is_err());
    assert!(validate_save_image_size(MAX_SIGNER_HELPER_SAVE_IMAGE_BYTES + 1).is_err());
}

#[test]
fn rejects_overlong_personal_sign_messages() {
    let message = vec![0_u8; MAX_SIGNER_HELPER_PERSONAL_SIGN_MESSAGE_BYTES + 1];
    assert!(validate_personal_sign_message(&message).is_err());
}

#[test]
fn rejects_overlong_typed_data_payloads() {
    let typed_data = serde_json::json!({
        "types": {
            "EIP712Domain": [],
            "Message": [{"name": "payload", "type": "string"}]
        },
        "primaryType": "Message",
        "domain": {},
        "message": {
            "payload": "x".repeat(MAX_SIGNER_HELPER_TYPED_DATA_BYTES)
        }
    });
    assert!(validate_typed_data_request(&typed_data).is_err());
}

#[test]
fn rejects_malformed_typed_data_before_signing() {
    let typed_data = serde_json::json!({
        "types": {
            "EIP712Domain": []
        },
        "primaryType": "Permit",
        "domain": {},
        "message": {}
    });

    let error = validate_typed_data_request(&typed_data).unwrap_err();
    assert!(error.to_string().contains("primaryType Permit"));
}

#[test]
fn validates_recovery_file_count() {
    assert!(validate_recovery_files(1).is_ok());
    assert!(validate_recovery_files(4).is_ok());
    assert!(validate_recovery_files(0).is_err());
    assert!(validate_recovery_files(5).is_err());
}

#[test]
fn recovery_file_drill_reports_policy_without_returning_secrets() {
    let recovery_root_key = [4_u8; 32];
    let pack = framkey_recovery::RecoveryBackupPack::standard(
        [1_u8; 16],
        1,
        [2_u8; 16],
        [3_u8; 16],
        1_700_000_000,
        &recovery_root_key,
        framkey_recovery::RecoveryBackupEntropy {
            group_polynomial_coefficients: [5_u8; 32],
            cloud_member_pad: [6_u8; 32],
        },
    );
    let cloud_only = pack
        .files
        .iter()
        .filter(|file| file.group_kind == framkey_recovery::RecoveryGroupKind::Cloud)
        .cloned()
        .collect::<Vec<_>>();
    let cloud_only_result = validate_recovery_files_drill(&cloud_only).unwrap();
    assert!(!cloud_only_result.can_recover);
    assert_eq!(cloud_only_result.satisfied_groups, vec!["cloud"]);
    assert!(cloud_only_result.failure_reason.is_some());

    let mut recoverable = cloud_only;
    recoverable.push(
        pack.files
            .iter()
            .find(|file| file.group_kind == framkey_recovery::RecoveryGroupKind::LocalPhysical)
            .unwrap()
            .clone(),
    );
    let recoverable_result = validate_recovery_files_drill(&recoverable).unwrap();
    assert!(recoverable_result.can_recover);
    assert_eq!(recoverable_result.recovery_share_file_count, 3);
    assert_eq!(
        recoverable_result.satisfied_groups,
        vec!["cloud", "local_physical"]
    );
    assert!(recoverable_result.failure_reason.is_none());
}

#[test]
fn rejects_overlong_transaction_data() {
    let transaction = SignerEvmTransaction {
        data: format!(
            "0x{}",
            "00".repeat(MAX_SIGNER_HELPER_TRANSACTION_DATA_BYTES + 1)
        ),
        ..valid_signer_transaction()
    };
    assert!(validate_sign_transaction_request(&transaction).is_err());
}

#[test]
fn rejects_malformed_transactions_before_signing() {
    let mut mixed_fees = valid_signer_transaction();
    mixed_fees.max_fee_per_gas = Some("0x2".to_owned());
    mixed_fees.max_priority_fee_per_gas = Some("0x1".to_owned());
    let error = validate_sign_transaction_request(&mixed_fees).unwrap_err();
    assert!(error.to_string().contains("cannot mix gasPrice"));

    let mut bad_to = valid_signer_transaction();
    bad_to.to = Some("0x1234".to_owned());
    let error = validate_sign_transaction_request(&bad_to).unwrap_err();
    assert!(error.to_string().contains("transaction to"));
}

#[test]
fn rejects_expected_address_mismatch_before_signing() {
    let actual: EvmAddress = "0x0000000000000000000000000000000000000001"
        .parse()
        .unwrap();
    let expected =
        parse_expected_address(Some("0x0000000000000000000000000000000000000002")).unwrap();
    let error = validate_expected_address(actual, expected).unwrap_err();
    assert!(error.to_string().contains("account mismatch"));
}

#[test]
fn rejects_malformed_expected_address_before_signing() {
    let error =
        parse_expected_address(Some("0000000000000000000000000000000000000002")).unwrap_err();
    assert!(error.to_string().contains("0x-prefixed"));
}

#[test]
fn accepts_matching_expected_address() {
    let actual: EvmAddress = "0x0000000000000000000000000000000000000001"
        .parse()
        .unwrap();
    let expected =
        parse_expected_address(Some("0x0000000000000000000000000000000000000001")).unwrap();
    validate_expected_address(actual, expected).unwrap();
}

#[test]
fn derives_public_evm_and_btc_accounts_for_multichain_secret() {
    let secret = SecretBytes::new([1; 32]);
    let accounts = chain_accounts_from_secret(WalletType::Secp256k1SingleKey, &secret).unwrap();

    let evm = accounts
        .iter()
        .find(|account| account.family == "evm")
        .expect("evm account");
    assert!(evm.address.starts_with("0x"));
    assert_eq!(evm.address_type, "eoa");

    let btc = accounts
        .iter()
        .find(|account| account.family == "btc" && account.network == "bitcoin-mainnet")
        .expect("btc mainnet account");
    assert_eq!(btc.network, "bitcoin-mainnet");
    assert_eq!(btc.address_type, "p2wpkh");
    assert!(btc.address.starts_with("bc1q"));

    let btc_testnet = accounts
        .iter()
        .find(|account| account.family == "btc" && account.network == "bitcoin-testnet4")
        .expect("btc testnet4 account");
    assert_eq!(btc_testnet.address_type, "p2wpkh");
    assert!(btc_testnet.address.starts_with("tb1q"));
}

#[test]
fn classifies_local_authentication_errors_as_local_authentication_failures() {
    let error = anyhow::anyhow!(
        "authorize FRAMKey local KEK access failed: macOS LocalAuthentication failed"
    );

    assert_eq!(
        classify_error(&error),
        IpcErrorCode::LocalAuthenticationFailed
    );
}

fn valid_signer_transaction() -> SignerEvmTransaction {
    SignerEvmTransaction {
        chain_id: 1,
        nonce: "0x0".to_owned(),
        gas_limit: "0x5208".to_owned(),
        to: Some("0x0000000000000000000000000000000000000001".to_owned()),
        value: "0x0".to_owned(),
        data: "0x".to_owned(),
        gas_price: Some("0x1".to_owned()),
        max_fee_per_gas: None,
        max_priority_fee_per_gas: None,
    }
}
