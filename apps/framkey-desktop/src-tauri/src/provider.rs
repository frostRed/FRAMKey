use anyhow::{Context, Result};
use framkey_evm::EvmAddress;
use framkey_ipc::{
    MAX_SIGNER_HELPER_PERSONAL_SIGN_MESSAGE_BYTES, MAX_SIGNER_HELPER_TYPED_DATA_BYTES,
};
use serde_json::{Map, Value, json};

use crate::review;
use crate::*;

pub(crate) fn handle_provider_request(
    state: &AppState,
    config: &DesktopConfig,
    request: &ProviderRequest,
) -> Result<ProviderResponse> {
    validate_provider_request_context(request)?;

    match request.method.as_str() {
        "eth_chainId" => Ok(ProviderResponse::Result(json!(config.chain_id))),
        "net_version" => Ok(ProviderResponse::Result(json!(chain_id_decimal(
            &config.chain_id
        )?))),
        "framkey_getStatus" | "wallet_getCapabilities" => {
            Ok(ProviderResponse::Result(status_result(config)))
        }
        "eth_accounts" => handle_eth_accounts_request(state, request),
        "eth_requestAccounts" => handle_request_accounts_request(state, config, request),
        "wallet_getPermissions" => handle_get_permissions_request(state, request),
        "wallet_requestPermissions" => handle_wallet_request_permissions(state, config, request),
        "wallet_revokePermissions" => handle_wallet_revoke_permissions(state, request),
        "eth_coinbase" => {
            let accounts = account_addresses_for_request(state, request)?;
            Ok(ProviderResponse::Result(
                accounts
                    .first()
                    .map(|account| json!(account))
                    .unwrap_or(Value::Null),
            ))
        }
        "framkey_getAccount" => {
            let account = state.load_and_connect_account(config)?;
            Ok(ProviderResponse::Result(account_result(config, account)))
        }
        "wallet_addEthereumChain" => handle_add_chain_request(state, config, request),
        "wallet_switchEthereumChain" => handle_switch_chain_request(state, config, request),
        "wallet_watchAsset" => handle_watch_asset_request(state, config, request),
        method if is_read_only_rpc_method(method) => proxy_read_rpc(config, request),
        "personal_sign" => {
            if let Some(response) = signing_permission_error_response(state, request)? {
                Ok(response)
            } else {
                Ok(ProviderResponse::Result(handle_personal_sign_request(
                    state, config, request,
                )?))
            }
        }
        method if is_typed_data_method(method) => {
            if let Some(response) = signing_permission_error_response(state, request)? {
                Ok(response)
            } else {
                handle_typed_data_request(state, config, request)
            }
        }
        "eth_sendTransaction" => {
            if let Some(response) = signing_permission_error_response(state, request)? {
                Ok(response)
            } else {
                handle_send_transaction_request(state, config, request)
            }
        }
        method if dangerous_method_kind(method).is_some() => {
            if let Some(response) = signing_permission_error_response(state, request)? {
                Ok(response)
            } else {
                let review = state.capture_review_request(config, request)?;
                Ok(ProviderResponse::Error(blocked_review_error(review)))
            }
        }
        _ => Err(anyhow::anyhow!(
            "unsupported FRAMKey provider method {}",
            request.method
        )),
    }
}

pub(crate) fn is_typed_data_method(method: &str) -> bool {
    matches!(
        method,
        "eth_signTypedData"
            | "eth_signTypedData_v1"
            | "eth_signTypedData_v3"
            | "eth_signTypedData_v4"
    )
}

pub(crate) fn handle_eth_accounts_request(
    state: &AppState,
    request: &ProviderRequest,
) -> Result<ProviderResponse> {
    let accounts = account_addresses_for_request(state, request)?;
    Ok(ProviderResponse::Result(json!(accounts)))
}

pub(crate) fn handle_request_accounts_request(
    state: &AppState,
    config: &DesktopConfig,
    request: &ProviderRequest,
) -> Result<ProviderResponse> {
    let origin = permission_origin_from_request(request)?;
    if state.account_permission_allowed(&origin)? {
        let account = state.connected_or_load_account(config)?;
        return Ok(ProviderResponse::Result(json!([account.address])));
    }

    let review = state.capture_review_request(config, request)?;
    eprintln!("eth_requestAccounts captured review_id={}", review.id);
    let approved = state.wait_for_review_approval(&review.id)?;
    eprintln!("eth_requestAccounts approved review_id={}", approved.id);
    if approved.kind != review::ReviewMethodKind::AccountConnection {
        anyhow::bail!(
            "approved review request {} is not an account connection",
            approved.id
        );
    }

    let account = state.connected_or_load_account(config)?;
    state.grant_account_permission(origin.clone())?;
    state.mark_review_completed(&review.id, Some(account.address.clone()))?;
    Ok(ProviderResponse::Result(json!([account.address])))
}

