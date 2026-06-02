use std::time::Instant;

use anyhow::Result;
use serde_json::json;
use tauri::{Manager, WebviewWindow};

use crate::*;

#[tauri::command]
pub(crate) fn framkey_status(state: tauri::State<'_, AppState>) -> ProviderEnvelope {
    match state.with_config(|config| Ok(status_result(config))) {
        Ok(result) => ProviderEnvelope::result("status", result),
        Err(error) => ProviderEnvelope::error("status", error_to_provider_error(error)),
    }
}

#[tauri::command]
pub(crate) async fn framkey_rpc_health(
    window: WebviewWindow,
    app: tauri::AppHandle,
) -> Result<ProviderEnvelope, String> {
    let config = match ensure_trusted_window(&window).and_then(|()| {
        let state = app.state::<AppState>();
        state.config_snapshot()
    }) {
        Ok(config) => config,
        Err(error) => {
            return Ok(ProviderEnvelope::error(
                "rpc_health",
                error_to_provider_error(error),
            ));
        }
    };

    tauri::async_runtime::spawn_blocking(move || match rpc_health_snapshot(&config) {
        Ok(result) => ProviderEnvelope::result("rpc_health", result),
        Err(error) => ProviderEnvelope::error("rpc_health", error_to_provider_error(error)),
    })
    .await
    .map_err(|error| format!("rpc health task failed: {error}"))
}

#[tauri::command]
pub(crate) async fn framkey_switch_session_chain(
    window: WebviewWindow,
    app: tauri::AppHandle,
    request: SwitchSessionChainRequest,
) -> Result<ProviderEnvelope, String> {
    let config = match ensure_trusted_window(&window).and_then(|()| {
        let state = app.state::<AppState>();
        state.config_snapshot()
    }) {
        Ok(config) => config,
        Err(error) => {
            return Ok(ProviderEnvelope::error(
                "switch_session_chain",
                error_to_provider_error(error),
            ));
        }
    };

    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        match switch_session_chain_from_trusted_ui(
            &state,
            &config,
            request,
            alchemy_token_from_env(),
            ChainSwitchRpcProbe::VerifyEndpoint,
        ) {
            Ok(result) => ProviderEnvelope::result("switch_session_chain", result),
            Err(error) => {
                ProviderEnvelope::error("switch_session_chain", error_to_provider_error(error))
            }
        }
    })
    .await
    .map_err(|error| format!("switch session chain task failed: {error}"))
}

#[tauri::command]
pub(crate) fn framkey_recovery_state(
    window: WebviewWindow,
    state: tauri::State<'_, AppState>,
) -> ProviderEnvelope {
    if let Err(error) = ensure_trusted_window(&window) {
        return ProviderEnvelope::error("recovery_state", error_to_provider_error(error));
    }
    match state.recovery_ui_state_snapshot() {
        Ok(result) => ProviderEnvelope::result("recovery_state", result),
        Err(error) => ProviderEnvelope::error("recovery_state", error_to_provider_error(error)),
    }
}

#[tauri::command]
pub(crate) fn framkey_clear_recovery_state(
    window: WebviewWindow,
    state: tauri::State<'_, AppState>,
) -> ProviderEnvelope {
    if let Err(error) = ensure_trusted_window(&window) {
        return ProviderEnvelope::error("clear_recovery_state", error_to_provider_error(error));
    }
    match state.clear_recovery_ui_state() {
        Ok(result) => ProviderEnvelope::result("clear_recovery_state", result),
        Err(error) => {
            ProviderEnvelope::error("clear_recovery_state", error_to_provider_error(error))
        }
    }
}

