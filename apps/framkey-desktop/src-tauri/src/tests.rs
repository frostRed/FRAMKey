use super::*;
use anyhow::Result;
use framkey_crypto::{encode_hex, random_array};
use framkey_gbxcart::GbaSaveType;
use framkey_ipc::SignerValidateRecoveryFilesResponse;
use framkey_recovery::{
    RecoveryBackupFile, RecoveryBackupPack, parse_recovery_backup_bundle, recovery_backup_file_name,
};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::Arc,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

fn test_siwe_message(account: &str, domain: &str, uri: &str, chain_id: u64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    format!(
        "{domain} wants you to sign in with your Ethereum account:\n\
         {account}\n\n\
         FRAMKey test sign-in\n\n\
         URI: {uri}\n\
         Version: 1\n\
         Chain ID: {chain_id}\n\
         Nonce: FRAMKey1\n\
         Issued At: {}\n\
         Expiration Time: {}",
        review::format_rfc3339_seconds(now),
        review::format_rfc3339_seconds(now.saturating_add(5 * 60))
    )
}

#[test]
fn captures_non_personal_signing_methods_without_signing() {
    let state = AppState::new();
    let config = fixture_config();
    state.load_and_connect_account(&config).unwrap();
    state
        .grant_account_permission("https://example.test".to_owned())
        .unwrap();
    let request = ProviderRequest {
        id: "1".to_owned(),
        method: "eth_signTypedData_v4".to_owned(),
        params: json!([
            "0x0000000000000000000000000000000000000001",
            {
                "domain": {"name": "FRAMKey Test"},
                "primaryType": "Message",
                "types": {
                    "EIP712Domain": [{"name": "name", "type": "string"}],
                    "Message": [{"name": "text", "type": "string"}]
                },
                "message": {"text": "blocked"}
            }
        ]),
        origin: Some("https://example.test".to_owned()),
    };

    let response = handle_provider_request(&state, &config, &request).unwrap();
    let ProviderResponse::Error(error) = response else {
        panic!("expected signing method to return provider error");
    };
    assert_eq!(error.code, 4200);
    assert!(error.message.contains("captured for trusted review"));

    let review_queue = state.review_queue_snapshot().unwrap();
    assert_eq!(review_queue.len(), 1);
    assert_eq!(review_queue[0].method, "eth_signTypedData_v4");
    assert_eq!(review_queue[0].summary["typedData"]["intent"], "typed_data");
}

#[test]
fn signing_requests_require_connected_origin() {
    let state = AppState::new();
    let config = fixture_config();
    let request = ProviderRequest {
        id: "unconnected-sign".to_owned(),
        method: "personal_sign".to_owned(),
        params: json!(["0x4652414d4b6579", null]),
        origin: Some("https://unconnected.example".to_owned()),
    };

    let response = handle_provider_request(&state, &config, &request).unwrap();
    let ProviderResponse::Error(error) = response else {
        panic!("expected unconnected signing request to return provider error");
    };
    assert_eq!(error.code, 4100);
    assert!(error.message.contains("eth_requestAccounts"));
    assert_eq!(state.review_queue_snapshot().unwrap().len(), 0);
}

#[test]
fn transaction_requests_require_connected_origin_before_preparation() {
    let state = AppState::new();
    let config = fixture_config();
    let request = ProviderRequest {
        id: "unconnected-transaction".to_owned(),
        method: "eth_sendTransaction".to_owned(),
        params: json!([
            {
                "to": "0x0000000000000000000000000000000000000001",
                "value": "0x0"
            }
        ]),
        origin: Some("https://unconnected.example".to_owned()),
    };

    let response = handle_provider_request(&state, &config, &request).unwrap();
    let ProviderResponse::Error(error) = response else {
        panic!("expected unconnected transaction request to return provider error");
    };
    assert_eq!(error.code, 4100);
    assert_eq!(state.review_queue_snapshot().unwrap().len(), 0);
}

#[test]
fn mock_status_reports_mock_transaction_capability() {
    let config = fixture_config();
    let status = status_result(&config);
    assert_eq!(status["capabilities"]["readOnlyAccounts"], true);
    assert_eq!(
        status["capabilities"]["accountPermissions"],
        "session_origin_approval"
    );
    assert_eq!(
        status["capabilities"]["sendTransaction"],
        "mock_approval_required"
    );
    assert_eq!(
        status["capabilities"]["trustedAutosmokeDurationMs"],
        json!(45_000)
    );
}

#[test]
fn status_reports_btc_testnet4_choice_and_controlled_send_strategy() {
    let config = fixture_config();
    let status = status_result(&config);

    assert_eq!(status["btc"]["testNetwork"]["selected"], "bitcoin-testnet4");
    assert_eq!(
        status["btc"]["testNetwork"]["signet"]["status"],
        "reserved_controlled_integration_testnet"
    );
    assert_eq!(status["btc"]["balanceRpc"]["status"], "enabled");
    assert_eq!(
        status["btc"]["psbtUtxo"]["status"],
        "enabled_controlled_trusted_ui"
    );

    let networks = status["supportedNetworks"]
        .as_array()
        .expect("supported networks");
    let btc_testnet = networks
        .iter()
        .find(|network| network["network"] == json!("bitcoin-testnet4"))
        .expect("btc testnet4 network");
    assert_eq!(btc_testnet["defaultAccount"], json!(true));
    assert_eq!(btc_testnet["selectedTestNetwork"], json!(true));

    let signet = networks
        .iter()
        .find(|network| network["network"] == json!("bitcoin-signet"))
        .expect("btc signet network");
    assert_eq!(signet["defaultAccount"], json!(false));
    assert_eq!(
        signet["capabilities"]["status"],
        "reserved_controlled_integration_testnet"
    );
}

#[test]
fn provider_status_hides_local_runtime_details_from_dapps() {
    let state = AppState::new();
    let mut config = fixture_config();
    config.device = DeviceConfig::File {
        path: PathBuf::from("/Users/example/.framkey/private-vault.sav"),
    };
    config.keychain_service = "io.framkey.private".to_owned();
    config.keychain_account = "private-account".to_owned();
    config.helper.path = PathBuf::from("/Users/example/bin/private-helper");
    config.rpc = Some(DesktopRpcConfig {
        endpoint_url: "https://eth-mainnet.g.alchemy.com/v2/secret-token".to_owned(),
        network: Some("eth-mainnet".to_owned()),
        timeout_ms: 1_000,
        provider: DesktopRpcProvider::Alchemy,
    });
    let request = ProviderRequest {
        id: "public-status".to_owned(),
        method: "framkey_getStatus".to_owned(),
        params: json!([]),
        origin: Some("https://app.example".to_owned()),
    };

    let ProviderResponse::Result(status) =
        handle_provider_request(&state, &config, &request).unwrap()
    else {
        panic!("expected provider status result");
    };

    assert!(status.get("device").is_none());
    assert!(status.get("keychain").is_none());
    assert!(status.get("signerHelper").is_none());
    let serialized = serde_json::to_string(&status).unwrap();
    assert!(!serialized.contains("private-vault.sav"));
    assert!(!serialized.contains("private-helper"));
    assert!(!serialized.contains("io.framkey.private"));
    assert!(!serialized.contains("private-account"));
    assert!(!serialized.contains("secret-token"));
    assert!(!serialized.contains("g.alchemy.com/v2"));
}