pub(crate) fn handle_get_permissions_request(
    state: &AppState,
    request: &ProviderRequest,
) -> Result<ProviderResponse> {
    let origin = permission_origin_from_request(request)?;
    let permissions = if state.account_permission_allowed(&origin)? {
        eth_accounts_permissions_value()
    } else {
        Value::Array(Vec::new())
    };
    Ok(ProviderResponse::Result(permissions))
}

pub(crate) fn handle_wallet_request_permissions(
    state: &AppState,
    config: &DesktopConfig,
    request: &ProviderRequest,
) -> Result<ProviderResponse> {
    if !requests_eth_accounts_permission(&request.params)? {
        return Ok(ProviderResponse::Error(ProviderError {
            code: 4200,
            message: "FRAMKey only supports wallet_requestPermissions for eth_accounts".to_owned(),
            data: Some(json!({
                "supportedPermission": "eth_accounts",
            })),
        }));
    }

    let origin = permission_origin_from_request(request)?;
    if state.account_permission_allowed(&origin)? {
        return Ok(ProviderResponse::Result(eth_accounts_permissions_value()));
    }

    let review = state.capture_review_request(config, request)?;
    eprintln!("wallet_requestPermissions captured review_id={}", review.id);
    let approved = state.wait_for_review_approval(&review.id)?;
    eprintln!(
        "wallet_requestPermissions approved review_id={}",
        approved.id
    );
    if approved.kind != review::ReviewMethodKind::AccountConnection {
        anyhow::bail!(
            "approved review request {} is not an account connection",
            approved.id
        );
    }

    let account = state.connected_or_load_account(config)?;
    state.grant_account_permission(origin)?;
    state.mark_review_completed(&review.id, Some(account.address))?;
    Ok(ProviderResponse::Result(eth_accounts_permissions_value()))
}

pub(crate) fn handle_wallet_revoke_permissions(
    state: &AppState,
    request: &ProviderRequest,
) -> Result<ProviderResponse> {
    if !requests_eth_accounts_permission(&request.params)? {
        return Ok(ProviderResponse::Error(ProviderError {
            code: 4200,
            message: "FRAMKey only supports wallet_revokePermissions for eth_accounts".to_owned(),
            data: Some(json!({
                "supportedPermission": "eth_accounts",
            })),
        }));
    }
    let origin = permission_origin_from_request(request)?;
    if !is_trusted_ui_origin(&origin) {
        state.revoke_account_permission(&origin)?;
    }
    Ok(ProviderResponse::Result(Value::Null))
}

pub(crate) fn handle_watch_asset_request(
    state: &AppState,
    config: &DesktopConfig,
    request: &ProviderRequest,
) -> Result<ProviderResponse> {
    let asset = watched_asset_from_provider_request(config, request)?;
    let review = state.capture_review_request(config, request)?;
    let approved = state.wait_for_review_approval(&review.id)?;
    if approved.kind != review::ReviewMethodKind::WatchAsset {
        anyhow::bail!(
            "approved review request {} is not a watch asset request",
            approved.id
        );
    }
    state.remember_watched_asset(asset)?;
    state.mark_review_completed(&review.id, None)?;
    Ok(ProviderResponse::Result(json!(true)))
}

pub(crate) fn account_addresses_for_request(
    state: &AppState,
    request: &ProviderRequest,
) -> Result<Vec<String>> {
    let Some(origin) = optional_permission_origin(request)? else {
        return Ok(Vec::new());
    };
    if !state.account_permission_allowed(&origin)? {
        return Ok(Vec::new());
    }
    Ok(state.connected_account_address()?.into_iter().collect())
}

pub(crate) fn signing_permission_error_response(
    state: &AppState,
    request: &ProviderRequest,
) -> Result<Option<ProviderResponse>> {
    let Some(origin) = optional_permission_origin(request)? else {
        return Ok(Some(account_access_required_error(None, &request.method)));
    };
    if state.account_permission_allowed(&origin)? && state.connected_account_address()?.is_some() {
        return Ok(None);
    }
    Ok(Some(account_access_required_error(
        Some(&origin),
        &request.method,
    )))
}