#[tauri::command]
pub(crate) async fn framkey_recovery_smoke_pack(
    window: WebviewWindow,
    app: tauri::AppHandle,
    request: RecoverySmokePackRequest,
) -> Result<ProviderEnvelope, String> {
    let config = match ensure_trusted_window(&window).and_then(|()| {
        let state = app.state::<AppState>();
        state.config_snapshot()
    }) {
        Ok(config) => config,
        Err(error) => {
            return Ok(ProviderEnvelope::error(
                "recovery_smoke_pack",
                error_to_provider_error(error),
            ));
        }
    };

    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        match recovery_smoke_pack_with_validator(&config, request, |config, recovery_files| {
            validate_recovery_files_with_helper(config, recovery_files)
        }) {
            Ok(result) => {
                state.remember_recovery_outcome(&result);
                ProviderEnvelope::result("recovery_smoke_pack", result)
            }
            Err(error) => {
                ProviderEnvelope::error("recovery_smoke_pack", error_to_provider_error(error))
            }
        }
    })
    .await
    .map_err(|error| format!("recovery smoke pack task failed: {error}"))
}

#[tauri::command]
pub(crate) async fn framkey_wallet_assets(
    window: WebviewWindow,
    app: tauri::AppHandle,
) -> Result<ProviderEnvelope, String> {
    let config = match ensure_trusted_window(&window).and_then(|()| {
        let state = app.state::<AppState>();
        state.config_snapshot()
    }) {
        Ok(config) => config,
        Err(error) => {
            return Ok(ProviderEnvelope::error(
                "wallet_assets",
                error_to_provider_error(error),
            ));
        }
    };

    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        match wallet_assets_snapshot(&state, &config) {
            Ok(result) => ProviderEnvelope::result("wallet_assets", result),
            Err(error) => ProviderEnvelope::error("wallet_assets", error_to_provider_error(error)),
        }
    })
    .await
    .map_err(|error| format!("wallet asset snapshot task failed: {error}"))
}

#[tauri::command]
pub(crate) async fn framkey_send_native_transfer(
    window: WebviewWindow,
    app: tauri::AppHandle,
    request: NativeTransferRequest,
) -> Result<ProviderEnvelope, String> {
    let config = match ensure_trusted_window(&window).and_then(|()| {
        let state = app.state::<AppState>();
        state.config_snapshot()
    }) {
        Ok(config) => config,
        Err(error) => {
            return Ok(ProviderEnvelope::error(
                "send_native_transfer",
                error_to_provider_error(error),
            ));
        }
    };

    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        match send_native_transfer_from_trusted_ui(&state, &config, request) {
            Ok(result) => ProviderEnvelope::result("send_native_transfer", result),
            Err(error) => {
                ProviderEnvelope::error("send_native_transfer", error_to_provider_error(error))
            }
        }
    })
    .await
    .map_err(|error| format!("send native transfer task failed: {error}"))
}

#[tauri::command]
pub(crate) async fn framkey_send_token_transfer(
    window: WebviewWindow,
    app: tauri::AppHandle,
    request: TokenTransferRequest,
) -> Result<ProviderEnvelope, String> {
    let config = match ensure_trusted_window(&window).and_then(|()| {
        let state = app.state::<AppState>();
        state.config_snapshot()
    }) {
        Ok(config) => config,
        Err(error) => {
            return Ok(ProviderEnvelope::error(
                "send_token_transfer",
                error_to_provider_error(error),
            ));
        }
    };

    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        match send_token_transfer_from_trusted_ui(&state, &config, request) {
            Ok(result) => ProviderEnvelope::result("send_token_transfer", result),
            Err(error) => {
                ProviderEnvelope::error("send_token_transfer", error_to_provider_error(error))
            }
        }
    })
    .await
    .map_err(|error| format!("send token transfer task failed: {error}"))
}

#[tauri::command]
pub(crate) async fn framkey_create_keychain_vault(
    window: WebviewWindow,
    app: tauri::AppHandle,
    request: CreateKeychainVaultRequest,
) -> Result<ProviderEnvelope, String> {
    let config = match ensure_trusted_window(&window).and_then(|()| {
        let state = app.state::<AppState>();
        state.config_snapshot()
    }) {
        Ok(config) => config,
        Err(error) => {
            return Ok(ProviderEnvelope::error(
                "create_keychain_vault",
                error_to_provider_error(error),
            ));
        }
    };

    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        match create_keychain_vault(&config, request) {
            Ok(result) => {
                state.remember_recovery_outcome(&result);
                ProviderEnvelope::result("create_keychain_vault", result)
            }
            Err(error) => {
                ProviderEnvelope::error("create_keychain_vault", error_to_provider_error(error))
            }
        }
    })
    .await
    .map_err(|error| format!("create keychain vault task failed: {error}"))
}