#[test]
fn framkey_get_account_is_restricted_to_trusted_provider_origin() {
    let state = AppState::new();
    let config = fixture_config();
    let untrusted = ProviderRequest {
        id: "untrusted-get-account".to_owned(),
        method: "framkey_getAccount".to_owned(),
        params: json!([]),
        origin: Some("https://app.example".to_owned()),
    };

    let error = match handle_provider_request(&state, &config, &untrusted) {
        Ok(_) => panic!("untrusted framkey_getAccount should be rejected"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("restricted to the trusted"));

    let trusted = ProviderRequest {
        origin: Some(TRUSTED_UI_ORIGIN.to_owned()),
        ..untrusted
    };
    let ProviderResponse::Result(account) =
        handle_provider_request(&state, &config, &trusted).unwrap()
    else {
        panic!("expected trusted account result");
    };
    assert!(account["address"].as_str().unwrap().starts_with("0x"));
}

#[test]
fn provider_request_origin_is_bound_to_window_context() {
    let request = ProviderRequest {
        id: "origin-bind".to_owned(),
        method: "eth_chainId".to_owned(),
        params: json!([]),
        origin: Some("https://app.example".to_owned()),
    };
    let dapp_url = tauri::Url::parse("https://app.example/swap?token=secret").unwrap();

    let bound = bind_provider_request_context(request.clone(), "dapp", Some(&dapp_url)).unwrap();
    assert_eq!(bound.origin.as_deref(), Some("https://app.example"));

    let forged = ProviderRequest {
        origin: Some(TRUSTED_UI_ORIGIN.to_owned()),
        ..request.clone()
    };
    assert!(
        bind_provider_request_context(forged, "dapp", Some(&dapp_url))
            .unwrap_err()
            .to_string()
            .contains("does not match window origin")
    );

    let trusted = bind_provider_request_context(request, "main", Some(&dapp_url)).unwrap();
    assert_eq!(trusted.origin.as_deref(), Some(TRUSTED_UI_ORIGIN));
}

#[test]
fn desktop_devtools_are_explicit_debug_opt_in() {
    assert!(!desktop_devtools_enabled_from_value(None));
    assert_eq!(
        desktop_devtools_enabled_from_value(Some("1")),
        cfg!(debug_assertions)
    );
    assert!(!desktop_devtools_enabled_from_value(Some("false")));
}

#[test]
fn dapp_compatibility_check_request_defaults_to_read_only() {
    let request = DappCompatibilityCheckRequest { mode: None };

    let normalized = request.normalized().unwrap();

    assert_eq!(
        normalized,
        NormalizedDappCompatibilityCheckRequest { mode: "read" }
    );
}

#[test]
fn dapp_compatibility_check_request_rejects_interactive_mode() {
    let request = DappCompatibilityCheckRequest {
        mode: Some("interactive".to_owned()),
    };

    let error = request.normalized().unwrap_err().to_string();

    assert!(error.contains("only supports read mode"));
}

#[test]
fn signer_helper_stderr_summary_redacts_contents() {
    let summary = signer_helper_stderr_summary(b"secret save image bytes and recovery material");

    assert!(summary.contains("bytes redacted"));
    assert!(!summary.contains("secret"));
    assert!(!summary.contains("recovery"));
    assert_eq!(signer_helper_stderr_summary(b""), "empty");
}

#[cfg(unix)]
#[test]
fn signer_helper_wait_drains_large_stdout_before_child_exit() {
    use std::os::unix::fs::PermissionsExt;

    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let script_path = std::env::temp_dir().join(format!(
        "framkey-desktop-large-stdout-{}-{unique}.sh",
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
fn dapp_session_location_sanitizes_url_and_origin() {
    let location =
        dapp_session_location("https://app.uniswap.org/swap?secret=token#fragment").unwrap();

    assert_eq!(
        location,
        DappSessionLocation {
            url: Some("https://app.uniswap.org/swap".to_owned()),
            origin: Some("https://app.uniswap.org".to_owned()),
        }
    );
}

#[test]
fn dapp_navigation_request_rejects_unknown_action() {
    let request = DappNavigationRequest {
        action: "open-ended".to_owned(),
    };

    let error = request.action().unwrap_err().to_string();

    assert!(error.contains("unsupported dApp navigation action"));
}

#[test]
fn dapp_session_state_tracks_page_load_without_query_or_fragment() {
    let state = AppState::new();

    state
        .remember_dapp_open_request(dapp_session_target(Some("uniswap")).unwrap())
        .unwrap();
    state
        .remember_dapp_page_load(
            "finished",
            "https://app.uniswap.org/swap?secret=token#fragment",
        )
        .unwrap();
    let snapshot = state.dapp_session_snapshot().unwrap();

    assert_eq!(snapshot["open"], json!(true));
    assert_eq!(snapshot["targetLabel"], json!("Uniswap"));
    assert_eq!(
        snapshot["currentUrl"],
        json!("https://app.uniswap.org/swap")
    );
    assert_eq!(snapshot["origin"], json!("https://app.uniswap.org"));
    assert_eq!(snapshot["loadStatus"], json!("loaded"));
    let serialized = serde_json::to_string(&snapshot).unwrap();
    assert!(!serialized.contains("secret=token"));
    assert!(!serialized.contains("fragment"));
}

#[test]
fn startup_dapp_target_stays_closed_without_explicit_start_or_smoke() {
    let target = startup_dapp_target_from_options(None, None, None, false, false);

    assert_eq!(target, None);
}

#[test]
fn startup_dapp_target_prefers_explicit_start_values() {
    let target = startup_dapp_target_from_options(
        Some("uniswap".to_owned()),
        Some("aave".to_owned()),
        Some("https://example.invalid".to_owned()),
        true,
        true,
    );

    assert_eq!(target.as_deref(), Some("uniswap"));
}

#[test]
fn startup_dapp_target_opens_local_for_smoke_only() {
    assert_eq!(
        startup_dapp_target_from_options(None, None, None, true, false).as_deref(),
        Some("local")
    );
    assert_eq!(
        startup_dapp_target_from_options(None, None, None, false, true).as_deref(),
        Some("local")
    );
}

#[test]
fn dapp_session_state_defaults_to_no_open_app() {
    let state = AppState::new();
    let snapshot = state.dapp_session_snapshot().unwrap();

    assert_eq!(snapshot["open"], json!(false));
    assert_eq!(snapshot["targetLabel"], json!("No app open"));
    assert_eq!(snapshot["requestedUrl"], Value::Null);
    assert_eq!(snapshot["origin"], Value::Null);
    assert_eq!(snapshot["loadStatus"], json!("not_loaded"));
}

#[test]
fn recovery_picker_paths_parse_line_separated_output() {
    let paths = parse_picker_paths(
        "/Users/example/FRAMKey-Recovery/backup-01.dat\n/Volumes/TF/backup-03.dat\n",
    )
    .unwrap();

    assert_eq!(
        paths,
        vec![
            "/Users/example/FRAMKey-Recovery/backup-01.dat",
            "/Volumes/TF/backup-03.dat",
        ]
    );
}

#[test]
fn recovery_picker_rejects_control_character_paths() {
    let error = parse_picker_paths("/tmp/backup-01.dat\n/tmp/bad\u{7}.dat\n")
        .unwrap_err()
        .to_string();

    assert!(error.contains("malformed path"));
}

#[test]
fn recovery_picker_treats_user_cancel_as_non_error() {
    assert!(is_macos_user_cancelled(
        "execution error: User canceled. (-128)"
    ));
    assert!(is_macos_user_cancelled(
        "execution error: User cancelled. (-128)"
    ));
    assert!(!is_macos_user_cancelled(
        "execution error: permission denied"
    ));
}

#[test]
fn alchemy_token_configures_read_rpc_and_default_live_simulation() {
    let token = "test-alchemy-token-for-config";
    let mut config = fixture_config();
    config.rpc = rpc_config_from_env(
        config.rpc.as_ref(),
        &config.chain_id,
        None,
        None,
        Some(token.to_owned()),
        None,
        None,
    )
    .unwrap();
    config.simulation = simulation_config_from_env(
        &config.simulation,
        None,
        None,
        Some(token.to_owned()),
        None,
        None,
        None,
        true,
    )
    .unwrap();

    let rpc = config.rpc.as_ref().unwrap();
    assert_eq!(rpc.network.as_deref(), Some(DEFAULT_ALCHEMY_NETWORK));
    assert_eq!(rpc.timeout_ms, DEFAULT_RPC_TIMEOUT_MS);
    assert_eq!(
        rpc.endpoint_url,
        format!("https://{DEFAULT_ALCHEMY_NETWORK}.g.alchemy.com/v2/{token}")
    );
    let DesktopSimulationConfig::AlchemyAssetChanges {
        endpoint_url,
        network,
        timeout_ms,
        ..
    } = &config.simulation
    else {
        panic!("Alchemy token should default transaction simulation to live Alchemy");
    };
    assert_eq!(
        endpoint_url,
        &format!("https://{DEFAULT_ALCHEMY_NETWORK}.g.alchemy.com/v2/{token}")
    );
    assert_eq!(network.as_deref(), Some(DEFAULT_ALCHEMY_NETWORK));
    assert_eq!(*timeout_ms, DEFAULT_SIMULATION_TIMEOUT_MS);

    let status = status_result(&config);
    assert_eq!(status["rpc"]["kind"], "alchemy_rpc");
    assert_eq!(status["rpc"]["network"], json!(DEFAULT_ALCHEMY_NETWORK));
    assert_eq!(status["rpc"]["timeoutMs"], json!(DEFAULT_RPC_TIMEOUT_MS));
    assert_eq!(status["simulation"]["kind"], "alchemy_asset_changes");
    assert_eq!(
        status["capabilities"]["simulation"],
        "alchemy_asset_changes"
    );
    let serialized = serde_json::to_string(&status).unwrap();
    assert!(!serialized.contains(token));
    assert!(!serialized.contains("g.alchemy.com/v2"));
}

#[test]
fn explicit_rpc_url_takes_priority_over_alchemy_token() {
    let token = "token-that-must-not-be-used";
    let endpoint_url = "https://example.invalid/rpc".to_owned();
    let rpc = rpc_config_from_env(
        None,
        "0x1",
        Some(endpoint_url.clone()),
        None,
        Some(token.to_owned()),
        Some("eth-sepolia".to_owned()),
        Some(12_000),
    )
    .unwrap()
    .unwrap();

    assert_eq!(rpc.endpoint_url, endpoint_url);
    assert_eq!(rpc.network.as_deref(), Some("eth-sepolia"));
    assert_eq!(rpc.timeout_ms, 12_000);
    assert_eq!(rpc.provider(), "custom");
    assert!(!rpc.supports_alchemy_token_api());
    assert!(!rpc.endpoint_url.contains(token));
}

#[test]
fn json_rpc_config_does_not_enable_alchemy_extensions_from_network_slug() {
    let rpc = ConfigRpc::JsonRpc {
        rpc_url: "https://example.invalid/rpc".to_owned(),
        network: Some(DEFAULT_ALCHEMY_NETWORK.to_owned()),
        timeout_ms: None,
    }
    .into_runtime()
    .unwrap();

    assert_eq!(rpc.kind(), "json_rpc");
    assert_eq!(rpc.provider(), "custom");
    assert!(!rpc.supports_alchemy_token_api());
}

#[test]
fn hyperevm_supported_chain_uses_static_json_rpc_without_alchemy_token() {
    let chain = supported_chain(HYPEREVM_CHAIN_ID).unwrap();
    assert_eq!(chain.name, "Hyperliquid");
    assert_eq!(chain.native_symbol, "HYPE");
    assert_eq!(chain.rpc_provider(), "hyperliquid");
    assert_eq!(chain.rpc_network(), HYPEREVM_NETWORK);
    assert!(!chain.requires_alchemy_token());

    let rpc = rpc_config_from_env(
        None,
        HYPEREVM_CHAIN_ID,
        None,
        None,
        Some("stray-alchemy-token".to_owned()),
        None,
        None,
    )
    .unwrap()
    .unwrap();

    assert_eq!(rpc.endpoint_url, HYPEREVM_RPC_URL);
    assert_eq!(rpc.network.as_deref(), Some(HYPEREVM_NETWORK));
    assert_eq!(rpc.kind(), "json_rpc");
    assert_eq!(rpc.provider(), "hyperliquid");
    assert!(!rpc.supports_alchemy_token_api());

    let status = status_result(&DesktopConfig {
        chain_id: HYPEREVM_CHAIN_ID.to_owned(),
        rpc: Some(rpc),
        ..fixture_config()
    });
    assert_eq!(status["network"]["chainId"], json!(HYPEREVM_CHAIN_ID));
    assert_eq!(status["network"]["nativeSymbol"], json!("HYPE"));
    assert_eq!(status["network"]["rpcProvider"], json!("hyperliquid"));
    assert_eq!(status["network"]["alchemyNetwork"], Value::Null);
    assert_eq!(status["rpc"]["kind"], json!("json_rpc"));
    assert_eq!(status["rpc"]["provider"], json!("hyperliquid"));
    assert_eq!(
        status["rpc"]["capabilities"]["alchemyTokenApi"],
        json!(false)
    );
    let serialized = serde_json::to_string(&status).unwrap();
    assert!(!serialized.contains(HYPEREVM_RPC_URL));
    assert!(!serialized.contains("stray-alchemy-token"));
}

#[test]
fn switch_session_chain_to_hyperevm_needs_no_token_and_disables_alchemy_simulation() {
    let mut config = fixture_config();
    config.simulation = DesktopSimulationConfig::AlchemyAssetChanges {
        endpoint_url: "https://eth-mainnet.g.alchemy.com/v2/token-for-test".to_owned(),
        network: Some("eth-mainnet".to_owned()),
        timeout_ms: 1_000,
        default_gas: DEFAULT_SIMULATION_DEFAULT_GAS.to_owned(),
    };
    let chain = supported_chain(HYPEREVM_CHAIN_ID).unwrap();

    config.switch_to_supported_chain(chain, None).unwrap();

    assert_eq!(config.chain_id, HYPEREVM_CHAIN_ID);
    let rpc = config.rpc.as_ref().unwrap();
    assert_eq!(rpc.endpoint_url, HYPEREVM_RPC_URL);
    assert_eq!(rpc.network.as_deref(), Some(HYPEREVM_NETWORK));
    assert_eq!(rpc.provider(), "hyperliquid");
    assert!(matches!(
        config.simulation,
        DesktopSimulationConfig::LocalDecoderOnly
    ));
}

#[test]
fn rpc_health_without_rpc_reports_missing_without_endpoint() {
    let config = fixture_config();

    let health = rpc_health_snapshot(&config).unwrap();

    assert_eq!(health["operation"], json!("rpc_health"));
    assert_eq!(health["provider"], json!("alchemy"));
    assert_eq!(health["configured"], json!(false));
    assert_eq!(health["healthy"], json!(false));
    assert_eq!(health["status"], json!("missing"));
    assert_eq!(health["expectedChainId"], json!("0x1"));
    assert_eq!(health["rpc"], Value::Null);
    assert_eq!(health["tokenExposed"], json!(false));
    assert_eq!(health["rpcUrlExposed"], json!(false));
    let serialized = serde_json::to_string(&health).unwrap();
    assert!(!serialized.contains("g.alchemy.com"));
    assert!(!serialized.contains("alchemy-token"));
}

#[test]
fn rpc_health_checks_chain_and_block_without_leaking_url() {
    let (rpc_url, request_rx) = spawn_rpc_body_sequence_server(vec![
        json!({"jsonrpc": "2.0", "id": 1, "result": "0x1"}),
        json!({"jsonrpc": "2.0", "id": 1, "result": "0x10"}),
    ]);
    let mut config = fixture_config();
    config.rpc = Some(DesktopRpcConfig {
        endpoint_url: rpc_url.clone(),
        network: Some("eth-mainnet".to_owned()),
        timeout_ms: 1_000,
        provider: DesktopRpcProvider::Alchemy,
    });

    let health = rpc_health_snapshot(&config).unwrap();

    assert_eq!(health["configured"], json!(true));
    assert_eq!(health["healthy"], json!(true));
    assert_eq!(health["status"], json!("ok"));
    assert_eq!(health["expectedChainId"], json!("0x1"));
    assert_eq!(health["observedChainId"], json!("0x1"));
    assert_eq!(health["chainMatches"], json!(true));
    assert_eq!(health["latestBlock"], json!("0x10"));
    assert_eq!(health["rpc"]["kind"], json!("alchemy_rpc"));
    assert_eq!(health["rpc"]["network"], json!("eth-mainnet"));
    assert_eq!(health["error"], Value::Null);

    let rpc_request: Value = serde_json::from_str(&request_rx.recv().unwrap()).unwrap();
    assert_eq!(rpc_request["method"], "eth_chainId");
    let rpc_request: Value = serde_json::from_str(&request_rx.recv().unwrap()).unwrap();
    assert_eq!(rpc_request["method"], "eth_blockNumber");
    let serialized = serde_json::to_string(&health).unwrap();
    assert!(!serialized.contains(&rpc_url));
}

#[test]
fn rpc_health_reports_wrong_chain() {
    let (rpc_url, request_rx) = spawn_rpc_body_sequence_server(vec![
        json!({"jsonrpc": "2.0", "id": 1, "result": "0xaa36a7"}),
        json!({"jsonrpc": "2.0", "id": 1, "result": "0x20"}),
    ]);
    let mut config = fixture_config();
    config.rpc = Some(DesktopRpcConfig {
        endpoint_url: rpc_url,
        network: Some("eth-sepolia".to_owned()),
        timeout_ms: 1_000,
        provider: DesktopRpcProvider::Alchemy,
    });

    let health = rpc_health_snapshot(&config).unwrap();

    assert_eq!(health["healthy"], json!(false));
    assert_eq!(health["status"], json!("wrong_chain"));
    assert_eq!(health["expectedChainId"], json!("0x1"));
    assert_eq!(health["observedChainId"], json!("0xaa36a7"));
    assert_eq!(health["chainMatches"], json!(false));
    assert_eq!(health["latestBlock"], json!("0x20"));
    assert_eq!(health["error"]["scope"], json!("chainId"));
    assert!(
        health["error"]["message"]
            .as_str()
            .unwrap()
            .contains("expected 0x1")
    );

    let rpc_request: Value = serde_json::from_str(&request_rx.recv().unwrap()).unwrap();
    assert_eq!(rpc_request["method"], "eth_chainId");
    let rpc_request: Value = serde_json::from_str(&request_rx.recv().unwrap()).unwrap();
    assert_eq!(rpc_request["method"], "eth_blockNumber");
}

#[test]
fn simulation_provider_selection_defaults_to_alchemy_but_respects_local_override() {
    let rpc_url = "https://example.invalid/alchemy-rpc".to_owned();
    let default_live = simulation_config_from_env(
        &DesktopSimulationConfig::LocalDecoderOnly,
        None,
        Some(rpc_url.clone()),
        Some("unused-token".to_owned()),
        Some("eth-mainnet".to_owned()),
        Some(1_000),
        Some("0x12345".to_owned()),
        true,
    )
    .unwrap();
    assert!(matches!(
        default_live,
        DesktopSimulationConfig::AlchemyAssetChanges { .. }
    ));

    let local = simulation_config_from_env(
        &DesktopSimulationConfig::LocalDecoderOnly,
        Some("local_decoder_only"),
        Some(rpc_url.clone()),
        Some("unused-token".to_owned()),
        Some("eth-mainnet".to_owned()),
        Some(1_000),
        Some("0x12345".to_owned()),
        true,
    )
    .unwrap();
    assert!(matches!(local, DesktopSimulationConfig::LocalDecoderOnly));

    let file_explicit_local = simulation_config_from_env(
        &DesktopSimulationConfig::LocalDecoderOnly,
        None,
        Some(rpc_url.clone()),
        Some("unused-token".to_owned()),
        Some("eth-mainnet".to_owned()),
        Some(1_000),
        Some("0x12345".to_owned()),
        false,
    )
    .unwrap();
    assert!(matches!(
        file_explicit_local,
        DesktopSimulationConfig::LocalDecoderOnly
    ));

    let live = simulation_config_from_env(
        &DesktopSimulationConfig::LocalDecoderOnly,
        Some("alchemy_asset_changes"),
        Some(rpc_url.clone()),
        Some("unused-token".to_owned()),
        Some("eth-mainnet".to_owned()),
        Some(1_000),
        Some("0x12345".to_owned()),
        true,
    )
    .unwrap();
    let DesktopSimulationConfig::AlchemyAssetChanges {
        endpoint_url,
        network,
        timeout_ms,
        default_gas,
    } = live
    else {
        panic!("explicit Alchemy simulation provider should enable live simulation");
    };
    assert_eq!(endpoint_url, rpc_url);
    assert_eq!(network.as_deref(), Some("eth-mainnet"));
    assert_eq!(timeout_ms, 1_000);
    assert_eq!(default_gas, "0x12345");
}

#[test]
fn wallet_assets_without_rpc_reports_missing_rpc() {
    let state = AppState::new();
    let config = fixture_config();
    state
        .remember_connected_account(fixture_connected_account(
            "0x0000000000000000000000000000000000000001",
        ))
        .unwrap();

    let assets = wallet_assets_snapshot(&state, &config).unwrap();
    assert_eq!(assets["address"].as_str().unwrap().len(), 42);
    assert_eq!(assets["rpc"], Value::Null);
    assert_eq!(assets["native"]["balance"], Value::Null);
    assert_eq!(assets["tokens"].as_array().unwrap().len(), 0);
    assert_eq!(assets["tokenScan"]["status"], "rpc_missing");
    assert_eq!(assets["errors"][0]["scope"], "rpc");
}

#[test]
fn wallet_assets_requires_connected_account_without_loading_vault() {
    let state = AppState::new();
    let mut config = fixture_config();
    config.wallet = DesktopWalletConfig::KeychainVault;
    config.device = DeviceConfig::File {
        path: unique_test_path("missing-vault.sav"),
    };

    let error = wallet_assets_snapshot(&state, &config).unwrap_err();
    let message = error.to_string();
    assert!(message.contains("wallet account is not connected"));
    assert!(!message.contains("missing-vault.sav"));
}

#[test]
fn wallet_watch_asset_requires_review_and_updates_portfolio() {
    let state = Arc::new(AppState::new());
    let config = fixture_config();
    let request = ProviderRequest {
        id: "watch-usdc".to_owned(),
        method: "wallet_watchAsset".to_owned(),
        params: json!({
            "type": "ERC20",
            "options": {
                "address": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
                "symbol": "USDC",
                "decimals": 6,
                "image": "https://static.alchemyapi.io/images/assets/3408.png"
            }
        }),
        origin: Some("https://app.uniswap.org".to_owned()),
    };

    let worker = thread::spawn({
        let state = Arc::clone(&state);
        let config = config.clone();
        let request = request.clone();
        move || handle_provider_request(&state, &config, &request)
    });

    let review = wait_for_pending_review(&state, "wallet_watchAsset");
    assert_eq!(review.kind, review::ReviewMethodKind::WatchAsset);
    assert_eq!(review.summary["intent"], json!("watch_asset"));
    assert_eq!(
        review.summary["contractAddress"],
        json!("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
    );
    state
        .decide_review_request(&review.id, &review.decision_token, ReviewDecision::Approve)
        .unwrap();

    let response = worker.join().unwrap().unwrap();
    match response {
        ProviderResponse::Result(value) => assert_eq!(value, json!(true)),
        ProviderResponse::Error(error) => {
            panic!("unexpected provider error: {}", error.message)
        }
    }
    let reviews = state.review_queue_snapshot().unwrap();
    assert_eq!(reviews[0].status, ReviewStatus::Completed);

    state
        .remember_connected_account(fixture_connected_account(
            "0x0000000000000000000000000000000000000001",
        ))
        .unwrap();
    let assets = wallet_assets_snapshot(&state, &config).unwrap();
    assert_eq!(assets["tokenScan"]["watched"], json!(1));
    assert_eq!(assets["tokens"].as_array().unwrap().len(), 1);
    assert_eq!(assets["tokens"][0]["watched"], json!(true));
    assert_eq!(assets["tokens"][0]["metadata"]["symbol"], json!("USDC"));
    assert_eq!(assets["tokens"][0]["balance"], json!("0x0"));
    let serialized = serde_json::to_string(&assets).unwrap();
    assert!(!serialized.contains("g.alchemy.com/v2"));
}

#[test]
fn wallet_watch_asset_rejects_malformed_assets_before_review() {
    let state = AppState::new();
    let config = fixture_config();
    let request = ProviderRequest {
        id: "watch-bad".to_owned(),
        method: "wallet_watchAsset".to_owned(),
        params: json!({
            "type": "ERC20",
            "options": {
                "address": "0xnot-an-address",
                "symbol": "BAD",
                "decimals": 18
            }
        }),
        origin: Some("https://app.aave.com".to_owned()),
    };

    let error = match handle_provider_request(&state, &config, &request) {
        Ok(_) => panic!("expected malformed watch asset request to fail"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("valid EVM address"));
    assert!(state.review_queue_snapshot().unwrap().is_empty());
}

#[test]
fn wallet_state_persistence_roundtrips_watched_assets_without_secret_material() {
    let dir = std::env::temp_dir().join(format!(
        "framkey-desktop-wallet-state-roundtrip-{}-{}",
        std::process::id(),
        random_suffix()
    ));
    let path = dir.join("wallet-state.json");
    let mut store = WatchedAssetStore::new();
    store.remember(fixture_watched_asset("USDC"));

    store.write_to_path(&path).unwrap();
    let restored = WatchedAssetStore::read_from_path(&path).unwrap();

    let assets = restored.for_chain("0x1");
    assert_eq!(assets.len(), 1);
    assert_eq!(assets[0].symbol, "USDC");
    assert_eq!(
        assets[0].contract_address,
        "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
    );
    let persisted = std::fs::read_to_string(&path).unwrap();
    assert!(persisted.contains("USDC"));
    assert!(persisted.contains("app.uniswap.org"));
    assert!(!persisted.contains("g.alchemy.com/v2"));
    assert!(!persisted.contains("decisionToken"));
    assert!(!persisted.contains("walletSecret"));
    assert!(!persisted.contains("recoveryRootKey"));
    assert!(!persisted.contains("shareHex"));
    assert!(!persisted.contains("rawTransaction"));

    std::fs::remove_dir_all(dir).unwrap();
}

#[cfg(unix)]
#[test]
fn wallet_state_persistence_writes_private_file_and_directory_modes() {
    let dir = std::env::temp_dir().join(format!(
        "framkey-desktop-wallet-state-mode-{}-{}",
        std::process::id(),
        random_suffix()
    ));
    let path = dir.join("wallet-state.json");
    let mut store = WatchedAssetStore::new();
    store.remember(fixture_watched_asset("USDC"));

    store.write_to_path(&path).unwrap();

    assert_eq!(unix_mode(&dir), PRIVATE_DIR_MODE);
    assert_eq!(unix_mode(&path), PRIVATE_FILE_MODE);

    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn wallet_state_persistence_corrupt_file_falls_back_to_empty_store() {
    let dir = std::env::temp_dir().join(format!(
        "framkey-desktop-wallet-state-corrupt-{}-{}",
        std::process::id(),
        random_suffix()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("wallet-state.json");
    std::fs::write(&path, "{not-json").unwrap();

    let state = AppState::new_with_wallet_ui_state_persistence(path.clone());
    state
        .remember_connected_account(fixture_connected_account(
            "0x0000000000000000000000000000000000000001",
        ))
        .unwrap();
    let assets = wallet_assets_snapshot(&state, &fixture_config()).unwrap();

    assert_eq!(assets["tokenScan"]["watched"], json!(0));
    assert_eq!(assets["walletState"]["persistence"]["enabled"], json!(true));
    assert!(
        assets["walletState"]["persistence"]["warning"]
            .as_str()
            .unwrap()
            .contains("parse wallet UI state")
    );

    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn wallet_watch_asset_persists_and_restores_portfolio() {
    let dir = std::env::temp_dir().join(format!(
        "framkey-desktop-wallet-state-restore-{}-{}",
        std::process::id(),
        random_suffix()
    ));
    let path = dir.join("wallet-state.json");
    let state = Arc::new(AppState::new_with_wallet_ui_state_persistence(path.clone()));
    let config = fixture_config();
    let request = ProviderRequest {
        id: "watch-usdc-persisted".to_owned(),
        method: "wallet_watchAsset".to_owned(),
        params: json!({
            "type": "ERC20",
            "options": {
                "address": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48",
                "symbol": "USDC",
                "decimals": 6
            }
        }),
        origin: Some("https://app.uniswap.org".to_owned()),
    };

    let worker = thread::spawn({
        let state = Arc::clone(&state);
        let config = config.clone();
        let request = request.clone();
        move || handle_provider_request(&state, &config, &request)
    });
    let review = wait_for_pending_review(&state, "wallet_watchAsset");
    state
        .decide_review_request(&review.id, &review.decision_token, ReviewDecision::Approve)
        .unwrap();
    worker.join().unwrap().unwrap();

    let restored_state = AppState::new_with_wallet_ui_state_persistence(path.clone());
    restored_state
        .remember_connected_account(fixture_connected_account(
            "0x0000000000000000000000000000000000000001",
        ))
        .unwrap();
    let assets = wallet_assets_snapshot(&restored_state, &config).unwrap();

    assert_eq!(assets["tokenScan"]["watched"], json!(1));
    assert_eq!(
        assets["walletState"]["persistence"]["restored"],
        json!(true)
    );
    assert_eq!(
        assets["walletState"]["persistence"]["watchedAssetsRestored"],
        json!(1)
    );
    assert_eq!(assets["tokens"][0]["metadata"]["symbol"], json!("USDC"));
    assert_eq!(assets["tokens"][0]["watched"], json!(true));

    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn wallet_assets_queries_alchemy_token_balances_and_metadata() {
    let token_contract = "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48";
    let (rpc_url, request_rx) = spawn_rpc_body_sequence_server(vec![
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0xde0b6b3a7640000",
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x1234",
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "address": "0x0000000000000000000000000000000000000001",
                "tokenBalances": [
                    {
                        "contractAddress": token_contract,
                        "tokenBalance": "0x00000000000000000000000000000000000000000000000000000000000f4240"
                    },
                    {
                        "contractAddress": "0x0000000000000000000000000000000000000002",
                        "tokenBalance": "0x0"
                    },
                    {
                        "contractAddress": "0x0000000000000000000000000000000000000003",
                        "error": "token balance unavailable"
                    }
                ]
            },
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "name": "USD Coin",
                "symbol": "USDC",
                "decimals": 6,
                "logo": "https://static.alchemyapi.io/images/assets/3408.png"
            },
        }),
    ]);
    let state = AppState::new();
    let mut config = fixture_config();
    config.rpc = Some(DesktopRpcConfig {
        endpoint_url: rpc_url,
        network: Some("fixture".to_owned()),
        timeout_ms: 1_000,
        provider: DesktopRpcProvider::Alchemy,
    });
    state
        .remember_connected_account(fixture_connected_account(
            "0x0000000000000000000000000000000000000001",
        ))
        .unwrap();

    let assets = wallet_assets_snapshot(&state, &config).unwrap();
    assert_eq!(assets["native"]["balance"], "0xde0b6b3a7640000");
    assert_eq!(assets["blockNumber"], "0x1234");
    assert_eq!(assets["tokenScan"]["returned"], 3);
    assert_eq!(assets["tokenScan"]["nonzero"], 1);
    assert_eq!(assets["tokenScan"]["balanceErrors"], 1);
    assert_eq!(assets["tokenScan"]["metadataQueried"], 1);
    assert_eq!(assets["tokens"].as_array().unwrap().len(), 1);
    assert_eq!(assets["tokens"][0]["contractAddress"], token_contract);
    assert_eq!(assets["tokens"][0]["metadata"]["symbol"], "USDC");
    assert_eq!(assets["tokens"][0]["metadata"]["decimals"], 6);
    assert_eq!(assets["tokens"][0]["metadata"]["logoAvailable"], true);

    let methods = (0..4)
        .map(|_| {
            let request: Value =
                serde_json::from_str(&request_rx.recv_timeout(Duration::from_secs(1)).unwrap())
                    .unwrap();
            request["method"].as_str().unwrap().to_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        methods,
        vec![
            "eth_getBalance",
            "eth_blockNumber",
            "alchemy_getTokenBalances",
            "alchemy_getTokenMetadata",
        ]
    );
}

#[test]
fn wallet_assets_on_hyperevm_skips_alchemy_token_discovery() {
    let (rpc_url, request_rx) = spawn_rpc_body_sequence_server(vec![
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x56bc75e2d63100000",
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x2318f8e",
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x0000000000000000000000000000000000000000000000000000000000000012",
        }),
    ]);
    let state = AppState::new();
    let mut config = fixture_config();
    config.chain_id = HYPEREVM_CHAIN_ID.to_owned();
    config.rpc = Some(DesktopRpcConfig {
        endpoint_url: rpc_url,
        network: Some(HYPEREVM_NETWORK.to_owned()),
        timeout_ms: 1_000,
        provider: DesktopRpcProvider::Hyperliquid,
    });
    state
        .remember_connected_account(fixture_connected_account(
            "0x0000000000000000000000000000000000000001",
        ))
        .unwrap();
    state
        .remember_watched_asset(WatchedAsset {
            chain_id: HYPEREVM_CHAIN_ID.to_owned(),
            asset_type: "erc20".to_owned(),
            contract_address: "0x1111111111111111111111111111111111111111".to_owned(),
            symbol: "TEST".to_owned(),
            decimals: 18,
            image: None,
            origin: Some("https://hyperevm.example".to_owned()),
            watched_at_unix_ms: now_unix_ms(),
        })
        .unwrap();

    let assets = wallet_assets_snapshot(&state, &config).unwrap();
    assert_eq!(assets["native"]["symbol"], json!("HYPE"));
    assert_eq!(assets["native"]["balance"], json!("0x56bc75e2d63100000"));
    assert_eq!(assets["blockNumber"], json!("0x2318f8e"));
    assert_eq!(
        assets["tokenScan"]["status"],
        json!("token_discovery_unsupported")
    );
    assert_eq!(
        assets["tokenScan"]["provider"],
        json!("unsupported_json_rpc")
    );
    assert_eq!(assets["tokenScan"]["watched"], json!(1));
    assert_eq!(assets["tokens"].as_array().unwrap().len(), 1);
    assert_eq!(assets["tokens"][0]["metadata"]["symbol"], json!("TEST"));
    assert_eq!(assets["tokens"][0]["metadata"]["decimals"], json!(18));
    assert_eq!(
        assets["tokens"][0]["metadata"]["metadataSource"],
        json!("erc20_decimals_call")
    );
    assert_eq!(
        assets["tokens"][0]["metadata"]["decimalsTrusted"],
        json!(true)
    );

    let methods = (0..3)
        .map(|_| {
            let request: Value =
                serde_json::from_str(&request_rx.recv_timeout(Duration::from_secs(1)).unwrap())
                    .unwrap();
            request["method"].as_str().unwrap().to_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        methods,
        vec!["eth_getBalance", "eth_blockNumber", "eth_call"]
    );
    assert!(request_rx.try_recv().is_err());
}

#[test]
fn native_send_amount_decimal_parser_is_exact_and_conservative() {
    assert_eq!(
        native_amount_decimal_to_wei_hex("1").unwrap(),
        "0xde0b6b3a7640000"
    );
    assert_eq!(
        native_amount_decimal_to_wei_hex("0.000000000000000001").unwrap(),
        "0x1"
    );
    assert_eq!(
        native_amount_decimal_to_wei_hex(".5").unwrap(),
        "0x6f05b59d3b20000"
    );
    assert!(native_amount_decimal_to_wei_hex("0").is_err());
    assert!(native_amount_decimal_to_wei_hex("1.0000000000000000001").is_err());
    assert!(native_amount_decimal_to_wei_hex("1e18").is_err());
    assert!(native_amount_decimal_to_wei_hex("-1").is_err());
}

#[test]
fn token_send_amount_decimal_parser_and_calldata_are_exact() {
    assert_eq!(
        token_amount_decimal_to_raw_hex("1.23", 6, "token transfer").unwrap(),
        "0x12c4b0"
    );
    assert_eq!(
        token_amount_decimal_to_raw_hex(".000001", 6, "token transfer").unwrap(),
        "0x1"
    );
    assert_eq!(
        token_amount_decimal_to_raw_hex("42", 0, "token transfer").unwrap(),
        "0x2a"
    );
    assert_eq!(
        raw_decimal_digits_to_hex(
            "115792089237316195423570985008687907853269984665640564039457584007913129639935"
        )
        .unwrap(),
        "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    );
    assert!(token_amount_decimal_to_raw_hex("0", 6, "token transfer").is_err());
    assert!(token_amount_decimal_to_raw_hex("1.0000001", 6, "token transfer").is_err());
    assert!(token_amount_decimal_to_raw_hex("1e6", 6, "token transfer").is_err());
    assert!(
        raw_decimal_digits_to_hex(
            "115792089237316195423570985008687907853269984665640564039457584007913129639936"
        )
        .is_err()
    );

    let data =
        erc20_transfer_calldata("0x0000000000000000000000000000000000000001", "0x12c4b0").unwrap();
    assert_eq!(
        data,
        "0xa9059cbb0000000000000000000000000000000000000000000000000000000000000001000000000000000000000000000000000000000000000000000000000012c4b0"
    );
}

#[test]
fn eip1559_fee_history_defaults_are_conservative() {
    let suggestion = eip1559_fee_suggestion_from_fee_history(&json!({
        "oldestBlock": "0x1",
        "baseFeePerGas": ["0x10", "0x20"],
        "gasUsedRatio": [0.5],
        "reward": [["0x3"]],
    }))
    .unwrap();
    assert_eq!(suggestion.next_base_fee_per_gas, "0x20");
    assert_eq!(suggestion.max_priority_fee_per_gas, "0x3");
    assert_eq!(
        max_fee_from_base_fee(
            &suggestion.next_base_fee_per_gas,
            &suggestion.max_priority_fee_per_gas,
        )
        .unwrap(),
        "0x43"
    );

    let suggestion = eip1559_fee_suggestion_from_fee_history(&json!({
        "oldestBlock": "0x1",
        "baseFeePerGas": ["0x10", "0x20"],
        "gasUsedRatio": [0.5],
        "reward": [[]],
    }))
    .unwrap();
    assert_eq!(suggestion.max_priority_fee_per_gas, "0x3b9aca00");
}

#[test]
fn eip1559_fee_history_falls_back_from_pending_to_latest() {
    let (rpc_url, request_rx) = spawn_rpc_body_sequence_server(vec![
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32000,
                "message": "invalid block range"
            },
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "oldestBlock": "0x1",
                "baseFeePerGas": ["0x10", "0x20"],
                "gasUsedRatio": [0.5],
                "reward": [["0x4"]]
            },
        }),
    ]);
    let mut config = fixture_config();
    config.rpc = Some(DesktopRpcConfig {
        endpoint_url: rpc_url,
        network: Some(HYPEREVM_NETWORK.to_owned()),
        timeout_ms: 1_000,
        provider: DesktopRpcProvider::Hyperliquid,
    });

    let suggestion = eip1559_fee_suggestion(&config).unwrap();

    assert_eq!(suggestion.next_base_fee_per_gas, "0x20");
    assert_eq!(suggestion.max_priority_fee_per_gas, "0x4");
    let request: Value = serde_json::from_str(&request_rx.recv().unwrap()).unwrap();
    assert_eq!(request["method"], "eth_feeHistory");
    assert_eq!(request["params"], json!(["0x1", "pending", [50]]));
    let request: Value = serde_json::from_str(&request_rx.recv().unwrap()).unwrap();
    assert_eq!(request["method"], "eth_feeHistory");
    assert_eq!(request["params"], json!(["0x1", "latest", [50]]));
}

#[test]
fn unsupported_transaction_envelopes_are_rejected_before_review() {
    let state = AppState::new();
    let config = fixture_config();
    let wallet = "0x000000000000000000000000000000000000000a";
    let request = ProviderRequest {
        id: "blob-tx".to_owned(),
        method: "eth_sendTransaction".to_owned(),
        params: json!([
            {
                "from": wallet,
                "to": "0x0000000000000000000000000000000000000001",
                "value": "0x0",
                "data": "0x",
                "nonce": "0x0",
                "gas": "0x5208",
                "gasPrice": "0x1",
                "type": "0x3",
                "maxFeePerBlobGas": "0x1"
            }
        ]),
        origin: Some("https://example.test".to_owned()),
    };

    let error = prepare_transaction(&state, &config, &request, wallet, true).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("transaction type 3 blob envelopes are not supported")
    );
    assert!(state.review_queue_snapshot().unwrap().is_empty());
}

#[test]
fn transaction_fee_caps_are_enforced() {
    let config = fixture_config();
    let high_gas_price = json!({
        "gasPrice": "0xe8d4a51001"
    });
    let error = transaction_fee_fields(&config, high_gas_price.as_object().unwrap()).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("gasPrice exceeds FRAMKey safety cap")
    );

    let inverted_eip1559 = json!({
        "maxFeePerGas": "0x10",
        "maxPriorityFeePerGas": "0x20"
    });
    let error = transaction_fee_fields(&config, inverted_eip1559.as_object().unwrap()).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("maxPriorityFeePerGas cannot exceed maxFeePerGas")
    );
}

#[test]
fn pending_nonce_reservation_advances_local_reuse() {
    let (rpc_url, request_rx) = spawn_rpc_body_sequence_server(vec![
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x5",
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x5",
        }),
    ]);
    let state = AppState::new();
    let mut config = fixture_config();
    config.rpc = Some(DesktopRpcConfig {
        endpoint_url: rpc_url,
        network: Some("fixture".to_owned()),
        timeout_ms: 1_000,
        provider: DesktopRpcProvider::Custom,
    });
    let wallet = "0x000000000000000000000000000000000000000a";
    let request = ProviderRequest {
        id: "nonce-reservation".to_owned(),
        method: "eth_sendTransaction".to_owned(),
        params: json!([
            {
                "from": wallet,
                "to": "0x0000000000000000000000000000000000000001",
                "value": "0x0",
                "data": "0x",
                "gas": "0x5208",
                "gasPrice": "0x1"
            }
        ]),
        origin: Some("https://example.test".to_owned()),
    };

    let first = prepare_transaction(&state, &config, &request, wallet, true).unwrap();
    let second = prepare_transaction(&state, &config, &request, wallet, true).unwrap();

    assert_eq!(first.transaction.nonce, "0x5");
    assert_eq!(second.transaction.nonce, "0x6");
    assert!(first.nonce_reservation.is_some());
    assert!(second.nonce_reservation.is_some());
    for _ in 0..2 {
        let rpc_request: Value = serde_json::from_str(&request_rx.recv().unwrap()).unwrap();
        assert_eq!(rpc_request["method"], "eth_getTransactionCount");
        assert_eq!(rpc_request["params"][1], json!("pending"));
    }
}

