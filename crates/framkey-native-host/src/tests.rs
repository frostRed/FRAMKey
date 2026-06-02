use crate::{
    config::{
        NativeDeviceConfig, NativeHostConfig, SignerHelperConfig, SignerHelperSandbox,
        parse_save_type, validate_chain_id,
    },
    constants::{DEFAULT_KEYCHAIN_ACCOUNT, DEFAULT_KEYCHAIN_SERVICE},
    error::error_to_ipc,
    handler::{NativeHostState, handle_request_result},
    signer_helper::{signer_helper_stderr_summary, wait_for_signer_helper_output},
};
use framkey_gbxcart::GbaSaveType;
use framkey_ipc::{IpcErrorCode, IpcRequest};
use serde_json::Value;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

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

    let mut state = NativeHostState::default();
    let error = handle_request_result(&config, &mut state, &request).unwrap_err();
    assert_eq!(error.code, IpcErrorCode::DangerousSignatureBlocked);
}

#[test]
fn eth_accounts_without_session_does_not_touch_configured_device() {
    let config = NativeHostConfig {
        chain_id: "0x1".to_owned(),
        device: NativeDeviceConfig::File {
            path: PathBuf::from("/definitely/missing/framkey-fixture.sav"),
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
        method: "eth_accounts".to_owned(),
        params: Value::Null,
        origin: Some("https://example.test".to_owned()),
    };

    let mut state = NativeHostState::default();
    let result = handle_request_result(&config, &mut state, &request).unwrap();

    assert_eq!(result, serde_json::json!([]));
}

#[test]
fn status_redacts_local_device_paths() {
    let config = NativeHostConfig {
        chain_id: "0x1".to_owned(),
        device: NativeDeviceConfig::File {
            path: PathBuf::from("/Users/example/.framkey/private-vault.sav"),
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
        method: "framkey_getStatus".to_owned(),
        params: Value::Null,
        origin: Some("https://example.test".to_owned()),
    };

    let mut state = NativeHostState::default();
    let result = handle_request_result(&config, &mut state, &request).unwrap();

    assert_eq!(result["device"]["kind"], "file");
    assert_eq!(result["device"]["pathConfigured"], true);
    assert!(result["device"].get("path").is_none());
    assert_eq!(result["keychain"]["configured"], true);
    assert!(result["keychain"].get("service").is_none());
    assert!(result["keychain"].get("account").is_none());
    assert!(!result.to_string().contains("private-vault.sav"));
    assert!(!result.to_string().contains(DEFAULT_KEYCHAIN_SERVICE));
    assert!(!result.to_string().contains(DEFAULT_KEYCHAIN_ACCOUNT));
}

#[test]
fn config_validation_rejects_ambiguous_keychain_names_and_device_hints() {
    let mut config = NativeHostConfig {
        chain_id: "0x1".to_owned(),
        device: NativeDeviceConfig::File {
            path: PathBuf::from("fixture.sav"),
        },
        keychain_service: " \t".to_owned(),
        keychain_account: DEFAULT_KEYCHAIN_ACCOUNT.to_owned(),
        helper: SignerHelperConfig {
            path: PathBuf::from("framkey-signer-helper"),
            expected_blake3: None,
            sandbox: SignerHelperSandbox::DisabledByConfig,
        },
    };
    assert!(config.validate().is_err());

    config.keychain_service = DEFAULT_KEYCHAIN_SERVICE.to_owned();
    config.keychain_account = " default".to_owned();
    assert!(config.validate().is_err());

    config.keychain_account = "default\u{7f}".to_owned();
    assert!(config.validate().is_err());

    config.keychain_account = DEFAULT_KEYCHAIN_ACCOUNT.to_owned();
    config.device = NativeDeviceConfig::GbxCart {
        port: Some(" \t".to_owned()),
        save_type: GbaSaveType::SramFram512Kbit,
        expected_save_size: None,
    };
    assert!(config.validate().is_err());
}

#[test]
fn local_authentication_errors_map_to_local_authentication_failed() {
    let error = error_to_ipc(anyhow::anyhow!(
        "authorize FRAMKey local KEK access failed: macOS LocalAuthentication failed"
    ));

    assert_eq!(error.code, IpcErrorCode::LocalAuthenticationFailed);
}

#[test]
fn signer_helper_stderr_summary_redacts_contents() {
    let summary = signer_helper_stderr_summary(b"secret keychain and recovery bytes");

    assert_eq!(summary, "34 bytes redacted");
    assert!(!summary.contains("secret"));
    assert!(!summary.contains("recovery"));
    assert_eq!(signer_helper_stderr_summary(b""), "empty");
}

#[cfg(unix)]
#[test]
fn signer_helper_wait_times_out_and_kills_child() {
    use std::{fs, os::unix::fs::PermissionsExt};

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let script_path = std::env::temp_dir().join(format!(
        "framkey-native-host-timeout-{}-{unique}.sh",
        std::process::id()
    ));
    fs::write(&script_path, "#!/bin/sh\nsleep 5\n").unwrap();
    let mut permissions = fs::metadata(&script_path).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&script_path, permissions).unwrap();

    let child = Command::new(&script_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let started_at = Instant::now();
    let error = wait_for_signer_helper_output(child, Duration::from_millis(1)).unwrap_err();
    let _ = fs::remove_file(&script_path);

    assert!(error.to_string().contains("timed out after 1 ms"));
    assert!(started_at.elapsed() < Duration::from_secs(2));
}

#[cfg(unix)]
#[test]
fn signer_helper_wait_drains_large_stdout_before_child_exit() {
    use std::{fs, os::unix::fs::PermissionsExt};

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let script_path = std::env::temp_dir().join(format!(
        "framkey-native-host-large-stdout-{}-{unique}.sh",
        std::process::id()
    ));
    fs::write(
        &script_path,
        "#!/bin/sh\ndd if=/dev/zero bs=1024 count=1024 2>/dev/null\nprintf done >&2\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&script_path).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&script_path, permissions).unwrap();

    let child = Command::new(&script_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let started_at = Instant::now();
    let output = wait_for_signer_helper_output(child, Duration::from_secs(5)).unwrap();
    let _ = fs::remove_file(&script_path);

    assert!(output.status.success());
    assert_eq!(output.stdout.len(), 1024 * 1024);
    assert_eq!(output.stderr, b"done");
    assert!(started_at.elapsed() < Duration::from_secs(2));
}

#[test]
fn validates_chain_ids() {
    validate_chain_id("0x1").unwrap();
    validate_chain_id("0xaa36a7").unwrap();
    assert!(validate_chain_id("1").is_err());
    assert!(validate_chain_id("0x").is_err());
    assert!(validate_chain_id("0xzz").is_err());
}