#[tauri::command]
pub(crate) async fn framkey_recover_keychain_vault(
    window: WebviewWindow,
    app: tauri::AppHandle,
    request: RecoverKeychainVaultRequest,
) -> Result<ProviderEnvelope, String> {
    let config = match ensure_trusted_window(&window).and_then(|()| {
        let state = app.state::<AppState>();
        state.config_snapshot()
    }) {
        Ok(config) => config,
        Err(error) => {
            return Ok(ProviderEnvelope::error(
                "recover_keychain_vault",
                error_to_provider_error(error),
            ));
        }
    };

    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        match recover_keychain_vault(&config, request) {
            Ok(result) => {
                state.remember_recovery_outcome(&result);
                ProviderEnvelope::result("recover_keychain_vault", result)
            }
            Err(error) => {
                ProviderEnvelope::error("recover_keychain_vault", error_to_provider_error(error))
            }
        }
    })
    .await
    .map_err(|error| format!("recover keychain vault task failed: {error}"))
}

#[tauri::command]
pub(crate) async fn framkey_validate_recovery_set(
    window: WebviewWindow,
    app: tauri::AppHandle,
    request: ValidateRecoverySetRequest,
) -> Result<ProviderEnvelope, String> {
    let config = match ensure_trusted_window(&window).and_then(|()| {
        let state = app.state::<AppState>();
        state.config_snapshot()
    }) {
        Ok(config) => config,
        Err(error) => {
            return Ok(ProviderEnvelope::error(
                "validate_recovery_set",
                error_to_provider_error(error),
            ));
        }
    };

    let app_handle = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app_handle.state::<AppState>();
        match validate_recovery_set(&config, request) {
            Ok(result) => {
                state.remember_recovery_outcome(&result);
                ProviderEnvelope::result("validate_recovery_set", result)
            }
            Err(error) => {
                ProviderEnvelope::error("validate_recovery_set", error_to_provider_error(error))
            }
        }
    })
    .await
    .map_err(|error| format!("validate recovery set task failed: {error}"))
}

#[tauri::command]
pub(crate) async fn framkey_reveal_path(
    window: WebviewWindow,
    request: RevealPathRequest,
) -> Result<ProviderEnvelope, String> {
    let path = match ensure_trusted_window(&window).and_then(|()| request.path()) {
        Ok(path) => path,
        Err(error) => {
            return Ok(ProviderEnvelope::error(
                "reveal_path",
                error_to_provider_error(error),
            ));
        }
    };

    tauri::async_runtime::spawn_blocking(move || match reveal_path(&path) {
        Ok(result) => ProviderEnvelope::result("reveal_path", result),
        Err(error) => ProviderEnvelope::error("reveal_path", error_to_provider_error(error)),
    })
    .await
    .map_err(|error| format!("reveal path task failed: {error}"))
}

#[tauri::command]
pub(crate) async fn framkey_pick_vault_backup_file(
    window: WebviewWindow,
) -> Result<ProviderEnvelope, String> {
    if let Err(error) = ensure_trusted_window(&window) {
        return Ok(ProviderEnvelope::error(
            "pick_vault_backup_file",
            error_to_provider_error(error),
        ));
    }

    tauri::async_runtime::spawn_blocking(|| match pick_vault_backup_file() {
        Ok(result) => ProviderEnvelope::result("pick_vault_backup_file", result),
        Err(error) => {
            ProviderEnvelope::error("pick_vault_backup_file", error_to_provider_error(error))
        }
    })
    .await
    .map_err(|error| format!("pick vault backup file task failed: {error}"))
}