#[test]
fn pending_nonce_reservation_reuses_released_gap_before_higher_nonce() {
    let state = AppState::new();
    let wallet = "0x000000000000000000000000000000000000000a";

    let first = state
        .reserve_transaction_nonce("0x1", wallet, "0x5")
        .unwrap();
    let second = state
        .reserve_transaction_nonce("0x1", wallet, "0x5")
        .unwrap();
    assert_eq!(first.nonce, "0x5");
    assert_eq!(second.nonce, "0x6");

    state.release_transaction_nonce(&first);
    let third = state
        .reserve_transaction_nonce("0x1", wallet, "0x5")
        .unwrap();
    assert_eq!(third.nonce, "0x5");

    state.release_transaction_nonce(&second);
    let fourth = state
        .reserve_transaction_nonce("0x1", wallet, "0x5")
        .unwrap();
    assert_eq!(fourth.nonce, "0x6");
}

#[test]
fn trusted_native_send_requires_review_and_records_activity() {
    let (rpc_url, request_rx) = spawn_rpc_body_sequence_server(vec![
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x0",
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x5208",
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "oldestBlock": "0x1",
                "baseFeePerGas": ["0x1", "0x2"],
                "gasUsedRatio": [0.5],
                "reward": [["0x1"]],
            },
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        }),
    ]);
    let state = Arc::new(AppState::new());
    let mut config = fixture_config();
    config.rpc = Some(DesktopRpcConfig {
        endpoint_url: rpc_url,
        network: Some("fixture".to_owned()),
        timeout_ms: 1_000,
        provider: DesktopRpcProvider::Custom,
    });
    state.load_and_connect_account(&config).unwrap();
    let request = NativeTransferRequest {
        to: "0x0000000000000000000000000000000000000001".to_owned(),
        amount: "0.000000000000000001".to_owned(),
        chain_id: Some("0x1".to_owned()),
    };

    let worker_state = Arc::clone(&state);
    let worker_config = config.clone();
    let worker = thread::spawn(move || {
        send_native_transfer_from_trusted_ui(&worker_state, &worker_config, request)
    });

    let review = wait_for_pending_review(&state, "eth_sendTransaction");
    assert_eq!(review.kind, review::ReviewMethodKind::Transaction);
    assert_eq!(review.origin.as_deref(), Some(TRUSTED_UI_ORIGIN));
    assert_eq!(
        review.summary["to"],
        json!("0x0000000000000000000000000000000000000001")
    );
    assert_eq!(review.summary["value"], json!("0x1"));
    let decision = if review.summary["policy"]["canSign"] == json!(true) {
        ReviewDecision::Approve
    } else {
        ReviewDecision::ApproveWithRisk
    };
    state
        .decide_review_request(&review.id, &review.decision_token, decision)
        .unwrap();

    let result = worker.join().unwrap().unwrap();
    assert_eq!(result["operation"], json!("send_native_transfer"));
    assert_eq!(result["status"], json!("broadcast"));
    assert_eq!(
        result["transactionHash"],
        json!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    );
    assert_eq!(result["value"], json!("0x1"));
    assert_eq!(result["reviewOrigin"], json!(TRUSTED_UI_ORIGIN));

    let methods = (0..4)
        .map(|_| {
            let request: Value = serde_json::from_str(&request_rx.recv().unwrap()).unwrap();
            request["method"].as_str().unwrap().to_owned()
        })
        .collect::<Vec<_>>();
    assert_eq!(
        methods,
        vec![
            "eth_getTransactionCount",
            "eth_estimateGas",
            "eth_feeHistory",
            "eth_sendRawTransaction",
        ]
    );

    let activity = transaction_activity_snapshot(&state, &config, false).unwrap();
    assert_eq!(activity["count"], json!(1));
    assert_eq!(activity["items"][0]["origin"], json!(TRUSTED_UI_ORIGIN));
    assert_eq!(activity["items"][0]["status"], json!("broadcast"));
    assert_eq!(
        activity["items"][0]["transactionHash"],
        json!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    );
    let serialized = serde_json::to_string(&activity).unwrap();
    assert!(!serialized.contains("rawTransaction"));
    assert!(!serialized.contains("decisionToken"));
}

#[test]
fn aave_borrow_review_attaches_account_health_evidence() {
    let from = "0x000000000000000000000000000000000000000a";
    let pool = "0x87870bca3f3fd6335c3f4ce8392d69350b4fa4e2";
    let data = format!(
        "0xa415bcad{}{}{}{}{}",
        abi_address_word("0x1111111111111111111111111111111111111111"),
        abi_u256_word(50_000_000),
        abi_u256_word(2),
        abi_u256_word(0),
        abi_address_word(from),
    );
    let (rpc_url, request_rx) = spawn_rpc_body_sequence_server(vec![
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": aave_account_data_result(2_000_000_000_000_000_000),
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x",
        }),
    ]);
    let state = AppState::new();
    let mut config = fixture_config();
    config.rpc = Some(DesktopRpcConfig {
        endpoint_url: rpc_url.clone(),
        network: Some("fixture".to_owned()),
        timeout_ms: 1_000,
        provider: DesktopRpcProvider::Custom,
    });
    let request = ProviderRequest {
        id: "aave-borrow-review".to_owned(),
        method: "eth_sendTransaction".to_owned(),
        params: json!([
            {
                "chainId": "0x1",
                "from": from,
                "to": pool,
                "value": "0x0",
                "data": data,
            }
        ]),
        origin: Some("https://app.aave.com".to_owned()),
    };

    let review = state.capture_review_request(&config, &request).unwrap();

    assert_eq!(review.kind, review::ReviewMethodKind::Transaction);
    assert_eq!(
        review.summary["simulation"]["protocolEvidence"]["aave"]["status"],
        json!("ok")
    );
    assert_eq!(
        review.summary["simulation"]["protocolEvidence"]["aave"]["healthFactor"],
        json!("2000000000000000000")
    );
    assert_eq!(
        review.summary["simulation"]["protocolEvidence"]["aave"]["transactionDryRun"]["status"],
        json!("ok")
    );
    assert_eq!(review.summary["policy"]["decision"], json!("allowed"));
    assert!(review.summary["policy"]["canSign"].as_bool().unwrap());
    assert!(policy_blocker(&review.summary, "aave_borrow_health_factor_unknown").is_none());
    assert!(policy_blocker(&review.summary, "aave_transaction_dry_run_missing").is_none());
    assert!(policy_blocker(&review.summary, "aave_health_factor_caution").is_none());

    let rpc_request: Value = serde_json::from_str(&request_rx.recv().unwrap()).unwrap();
    assert_eq!(rpc_request["method"], "eth_call");
    assert_eq!(rpc_request["params"][0]["to"], json!(pool));
    let dry_run_request: Value = serde_json::from_str(&request_rx.recv().unwrap()).unwrap();
    assert_eq!(dry_run_request["method"], "eth_call");
    assert_eq!(dry_run_request["params"][0]["to"], json!(pool));
    assert_eq!(dry_run_request["params"][0]["data"], json!(data));
    assert!(
        rpc_request["params"][0]["data"]
            .as_str()
            .unwrap()
            .starts_with("0xbf92857c")
    );
    let serialized = serde_json::to_string(&review.summary).unwrap();
    assert!(!serialized.contains(&rpc_url));
}

#[test]
fn trusted_native_send_rejects_invalid_request_before_review() {
    let state = AppState::new();
    let mut config = fixture_config();
    config.rpc = Some(DesktopRpcConfig {
        endpoint_url: "http://127.0.0.1:9".to_owned(),
        network: Some("fixture".to_owned()),
        timeout_ms: 1_000,
        provider: DesktopRpcProvider::Custom,
    });

    let error = send_native_transfer_from_trusted_ui(
        &state,
        &config,
        NativeTransferRequest {
            to: "not-an-address".to_owned(),
            amount: "1".to_owned(),
            chain_id: Some("0x1".to_owned()),
        },
    )
    .unwrap_err();

    assert!(error.to_string().contains("valid EVM address"));
    assert!(state.review_queue_snapshot().unwrap().is_empty());

    let error = send_native_transfer_from_trusted_ui(
        &state,
        &config,
        NativeTransferRequest {
            to: "0x0000000000000000000000000000000000000001".to_owned(),
            amount: "0".to_owned(),
            chain_id: Some("0x1".to_owned()),
        },
    )
    .unwrap_err();

    assert!(error.to_string().contains("greater than zero"));
    assert!(state.review_queue_snapshot().unwrap().is_empty());
}