pub(crate) fn account_access_required_error(
    origin: Option<&str>,
    method: &str,
) -> ProviderResponse {
    ProviderResponse::Error(ProviderError {
        code: 4100,
        message: "FRAMKey account access is not connected for this origin; call eth_requestAccounts first".to_owned(),
        data: Some(json!({
            "origin": origin,
            "method": method,
            "requiredPermission": "eth_accounts",
        })),
    })
}

pub(crate) fn requests_eth_accounts_permission(params: &Value) -> Result<bool> {
    let items = params
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("wallet permission params must be an array"))?;
    let Some(object) = items.first().and_then(Value::as_object) else {
        anyhow::bail!("wallet permission params must contain one permission object");
    };
    Ok(object.contains_key("eth_accounts"))
}

pub(crate) fn optional_permission_origin(request: &ProviderRequest) -> Result<Option<String>> {
    match request.origin.as_deref() {
        Some(origin) => Ok(Some(validate_permission_origin(origin)?)),
        None => Ok(None),
    }
}

pub(crate) fn permission_origin_from_request(request: &ProviderRequest) -> Result<String> {
    optional_permission_origin(request)?
        .ok_or_else(|| anyhow::anyhow!("account permission request requires an origin"))
}

pub(crate) fn validate_permission_origin(origin: &str) -> Result<String> {
    let origin = origin.trim();
    if origin.is_empty() || origin.len() > 2048 || origin.chars().any(char::is_control) {
        anyhow::bail!("account permission origin is malformed");
    }
    Ok(origin.to_owned())
}

pub(crate) fn watched_asset_from_provider_request(
    config: &DesktopConfig,
    request: &ProviderRequest,
) -> Result<WatchedAsset> {
    let params = watch_asset_params_object(&request.params)?;
    let asset_type = params
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("wallet_watchAsset requires params.type"))?;
    if !asset_type.eq_ignore_ascii_case("ERC20") {
        anyhow::bail!("wallet_watchAsset currently supports ERC-20 assets only");
    }
    let options = params
        .get("options")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow::anyhow!("wallet_watchAsset requires params.options"))?;
    let address = options
        .get("address")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("wallet_watchAsset requires options.address"))?;
    let contract_address = address
        .parse::<EvmAddress>()
        .map_err(|_| {
            anyhow::anyhow!("wallet_watchAsset options.address is not a valid EVM address")
        })?
        .to_string();
    let symbol = options
        .get("symbol")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("wallet_watchAsset requires options.symbol"))
        .and_then(validate_watch_asset_symbol)?;
    let decimals = options
        .get("decimals")
        .ok_or_else(|| anyhow::anyhow!("wallet_watchAsset requires options.decimals"))
        .and_then(validate_watch_asset_decimals)?;
    let image = options
        .get("image")
        .and_then(Value::as_str)
        .map(validate_watch_asset_image)
        .transpose()?;
    let origin = request
        .origin
        .as_deref()
        .map(validate_permission_origin)
        .transpose()?;

    Ok(WatchedAsset {
        chain_id: normalize_chain_id(&config.chain_id)?,
        asset_type: "erc20".to_owned(),
        contract_address,
        symbol,
        decimals,
        image,
        origin,
        watched_at_unix_ms: now_unix_ms(),
    })
}

pub(crate) fn normalize_watched_asset(asset: WatchedAsset) -> Result<WatchedAsset> {
    if !asset.asset_type.eq_ignore_ascii_case("erc20") {
        anyhow::bail!("persisted watched asset type is unsupported");
    }
    let chain_id = normalize_chain_id(&asset.chain_id)?;
    let contract_address = asset
        .contract_address
        .parse::<EvmAddress>()
        .map_err(|_| anyhow::anyhow!("persisted watched asset address is malformed"))?
        .to_string();
    let symbol = validate_watch_asset_symbol(&asset.symbol)?;
    let image = asset
        .image
        .as_deref()
        .map(validate_watch_asset_image)
        .transpose()?;
    let origin = asset
        .origin
        .as_deref()
        .map(validate_permission_origin)
        .transpose()?;

    Ok(WatchedAsset {
        chain_id,
        asset_type: "erc20".to_owned(),
        contract_address,
        symbol,
        decimals: asset.decimals,
        image,
        origin,
        watched_at_unix_ms: asset.watched_at_unix_ms,
    })
}