#[tauri::command]
pub(crate) async fn framkey_pick_recovery_files(
    window: WebviewWindow,
) -> Result<ProviderEnvelope, String> {
    if let Err(error) = ensure_trusted_window(&window) {
        return Ok(ProviderEnvelope::error(
            "pick_recovery_files",
            error_to_provider_error(error),
        ));
    }

    tauri::async_runtime::spawn_blocking(|| match pick_recovery_files() {
        Ok(result) => ProviderEnvelope::result("pick_recovery_files", result),
        Err(error) => {
            ProviderEnvelope::error("pick_recovery_files", error_to_provider_error(error))
        }
    })
    .await
    .map_err(|error| format!("pick recovery files task failed: {error}"))
}

#[tauri::command]
pub(crate) async fn framkey_pick_recovery_out_dir(
    window: WebviewWindow,
) -> Result<ProviderEnvelope, String> {
    if let Err(error) = ensure_trusted_window(&window) {
        return Ok(ProviderEnvelope::error(
            "pick_recovery_out_dir",
            error_to_provider_error(error),
        ));
    }

    tauri::async_runtime::spawn_blocking(|| match pick_recovery_out_dir() {
        Ok(result) => ProviderEnvelope::result("pick_recovery_out_dir", result),
        Err(error) => {
            ProviderEnvelope::error("pick_recovery_out_dir", error_to_provider_error(error))
        }
    })
    .await
    .map_err(|error| format!("pick recovery output directory task failed: {error}"))
}

#[tauri::command]
pub(crate) async fn framkey_provider_request(
    app: tauri::AppHandle,
    request: ProviderRequest,
) -> Result<ProviderEnvelope, String> {
    let response_id = request.id.clone();
    eprintln!(
        "framkey_provider_request id={} method={}",
        request.id, request.method
    );
    tauri::async_runtime::spawn_blocking(move || {
        let started = Instant::now();
        let state = app.state::<AppState>();
        let envelope = match state
            .config_snapshot()
            .and_then(|config| handle_provider_request(state.inner(), &config, &request))
        {
            Ok(ProviderResponse::Result(result)) => {
                eprintln!("framkey_provider_request id={response_id} completed result");
                ProviderEnvelope::result(response_id, result)
            }
            Ok(ProviderResponse::Error(error)) => {
                eprintln!("framkey_provider_request id={response_id} completed provider_error");
                ProviderEnvelope::error(response_id, error)
            }
            Err(error) => {
                eprintln!(
                    "framkey_provider_request id={response_id} completed error: {}",
                    error
                );
                ProviderEnvelope::error(response_id, error_to_provider_error(error))
            }
        };
        if let Ok(event) =
            state.record_provider_request_event(&request, &envelope, started.elapsed())
        {
            print_provider_event_if_enabled(&event);
        }
        envelope
    })
    .await
    .map_err(|error| format!("provider request task failed: {error}"))
}

#[tauri::command]
pub(crate) fn framkey_review_queue(
    window: WebviewWindow,
    state: tauri::State<'_, AppState>,
) -> ProviderEnvelope {
    match ensure_trusted_window(&window).and_then(|()| state.review_queue_snapshot()) {
        Ok(requests) => ProviderEnvelope::result("review_queue", review_queue_result(requests)),
        Err(error) => ProviderEnvelope::error("review_queue", error_to_provider_error(error)),
    }
}

#[tauri::command]
pub(crate) async fn framkey_transaction_activity(
    window: WebviewWindow,
    app: tauri::AppHandle,
    request: TransactionActivityRequest,
) -> Result<ProviderEnvelope, String> {
    let config = match ensure_trusted_window(&window).and_then(|()| {
        let state = app.state::<AppState>();
        state.config_snapshot()
    }) {
        Ok(config) => config,
        Err(error) => {
            return Ok(ProviderEnvelope::error(
                "transaction_activity",
                error_to_provider_error(error),
            ));
        }
    };

    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        match transaction_activity_snapshot(&state, &config, request.refresh_receipts) {
            Ok(result) => ProviderEnvelope::result("transaction_activity", result),
            Err(error) => {
                ProviderEnvelope::error("transaction_activity", error_to_provider_error(error))
            }
        }
    })
    .await
    .map_err(|error| format!("transaction activity task failed: {error}"))
}