#[test]
fn trusted_token_send_requires_review_and_records_activity() {
    let (rpc_url, request_rx) = spawn_rpc_body_sequence_server(vec![
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x0000000000000000000000000000000000000000000000000000000000000006",
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x0",
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x11170",
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "oldestBlock": "0x1",
                "baseFeePerGas": ["0x1", "0x2"],
                "gasUsedRatio": [0.5],
                "reward": [["0x1"]],
            },
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "name": "USD Coin",
                "symbol": "USDC",
                "decimals": 6,
                "logo": null,
            },
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        }),
    ]);
    let state = Arc::new(AppState::new());
    let mut config = fixture_config();
    config.rpc = Some(DesktopRpcConfig {
        endpoint_url: rpc_url,
        network: Some("fixture".to_owned()),
        timeout_ms: 1_000,
        provider: DesktopRpcProvider::Alchemy,
    });
    state.load_and_connect_account(&config).unwrap();
    let request = TokenTransferRequest {
        token_contract: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_owned(),
        to: "0x0000000000000000000000000000000000000001".to_owned(),
        amount: "1.23".to_owned(),
        decimals: Some(6),
        symbol: Some("USDC".to_owned()),
        chain_id: Some("0x1".to_owned()),
    };

    let worker_state = Arc::clone(&state);
    let worker_config = config.clone();
    let worker = thread::spawn(move || {
        send_token_transfer_from_trusted_ui(&worker_state, &worker_config, request)
    });

    let review = wait_for_pending_review(&state, "eth_sendTransaction");
    assert_eq!(review.kind, review::ReviewMethodKind::Transaction);
    assert_eq!(review.origin.as_deref(), Some(TRUSTED_UI_ORIGIN));
    assert_eq!(
        review.summary["to"],
        json!("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
    );
    assert_eq!(review.summary["value"], json!("0x0"));
    assert_eq!(
        review.summary["simulation"]["decodedCall"]["function"],
        json!("transfer(address,uint256)")
    );
    assert_eq!(review.summary["policy"]["decision"], json!("allowed"));
    assert_eq!(review.summary["policy"]["canSign"], json!(true));
    state
        .decide_review_request(&review.id, &review.decision_token, ReviewDecision::Approve)
        .unwrap();

    let result = worker.join().unwrap().unwrap();
    assert_eq!(result["operation"], json!("send_token_transfer"));
    assert_eq!(result["status"], json!("broadcast"));
    assert_eq!(
        result["transactionHash"],
        json!("0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
    );
    assert_eq!(result["rawAmount"], json!("0x12c4b0"));
    assert_eq!(result["symbol"], json!("USDC"));
    assert_eq!(result["reviewOrigin"], json!(TRUSTED_UI_ORIGIN));

    let requests = (0..6)
        .map(|_| serde_json::from_str::<Value>(&request_rx.recv().unwrap()).unwrap())
        .collect::<Vec<_>>();
    let methods = requests
        .iter()
        .map(|request| request["method"].as_str().unwrap().to_owned())
        .collect::<Vec<_>>();
    assert_eq!(
        methods,
        vec![
            "eth_call",
            "eth_getTransactionCount",
            "eth_estimateGas",
            "eth_feeHistory",
            "alchemy_getTokenMetadata",
            "eth_sendRawTransaction",
        ]
    );
    assert_eq!(requests[0]["params"][0]["data"], json!("0x313ce567"));
    assert_eq!(
        requests[2]["params"][0]["to"],
        json!("0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48")
    );
    assert_eq!(requests[2]["params"][0]["value"], json!("0x0"));
    assert!(
        requests[2]["params"][0]["data"]
            .as_str()
            .unwrap()
            .starts_with("0xa9059cbb")
    );

    let activity = transaction_activity_snapshot(&state, &config, false).unwrap();
    assert_eq!(activity["count"], json!(1));
    assert_eq!(activity["items"][0]["origin"], json!(TRUSTED_UI_ORIGIN));
    assert_eq!(activity["items"][0]["status"], json!("broadcast"));
    assert_eq!(
        activity["items"][0]["call"],
        json!("transfer(address,uint256)")
    );
    let serialized = serde_json::to_string(&activity).unwrap();
    assert!(!serialized.contains("rawTransaction"));
    assert!(!serialized.contains("decisionToken"));
}

#[test]
fn trusted_token_send_rejects_invalid_request_before_review() {
    let state = AppState::new();
    let mut config = fixture_config();
    config.rpc = Some(DesktopRpcConfig {
        endpoint_url: "http://127.0.0.1:9".to_owned(),
        network: Some("fixture".to_owned()),
        timeout_ms: 1_000,
        provider: DesktopRpcProvider::Custom,
    });

    let error = send_token_transfer_from_trusted_ui(
        &state,
        &config,
        TokenTransferRequest {
            token_contract: "not-an-address".to_owned(),
            to: "0x0000000000000000000000000000000000000001".to_owned(),
            amount: "1".to_owned(),
            decimals: Some(6),
            symbol: Some("USDC".to_owned()),
            chain_id: Some("0x1".to_owned()),
        },
    )
    .unwrap_err();

    assert!(error.to_string().contains("token contract"));
    assert!(state.review_queue_snapshot().unwrap().is_empty());

    let error = send_token_transfer_from_trusted_ui(
        &state,
        &config,
        TokenTransferRequest {
            token_contract: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_owned(),
            to: "0x0000000000000000000000000000000000000001".to_owned(),
            amount: "0.0000001".to_owned(),
            decimals: Some(6),
            symbol: Some("USDC".to_owned()),
            chain_id: Some("0x1".to_owned()),
        },
    )
    .unwrap_err();

    assert!(error.to_string().contains("more than 6 decimal places"));
    assert!(state.review_queue_snapshot().unwrap().is_empty());
}

#[test]
fn trusted_token_send_rejects_dapp_decimal_mismatch_before_review() {
    let (rpc_url, request_rx) = spawn_rpc_body_sequence_server(vec![json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": "0x0000000000000000000000000000000000000000000000000000000000000012",
    })]);
    let state = AppState::new();
    let mut config = fixture_config();
    config.rpc = Some(DesktopRpcConfig {
        endpoint_url: rpc_url,
        network: Some("fixture".to_owned()),
        timeout_ms: 1_000,
        provider: DesktopRpcProvider::Custom,
    });

    let error = send_token_transfer_from_trusted_ui(
        &state,
        &config,
        TokenTransferRequest {
            token_contract: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_owned(),
            to: "0x0000000000000000000000000000000000000001".to_owned(),
            amount: "1".to_owned(),
            decimals: Some(6),
            symbol: Some("USDC".to_owned()),
            chain_id: Some("0x1".to_owned()),
        },
    )
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("do not match trusted contract decimals")
    );
    assert!(state.review_queue_snapshot().unwrap().is_empty());
    let rpc_request: Value = serde_json::from_str(&request_rx.recv().unwrap()).unwrap();
    assert_eq!(rpc_request["method"], "eth_call");
    assert_eq!(rpc_request["params"][0]["data"], json!("0x313ce567"));
}

#[test]
fn eth_accounts_requires_origin_permission_and_connected_session() {
    let state = AppState::new();
    let mut config = fixture_config();
    config.wallet = DesktopWalletConfig::KeychainVault;
    config.device = DeviceConfig::File {
        path: unique_test_path("eth-accounts-missing-vault.sav"),
    };
    let request = ProviderRequest {
        id: "accounts-before-grant".to_owned(),
        method: "eth_accounts".to_owned(),
        params: json!([]),
        origin: Some("https://example.test".to_owned()),
    };

    let ProviderResponse::Result(accounts) =
        handle_provider_request(&state, &config, &request).unwrap()
    else {
        panic!("expected eth_accounts result");
    };
    assert_eq!(accounts, json!([]));

    state
        .grant_account_permission("https://example.test".to_owned())
        .unwrap();
    let ProviderResponse::Result(accounts) =
        handle_provider_request(&state, &config, &request).unwrap()
    else {
        panic!("expected eth_accounts result after grant");
    };
    assert_eq!(accounts, json!([]));

    state
        .remember_connected_account(fixture_connected_account(
            "0x0000000000000000000000000000000000000001",
        ))
        .unwrap();
    let ProviderResponse::Result(accounts) =
        handle_provider_request(&state, &config, &request).unwrap()
    else {
        panic!("expected eth_accounts result with connected account");
    };
    assert_eq!(
        accounts,
        json!(["0x0000000000000000000000000000000000000001"])
    );
}

#[test]
fn repeated_request_accounts_uses_connected_session_without_loading_vault() {
    let state = AppState::new();
    let mut config = fixture_config();
    config.wallet = DesktopWalletConfig::KeychainVault;
    config.device = DeviceConfig::File {
        path: unique_test_path("request-accounts-missing-vault.sav"),
    };
    state
        .remember_connected_account(fixture_connected_account(
            "0x0000000000000000000000000000000000000001",
        ))
        .unwrap();
    state
        .grant_account_permission("https://dapp.example".to_owned())
        .unwrap();
    let request = ProviderRequest {
        id: "request-accounts-already-connected".to_owned(),
        method: "eth_requestAccounts".to_owned(),
        params: json!([]),
        origin: Some("https://dapp.example".to_owned()),
    };

    let ProviderResponse::Result(accounts) =
        handle_provider_request(&state, &config, &request).unwrap()
    else {
        panic!("expected eth_requestAccounts result");
    };
    assert_eq!(
        accounts,
        json!(["0x0000000000000000000000000000000000000001"])
    );
    assert!(state.review_queue_snapshot().unwrap().is_empty());
}

#[test]
fn repeated_trusted_get_account_uses_connected_session_without_loading_vault() {
    let state = AppState::new();
    let mut config = fixture_config();
    config.wallet = DesktopWalletConfig::KeychainVault;
    config.device = DeviceConfig::File {
        path: unique_test_path("trusted-get-account-missing-vault.sav"),
    };
    state
        .remember_connected_account(DesktopAccount {
            address: "0x0000000000000000000000000000000000000001".to_owned(),
            accounts: json!([]),
            wallet: json!({
                "kind": "keychain_vault",
                "mock": false,
            }),
            metadata: json!({
                "walletSecretHash": "should-not-remain-in-session",
                "walletId": "wallet-id-should-not-remain",
            }),
            keychain: Some(json!({
                "service": "service-should-not-remain",
                "account": "account-should-not-remain",
                "itemId": "item-id-should-not-remain",
                "deviceId": "device-id-should-not-remain",
                "kekId": "kek-id-should-not-remain",
            })),
            helper_report: Some(json!({
                "path": "/private/helper/path/should-not-remain",
                "blake3": "helper-hash-should-not-remain",
            })),
        })
        .unwrap();
    let request = ProviderRequest {
        id: "trusted-get-account-already-connected".to_owned(),
        method: "framkey_getAccount".to_owned(),
        params: json!([]),
        origin: Some(TRUSTED_UI_ORIGIN.to_owned()),
    };

    let ProviderResponse::Result(account) =
        handle_provider_request(&state, &config, &request).unwrap()
    else {
        panic!("expected trusted framkey_getAccount result");
    };
    assert_eq!(
        account["address"],
        json!("0x0000000000000000000000000000000000000001")
    );
    assert_eq!(account["wallet"]["kind"], json!("connected_session"));
    assert_eq!(account["wallet"]["scope"], json!("address_only"));
    assert_eq!(account["metadata"], json!({}));
    assert_eq!(account["keychain"], Value::Null);
    assert_eq!(account["signerHelper"], Value::Null);
    let serialized = serde_json::to_string(&account).unwrap();
    assert!(!serialized.contains("should-not-remain"));
}

#[test]
fn mock_wallet_account_exposes_btc_receive_balance_and_controlled_send() {
    let state = AppState::new();
    let config = fixture_config();

    let account = state.load_account(&config).unwrap();
    let accounts = account.accounts.as_array().expect("accounts array");

    let evm = accounts
        .iter()
        .find(|account| account["family"] == json!("evm"))
        .expect("evm account");
    assert_eq!(evm["address"], json!(account.address));
    assert_eq!(evm["capabilities"]["dappProvider"], json!(true));

    let btc_accounts: Vec<_> = accounts
        .iter()
        .filter(|account| account["family"] == json!("btc"))
        .collect();
    assert_eq!(btc_accounts.len(), 2);

    let btc = btc_accounts
        .iter()
        .copied()
        .find(|account| account["network"] == json!("bitcoin-mainnet"))
        .expect("btc mainnet account");
    assert_eq!(btc["network"], json!("bitcoin-mainnet"));
    assert!(
        btc["address"]
            .as_str()
            .expect("btc address")
            .starts_with("bc1q")
    );
    assert_eq!(btc["capabilities"]["receive"], json!(true));
    assert_eq!(btc["capabilities"]["balance"], json!(true));
    assert_eq!(btc["capabilities"]["send"], json!(true));
    assert_eq!(btc["capabilities"]["psbtSign"], json!(true));
    assert_eq!(
        btc["capabilities"]["balanceStatus"],
        json!("enabled_esplora_utxo_index")
    );

    let btc_testnet = btc_accounts
        .iter()
        .copied()
        .find(|account| account["network"] == json!("bitcoin-testnet4"))
        .expect("btc testnet4 account");
    assert_eq!(btc_testnet["selectedTestNetwork"], json!(true));
    assert!(
        btc_testnet["address"]
            .as_str()
            .expect("btc testnet address")
            .starts_with("tb1q")
    );
    assert_eq!(btc_testnet["capabilities"]["receive"], json!(true));
    assert_eq!(btc_testnet["capabilities"]["balance"], json!(true));
    assert_eq!(btc_testnet["capabilities"]["send"], json!(true));
    assert_eq!(btc_testnet["capabilities"]["psbtSign"], json!(true));
}

#[test]
fn btc_balance_snapshot_reads_esplora_utxos() {
    let utxos = json!([
        {
            "txid": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "vout": 0,
            "value": 12_345,
            "status": {
                "confirmed": true,
                "block_height": 10,
                "block_hash": "bbbb",
                "block_time": 123,
            },
        },
        {
            "txid": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            "vout": 1,
            "value": 2_000,
            "status": {
                "confirmed": false,
            },
        },
    ]);
    let (esplora_url, request_rx) =
        spawn_esplora_sequence_server(vec![EsploraResponse::json(utxos)]);
    let state = AppState::new();
    let mut config = fixture_config();
    config.btc.mainnet_esplora_url = Some(esplora_url);
    let account = state.load_and_connect_account(&config).unwrap();
    let btc = account
        .accounts
        .as_array()
        .unwrap()
        .iter()
        .find(|account| account["network"] == json!("bitcoin-mainnet"))
        .unwrap()
        .clone();

    let result = btc_balance_snapshot(
        &state,
        &config,
        BtcBalanceRequest {
            network: "bitcoin-mainnet".to_owned(),
        },
    )
    .unwrap();

    assert_eq!(result["network"], json!("bitcoin-mainnet"));
    assert_eq!(result["address"], btc["address"]);
    assert_eq!(result["confirmedSat"], json!(12_345));
    assert_eq!(result["unconfirmedSat"], json!(2_000));
    assert_eq!(result["spendableSat"], json!(12_345));
    let request = request_rx.recv().unwrap();
    assert_eq!(request.method, "GET");
    assert!(request.path.starts_with(&format!(
        "/address/{}/utxo",
        btc["address"].as_str().unwrap()
    )));
}

#[test]
fn btc_balance_requires_connected_account_session() {
    let state = AppState::new();
    let config = fixture_config();

    let error = btc_balance_snapshot(
        &state,
        &config,
        BtcBalanceRequest {
            network: "bitcoin-mainnet".to_owned(),
        },
    )
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("wallet account is not connected")
    );
}

#[test]
fn trusted_btc_send_requires_review_signs_and_broadcasts() {
    let state = Arc::new(AppState::new());
    let mut config = fixture_config();
    let account = state.load_and_connect_account(&config).unwrap();
    let btc = account
        .accounts
        .as_array()
        .unwrap()
        .iter()
        .find(|account| account["network"] == json!("bitcoin-testnet4"))
        .unwrap()
        .clone();
    let btc_address = btc["address"].as_str().unwrap().to_owned();
    let utxos = json!([
        {
            "txid": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            "vout": 0,
            "value": 20_000,
            "status": {
                "confirmed": true,
                "block_height": 100,
            },
        },
    ]);
    let (esplora_url, request_rx) = spawn_esplora_sequence_server(vec![
        EsploraResponse::json(utxos),
        EsploraResponse::computed_txid(),
        EsploraResponse::json(json!([])),
    ]);
    config.btc.testnet4_esplora_url = Some(esplora_url);

    let worker_state = Arc::clone(&state);
    let worker_config = config.clone();
    let worker = thread::spawn(move || {
        send_btc_transfer_from_trusted_ui(
            &worker_state,
            &worker_config,
            BtcTransferRequest {
                network: "bitcoin-testnet4".to_owned(),
                to_address: btc_address,
                amount_sat: "1000".to_owned(),
                fee_rate_sat_vb: Some(2),
            },
        )
    });

    let review = wait_for_pending_review(&state, "framkey_btcSendTransaction");
    assert_eq!(review.kind, review::ReviewMethodKind::BtcTransaction);
    assert_eq!(review.origin.as_deref(), Some(TRUSTED_UI_ORIGIN));
    assert_eq!(review.summary["network"], json!("bitcoin-testnet4"));
    assert_eq!(review.summary["policy"]["canSign"], json!(true));
    state
        .decide_review_request(&review.id, &review.decision_token, ReviewDecision::Approve)
        .unwrap();

    let result = worker.join().unwrap().unwrap();
    assert_eq!(result["operation"], json!("send_btc_transfer"));
    assert_eq!(result["status"], json!("broadcast"));
    assert_eq!(result["network"], json!("bitcoin-testnet4"));
    assert_eq!(result["amountSat"], json!(1000));
    assert_eq!(result["transactionId"].as_str().unwrap().len(), 64);

    let first = request_rx.recv().unwrap();
    assert_eq!(first.method, "GET");
    let second = request_rx.recv().unwrap();
    assert_eq!(second.method, "POST");
    assert_eq!(second.path, "/tx");
    assert!(second.body.len() > 100);

    let activity = state.transaction_activity_snapshot().unwrap();
    assert_eq!(activity.len(), 1);
    assert_eq!(activity[0].method, "framkey_btcSendTransaction");
    assert_eq!(activity[0].status, "broadcast");
    assert_eq!(activity[0].chain_id.as_deref(), Some("bitcoin-testnet4"));
    assert_eq!(activity[0].receipt_status, None);
}

#[test]
fn trusted_btc_send_requires_connected_account_session() {
    let state = AppState::new();
    let config = fixture_config();

    let error = send_btc_transfer_from_trusted_ui(
        &state,
        &config,
        BtcTransferRequest {
            network: "bitcoin-testnet4".to_owned(),
            to_address: "tb1q3w0hl5vxesce4rq0x6rpk6q4drj58mtcx7kwku".to_owned(),
            amount_sat: "1000".to_owned(),
            fee_rate_sat_vb: Some(2),
        },
    )
    .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("wallet account is not connected")
    );
    assert!(state.review_queue_snapshot().unwrap().is_empty());
}