pub(crate) fn watch_asset_params_object(params: &Value) -> Result<&Map<String, Value>> {
    params
        .as_object()
        .or_else(|| {
            params
                .as_array()
                .and_then(|items| items.first())
                .and_then(Value::as_object)
        })
        .ok_or_else(|| anyhow::anyhow!("wallet_watchAsset params must be an object"))
}

pub(crate) fn validate_watch_asset_symbol(symbol: &str) -> Result<String> {
    let symbol = symbol.trim();
    if symbol.is_empty() {
        anyhow::bail!("wallet_watchAsset options.symbol is empty");
    }
    if symbol.chars().any(char::is_control) {
        anyhow::bail!("wallet_watchAsset options.symbol is malformed");
    }
    if symbol.chars().count() > 24 {
        anyhow::bail!("wallet_watchAsset options.symbol is too long");
    }
    Ok(symbol.to_owned())
}

pub(crate) fn validate_watch_asset_decimals(value: &Value) -> Result<u8> {
    let decimals = match value {
        Value::Number(number) => number.as_u64().ok_or_else(|| {
            anyhow::anyhow!("wallet_watchAsset options.decimals must be an integer")
        })?,
        Value::String(text) => text.parse::<u64>().map_err(|_| {
            anyhow::anyhow!("wallet_watchAsset options.decimals must be an integer")
        })?,
        _ => anyhow::bail!("wallet_watchAsset options.decimals must be an integer"),
    };
    if decimals > 255 {
        anyhow::bail!("wallet_watchAsset options.decimals is too large");
    }
    Ok(decimals as u8)
}

pub(crate) fn validate_watch_asset_image(image: &str) -> Result<String> {
    let image = image.trim();
    if image.is_empty() || image.len() > 2048 || image.chars().any(char::is_control) {
        anyhow::bail!("wallet_watchAsset options.image is malformed");
    }
    Ok(image.to_owned())
}

pub(crate) fn is_trusted_ui_origin(origin: &str) -> bool {
    origin == TRUSTED_UI_ORIGIN
}

pub(crate) fn eth_accounts_permissions_value() -> Value {
    json!([
        {
            "parentCapability": "eth_accounts",
            "caveats": [],
        }
    ])
}

pub(crate) fn account_permissions_result(origins: Vec<String>) -> Value {
    json!({
        "scope": "session",
        "permission": "eth_accounts",
        "origins": origins,
    })
}

pub(crate) fn review_queue_result(requests: Vec<ReviewRequest>) -> Value {
    json!({
        "count": requests.len(),
        "maxItems": review::MAX_REVIEW_QUEUE_ITEMS,
        "requests": requests,
    })
}

pub(crate) fn transaction_activity_snapshot(
    state: &AppState,
    config: &DesktopConfig,
    refresh_receipts: bool,
) -> Result<Value> {
    let receipt_refresh = if refresh_receipts {
        if config.rpc.is_some() {
            state.refresh_transaction_receipts(config)?
        } else {
            json!({
                "queried": 0,
                "included": 0,
                "pending": 0,
                "errors": [{
                    "scope": "rpc",
                    "message": "Alchemy RPC is not configured",
                }],
                "limit": TRANSACTION_RECEIPT_REFRESH_LIMIT,
            })
        }
    } else {
        Value::Null
    };
    let items = state.transaction_activity_snapshot()?;
    let persistence = state.transaction_activity_persistence_snapshot()?;
    Ok(json!({
        "count": items.len(),
        "maxItems": TRANSACTION_ACTIVITY_LIMIT,
        "processLocal": !persistence.enabled,
        "persistence": persistence,
        "receiptRefresh": receipt_refresh,
        "items": items,
    }))
}

