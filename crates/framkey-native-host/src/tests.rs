use crate::{
    config::{
        NativeDeviceConfig, NativeHostConfig, SignerHelperConfig, SignerHelperSandbox,
        parse_save_type, validate_chain_id,
    },
    constants::{DEFAULT_KEYCHAIN_ACCOUNT, DEFAULT_KEYCHAIN_SERVICE},
    handler::handle_request_result,
};
use framkey_gbxcart::GbaSaveType;
use framkey_ipc::{IpcErrorCode, IpcRequest};
use serde_json::Value;
use std::path::PathBuf;

#[test]
fn parses_save_type_aliases() {
    assert_eq!(
        parse_save_type("gba-sram-fram-512kbit").unwrap(),
        GbaSaveType::SramFram512Kbit
    );
    assert_eq!(
        parse_save_type("gba-fram-1m").unwrap(),
        GbaSaveType::SramFram1Mbit
    );
}

#[test]
fn rejects_signing_methods() {
    let config = NativeHostConfig {
        chain_id: "0x1".to_owned(),
        device: NativeDeviceConfig::File {
            path: PathBuf::from("fixture.sav"),
        },
        keychain_service: DEFAULT_KEYCHAIN_SERVICE.to_owned(),
        keychain_account: DEFAULT_KEYCHAIN_ACCOUNT.to_owned(),
        helper: SignerHelperConfig {
            path: PathBuf::from("framkey-signer-helper"),
            expected_blake3: None,
            sandbox: SignerHelperSandbox::DisabledByConfig,
        },
    };
    let request = IpcRequest {
        id: "1".to_owned(),
        method: "personal_sign".to_owned(),
        params: Value::Null,
        origin: Some("https://example.test".to_owned()),
    };

    let error = handle_request_result(&config, &request).unwrap_err();
    assert_eq!(error.code, IpcErrorCode::DangerousSignatureBlocked);
}

#[test]
fn validates_chain_ids() {
    validate_chain_id("0x1").unwrap();
    validate_chain_id("0xaa36a7").unwrap();
    assert!(validate_chain_id("1").is_err());
    assert!(validate_chain_id("0x").is_err());
    assert!(validate_chain_id("0xzz").is_err());
}