#[test]
fn provider_rejects_btc_send_method_without_review_capture() {
    let state = AppState::new();
    let config = fixture_config();
    state
        .remember_connected_account(fixture_connected_account(
            "0x0000000000000000000000000000000000000001",
        ))
        .unwrap();
    state
        .grant_account_permission("https://dapp.example".to_owned())
        .unwrap();
    let request = ProviderRequest {
        id: "btc-from-dapp".to_owned(),
        method: "framkey_btcSendTransaction".to_owned(),
        params: json!({}),
        origin: Some("https://dapp.example".to_owned()),
    };

    let error = match handle_provider_request(&state, &config, &request) {
        Ok(_) => panic!("expected BTC provider method to be unsupported"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("trusted UI-only"));
    assert!(state.review_queue_snapshot().unwrap().is_empty());
}

#[test]
fn btc_broadcast_failure_redacts_backend_body() {
    let state = Arc::new(AppState::new());
    let mut config = fixture_config();
    let account = state.load_and_connect_account(&config).unwrap();
    let btc = account
        .accounts
        .as_array()
        .unwrap()
        .iter()
        .find(|account| account["network"] == json!("bitcoin-testnet4"))
        .unwrap()
        .clone();
    let btc_address = btc["address"].as_str().unwrap().to_owned();
    let utxos = json!([
        {
            "txid": "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
            "vout": 0,
            "value": 20_000,
            "status": {
                "confirmed": true,
                "block_height": 100,
            },
        },
    ]);
    let leaked_body = "backend echoed signed transaction raw hex 020000000001...";
    let (esplora_url, _request_rx) = spawn_esplora_sequence_server(vec![
        EsploraResponse::json(utxos),
        EsploraResponse::error(400, leaked_body),
    ]);
    config.btc.testnet4_esplora_url = Some(esplora_url);

    let worker_state = Arc::clone(&state);
    let worker_config = config.clone();
    let worker = thread::spawn(move || {
        send_btc_transfer_from_trusted_ui(
            &worker_state,
            &worker_config,
            BtcTransferRequest {
                network: "bitcoin-testnet4".to_owned(),
                to_address: btc_address,
                amount_sat: "1000".to_owned(),
                fee_rate_sat_vb: Some(2),
            },
        )
    });

    let review = wait_for_pending_review(&state, "framkey_btcSendTransaction");
    state
        .decide_review_request(&review.id, &review.decision_token, ReviewDecision::Approve)
        .unwrap();

    let error = worker.join().unwrap().unwrap_err();
    let message = error.to_string();
    assert!(message.contains("HTTP 400"));
    assert!(!message.contains(leaked_body));
    let activity = state.transaction_activity_snapshot().unwrap();
    assert_eq!(activity[0].status, "failed");
    assert!(
        !activity[0]
            .error
            .as_deref()
            .unwrap_or("")
            .contains(leaked_body)
    );
}

#[test]
fn transaction_review_uses_connected_session_address_without_loading_vault() {
    let state = Arc::new(AppState::new());
    let mut config = fixture_config();
    config.wallet = DesktopWalletConfig::KeychainVault;
    config.device = DeviceConfig::File {
        path: unique_test_path("transaction-missing-vault.sav"),
    };
    state
        .remember_connected_account(fixture_connected_account(
            "0x0000000000000000000000000000000000000001",
        ))
        .unwrap();
    state
        .grant_account_permission("https://dapp.example".to_owned())
        .unwrap();
    let request = ProviderRequest {
        id: "transaction-without-from".to_owned(),
        method: "eth_sendTransaction".to_owned(),
        params: json!([
            {
                "to": "0x0000000000000000000000000000000000000002",
                "value": "0x0",
                "nonce": "0x0",
                "gas": "0x5208",
                "gasPrice": "0x1"
            }
        ]),
        origin: Some("https://dapp.example".to_owned()),
    };

    let worker_state = Arc::clone(&state);
    let worker_config = config.clone();
    let worker =
        thread::spawn(move || handle_provider_request(&worker_state, &worker_config, &request));

    let review = wait_for_pending_review(&state, "eth_sendTransaction");
    assert_eq!(review.kind, review::ReviewMethodKind::Transaction);
    assert_eq!(
        review.summary["from"],
        json!("0x0000000000000000000000000000000000000001")
    );
    state
        .decide_review_request(&review.id, &review.decision_token, ReviewDecision::Reject)
        .unwrap();
    let error = match worker.join().unwrap() {
        Ok(_) => panic!("expected rejected transaction to fail"),
        Err(error) => error,
    };
    assert!(error.to_string().contains("rejected"));
}

#[test]
fn disconnect_account_session_clears_account_state() {
    let state = AppState::new();
    let config = fixture_config();
    state.load_and_connect_account(&config).unwrap();
    state
        .grant_account_permission("https://dapp.example".to_owned())
        .unwrap();

    let result = state.disconnect_account_session().unwrap();

    assert_eq!(result["accountCleared"], json!(true));
    assert_eq!(result["accountPermissionsCleared"], json!(1));
    assert!(state.connected_account_address().unwrap().is_none());
    assert!(state.account_permission_snapshot().unwrap().is_empty());
}

#[test]
fn disconnect_supersedes_in_flight_connect_result() {
    let state = AppState::new();
    let connect_sequence = state.begin_account_connect_intent().unwrap();

    let result = state.disconnect_account_session().unwrap();

    assert_eq!(result["accountCleared"], json!(false));
    let error = state
        .remember_connected_account_for_intent(
            &fixture_connected_account("0x0000000000000000000000000000000000000001"),
            connect_sequence,
        )
        .unwrap_err();
    assert!(error.to_string().contains("superseded"));
    assert!(state.connected_account_address().unwrap().is_none());
}

#[test]
fn newer_connect_supersedes_older_connect_result() {
    let state = AppState::new();
    let older_sequence = state.begin_account_connect_intent().unwrap();
    let newer_sequence = state.begin_account_connect_intent().unwrap();

    let error = state
        .remember_connected_account_for_intent(
            &fixture_connected_account("0x0000000000000000000000000000000000000001"),
            older_sequence,
        )
        .unwrap_err();
    assert!(error.to_string().contains("superseded"));
    assert!(state.connected_account_address().unwrap().is_none());

    state
        .remember_connected_account_for_intent(
            &fixture_connected_account("0x0000000000000000000000000000000000000002"),
            newer_sequence,
        )
        .unwrap();
    assert_eq!(
        state.connected_account_address().unwrap(),
        Some("0x0000000000000000000000000000000000000002".to_owned())
    );
}

#[test]
fn mock_eth_request_accounts_requires_review_and_grants_origin() {
    let state = Arc::new(AppState::new());
    let config = fixture_config();
    let request = ProviderRequest {
        id: "request-accounts-smoke".to_owned(),
        method: "eth_requestAccounts".to_owned(),
        params: json!([]),
        origin: Some("https://dapp.example".to_owned()),
    };

    let worker_state = Arc::clone(&state);
    let worker_config = config.clone();
    let worker =
        thread::spawn(move || handle_provider_request(&worker_state, &worker_config, &request));

    let review = wait_for_pending_review(&state, "eth_requestAccounts");
    assert_eq!(review.kind, review::ReviewMethodKind::AccountConnection);
    state
        .decide_review_request(&review.id, &review.decision_token, ReviewDecision::Approve)
        .unwrap();

    let response = worker.join().unwrap().unwrap();
    let ProviderResponse::Result(accounts) = response else {
        panic!("expected eth_requestAccounts result");
    };
    let address = accounts[0].as_str().unwrap().to_owned();
    assert!(address.starts_with("0x"));

    let account_request = ProviderRequest {
        id: "accounts-after-grant".to_owned(),
        method: "eth_accounts".to_owned(),
        params: json!([]),
        origin: Some("https://dapp.example".to_owned()),
    };
    let ProviderResponse::Result(granted_accounts) =
        handle_provider_request(&state, &config, &account_request).unwrap()
    else {
        panic!("expected eth_accounts result");
    };
    assert_eq!(granted_accounts, json!([address]));

    let completed = state
        .review_queue_snapshot()
        .unwrap()
        .into_iter()
        .find(|item| item.id == review.id)
        .unwrap();
    assert_eq!(completed.status, ReviewStatus::Completed);
    assert_eq!(
        state.account_permission_snapshot().unwrap(),
        vec!["https://dapp.example".to_owned()]
    );
}

#[test]
fn wallet_permissions_can_be_requested_queried_and_revoked() {
    let state = Arc::new(AppState::new());
    let config = fixture_config();
    let request = ProviderRequest {
        id: "wallet-request-permissions-smoke".to_owned(),
        method: "wallet_requestPermissions".to_owned(),
        params: json!([{ "eth_accounts": {} }]),
        origin: Some("https://permissions.example".to_owned()),
    };

    let worker_state = Arc::clone(&state);
    let worker_config = config.clone();
    let worker =
        thread::spawn(move || handle_provider_request(&worker_state, &worker_config, &request));

    let review = wait_for_pending_review(&state, "wallet_requestPermissions");
    assert_eq!(review.kind, review::ReviewMethodKind::AccountConnection);
    state
        .decide_review_request(&review.id, &review.decision_token, ReviewDecision::Approve)
        .unwrap();

    let response = worker.join().unwrap().unwrap();
    let ProviderResponse::Result(permissions) = response else {
        panic!("expected wallet_requestPermissions result");
    };
    assert_eq!(permissions[0]["parentCapability"], "eth_accounts");

    let get_request = ProviderRequest {
        id: "wallet-get-permissions-smoke".to_owned(),
        method: "wallet_getPermissions".to_owned(),
        params: json!([]),
        origin: Some("https://permissions.example".to_owned()),
    };
    let ProviderResponse::Result(permissions) =
        handle_provider_request(&state, &config, &get_request).unwrap()
    else {
        panic!("expected wallet_getPermissions result");
    };
    assert_eq!(permissions.as_array().unwrap().len(), 1);

    let revoke_request = ProviderRequest {
        id: "wallet-revoke-permissions-smoke".to_owned(),
        method: "wallet_revokePermissions".to_owned(),
        params: json!([{ "eth_accounts": {} }]),
        origin: Some("https://permissions.example".to_owned()),
    };
    let ProviderResponse::Result(result) =
        handle_provider_request(&state, &config, &revoke_request).unwrap()
    else {
        panic!("expected wallet_revokePermissions result");
    };
    assert_eq!(result, Value::Null);

    let ProviderResponse::Result(permissions) =
        handle_provider_request(&state, &config, &get_request).unwrap()
    else {
        panic!("expected wallet_getPermissions result after revoke");
    };
    assert_eq!(permissions, json!([]));
}

#[test]
fn keychain_status_reports_signer_helper_transaction_capability() {
    let mut config = fixture_config();
    config.wallet = DesktopWalletConfig::KeychainVault;
    let status = status_result(&config);
    assert_eq!(
        status["capabilities"]["sendTransaction"],
        "signer_helper_approval_required"
    );
}

#[test]
fn mock_personal_sign_provider_flow_requires_review_approval() {
    let state = Arc::new(AppState::new());
    let config = fixture_config();
    let account = state.load_and_connect_account(&config).unwrap();
    state
        .grant_account_permission("framkey://local-dapp".to_owned())
        .unwrap();
    let request = ProviderRequest {
        id: "personal-sign-smoke".to_owned(),
        method: "personal_sign".to_owned(),
        params: json!([
            test_siwe_message(&account.address, "local-dapp", "framkey://local-dapp", 1),
            account.address
        ]),
        origin: Some("framkey://local-dapp".to_owned()),
    };

    let worker_state = Arc::clone(&state);
    let worker_config = config.clone();
    let worker =
        thread::spawn(move || handle_provider_request(&worker_state, &worker_config, &request));

    let review = wait_for_pending_review(&state, "personal_sign");
    assert_eq!(review.kind, review::ReviewMethodKind::PersonalSign);
    state
        .decide_review_request(&review.id, &review.decision_token, ReviewDecision::Approve)
        .unwrap();

    let response = worker.join().unwrap().unwrap();
    let ProviderResponse::Result(signature) = response else {
        panic!("expected personal_sign result");
    };
    assert!(signature.as_str().unwrap().starts_with("0x"));

    let signed = state
        .review_queue_snapshot()
        .unwrap()
        .into_iter()
        .find(|item| item.id == review.id)
        .unwrap();
    assert_eq!(signed.status, ReviewStatus::Signed);
    assert!(signed.execution.as_ref().unwrap().address.is_some());
}

#[test]
fn mock_arbitrary_personal_sign_provider_flow_blocks_before_approval() {
    let state = AppState::new();
    let config = fixture_config();
    let account = state.load_and_connect_account(&config).unwrap();
    state
        .grant_account_permission("framkey://local-dapp".to_owned())
        .unwrap();
    let request = ProviderRequest {
        id: "personal-sign-blocked".to_owned(),
        method: "personal_sign".to_owned(),
        params: json!(["0x4652414d4b6579", account.address]),
        origin: Some("framkey://local-dapp".to_owned()),
    };

    let response = handle_provider_request(&state, &config, &request).unwrap();
    let ProviderResponse::Error(error) = response else {
        panic!("expected arbitrary personal_sign to return provider error");
    };
    assert_eq!(error.code, 4200);
    assert!(error.message.contains("blocked before signing"));

    let review = state
        .review_queue_snapshot()
        .unwrap()
        .into_iter()
        .find(|item| item.method == "personal_sign")
        .unwrap();
    assert_eq!(review.status, ReviewStatus::Pending);
    assert_eq!(review.summary["policy"]["canSign"], false);
    assert_eq!(
        review.summary["policy"]["blockers"][0]["code"],
        "unrecognized_personal_sign_message"
    );
}

#[test]
fn mock_typed_data_provider_flow_requires_review_approval() {
    let state = Arc::new(AppState::new());
    let config = fixture_config();
    let account = state.load_and_connect_account(&config).unwrap().address;
    state
        .grant_account_permission("framkey://local-dapp".to_owned())
        .unwrap();
    let request = ProviderRequest {
        id: "typed-data-smoke".to_owned(),
        method: "eth_signTypedData_v4".to_owned(),
        params: json!([
            account,
            permit_typed_data(&state.connected_account_address().unwrap().unwrap())
        ]),
        origin: Some("framkey://local-dapp".to_owned()),
    };

    let worker_state = Arc::clone(&state);
    let worker_config = config.clone();
    let worker =
        thread::spawn(move || handle_provider_request(&worker_state, &worker_config, &request));

    let review = wait_for_pending_review(&state, "eth_signTypedData_v4");
    assert_eq!(review.kind, review::ReviewMethodKind::TypedData);
    assert_eq!(review.summary["typedData"]["intent"], json!("erc20_permit"));
    state
        .decide_review_request(&review.id, &review.decision_token, ReviewDecision::Approve)
        .unwrap();

    let response = worker.join().unwrap().unwrap();
    let ProviderResponse::Result(signature) = response else {
        panic!("expected typed-data signature result");
    };
    assert!(signature.as_str().unwrap().starts_with("0x"));

    let signed = state
        .review_queue_snapshot()
        .unwrap()
        .into_iter()
        .find(|item| item.id == review.id)
        .unwrap();
    assert_eq!(signed.status, ReviewStatus::Signed);
    assert!(signed.execution.as_ref().unwrap().address.is_some());
    assert!(signed.execution.as_ref().unwrap().message_hash.is_some());
}

#[test]
fn mock_send_transaction_provider_flow_uses_ordinary_review_approval() {
    let (rpc_url, request_rx) = spawn_rpc_body_sequence_server(vec![
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "name": "Fixture Token",
                "symbol": "FIX",
                "decimals": 18,
                "logo": null,
            },
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "transactionHash": "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "status": "0x1",
                "blockNumber": "0x1234",
                "transactionIndex": "0x0",
                "gasUsed": "0x5208",
                "effectiveGasPrice": "0x1",
                "logs": [
                    {
                        "data": "0xdeadbeef"
                    }
                ]
            },
        }),
    ]);
    let state = Arc::new(AppState::new());
    let mut config = fixture_config();
    state.load_and_connect_account(&config).unwrap();
    state
        .grant_account_permission("framkey://local-dapp".to_owned())
        .unwrap();
    config.rpc = Some(DesktopRpcConfig {
        endpoint_url: rpc_url,
        network: Some("fixture".to_owned()),
        timeout_ms: 1_000,
        provider: DesktopRpcProvider::Alchemy,
    });
    let request = ProviderRequest {
        id: "send-transaction-smoke".to_owned(),
        method: "eth_sendTransaction".to_owned(),
        params: json!([
            {
                "to": "0x0000000000000000000000000000000000000001",
                "value": "0x0",
                "data": concat!(
                    "0xa9059cbb",
                    "0000000000000000000000000000000000000000000000000000000000000002",
                    "00000000000000000000000000000000000000000000000000000000000f4240"
                ),
                "nonce": "0x0",
                "gas": "0x5208",
                "gasPrice": "0x1"
            }
        ]),
        origin: Some("framkey://local-dapp".to_owned()),
    };

    let worker_state = Arc::clone(&state);
    let worker_config = config.clone();
    let worker =
        thread::spawn(move || handle_provider_request(&worker_state, &worker_config, &request));

    let review = wait_for_pending_review(&state, "eth_sendTransaction");
    assert_eq!(review.kind, review::ReviewMethodKind::Transaction);
    assert_eq!(review.summary["policy"]["decision"], json!("allowed"));
    assert_eq!(review.summary["policy"]["overrideAllowed"], json!(false));
    assert_eq!(review.summary["assetContext"]["status"], json!("ok"));
    assert_eq!(
        review.summary["assetContext"]["tokens"][0]["metadata"]["symbol"],
        json!("FIX")
    );
    state
        .decide_review_request(&review.id, &review.decision_token, ReviewDecision::Approve)
        .unwrap();

    let response = worker.join().unwrap().unwrap();
    let ProviderResponse::Result(tx_hash) = response else {
        panic!("expected eth_sendTransaction result");
    };
    assert_eq!(
        tx_hash,
        json!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    );

    let rpc_request: Value = serde_json::from_str(&request_rx.recv().unwrap()).unwrap();
    assert_eq!(rpc_request["method"], "alchemy_getTokenMetadata");
    let rpc_request: Value = serde_json::from_str(&request_rx.recv().unwrap()).unwrap();
    assert_eq!(rpc_request["method"], "eth_sendRawTransaction");
    assert!(rpc_request["params"][0].as_str().unwrap().starts_with("0x"));

    let signed = state
        .review_queue_snapshot()
        .unwrap()
        .into_iter()
        .find(|item| item.id == review.id)
        .unwrap();
    assert_eq!(signed.status, ReviewStatus::Signed);
    assert_eq!(
        signed.decision.as_ref().map(|record| record.decision),
        Some(ReviewDecision::Approve)
    );

    let activity = transaction_activity_snapshot(&state, &config, false).unwrap();
    assert_eq!(activity["count"], json!(1));
    assert_eq!(activity["items"][0]["status"], json!("broadcast"));
    assert_eq!(
        activity["items"][0]["transactionHash"],
        json!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    );
    assert!(
        activity["items"][0]["localTransactionHash"]
            .as_str()
            .is_some()
    );
    let serialized = serde_json::to_string(&activity).unwrap();
    assert!(!serialized.contains("raw_transaction"));
    assert!(!serialized.contains("rawTransaction"));
    assert!(!serialized.contains("deadbeef"));

    let activity = transaction_activity_snapshot(&state, &config, true).unwrap();
    assert_eq!(activity["receiptRefresh"]["queried"], json!(1));
    assert_eq!(activity["receiptRefresh"]["included"], json!(1));
    assert_eq!(activity["items"][0]["status"], json!("confirmed"));
    assert_eq!(
        activity["items"][0]["receipt"]["blockNumber"],
        json!("0x1234")
    );

    let rpc_request: Value = serde_json::from_str(&request_rx.recv().unwrap()).unwrap();
    assert_eq!(rpc_request["method"], "eth_getTransactionReceipt");
    assert_eq!(
        rpc_request["params"][0],
        json!("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    );
}

#[test]
fn transaction_asset_context_on_hyperevm_skips_alchemy_metadata() {
    let state = AppState::new();
    let mut config = fixture_config();
    config.chain_id = HYPEREVM_CHAIN_ID.to_owned();
    config.rpc = Some(DesktopRpcConfig {
        endpoint_url: HYPEREVM_RPC_URL.to_owned(),
        network: Some(HYPEREVM_NETWORK.to_owned()),
        timeout_ms: 1_000,
        provider: DesktopRpcProvider::Hyperliquid,
    });
    let request = ProviderRequest {
        id: "hyperevm-approve-review".to_owned(),
        method: "eth_sendTransaction".to_owned(),
        params: json!([
            {
                "from": "0x1111111111111111111111111111111111111111",
                "to": "0x2222222222222222222222222222222222222222",
                "value": "0x0",
                "data": concat!(
                    "0x095ea7b3",
                    "0000000000000000000000003333333333333333333333333333333333333333",
                    "0000000000000000000000000000000000000000000000000000000000000001"
                ),
            }
        ]),
        origin: Some("https://hyperevm.example".to_owned()),
    };

    let review = state.capture_review_request(&config, &request).unwrap();

    assert_eq!(
        review.summary["assetContext"]["status"],
        json!("metadata_unsupported")
    );
    assert_eq!(
        review.summary["assetContext"]["provider"],
        json!("unsupported_json_rpc")
    );
    assert_eq!(
        review.summary["assetContext"]["tokens"][0]["metadataError"],
        json!("Token metadata requires an Alchemy-backed RPC")
    );
}

#[test]
fn transaction_activity_persistence_roundtrips_sanitized_entries() {
    let path = unique_test_path("transaction-activity-roundtrip.json");
    let mut log = TransactionActivityLog::new();
    log.items.push_front(fixture_activity_entry("broadcast"));
    log.write_to_path(&path).unwrap();

    let persisted = fs::read_to_string(&path).unwrap();
    assert!(
        persisted.contains("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    );
    assert!(!persisted.contains("raw_transaction"));
    assert!(!persisted.contains("rawTransaction"));
    assert!(!persisted.contains("deadbeef"));

    let restored = TransactionActivityLog::read_from_path(&path).unwrap();
    assert_eq!(restored.len(), 1);
    assert_eq!(
        restored.snapshot()[0].transaction_hash.as_deref(),
        Some("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
    );

    let state = AppState::new_with_transaction_activity_persistence(path.clone());
    let activity = transaction_activity_snapshot(&state, &fixture_config(), false).unwrap();
    assert_eq!(activity["count"], json!(1));
    assert_eq!(activity["processLocal"], json!(false));
    assert_eq!(activity["persistence"]["enabled"], json!(true));
    assert_eq!(activity["persistence"]["restored"], json!(true));
    assert_eq!(activity["persistence"]["itemsRestored"], json!(1));

    let _ = fs::remove_file(path);
}

#[test]
fn transaction_activity_clear_persists_empty_history() {
    let path = unique_test_path("transaction-activity-clear.json");
    let mut log = TransactionActivityLog::new();
    log.items.push_front(fixture_activity_entry("expired"));
    log.write_to_path(&path).unwrap();

    let state = AppState::new_with_transaction_activity_persistence(path.clone());
    let before = transaction_activity_snapshot(&state, &fixture_config(), false).unwrap();
    assert_eq!(before["count"], json!(1));
    assert_eq!(before["persistence"]["itemsRestored"], json!(1));

    assert_eq!(state.clear_transaction_activity().unwrap(), 1);
    let after = transaction_activity_snapshot(&state, &fixture_config(), false).unwrap();
    assert_eq!(after["count"], json!(0));
    assert_eq!(after["items"].as_array().unwrap().len(), 0);
    assert_eq!(after["persistence"]["restored"], json!(false));
    assert_eq!(after["persistence"]["itemsRestored"], json!(0));

    let persisted = TransactionActivityLog::read_from_path(&path).unwrap();
    assert_eq!(persisted.len(), 0);

    let _ = fs::remove_file(path);
}

#[test]
fn transaction_activity_persistence_corrupt_file_falls_back_to_empty_log() {
    let path = unique_test_path("transaction-activity-corrupt.json");
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(&path, b"{not json").unwrap();

    let state = AppState::new_with_transaction_activity_persistence(path.clone());
    let activity = transaction_activity_snapshot(&state, &fixture_config(), false).unwrap();
    assert_eq!(activity["count"], json!(0));
    assert_eq!(activity["persistence"]["enabled"], json!(true));
    assert!(
        activity["persistence"]["warning"]
            .as_str()
            .unwrap()
            .contains("failed to parse")
    );

    let _ = fs::remove_file(path);
}

#[test]
fn transaction_activity_persistence_expires_transient_reviews_on_restore() {
    let path = unique_test_path("transaction-activity-transient.json");
    let mut log = TransactionActivityLog::new();
    log.items
        .push_front(fixture_activity_entry("review_pending"));
    log.write_to_path(&path).unwrap();

    let restored = TransactionActivityLog::read_from_path(&path).unwrap();
    let item = &restored.snapshot()[0];
    assert_eq!(item.status, "expired");
    assert_eq!(
        item.guidance
            .as_ref()
            .and_then(|guidance| guidance.reason_code.as_deref()),
        Some("review_not_restored")
    );
    assert_eq!(
        item.guidance
            .as_ref()
            .map(|guidance| guidance.primary_action.as_str()),
        Some("Retry from dApp")
    );

    let _ = fs::remove_file(path);
}

#[cfg(unix)]
#[test]
fn transaction_activity_persistence_writes_private_file_and_directory_modes() {
    let dir = std::env::temp_dir().join(format!(
        "framkey-desktop-activity-mode-{}-{}",
        std::process::id(),
        random_suffix()
    ));
    let path = dir.join("activity.json");
    let mut log = TransactionActivityLog::new();
    log.items.push_front(fixture_activity_entry("failed"));
    log.write_to_path(&path).unwrap();

    assert_eq!(unix_mode(&dir), PRIVATE_DIR_MODE);
    assert_eq!(unix_mode(&path), PRIVATE_FILE_MODE);

    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn transaction_activity_guidance_reuses_blocked_simulation_next_step() {
    let (rpc_url, request_rx) = spawn_rpc_body_sequence_server(vec![json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": {
            "code": -32000,
            "message": "simulation unavailable"
        }
    })]);
    let state = AppState::new();
    let mut config = fixture_config();
    config.simulation = DesktopSimulationConfig::AlchemyAssetChanges {
        endpoint_url: rpc_url,
        network: Some("fixture".to_owned()),
        timeout_ms: 1_000,
        default_gas: DEFAULT_SIMULATION_DEFAULT_GAS.to_owned(),
    };
    let request = ProviderRequest {
        id: "blocked-simulation".to_owned(),
        method: "eth_sendTransaction".to_owned(),
        params: json!([
            {
                "from": "0x1111111111111111111111111111111111111111",
                "to": "0x2222222222222222222222222222222222222222",
                "value": "0x0",
                "data": "0x"
            }
        ]),
        origin: Some("framkey://local-dapp".to_owned()),
    };

    let review = state.capture_review_request(&config, &request).unwrap();
    assert_eq!(review.summary["guidance"]["status"], json!("blocked"));

    let activity = transaction_activity_snapshot(&state, &config, false).unwrap();
    assert_eq!(activity["count"], json!(1));
    assert_eq!(activity["items"][0]["status"], json!("review_pending"));
    assert_eq!(
        activity["items"][0]["guidance"]["reasonCode"],
        json!("simulation_provider_failed")
    );
    assert_eq!(
        activity["items"][0]["guidance"]["primaryAction"],
        json!("Cannot Sign")
    );
    assert!(
        activity["items"][0]["guidance"]["nextStep"]
            .as_str()
            .unwrap()
            .contains("Check RPC health")
    );

    let rpc_request: Value = serde_json::from_str(&request_rx.recv().unwrap()).unwrap();
    assert_eq!(rpc_request["method"], "alchemy_simulateAssetChanges");
}

#[test]
fn transaction_activity_guidance_explains_insufficient_funds_failure() {
    let (rpc_url, request_rx) = spawn_rpc_body_sequence_server(vec![
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "symbol": "FIX",
                "decimals": 18,
                "name": "Fixture Token"
            }
        }),
        json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32000,
                "message": "insufficient funds for gas * price + value"
            }
        }),
    ]);
    let state = Arc::new(AppState::new());
    let mut config = fixture_config();
    state.load_and_connect_account(&config).unwrap();
    state
        .grant_account_permission("framkey://local-dapp".to_owned())
        .unwrap();
    config.rpc = Some(DesktopRpcConfig {
        endpoint_url: rpc_url,
        network: Some("fixture".to_owned()),
        timeout_ms: 1_000,
        provider: DesktopRpcProvider::Alchemy,
    });
    let request = ProviderRequest {
        id: "insufficient-funds".to_owned(),
        method: "eth_sendTransaction".to_owned(),
        params: json!([
            {
                "to": "0x0000000000000000000000000000000000000001",
                "value": "0x0",
                "data": concat!(
                    "0xa9059cbb",
                    "0000000000000000000000000000000000000000000000000000000000000002",
                    "00000000000000000000000000000000000000000000000000000000000f4240"
                ),
                "nonce": "0x0",
                "gas": "0x5208",
                "gasPrice": "0x1"
            }
        ]),
        origin: Some("framkey://local-dapp".to_owned()),
    };

    let worker_state = Arc::clone(&state);
    let worker_config = config.clone();
    let worker =
        thread::spawn(move || handle_provider_request(&worker_state, &worker_config, &request));

    let review = wait_for_pending_review(&state, "eth_sendTransaction");
    state
        .decide_review_request(&review.id, &review.decision_token, ReviewDecision::Approve)
        .unwrap();

    let response = worker.join().unwrap().unwrap();
    let ProviderResponse::Error(error) = response else {
        panic!("expected eth_sendTransaction broadcast failure");
    };
    assert!(error.message.contains("insufficient funds"));

    let activity = transaction_activity_snapshot(&state, &config, false).unwrap();
    assert_eq!(activity["count"], json!(1));
    assert_eq!(activity["items"][0]["status"], json!("failed"));
    assert_eq!(
        activity["items"][0]["guidance"]["reasonCode"],
        json!("insufficient_funds")
    );
    assert_eq!(
        activity["items"][0]["guidance"]["primaryAction"],
        json!("Fund Account")
    );
    assert!(
        activity["items"][0]["guidance"]["nextStep"]
            .as_str()
            .unwrap()
            .contains("Add native gas funds")
    );
    let serialized = serde_json::to_string(&activity).unwrap();
    assert!(!serialized.contains("raw_transaction"));
    assert!(!serialized.contains("rawTransaction"));

    let rpc_request: Value = serde_json::from_str(&request_rx.recv().unwrap()).unwrap();
    assert_eq!(rpc_request["method"], "alchemy_getTokenMetadata");
    let rpc_request: Value = serde_json::from_str(&request_rx.recv().unwrap()).unwrap();
    assert_eq!(rpc_request["method"], "eth_sendRawTransaction");
}