#[tauri::command]
pub(crate) fn framkey_decide_review_request(
    window: WebviewWindow,
    state: tauri::State<'_, AppState>,
    review_id: String,
    decision_token: String,
    decision: ReviewDecision,
) -> ProviderEnvelope {
    eprintln!("framkey_decide_review_request review_id={review_id} decision={decision:?}");
    match ensure_trusted_window(&window)
        .and_then(|()| state.decide_review_request(&review_id, &decision_token, decision))
    {
        Ok(outcome) => ProviderEnvelope::result("decide_review_request", json!(outcome)),
        Err(error) => {
            ProviderEnvelope::error("decide_review_request", error_to_provider_error(error))
        }
    }
}

#[tauri::command]
pub(crate) fn framkey_dismiss_review_request(
    window: WebviewWindow,
    state: tauri::State<'_, AppState>,
    review_id: String,
) -> ProviderEnvelope {
    match ensure_trusted_window(&window).and_then(|()| state.dismiss_review_request(&review_id)) {
        Ok(dismissed) => ProviderEnvelope::result(
            "dismiss_review_request",
            json!({
                "reviewId": review_id,
                "dismissed": dismissed,
            }),
        ),
        Err(error) => {
            ProviderEnvelope::error("dismiss_review_request", error_to_provider_error(error))
        }
    }
}

#[tauri::command]
pub(crate) fn framkey_clear_review_queue(
    window: WebviewWindow,
    state: tauri::State<'_, AppState>,
) -> ProviderEnvelope {
    match ensure_trusted_window(&window).and_then(|()| state.clear_review_queue()) {
        Ok(cleared) => ProviderEnvelope::result("clear_review_queue", json!({"cleared": cleared})),
        Err(error) => ProviderEnvelope::error("clear_review_queue", error_to_provider_error(error)),
    }
}

#[tauri::command]
pub(crate) fn framkey_account_permissions(
    window: WebviewWindow,
    state: tauri::State<'_, AppState>,
) -> ProviderEnvelope {
    match ensure_trusted_window(&window).and_then(|()| state.account_permission_snapshot()) {
        Ok(origins) => {
            ProviderEnvelope::result("account_permissions", account_permissions_result(origins))
        }
        Err(error) => {
            ProviderEnvelope::error("account_permissions", error_to_provider_error(error))
        }
    }
}

#[tauri::command]
pub(crate) fn framkey_revoke_account_permission(
    window: WebviewWindow,
    state: tauri::State<'_, AppState>,
    origin: String,
) -> ProviderEnvelope {
    match ensure_trusted_window(&window)
        .and_then(|()| validate_permission_origin(&origin))
        .and_then(|origin| state.revoke_account_permission(&origin))
    {
        Ok(revoked) => ProviderEnvelope::result(
            "revoke_account_permission",
            json!({
                "origin": origin,
                "permission": "eth_accounts",
                "revoked": revoked,
            }),
        ),
        Err(error) => {
            ProviderEnvelope::error("revoke_account_permission", error_to_provider_error(error))
        }
    }
}

#[tauri::command]
pub(crate) fn framkey_disconnect_account(
    window: WebviewWindow,
    state: tauri::State<'_, AppState>,
) -> ProviderEnvelope {
    match ensure_trusted_window(&window).and_then(|()| state.disconnect_account_session()) {
        Ok(disconnected) => ProviderEnvelope::result("disconnect_account", disconnected),
        Err(error) => ProviderEnvelope::error("disconnect_account", error_to_provider_error(error)),
    }
}