pub(crate) fn handle_personal_sign_request(
    state: &AppState,
    config: &DesktopConfig,
    request: &ProviderRequest,
) -> Result<Value> {
    let payload = review::personal_sign_payload(&request.params)?;
    if payload.message.len() > MAX_SIGNER_HELPER_PERSONAL_SIGN_MESSAGE_BYTES {
        anyhow::bail!(
            "personal_sign message exceeds {} bytes",
            MAX_SIGNER_HELPER_PERSONAL_SIGN_MESSAGE_BYTES
        );
    }

    let review = state.capture_review_request(config, request)?;
    eprintln!("personal_sign captured review_id={}", review.id);
    let approved = state.wait_for_review_approval(&review.id)?;
    eprintln!("personal_sign approved review_id={}", approved.id);
    if approved.kind != review::ReviewMethodKind::PersonalSign {
        anyhow::bail!(
            "approved review request {} is not personal_sign",
            approved.id
        );
    }

    let signed = (|| match config.wallet {
        DesktopWalletConfig::KeychainVault => {
            let save_image = read_configured_save_image(config)?;
            personal_sign_with_helper(
                config,
                save_image,
                payload.message,
                payload.expected_address,
            )
        }
        DesktopWalletConfig::MockInMemory => {
            state.personal_sign_with_mock_wallet(payload.message, payload.expected_address)
        }
    })();

    match signed {
        Ok(signed) => {
            state.mark_review_signed(&review.id, &signed)?;
            eprintln!("personal_sign signed review_id={}", review.id);
            Ok(json!(signed.signature))
        }
        Err(error) => {
            let message = error.to_string();
            let _ = state.mark_review_sign_failed(&review.id, &message);
            eprintln!("personal_sign failed review_id={}: {}", review.id, message);
            Err(error)
        }
    }
}

pub(crate) fn handle_typed_data_request(
    state: &AppState,
    config: &DesktopConfig,
    request: &ProviderRequest,
) -> Result<ProviderResponse> {
    let payload = review::typed_data_payload(&request.params)?;
    let typed_data_bytes = serde_json::to_vec(&payload.typed_data)
        .context("failed to encode typed-data payload for size check")?;
    if typed_data_bytes.len() > MAX_SIGNER_HELPER_TYPED_DATA_BYTES {
        anyhow::bail!(
            "typed-data payload exceeds {} bytes",
            MAX_SIGNER_HELPER_TYPED_DATA_BYTES
        );
    }

    let review = state.capture_review_request(config, request)?;
    if request.method != "eth_signTypedData_v4"
        || review::signable_typed_data_intent(&review).is_none()
    {
        return Ok(ProviderResponse::Error(blocked_review_error(review)));
    }

    eprintln!("eth_signTypedData_v4 captured review_id={}", review.id);
    let approved = state.wait_for_review_approval(&review.id)?;
    eprintln!("eth_signTypedData_v4 approved review_id={}", approved.id);
    if approved.kind != review::ReviewMethodKind::TypedData {
        anyhow::bail!("approved review request {} is not typed data", approved.id);
    }
    let typed_data_broker_mode = match review::typed_data_signing_authorization(&approved) {
        Ok(mode) => mode,
        Err(error) => {
            let message = error.to_string();
            let _ = state.mark_review_sign_failed(&review.id, &message);
            return Err(error);
        }
    };

    let signed = (|| match config.wallet {
        DesktopWalletConfig::KeychainVault => {
            let save_image = read_configured_save_image(config)?;
            sign_typed_data_with_helper(
                config,
                save_image,
                payload.typed_data,
                payload.expected_address,
            )
        }
        DesktopWalletConfig::MockInMemory => {
            state.sign_typed_data_with_mock_wallet(payload.typed_data, payload.expected_address)
        }
    })();

    match signed {
        Ok(signed) => {
            state.mark_review_signature(&review.id, &signed.address, &signed.typed_data_hash)?;
            eprintln!(
                "eth_signTypedData_v4 signed review_id={} broker_mode={}",
                review.id, typed_data_broker_mode
            );
            Ok(ProviderResponse::Result(json!(signed.signature)))
        }
        Err(error) => {
            let message = error.to_string();
            let _ = state.mark_review_sign_failed(&review.id, &message);
            eprintln!(
                "eth_signTypedData_v4 signing failed review_id={}: {}",
                review.id, message
            );
            Err(error)
        }
    }
}