#[test]
fn switch_session_chain_updates_alchemy_rpc_without_leaking_token() {
    let mut config = fixture_config();
    let token = "switch-token-for-test";
    let chain = supported_chain("0x2105").unwrap();

    config
        .switch_to_supported_chain(chain, Some(token))
        .unwrap();

    assert_eq!(config.chain_id, "0x2105");
    let rpc = config.rpc.as_ref().unwrap();
    assert_eq!(rpc.network.as_deref(), Some("base-mainnet"));
    assert_eq!(
        rpc.endpoint_url,
        "https://base-mainnet.g.alchemy.com/v2/switch-token-for-test"
    );
    let status = status_result(&config);
    assert_eq!(status["network"]["name"], json!("Base"));
    assert_eq!(status["network"]["alchemyNetwork"], json!("base-mainnet"));
    assert_eq!(
        status["supportedChains"].as_array().unwrap().len(),
        SUPPORTED_CHAINS.len()
    );
    let serialized = serde_json::to_string(&status).unwrap();
    assert!(!serialized.contains(token));
    assert!(!serialized.contains("g.alchemy.com/v2"));
}

#[test]
fn wallet_switch_ethereum_chain_requires_trusted_approval() {
    let state = Arc::new(AppState::new());
    let config = fixture_config();
    *state.config.lock().unwrap() = Some(config.clone());
    let request = ProviderRequest {
        id: "switch-1".to_owned(),
        method: "wallet_switchEthereumChain".to_owned(),
        params: json!([{"chainId": "0x2105"}]),
        origin: Some("https://app.uniswap.org".to_owned()),
    };

    let worker = thread::spawn({
        let state = Arc::clone(&state);
        let config = config.clone();
        let request = request.clone();
        move || {
            handle_switch_chain_request_with_probe(
                state.as_ref(),
                &config,
                &request,
                Some("switch-token-for-test".to_owned()),
                ChainSwitchRpcProbe::Skip,
            )
        }
    });

    let review = wait_for_pending_review(&state, "wallet_switchEthereumChain");
    assert_eq!(review.kind, review::ReviewMethodKind::NetworkSwitch);
    assert_eq!(review.summary["requestedChainId"], json!("0x2105"));
    state
        .decide_review_request(&review.id, &review.decision_token, ReviewDecision::Approve)
        .unwrap();

    let response = worker.join().unwrap().unwrap();
    match response {
        ProviderResponse::Result(value) => assert_eq!(value, Value::Null),
        ProviderResponse::Error(error) => {
            panic!("unexpected provider error: {}", error.message)
        }
    }

    let active = state.config_snapshot().unwrap();
    assert_eq!(active.chain_id, "0x2105");
    assert_eq!(
        active.rpc.as_ref().unwrap().network.as_deref(),
        Some("base-mainnet")
    );
    let reviews = state.review_queue_snapshot().unwrap();
    assert_eq!(reviews[0].status, ReviewStatus::Completed);
}

#[test]
fn wallet_switch_ethereum_chain_to_hyperevm_requires_approval_without_token() {
    let state = Arc::new(AppState::new());
    let config = fixture_config();
    *state.config.lock().unwrap() = Some(config.clone());
    let request = ProviderRequest {
        id: "switch-hyperevm".to_owned(),
        method: "wallet_switchEthereumChain".to_owned(),
        params: json!([{"chainId": HYPEREVM_CHAIN_ID}]),
        origin: Some("https://hyperevm.example".to_owned()),
    };

    let worker = thread::spawn({
        let state = Arc::clone(&state);
        let config = config.clone();
        let request = request.clone();
        move || {
            handle_switch_chain_request_with_probe(
                state.as_ref(),
                &config,
                &request,
                None,
                ChainSwitchRpcProbe::Skip,
            )
        }
    });

    let review = wait_for_pending_review(&state, "wallet_switchEthereumChain");
    assert_eq!(review.kind, review::ReviewMethodKind::NetworkSwitch);
    assert_eq!(review.summary["requestedChainId"], json!(HYPEREVM_CHAIN_ID));
    assert_eq!(review.summary["rpcSource"], json!("trusted_chain_session"));
    state
        .decide_review_request(&review.id, &review.decision_token, ReviewDecision::Approve)
        .unwrap();

    let response = worker.join().unwrap().unwrap();
    match response {
        ProviderResponse::Result(value) => assert_eq!(value, Value::Null),
        ProviderResponse::Error(error) => {
            panic!("unexpected provider error: {}", error.message)
        }
    }

    let active = state.config_snapshot().unwrap();
    assert_eq!(active.chain_id, HYPEREVM_CHAIN_ID);
    let rpc = active.rpc.as_ref().unwrap();
    assert_eq!(rpc.endpoint_url, HYPEREVM_RPC_URL);
    assert_eq!(rpc.network.as_deref(), Some(HYPEREVM_NETWORK));
    assert_eq!(rpc.provider(), "hyperliquid");
    let reviews = state.review_queue_snapshot().unwrap();
    assert_eq!(reviews[0].status, ReviewStatus::Completed);
}

#[test]
fn wallet_add_ethereum_chain_requires_trusted_approval_without_switching() {
    let state = Arc::new(AppState::new());
    let config = fixture_config();
    *state.config.lock().unwrap() = Some(config.clone());
    let request = ProviderRequest {
        id: "add-chain-1".to_owned(),
        method: "wallet_addEthereumChain".to_owned(),
        params: json!([
            {
                "chainId": "0x2105",
                "chainName": "Base",
                "nativeCurrency": {"name": "Ether", "symbol": "ETH", "decimals": 18},
                "rpcUrls": ["https://developer-provided-rpc.example/base"],
                "blockExplorerUrls": ["https://basescan.org"]
            }
        ]),
        origin: Some("https://app.uniswap.org".to_owned()),
    };

    let worker = thread::spawn({
        let state = Arc::clone(&state);
        let config = config.clone();
        let request = request.clone();
        move || {
            handle_add_chain_request_with_probe(
                state.as_ref(),
                &config,
                &request,
                Some("switch-token-for-test".to_owned()),
                ChainSwitchRpcProbe::Skip,
            )
        }
    });

    let review = wait_for_pending_review(&state, "wallet_addEthereumChain");
    assert_eq!(review.kind, review::ReviewMethodKind::NetworkSwitch);
    assert_eq!(review.summary["intent"], json!("add_network"));
    assert_eq!(review.summary["requestedChainId"], json!("0x2105"));
    assert_eq!(review.summary["providedRpcUrlCount"], json!(1));
    assert_eq!(review.summary["rpcSource"], json!("trusted_chain_endpoint"));
    state
        .decide_review_request(&review.id, &review.decision_token, ReviewDecision::Approve)
        .unwrap();

    let response = worker.join().unwrap().unwrap();
    match response {
        ProviderResponse::Result(value) => assert_eq!(value, Value::Null),
        ProviderResponse::Error(error) => {
            panic!("unexpected provider error: {}", error.message)
        }
    }

    let active = state.config_snapshot().unwrap();
    assert_eq!(active.chain_id, "0x1");
    assert!(active.rpc.is_none());
    let reviews = state.review_queue_snapshot().unwrap();
    assert_eq!(reviews[0].status, ReviewStatus::Completed);
}

#[test]
fn wallet_add_ethereum_chain_hyperevm_requires_approval_without_token_or_switching() {
    let state = Arc::new(AppState::new());
    let config = fixture_config();
    *state.config.lock().unwrap() = Some(config.clone());
    let request = ProviderRequest {
        id: "add-hyperevm".to_owned(),
        method: "wallet_addEthereumChain".to_owned(),
        params: json!([
            {
                "chainId": HYPEREVM_CHAIN_ID,
                "chainName": "Hyperliquid",
                "nativeCurrency": {"name": "HYPE", "symbol": "HYPE", "decimals": 18},
                "rpcUrls": ["https://developer-provided-rpc.example/hyperevm"],
                "blockExplorerUrls": ["https://hyperevmscan.io"]
            }
        ]),
        origin: Some("https://hyperevm.example".to_owned()),
    };

    let worker = thread::spawn({
        let state = Arc::clone(&state);
        let config = config.clone();
        let request = request.clone();
        move || {
            handle_add_chain_request_with_probe(
                state.as_ref(),
                &config,
                &request,
                None,
                ChainSwitchRpcProbe::Skip,
            )
        }
    });

    let review = wait_for_pending_review(&state, "wallet_addEthereumChain");
    assert_eq!(review.kind, review::ReviewMethodKind::NetworkSwitch);
    assert_eq!(review.summary["intent"], json!("add_network"));
    assert_eq!(review.summary["requestedChainId"], json!(HYPEREVM_CHAIN_ID));
    assert_eq!(review.summary["providedRpcUrlCount"], json!(1));
    assert_eq!(review.summary["rpcSource"], json!("trusted_chain_endpoint"));
    state
        .decide_review_request(&review.id, &review.decision_token, ReviewDecision::Approve)
        .unwrap();

    let response = worker.join().unwrap().unwrap();
    match response {
        ProviderResponse::Result(value) => assert_eq!(value, Value::Null),
        ProviderResponse::Error(error) => {
            panic!("unexpected provider error: {}", error.message)
        }
    }

    let active = state.config_snapshot().unwrap();
    assert_eq!(active.chain_id, "0x1");
    assert!(active.rpc.is_none());
    let reviews = state.review_queue_snapshot().unwrap();
    assert_eq!(reviews[0].status, ReviewStatus::Completed);
}