#[tauri::command]
pub(crate) fn framkey_provider_telemetry(
    window: WebviewWindow,
    state: tauri::State<'_, AppState>,
    event: ProviderTelemetryEvent,
) -> ProviderEnvelope {
    match state.record_provider_telemetry_event(window.label(), event) {
        Ok(recorded) => {
            print_provider_event_if_enabled(&recorded);
            ProviderEnvelope::result(
                "provider_telemetry",
                json!({
                    "recorded": true,
                    "window": window.label(),
                }),
            )
        }
        Err(error) => ProviderEnvelope::error("provider_telemetry", error_to_provider_error(error)),
    }
}

#[tauri::command]
pub(crate) fn framkey_provider_events(
    window: WebviewWindow,
    state: tauri::State<'_, AppState>,
) -> ProviderEnvelope {
    match ensure_trusted_window(&window).and_then(|()| state.provider_events_snapshot()) {
        Ok(events) => ProviderEnvelope::result(
            "provider_events",
            json!({
                "limit": PROVIDER_EVENT_LOG_LIMIT,
                "events": events,
            }),
        ),
        Err(error) => ProviderEnvelope::error("provider_events", error_to_provider_error(error)),
    }
}

#[tauri::command]
pub(crate) fn framkey_clear_provider_events(
    window: WebviewWindow,
    state: tauri::State<'_, AppState>,
) -> ProviderEnvelope {
    match ensure_trusted_window(&window).and_then(|()| state.clear_provider_events()) {
        Ok(cleared) => ProviderEnvelope::result(
            "clear_provider_events",
            json!({
                "cleared": cleared,
            }),
        ),
        Err(error) => {
            ProviderEnvelope::error("clear_provider_events", error_to_provider_error(error))
        }
    }
}

#[tauri::command]
pub(crate) fn framkey_run_dapp_compatibility_check(
    window: WebviewWindow,
    app: tauri::AppHandle,
    request: DappCompatibilityCheckRequest,
) -> ProviderEnvelope {
    match ensure_trusted_window(&window)
        .and_then(|()| request.normalized())
        .and_then(|request| run_dapp_compatibility_check(&app, &request))
    {
        Ok(result) => ProviderEnvelope::result("run_dapp_compatibility_check", result),
        Err(error) => ProviderEnvelope::error(
            "run_dapp_compatibility_check",
            error_to_provider_error(error),
        ),
    }
}

#[tauri::command]
pub(crate) fn framkey_dapp_session(
    window: WebviewWindow,
    state: tauri::State<'_, AppState>,
) -> ProviderEnvelope {
    match ensure_trusted_window(&window).and_then(|()| state.dapp_session_snapshot()) {
        Ok(result) => ProviderEnvelope::result("dapp_session", result),
        Err(error) => ProviderEnvelope::error("dapp_session", error_to_provider_error(error)),
    }
}

#[tauri::command]
pub(crate) fn framkey_navigate_dapp(
    window: WebviewWindow,
    app: tauri::AppHandle,
    request: DappNavigationRequest,
) -> ProviderEnvelope {
    match ensure_trusted_window(&window)
        .and_then(|()| request.action())
        .and_then(|action| navigate_dapp_window(&app, action))
    {
        Ok(result) => ProviderEnvelope::result("navigate_dapp", result),
        Err(error) => ProviderEnvelope::error("navigate_dapp", error_to_provider_error(error)),
    }
}

#[tauri::command]
pub(crate) fn framkey_smoke_event(window: WebviewWindow, event: SmokeEvent) -> ProviderEnvelope {
    if !runtime_smoke_enabled()
        && (!env_truthy("FRAMKEY_DESKTOP_TRUSTED_AUTOSMOKE")
            || ensure_trusted_window(&window).is_err())
    {
        return ProviderEnvelope::error(
            "smoke_event",
            ProviderError {
                code: 4200,
                message: "trusted autosmoke is disabled".to_owned(),
                data: None,
            },
        );
    }
    eprintln!(
        "framkey_runtime_smoke window={} stage={} detail={}",
        window.label(),
        event.stage,
        event.detail
    );
    ProviderEnvelope::result(
        "smoke_event",
        json!({
            "recorded": true,
            "window": window.label(),
            "stage": event.stage,
        }),
    )
}