pub(crate) fn handle_send_transaction_request(
    state: &AppState,
    config: &DesktopConfig,
    request: &ProviderRequest,
) -> Result<ProviderResponse> {
    let wallet_address = state.transaction_wallet_address(config, request)?;
    let prepared = prepare_transaction(
        config,
        request,
        &wallet_address,
        config.wallet == DesktopWalletConfig::MockInMemory,
    )?;
    let review = state.capture_review_request(config, &prepared.review_request)?;
    eprintln!("eth_sendTransaction captured review_id={}", review.id);
    let approved = state.wait_for_review_approval(&review.id)?;
    eprintln!("eth_sendTransaction approved review_id={}", approved.id);
    if approved.kind != review::ReviewMethodKind::Transaction {
        anyhow::bail!(
            "approved review request {} is not eth_sendTransaction",
            approved.id
        );
    }
    let transaction_broker_mode = match review::transaction_signing_authorization(&approved) {
        Ok(mode) => mode,
        Err(error) => {
            let message = error.to_string();
            let _ = state.mark_review_sign_failed(&review.id, &message);
            return Err(error);
        }
    };

    let signed = match config.wallet {
        DesktopWalletConfig::MockInMemory => {
            state.sign_transaction_with_mock_wallet(&prepared.transaction, &wallet_address)
        }
        DesktopWalletConfig::KeychainVault => {
            let save_image = read_configured_save_image(config)?;
            sign_transaction_with_helper(
                config,
                save_image,
                prepared.transaction.clone(),
                Some(wallet_address.clone()),
            )
            .map(DesktopSignedTransaction::from)
        }
    };
    let signed = match signed {
        Ok(signed) => signed,
        Err(error) => {
            let message = error.to_string();
            let _ = state.mark_review_sign_failed(&review.id, &message);
            eprintln!(
                "eth_sendTransaction signing failed review_id={}: {}",
                review.id, message
            );
            return Err(error);
        }
    };

    let broadcast = rpc_provider_request(
        config,
        &request.id,
        "eth_sendRawTransaction",
        json!([signed.raw_transaction]),
    )?;
    match broadcast {
        ProviderResponse::Result(tx_hash) => {
            let tx_hash = tx_hash
                .as_str()
                .ok_or_else(|| anyhow::anyhow!("eth_sendRawTransaction returned non-string hash"))?
                .to_owned();
            state.mark_review_transaction_broadcast(
                &review.id,
                &signed.address,
                &tx_hash,
                &signed.transaction_hash,
            )?;
            eprintln!(
                "eth_sendTransaction broadcast review_id={} broker_mode={} kind={} local_hash={}",
                review.id,
                transaction_broker_mode,
                signed.transaction_kind,
                signed.transaction_hash
            );
            Ok(ProviderResponse::Result(json!(tx_hash)))
        }
        ProviderResponse::Error(error) => {
            let message = error.message.clone();
            let _ = state.mark_review_sign_failed(&review.id, &message);
            eprintln!(
                "eth_sendTransaction broadcast failed review_id={}: {}",
                review.id, message
            );
            Ok(ProviderResponse::Error(error))
        }
    }
}

pub(crate) fn blocked_review_error(review: ReviewRequest) -> ProviderError {
    ProviderError {
        code: 4200,
        message: format!(
            "{} was captured for trusted review and blocked before signing",
            review.method
        ),
        data: Some(json!({
            "reviewRequest": review.provider_view(),
            "signingEnabled": false,
            "approvalBroker": "dry_run",
        })),
    }
}

pub(crate) fn validate_provider_request_context(request: &ProviderRequest) -> Result<()> {
    if let Some(origin) = &request.origin {
        if origin.len() > 2048 || origin.chars().any(char::is_control) {
            anyhow::bail!("invalid provider request origin");
        }
    }

    let params_len = serde_json::to_vec(&request.params)
        .context("failed to encode provider request params for validation")?
        .len();
    if params_len > 64 * 1024 {
        anyhow::bail!("provider request params exceed 64 KiB");
    }

    Ok(())
}

pub(crate) fn handle_switch_chain_request(
    state: &AppState,
    config: &DesktopConfig,
    request: &ProviderRequest,
) -> Result<ProviderResponse> {
    handle_switch_chain_request_with_token(state, config, request, alchemy_token_from_env())
}

pub(crate) fn handle_switch_chain_request_with_token(
    state: &AppState,
    config: &DesktopConfig,
    request: &ProviderRequest,
    alchemy_token: Option<String>,
) -> Result<ProviderResponse> {
    handle_switch_chain_request_with_probe(
        state,
        config,
        request,
        alchemy_token,
        ChainSwitchRpcProbe::VerifyEndpoint,
    )
}

pub(crate) fn handle_add_chain_request(
    state: &AppState,
    config: &DesktopConfig,
    request: &ProviderRequest,
) -> Result<ProviderResponse> {
    handle_add_chain_request_with_token(state, config, request, alchemy_token_from_env())
}

pub(crate) fn handle_add_chain_request_with_token(
    state: &AppState,
    config: &DesktopConfig,
    request: &ProviderRequest,
    alchemy_token: Option<String>,
) -> Result<ProviderResponse> {
    handle_add_chain_request_with_probe(
        state,
        config,
        request,
        alchemy_token,
        ChainSwitchRpcProbe::VerifyEndpoint,
    )
}