#[test]
fn trusted_ui_switch_session_chain_returns_updated_status() {
    let state = AppState::new();
    let config = fixture_config();
    *state.config.lock().unwrap() = Some(config.clone());
    let request = SwitchSessionChainRequest {
        chain_id: "0xaa36a7".to_owned(),
    };

    let result = switch_session_chain_from_trusted_ui(
        &state,
        &config,
        request,
        Some("switch-token-for-test".to_owned()),
        ChainSwitchRpcProbe::Skip,
    )
    .unwrap();

    assert_eq!(result["operation"], json!("switch_session_chain"));
    assert_eq!(result["switched"], json!(true));
    assert_eq!(result["chainId"], json!("0xaa36a7"));
    assert_eq!(result["network"]["name"], json!("Sepolia"));
    assert_eq!(result["status"]["chainId"], json!("0xaa36a7"));
    assert_eq!(
        result["status"]["network"]["alchemyNetwork"],
        json!("eth-sepolia")
    );
    let active = state.config_snapshot().unwrap();
    assert_eq!(active.chain_id, "0xaa36a7");
    assert_eq!(
        active.rpc.as_ref().unwrap().network.as_deref(),
        Some("eth-sepolia")
    );
    let serialized = serde_json::to_string(&result).unwrap();
    assert!(!serialized.contains("switch-token-for-test"));
    assert!(!serialized.contains("g.alchemy.com/v2"));
}

#[test]
fn trusted_ui_switch_session_chain_to_hyperevm_needs_no_token() {
    let state = AppState::new();
    let config = fixture_config();
    *state.config.lock().unwrap() = Some(config.clone());
    let request = SwitchSessionChainRequest {
        chain_id: HYPEREVM_CHAIN_ID.to_owned(),
    };

    let result = switch_session_chain_from_trusted_ui(
        &state,
        &config,
        request,
        None,
        ChainSwitchRpcProbe::Skip,
    )
    .unwrap();

    assert_eq!(result["operation"], json!("switch_session_chain"));
    assert_eq!(result["switched"], json!(true));
    assert_eq!(result["chainId"], json!(HYPEREVM_CHAIN_ID));
    assert_eq!(result["network"]["name"], json!("Hyperliquid"));
    assert_eq!(result["network"]["nativeSymbol"], json!("HYPE"));
    assert_eq!(result["status"]["rpc"]["provider"], json!("hyperliquid"));
    assert_eq!(
        result["status"]["rpc"]["capabilities"]["alchemyTokenApi"],
        json!(false)
    );
    let active = state.config_snapshot().unwrap();
    assert_eq!(active.chain_id, HYPEREVM_CHAIN_ID);
    assert_eq!(active.rpc.as_ref().unwrap().endpoint_url, HYPEREVM_RPC_URL);
}

#[test]
fn recovery_smoke_pack_writes_files_and_validates_recommended_set() {
    let dir = std::env::temp_dir().join(format!(
        "framkey-desktop-recovery-smoke-{}-{}",
        std::process::id(),
        now_unix_ms()
    ));
    let config = fixture_config();
    let result = recovery_smoke_pack_with_validator(
        &config,
        RecoverySmokePackRequest {
            out_dir: Some(dir.display().to_string()),
            generation: Some(7),
        },
        |_config, recovery_files| direct_validate_recovery_files(&recovery_files),
    )
    .unwrap();

    assert_eq!(result["operation"], json!("recovery_smoke_pack"));
    assert_eq!(result["developmentOnly"], json!(true));
    assert_eq!(result["recoveryBackups"]["shareFileCount"], json!(4));
    assert_eq!(result["recoveryBackups"]["backupFileCount"], json!(4));
    assert_eq!(result["cloudOnlyDrill"]["canRecover"], json!(false));
    assert_eq!(result["recommendedDrill"]["canRecover"], json!(true));
    assert_eq!(result["configuredVaultDeviceTouched"], json!(false));
    assert_eq!(result["walletSecretTouched"], json!(false));
    assert_eq!(result["recoveryShareBytesPrinted"], json!(false));
    assert!(dir.join("backup-01.dat").exists());
    assert!(dir.join("backup-02.dat").exists());
    assert!(dir.join("backup-03.dat").exists());
    assert!(dir.join("backup-04.dat").exists());
    let serialized = serde_json::to_string(&result).unwrap();
    assert!(!serialized.contains("share_hex"));
    assert!(!serialized.contains("shareHex"));
    assert!(!serialized.contains("recovery_root_key"));
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn chain_switch_rpc_probe_requires_target_chain_id() {
    let chain = supported_chain("0xaa36a7").unwrap();
    let (rpc_url, request_rx) = spawn_rpc_body_sequence_server(vec![
        json!({"jsonrpc": "2.0", "id": 1, "result": "0xaa36a7"}),
    ]);

    verify_supported_chain_endpoint(chain, &rpc_url, 1_000).unwrap();

    let rpc_request: Value = serde_json::from_str(&request_rx.recv().unwrap()).unwrap();
    assert_eq!(rpc_request["method"], "eth_chainId");
    assert_eq!(rpc_request["params"], json!([]));
}

#[test]
fn chain_switch_rpc_probe_rejects_wrong_chain_id() {
    let chain = supported_chain("0xaa36a7").unwrap();
    let (rpc_url, _request_rx) =
        spawn_rpc_body_sequence_server(vec![json!({"jsonrpc": "2.0", "id": 1, "result": "0x1"})]);

    let error = verify_supported_chain_endpoint(chain, &rpc_url, 1_000).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("chain switch RPC probe reported 0x1")
    );
}

#[test]
fn wallet_switch_ethereum_chain_fails_closed_without_token_or_supported_chain() {
    let state = AppState::new();
    let config = fixture_config();
    let base_request = ProviderRequest {
        id: "switch-no-token".to_owned(),
        method: "wallet_switchEthereumChain".to_owned(),
        params: json!([{"chainId": "0x2105"}]),
        origin: Some("https://app.aave.com".to_owned()),
    };
    let response =
        handle_switch_chain_request_with_token(&state, &config, &base_request, None).unwrap();
    match response {
        ProviderResponse::Error(error) => {
            assert_eq!(error.code, 4902);
            assert_eq!(error.data.unwrap()["requestedChainId"], json!("0x2105"));
        }
        ProviderResponse::Result(_) => panic!("expected missing-token switch error"),
    }
    assert!(state.review_queue_snapshot().unwrap().is_empty());

    let unsupported_request = ProviderRequest {
        id: "switch-unsupported".to_owned(),
        method: "wallet_switchEthereumChain".to_owned(),
        params: json!([{"chainId": "0x38"}]),
        origin: Some("https://app.aave.com".to_owned()),
    };
    let response = handle_switch_chain_request_with_token(
        &state,
        &config,
        &unsupported_request,
        Some("switch-token-for-test".to_owned()),
    )
    .unwrap();
    match response {
        ProviderResponse::Error(error) => {
            assert_eq!(error.code, 4902);
            assert_eq!(error.data.unwrap()["requestedChainId"], json!("0x38"));
        }
        ProviderResponse::Result(_) => panic!("expected unsupported-chain switch error"),
    }
    assert!(state.review_queue_snapshot().unwrap().is_empty());
}

#[test]
fn wallet_add_ethereum_chain_fails_closed_without_token_or_supported_chain() {
    let state = AppState::new();
    let config = fixture_config();
    let base_request = ProviderRequest {
        id: "add-no-token".to_owned(),
        method: "wallet_addEthereumChain".to_owned(),
        params: json!([{"chainId": "0x2105"}]),
        origin: Some("https://app.aave.com".to_owned()),
    };
    let response =
        handle_add_chain_request_with_token(&state, &config, &base_request, None).unwrap();
    match response {
        ProviderResponse::Error(error) => {
            assert_eq!(error.code, 4902);
            assert_eq!(error.data.unwrap()["requestedChainId"], json!("0x2105"));
        }
        ProviderResponse::Result(_) => panic!("expected missing-token add-chain error"),
    }
    assert!(state.review_queue_snapshot().unwrap().is_empty());

    let unsupported_request = ProviderRequest {
        id: "add-unsupported".to_owned(),
        method: "wallet_addEthereumChain".to_owned(),
        params: json!([{"chainId": "0x38"}]),
        origin: Some("https://app.aave.com".to_owned()),
    };
    let response = handle_add_chain_request_with_token(
        &state,
        &config,
        &unsupported_request,
        Some("switch-token-for-test".to_owned()),
    )
    .unwrap();
    match response {
        ProviderResponse::Error(error) => {
            assert_eq!(error.code, 4902);
            let data = error.data.unwrap();
            assert_eq!(data["method"], json!("wallet_addEthereumChain"));
            assert_eq!(data["requestedChainId"], json!("0x38"));
        }
        ProviderResponse::Result(_) => panic!("expected unsupported add-chain error"),
    }
    assert!(state.review_queue_snapshot().unwrap().is_empty());
}

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
fn vault_image_size_defaults_file_targets_to_64_kib() {
    let path = std::env::temp_dir().join(format!(
        "framkey-desktop-missing-save-{}-{}.sav",
        std::process::id(),
        random_suffix()
    ));
    let device = DeviceConfig::File { path: path.clone() };

    assert_eq!(
        device.vault_image_size().unwrap(),
        GbaSaveType::SramFram512Kbit.save_size()
    );
    assert!(!path.exists());
}

#[test]
fn vault_image_size_rejects_eeprom_targets() {
    let device = DeviceConfig::GbxCart {
        port: None,
        save_type: GbaSaveType::Eeprom64k,
        expected_save_size: None,
    };

    let error = device.vault_image_size().unwrap_err().to_string();
    assert!(error.contains("too small"));
}

#[test]
fn read_configured_save_image_rejects_invalid_vault_before_helper() {
    let path = unique_test_path("invalid-vault-before-helper.sav");
    fs::write(&path, vec![0_u8; GbaSaveType::SramFram512Kbit.save_size()]).unwrap();
    let mut config = fixture_config();
    config.device = DeviceConfig::File { path: path.clone() };
    config.wallet = DesktopWalletConfig::KeychainVault;

    let error = read_configured_save_image(&config).unwrap_err();
    let error_chain = format!("{error:#}");

    assert!(error_chain.contains("configured save image"));
    assert!(error_chain.contains("no valid FRAMKey Reed-Solomon superblock found"));
    fs::remove_file(path).unwrap();
}

#[test]
fn writes_recovery_backup_pack_without_printing_share_bytes() {
    let dir = std::env::temp_dir().join(format!(
        "framkey-desktop-recovery-pack-{}-{}",
        std::process::id(),
        random_suffix()
    ));
    let recovery_root_key = [4_u8; 32];
    let pack = RecoveryBackupPack::standard(
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

    let encrypted_vault_backup = b"encrypted vault backup fixture";
    let summary = write_recovery_backup_pack(&dir, &pack, Some(encrypted_vault_backup)).unwrap();
    assert_eq!(summary["shareFileCount"], 4);
    assert_eq!(summary["backupFileCount"], 4);
    assert_eq!(summary["embeddedVaultBackupCount"], 4);
    assert_eq!(summary["cloudAloneRecovers"], false);
    assert!(dir.join("backup-01.dat").exists());
    assert!(dir.join("backup-02.dat").exists());
    assert!(dir.join("backup-03.dat").exists());
    assert!(dir.join("backup-04.dat").exists());
    let files = summary["files"].as_array().unwrap();
    assert_eq!(files.len(), 4);
    assert!(files.iter().all(|file| file.get("shareHex").is_none()));
    assert!(
        files
            .iter()
            .filter(|file| file.get("kind").and_then(Value::as_str) == Some("bundle"))
            .all(|file| file["shareBytesPrinted"] == false)
    );
    assert_eq!(
        files
            .iter()
            .filter(|file| file.get("kind").and_then(Value::as_str) == Some("bundle"))
            .count(),
        4
    );
    assert!(
        files
            .iter()
            .filter(|file| file.get("kind").and_then(Value::as_str) == Some("bundle"))
            .all(|file| file["encryptedVaultData"] == "embedded")
    );
    let parsed =
        parse_recovery_backup_bundle(&std::fs::read(dir.join("backup-01.dat")).unwrap()).unwrap();
    assert_eq!(
        parsed.encrypted_vault_backup_bytes().unwrap(),
        encrypted_vault_backup
    );
    assert_eq!(parsed.recovery_file.group_kind.as_str(), "cloud");

    let error = write_recovery_backup_pack(&dir, &pack, Some(encrypted_vault_backup))
        .unwrap_err()
        .to_string();
    assert!(error.contains("failed to create"));
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn recovery_backup_pack_rejects_duplicate_target_names_before_writing() {
    let dir = std::env::temp_dir().join(format!(
        "framkey-desktop-recovery-pack-duplicate-{}-{}",
        std::process::id(),
        random_suffix()
    ));
    let mut pack = fixture_recovery_pack();
    pack.files[3].group_kind = framkey_recovery::RecoveryGroupKind::LocalPhysical;

    let error = write_recovery_backup_pack(&dir, &pack, Some(b"encrypted vault backup fixture"))
        .unwrap_err()
        .to_string();

    assert!(error.contains("maps multiple backup files to backup-03.dat"));
    assert!(!dir.exists());
}

#[cfg(unix)]
#[test]
fn recovery_backup_pack_writes_private_file_and_directory_modes() {
    let dir = std::env::temp_dir().join(format!(
        "framkey-desktop-recovery-pack-mode-{}-{}",
        std::process::id(),
        random_suffix()
    ));
    let pack = fixture_recovery_pack();
    let encrypted_vault_backup = b"encrypted vault backup fixture";

    write_recovery_backup_pack(&dir, &pack, Some(encrypted_vault_backup)).unwrap();

    assert_eq!(unix_mode(&dir), PRIVATE_DIR_MODE);
    for file in &pack.files {
        assert_eq!(
            unix_mode(&dir.join(recovery_backup_file_name(file))),
            PRIVATE_FILE_MODE
        );
    }

    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn recovery_backup_set_out_dir_uses_unique_child_directory() {
    let parent = std::env::temp_dir().join(format!(
        "framkey-desktop-recovery-parent-{}-{}",
        std::process::id(),
        random_suffix()
    ));
    let pack = fixture_recovery_pack();

    let first = recovery_backup_set_out_dir(&parent, &pack).unwrap();
    assert_eq!(first.parent(), Some(parent.as_path()));
    assert!(
        first
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("framkey-backup-g1-")
    );
    assert!(first.exists());
    #[cfg(unix)]
    assert_eq!(unix_mode(&first), PRIVATE_DIR_MODE);

    let second = recovery_backup_set_out_dir(&parent, &pack).unwrap();
    assert_eq!(second.parent(), Some(parent.as_path()));
    assert_ne!(first, second);
    assert!(
        second
            .file_name()
            .unwrap()
            .to_string_lossy()
            .ends_with("-2")
    );
    assert!(second.exists());
    #[cfg(unix)]
    assert_eq!(unix_mode(&second), PRIVATE_DIR_MODE);

    std::fs::remove_dir_all(parent).unwrap();
}

#[test]
fn recovery_ui_state_persists_sanitized_backup_plan_without_secret_material() {
    let dir = std::env::temp_dir().join(format!(
        "framkey-desktop-recovery-state-{}-{}",
        std::process::id(),
        random_suffix()
    ));
    let pack = fixture_recovery_pack();
    let encrypted_vault_backup = b"encrypted vault backup fixture";
    let mut recovery_backups =
        write_recovery_backup_pack(&dir, &pack, Some(encrypted_vault_backup)).unwrap();
    recovery_backups["files"][0]["shareHex"] = json!("secret-share-material");
    recovery_backups["files"][0]["recoveryRootKey"] = json!("secret-rrk");
    let outcome = json!({
        "operation": "recovery_smoke_pack",
        "developmentOnly": true,
        "outDir": dir.display().to_string(),
        "generation": 11,
        "recoveryBackups": recovery_backups,
        "recommendedDrill": {
            "operation": "validate_recovery_set",
            "backupSetId": "backup-state-test",
            "walletId": "wallet-state-test",
            "generation": 11,
            "policyId": "grouped-cloud-plus-physical-v1",
            "recoveryFiles": [dir.join("backup-01.dat").display().to_string()],
            "recoveryShareFileCount": 3,
            "satisfiedGroups": ["cloud", "local_physical"],
            "canRecover": true,
            "walletSecretTouched": false,
            "recoveryRootKeyPrinted": false,
            "recoveryShareBytesPrinted": false,
            "configuredVaultDeviceTouched": false,
            "plaintextSecretProcess": "not_required_for_drill",
            "shareHex": "secret-share-material",
        },
        "walletSecret": "secret-wallet-material",
        "recoveryRootKey": "secret-rrk",
        "recoveryShareBytes": "secret-share-material",
    });
    let mut state = RecoveryUiState::new();

    assert!(state.remember(&outcome));

    let path = dir.join("recovery-state.json");
    state.write_to_path(&path).unwrap();
    let restored = RecoveryUiState::read_from_path(&path).unwrap();
    assert!(restored.backup_outcome.is_some());
    assert!(restored.drill_outcome.is_some());
    let serialized = serde_json::to_string(&restored).unwrap();
    assert!(serialized.contains("backup-01.dat"));
    assert!(serialized.contains("backup-04.dat"));
    assert!(!serialized.contains("secret-wallet-material"));
    assert!(!serialized.contains("secret-rrk"));
    assert!(!serialized.contains("secret-share-material"));
    assert!(!serialized.contains("shareHex"));
    #[cfg(unix)]
    assert_eq!(unix_mode(&path), PRIVATE_FILE_MODE);

    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn recovery_backup_pack_failure_removes_partial_new_files() {
    let dir = std::env::temp_dir().join(format!(
        "framkey-desktop-recovery-pack-partial-{}-{}",
        std::process::id(),
        random_suffix()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let existing_path = dir.join("backup-04.dat");
    std::fs::write(&existing_path, "existing backup").unwrap();

    let recovery_root_key = [4_u8; 32];
    let pack = RecoveryBackupPack::standard(
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

    let encrypted_vault_backup = b"encrypted vault backup fixture";
    let error = write_recovery_backup_pack(&dir, &pack, Some(encrypted_vault_backup))
        .unwrap_err()
        .to_string();
    assert!(error.contains("failed to create"));
    for file in &pack.files[..3] {
        assert!(!dir.join(recovery_backup_file_name(file)).exists());
    }
    assert_eq!(
        std::fs::read_to_string(&existing_path).unwrap(),
        "existing backup"
    );
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn reveal_path_request_rejects_empty_or_control_paths() {
    let empty = RevealPathRequest {
        path: "  ".to_owned(),
    };
    assert!(empty.path().is_err());

    let control = RevealPathRequest {
        path: "valid\ninvalid".to_owned(),
    };
    assert!(control.path().is_err());
}

#[test]
fn recovery_recover_request_requires_local_vault_backup_path() {
    let empty = RecoverKeychainVaultRequest {
        vault_backup_path: "  ".to_owned(),
        recovery_files: vec!["/tmp/backup-03.dat".to_owned()],
        confirm_overwrite: true,
    };
    assert!(empty.validate().is_err());

    let control = RecoverKeychainVaultRequest {
        vault_backup_path: "/tmp/backup\n01.dat".to_owned(),
        recovery_files: vec!["/tmp/backup-03.dat".to_owned()],
        confirm_overwrite: true,
    };
    assert!(control.validate().is_err());
}

#[test]
fn recovery_recover_sanitizer_keeps_vault_backup_metadata_only() {
    let outcome = json!({
        "operation": "recover_keychain_vault",
        "vaultBackupPath": "/Users/example/FRAMKey-Recovery/backup-01.dat",
        "vaultBackupBlake3": "ab".repeat(32),
        "saveSize": 65536,
        "saveImageBlake3": "cd".repeat(32),
        "recoveryFiles": ["/Users/example/FRAMKey-Recovery/backup-03.dat"],
        "recoveryShareFileCount": 3,
        "walletSecretTouched": false,
        "recoveryShareBytesPrinted": false,
        "plaintextSecretProcess": "not_required_for_rewrap",
        "walletSecret": "secret-wallet-material",
        "recoveryRootKey": "secret-rrk",
        "shareHex": "secret-share-material",
    });

    let sanitized = sanitize_recovery_recover_outcome(&outcome).unwrap();
    assert_eq!(
        sanitized["vaultBackupPath"],
        json!("/Users/example/FRAMKey-Recovery/backup-01.dat")
    );
    assert_eq!(sanitized["vaultBackupBlake3"], json!("ab".repeat(32)));
    let serialized = serde_json::to_string(&sanitized).unwrap();
    assert!(!serialized.contains("secret-wallet-material"));
    assert!(!serialized.contains("secret-rrk"));
    assert!(!serialized.contains("secret-share-material"));
    assert!(!serialized.contains("shareHex"));
}

#[test]
fn default_signer_helper_path_can_resolve_bundled_sidecar() {
    let dir = std::env::temp_dir().join(format!(
        "framkey-desktop-helper-bundle-{}-{}",
        std::process::id(),
        random_suffix()
    ));
    let macos_dir = dir.join("FRAMKey.app/Contents/MacOS");
    let resources_dir = dir.join("FRAMKey.app/Contents/Resources");
    std::fs::create_dir_all(&macos_dir).unwrap();
    std::fs::create_dir_all(&resources_dir).unwrap();
    let current_exe = macos_dir.join("framkey-desktop");
    std::fs::write(&current_exe, "desktop").unwrap();
    let sidecar_name = signer_helper_file_names()
        .last()
        .expect("at least one signer helper name")
        .clone();
    let sidecar = resources_dir.join(sidecar_name);
    std::fs::write(&sidecar, "helper").unwrap();

    let resolved = default_signer_helper_path_for_exe(&current_exe).unwrap();

    assert_eq!(resolved, sidecar.canonicalize().unwrap());
    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn signer_helper_status_reports_missing_without_hashing() {
    let path = std::env::temp_dir().join(format!(
        "framkey-missing-signer-helper-{}-{}",
        std::process::id(),
        random_suffix()
    ));
    let helper = SignerHelperConfig {
        path: path.clone(),
        expected_blake3: Some("00".repeat(32)),
        sandbox: SignerHelperSandbox::MacosSandboxExecNoNetwork,
    };

    let status = signer_helper_status_value(&helper);

    assert_eq!(status["exists"], json!(false));
    assert_eq!(status["ready"], json!(false));
    assert_eq!(status["readiness"], json!("missing"));
    assert_eq!(status["hashPinned"], json!(true));
    assert_eq!(status["hashMatches"], json!(false));
    assert!(status.get("blake3").unwrap().is_null());
    assert_eq!(status["path"], json!(path.display().to_string()));
}

#[test]
fn signer_helper_cdhash_parser_accepts_only_codesign_hashes() {
    let output = "\
Executable=/tmp/framkey-signer-helper
Identifier=framkey-signer-helper
CDHash=2316c52c2b96f94fb72411610396b7b6ef715944
Signature=adhoc
";

    assert_eq!(
        parse_codesign_cdhash(output),
        Some("2316c52c2b96f94fb72411610396b7b6ef715944")
    );
    assert_eq!(parse_codesign_cdhash("CDHash=not-a-cdhash"), None);
    assert_eq!(
        parse_codesign_cdhash("CDHash=2316c52c2b96f94fb72411610396b7b6ef71594400"),
        None
    );
}

#[test]
fn provider_event_log_redacts_params_and_caps_entries() {
    let state = AppState::new();
    let request = ProviderRequest {
        id: "request-with-params".to_owned(),
        method: "eth_call".to_owned(),
        params: json!([{ "data": "0xdeadbeef" }, "latest"]),
        origin: Some("https://example.test".to_owned()),
    };
    let envelope = ProviderEnvelope::error(
        "request-with-params",
        ProviderError {
            code: 4200,
            message: "blocked without raw data".to_owned(),
            data: Some(json!({ "raw": "0xdeadbeef" })),
        },
    );
    state
        .record_provider_request_event(&request, &envelope, Duration::from_millis(12))
        .unwrap();
    let events = state.provider_events_snapshot().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, "provider_request");
    assert_eq!(events[0].method.as_deref(), Some("eth_call"));
    assert_eq!(events[0].error_code, Some(4200));

    let serialized = serde_json::to_string(&events).unwrap();
    assert!(!serialized.contains("deadbeef"));
    assert!(!serialized.contains("\"raw\""));

    for index in 0..(PROVIDER_EVENT_LOG_LIMIT + 5) {
        state
            .record_provider_telemetry_event(
                "dapp",
                ProviderTelemetryEvent {
                    event: "provider_injected".to_owned(),
                    origin: Some("https://example.test".to_owned()),
                    url: Some("https://example.test/app?token=should_not_log#frag".to_owned()),
                    detail: json!({ "index": index }),
                },
            )
            .unwrap();
    }
    let events = state.provider_events_snapshot().unwrap();
    assert_eq!(events.len(), PROVIDER_EVENT_LOG_LIMIT);
    assert!(events[0].sequence > 1);
    assert_eq!(
        state.clear_provider_events().unwrap(),
        PROVIDER_EVENT_LOG_LIMIT
    );
    let serialized = serde_json::to_string(&events).unwrap();
    assert!(!serialized.contains("should_not_log"));
    assert!(!serialized.contains("#frag"));
    assert!(state.provider_events_snapshot().unwrap().is_empty());
}

#[test]
fn provider_telemetry_detail_is_schema_whitelisted() {
    let state = AppState::new();
    state
        .record_provider_telemetry_event(
            "dapp",
            ProviderTelemetryEvent {
                event: "provider_smoke_request".to_owned(),
                origin: Some("https://example.test".to_owned()),
                url: Some("https://example.test/app?token=should_not_log#frag".to_owned()),
                detail: json!({
                    "provider": "dev.framkey",
                    "method": "eth_signTypedData_v4",
                    "ok": true,
                    "resultPreview": "signature",
                    "rawParams": ["0xdeadbeef"],
                    "nested": { "signature": "0xaaaaaaaaaaaaaaaa" },
                }),
            },
        )
        .unwrap();

    let events = state.provider_events_snapshot().unwrap();
    assert_eq!(events.len(), 1);
    let detail = events[0].detail.as_ref().unwrap();
    assert_eq!(detail["provider"], json!("dev.framkey"));
    assert_eq!(detail["method"], json!("eth_signTypedData_v4"));
    assert_eq!(detail["ok"], json!(true));
    assert_eq!(detail["resultPreview"], json!("signature"));
    assert_eq!(detail["_omittedKeys"], json!(2));

    let serialized = serde_json::to_string(&events).unwrap();
    assert!(!serialized.contains("deadbeef"));
    assert!(!serialized.contains("aaaaaaaa"));
    assert!(!serialized.contains("should_not_log"));
    assert!(!serialized.contains("#frag"));
}

#[test]
fn mock_gas_fallback_distinguishes_native_and_contract_calls() {
    assert_eq!(
        default_mock_gas_limit("0x"),
        DEFAULT_MOCK_NATIVE_TRANSFER_GAS
    );
    assert_eq!(
        default_mock_gas_limit("0x0"),
        DEFAULT_MOCK_NATIVE_TRANSFER_GAS
    );
    assert_eq!(
        default_mock_gas_limit("0xa9059cbb"),
        DEFAULT_MOCK_CONTRACT_CALL_GAS
    );
}

#[test]
fn desktop_config_rejects_blank_keychain_names() {
    let mut config = fixture_config();
    config.keychain_service = " \t".to_owned();
    assert!(config.validate().is_err());

    let mut config = fixture_config();
    config.keychain_account = "\n".to_owned();
    assert!(config.validate().is_err());

    let mut config = fixture_config();
    config.keychain_account = " default".to_owned();
    assert!(config.validate().is_err());

    let mut config = fixture_config();
    config.device = DeviceConfig::GbxCart {
        port: Some(" \t".to_owned()),
        save_type: GbaSaveType::SramFram512Kbit,
        expected_save_size: None,
    };
    assert!(config.validate().is_err());

    let mut config = fixture_config();
    config.device = DeviceConfig::File {
        path: PathBuf::new(),
    };
    assert!(config.validate().is_err());

    let mut config = fixture_config();
    config.helper.path = PathBuf::new();
    assert!(config.validate().is_err());
}

#[test]
fn desktop_default_gbxcart_port_uses_auto_discovery() {
    let config = DesktopConfig::default_for_repo().unwrap();
    let DeviceConfig::GbxCart {
        port, save_type, ..
    } = config.device
    else {
        panic!("desktop default device should be GBxCart");
    };

    assert_eq!(port, None);
    assert_eq!(save_type, GbaSaveType::SramFram512Kbit);
}

fn fixture_config() -> DesktopConfig {
    DesktopConfig {
        chain_id: "0x1".to_owned(),
        device: DeviceConfig::File {
            path: PathBuf::from("fixture.sav"),
        },
        wallet: DesktopWalletConfig::MockInMemory,
        keychain_service: DEFAULT_KEYCHAIN_SERVICE.to_owned(),
        keychain_account: DEFAULT_KEYCHAIN_ACCOUNT.to_owned(),
        helper: SignerHelperConfig {
            path: PathBuf::from("framkey-signer-helper"),
            expected_blake3: None,
            sandbox: SignerHelperSandbox::DisabledByConfig,
        },
        simulation: DesktopSimulationConfig::LocalDecoderOnly,
        rpc: None,
        btc: DesktopBtcConfig::default(),
    }
}

fn abi_u256_word(value: u128) -> String {
    format!("{value:064x}")
}

fn abi_address_word(value: &str) -> String {
    let address = value.strip_prefix("0x").unwrap_or(value);
    format!("{address:0>64}")
}

fn aave_account_data_result(health_factor: u128) -> String {
    format!(
        "0x{}{}{}{}{}{}",
        abi_u256_word(1_000_000_000),
        abi_u256_word(100_000_000),
        abi_u256_word(500_000_000),
        abi_u256_word(8_000),
        abi_u256_word(7_500),
        abi_u256_word(health_factor),
    )
}

fn policy_blocker<'a>(summary: &'a Value, code: &str) -> Option<&'a Value> {
    summary
        .get("policy")
        .and_then(|policy| policy.get("blockers"))
        .and_then(Value::as_array)?
        .iter()
        .find(|blocker| blocker.get("code").and_then(Value::as_str) == Some(code))
}

fn fixture_connected_account(address: &str) -> DesktopAccount {
    DesktopAccount {
        address: address.to_owned(),
        accounts: json!([]),
        wallet: json!({
            "kind": "test_connected_session",
            "mock": true,
        }),
        metadata: json!({}),
        keychain: None,
        helper_report: None,
    }
}

fn unique_test_path(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "framkey-desktop-test-{}-{}-{name}",
        std::process::id(),
        now_unix_ms()
    ))
}

fn fixture_watched_asset(symbol: &str) -> WatchedAsset {
    WatchedAsset {
        chain_id: "0x1".to_owned(),
        asset_type: "erc20".to_owned(),
        contract_address: "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48".to_owned(),
        symbol: symbol.to_owned(),
        decimals: 6,
        image: None,
        origin: Some("https://app.uniswap.org".to_owned()),
        watched_at_unix_ms: now_unix_ms(),
    }
}

fn fixture_activity_entry(status: &str) -> TransactionActivityEntry {
    TransactionActivityEntry {
        id: "review-persisted".to_owned(),
        review_id: "review-persisted".to_owned(),
        provider_request_id: "provider-persisted".to_owned(),
        method: "eth_sendTransaction".to_owned(),
        origin: Some("https://app.uniswap.org".to_owned()),
        chain_id: Some("0x1".to_owned()),
        from: Some("0x1111111111111111111111111111111111111111".to_owned()),
        to: Some("0x2222222222222222222222222222222222222222".to_owned()),
        value: Some("0x0".to_owned()),
        data_bytes: Some(4),
        call: Some("approve".to_owned()),
        policy_decision: Some("requires_user_override".to_owned()),
        simulation_status: Some("local_decoded".to_owned()),
        guidance: transaction_activity_lifecycle_guidance(status),
        status: status.to_owned(),
        address: Some("0x1111111111111111111111111111111111111111".to_owned()),
        transaction_hash: Some(
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
        ),
        local_transaction_hash: Some(
            "0xbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb".to_owned(),
        ),
        error: None,
        receipt_status: Some("pending".to_owned()),
        receipt: None,
        receipt_checked_at_unix_ms: None,
        created_at_unix_ms: now_unix_ms(),
        updated_at_unix_ms: now_unix_ms(),
    }
}

fn fixture_recovery_pack() -> RecoveryBackupPack {
    let recovery_root_key = [4_u8; 32];
    RecoveryBackupPack::standard(
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
    )
}

#[cfg(unix)]
fn unix_mode(path: &Path) -> u32 {
    use std::os::unix::fs::PermissionsExt;

    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

fn permit_typed_data(owner: &str) -> Value {
    let deadline = current_test_unix_seconds()
        .saturating_add(60 * 60)
        .to_string();
    json!({
        "domain": {
            "name": "USD Coin",
            "version": "2",
            "chainId": 1,
            "verifyingContract": "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"
        },
        "primaryType": "Permit",
        "types": {
            "EIP712Domain": [
                {"name": "name", "type": "string"},
                {"name": "version", "type": "string"},
                {"name": "chainId", "type": "uint256"},
                {"name": "verifyingContract", "type": "address"}
            ],
            "Permit": [
                {"name": "owner", "type": "address"},
                {"name": "spender", "type": "address"},
                {"name": "value", "type": "uint256"},
                {"name": "nonce", "type": "uint256"},
                {"name": "deadline", "type": "uint256"}
            ]
        },
        "message": {
            "owner": owner,
            "spender": "0x000000000022d473030f116ddee9f6b43ac78ba3",
            "value": "1000000",
            "nonce": "0",
            "deadline": deadline
        }
    })
}

fn current_test_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn random_suffix() -> String {
    encode_hex(blake3::hash(&random_array::<16>().unwrap()).as_bytes())
}

fn wait_for_pending_review(state: &AppState, method: &str) -> ReviewRequest {
    for _ in 0..80 {
        let requests = state.review_queue_snapshot().unwrap();
        if let Some(review) = requests
            .into_iter()
            .find(|item| item.method == method && item.status == ReviewStatus::Pending)
        {
            return review;
        }
        thread::sleep(Duration::from_millis(25));
    }
    panic!("timed out waiting for pending {method} review");
}

fn direct_validate_recovery_files(
    files: &[RecoveryBackupFile],
) -> Result<SignerValidateRecoveryFilesResponse> {
    let first = files
        .first()
        .ok_or_else(|| anyhow::anyhow!("at least one recovery backup file is required"))?;
    let mut group_members: BTreeMap<String, BTreeSet<u8>> = BTreeMap::new();
    let mut group_thresholds: BTreeMap<String, u8> = BTreeMap::new();
    for file in files {
        if file.backup_set_id != first.backup_set_id
            || file.wallet_id != first.wallet_id
            || file.generation != first.generation
            || file.policy_id != first.policy_id
        {
            anyhow::bail!("recovery backup files do not belong to the same backup set");
        }
        let group = file.group_kind.as_str().to_owned();
        group_members
            .entry(group.clone())
            .or_default()
            .insert(file.member_index);
        group_thresholds
            .entry(group)
            .or_insert(file.member_threshold);
    }
    let satisfied_groups = group_members
        .iter()
        .filter_map(|(group, members)| {
            let threshold = group_thresholds.get(group).copied().unwrap_or(1);
            (members.len() >= usize::from(threshold)).then(|| group.clone())
        })
        .collect::<Vec<_>>();
    let recovery_result = framkey_recovery::reconstruct_recovery_root_key(files);
    Ok(SignerValidateRecoveryFilesResponse {
        backup_set_id: first.backup_set_id.clone(),
        wallet_id: first.wallet_id.clone(),
        generation: first.generation,
        policy_id: first.policy_id.clone(),
        recovery_share_file_count: files.len(),
        satisfied_groups,
        can_recover: recovery_result.is_ok(),
        failure_reason: recovery_result.err().map(|error| error.to_string()),
    })
}

fn spawn_rpc_body_sequence_server(
    bodies: Vec<Value>,
) -> (String, std::sync::mpsc::Receiver<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (request_tx, request_rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        for body in bodies {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = Vec::new();
            let mut chunk = [0_u8; 4096];
            loop {
                let read = stream.read(&mut chunk).unwrap();
                if read == 0 {
                    break;
                }
                buffer.extend_from_slice(&chunk[..read]);
                if let Some(request_body) = request_body_from_http(&buffer) {
                    request_tx.send(request_body).unwrap();
                    break;
                }
            }

            let body = body.to_string();
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
    });
    (format!("http://{address}"), request_rx)
}

#[derive(Debug, Clone)]
struct EsploraHttpRequest {
    method: String,
    path: String,
    body: String,
}

enum EsploraResponse {
    Json(Value),
    ComputedTxid,
    Error { status: u16, body: String },
}

impl EsploraResponse {
    fn json(value: Value) -> Self {
        Self::Json(value)
    }

    fn computed_txid() -> Self {
        Self::ComputedTxid
    }

    fn error(status: u16, body: &str) -> Self {
        Self::Error {
            status,
            body: body.to_owned(),
        }
    }
}

fn spawn_esplora_sequence_server(
    responses: Vec<EsploraResponse>,
) -> (String, std::sync::mpsc::Receiver<EsploraHttpRequest>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let (request_tx, request_rx) = std::sync::mpsc::channel();
    thread::spawn(move || {
        for response in responses {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buffer = Vec::new();
            let mut chunk = [0_u8; 4096];
            let request = loop {
                let read = stream.read(&mut chunk).unwrap();
                if read == 0 {
                    panic!("connection closed before request was complete");
                }
                buffer.extend_from_slice(&chunk[..read]);
                if let Some(request) = http_request_from_buffer(&buffer) {
                    break request;
                }
            };
            request_tx.send(request.clone()).unwrap();

            let (status, content_type, body) = match response {
                EsploraResponse::Json(value) => (200, "application/json", value.to_string()),
                EsploraResponse::ComputedTxid => (
                    200,
                    "text/plain",
                    framkey_btc::transaction_id_from_raw_hex(&request.body).unwrap(),
                ),
                EsploraResponse::Error { status, body } => (status, "text/plain", body),
            };
            let http_response = format!(
                "HTTP/1.1 {status} OK\r\ncontent-type: {content_type}\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(http_response.as_bytes()).unwrap();
        }
    });
    (format!("http://{address}"), request_rx)
}

fn http_request_from_buffer(buffer: &[u8]) -> Option<EsploraHttpRequest> {
    let marker = b"\r\n\r\n";
    let header_end = buffer
        .windows(marker.len())
        .position(|window| window == marker)?
        + marker.len();
    let headers = String::from_utf8_lossy(&buffer[..header_end]);
    let request_line = headers.lines().next()?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts.next()?.to_owned();
    let path = request_parts.next()?.to_owned();
    let content_length = headers
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse::<usize>().ok())?
        })
        .unwrap_or(0);
    if buffer.len() < header_end + content_length {
        return None;
    }
    let body = String::from_utf8(buffer[header_end..header_end + content_length].to_vec()).ok()?;
    Some(EsploraHttpRequest { method, path, body })
}

fn request_body_from_http(buffer: &[u8]) -> Option<String> {
    let marker = b"\r\n\r\n";
    let header_end = buffer
        .windows(marker.len())
        .position(|window| window == marker)?
        + marker.len();
    let headers = String::from_utf8_lossy(&buffer[..header_end]);
    let content_length = headers.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        name.eq_ignore_ascii_case("content-length")
            .then(|| value.trim().parse::<usize>().ok())?
    })?;
    if buffer.len() < header_end + content_length {
        return None;
    }
    String::from_utf8(buffer[header_end..header_end + content_length].to_vec()).ok()
}
