use std::{
    collections::{BTreeMap, BTreeSet},
    time::Instant,
};

use anyhow::Result;
use framkey_evm::EvmAddress;
use framkey_simulation::TransactionSimulationReport;
use serde_json::{Value, json};

use crate::*;

pub(crate) fn status_result(config: &DesktopConfig) -> Value {
    json!({
        "app": "framkey-desktop",
        "version": env!("CARGO_PKG_VERSION"),
        "chainId": config.chain_id,
        "network": active_chain_value(&config.chain_id),
        "supportedChains": supported_alchemy_chains_value(),
        "wallet": config.wallet.describe(),
        "device": config.device.describe(),
        "keychain": {
            "service": config.keychain_service,
            "account": config.keychain_account,
        },
        "capabilities": {
            "readOnlyAccounts": true,
            "accountPermissions": "session_origin_approval",
            "runtimeSmoke": runtime_smoke_enabled(),
            "trustedAutosmoke": trusted_autosmoke_enabled(),
            "recoveryAutosmoke": recovery_autosmoke_enabled(),
            "walletSendAutosmoke": wallet_send_autosmoke_enabled(),
            "trustedAutosmokeDurationMs": trusted_autosmoke_duration_ms(),
            "requestReview": true,
            "approvalBroker": "controlled_personal_sign",
            "typedDataApprovalBroker": "controlled_typed_data_signing",
            "personalSign": "approval_required",
            "sendTransaction": config.wallet.send_transaction_capability(),
            "nativeSend": config.wallet.send_transaction_capability(),
            "tokenSend": config.wallet.send_transaction_capability(),
            "signTypedData": "permit_approval_required",
            "networkAdd": "trusted_approval_known_alchemy_chains",
            "networkSwitch": "trusted_approval_known_alchemy_chains",
            "watchAsset": "trusted_approval_erc20_persistent_local",
            "simulation": config.simulation.capability_value(),
            "rpcProxy": config.rpc.is_some(),
        },
        "simulation": config.simulation.describe(),
        "rpc": config.rpc.as_ref().map(DesktopRpcConfig::describe),
        "signerHelper": signer_helper_status_value(&config.helper),
        "trustModel": {
            "trustedWalletUi": true,
            "untrustedDappWebView": true,
            "signingEnabled": "personal_sign_and_permit_typed_data_after_approval",
        }
    })
}

pub(crate) fn rpc_health_snapshot(config: &DesktopConfig) -> Result<Value> {
    let expected_chain_id = normalize_chain_id(&config.chain_id)?;
    let checked_at = now_unix_ms();
    let Some(_rpc) = &config.rpc else {
        return Ok(json!({
            "operation": "rpc_health",
            "provider": "alchemy",
            "configured": false,
            "healthy": false,
            "status": "missing",
            "expectedChainId": expected_chain_id,
            "observedChainId": Value::Null,
            "chainMatches": false,
            "latestBlock": Value::Null,
            "latencyMs": Value::Null,
            "checkedAtUnixMs": checked_at,
            "rpc": Value::Null,
            "tokenExposed": false,
            "rpcUrlExposed": false,
            "error": {
                "scope": "rpc",
                "message": "Alchemy RPC is not configured",
            },
        }));
    };

    let started = Instant::now();
    let observed_chain_id = match rpc_string_result(config, "eth_chainId", json!([])) {
        Ok(chain_id) => match normalize_chain_id(&chain_id) {
            Ok(chain_id) => chain_id,
            Err(error) => {
                return Ok(rpc_health_result(
                    config,
                    "invalid_chain",
                    false,
                    None,
                    false,
                    None,
                    duration_ms(started.elapsed()),
                    Some(rpc_health_error("chainId", &error)),
                ));
            }
        },
        Err(error) => {
            return Ok(rpc_health_result(
                config,
                "rpc_error",
                false,
                None,
                false,
                None,
                duration_ms(started.elapsed()),
                Some(rpc_health_error("chainId", &error)),
            ));
        }
    };

    let chain_matches = observed_chain_id == expected_chain_id;
    let latest_block = match rpc_string_result(config, "eth_blockNumber", json!([])) {
        Ok(block_number) => Some(block_number),
        Err(error) => {
            return Ok(rpc_health_result(
                config,
                "rpc_error",
                false,
                Some(observed_chain_id),
                chain_matches,
                None,
                duration_ms(started.elapsed()),
                Some(rpc_health_error("blockNumber", &error)),
            ));
        }
    };

    let status = if chain_matches { "ok" } else { "wrong_chain" };
    let error = (!chain_matches).then(|| {
        json!({
            "scope": "chainId",
            "message": format!("read RPC returned {observed_chain_id}, expected {expected_chain_id}"),
        })
    });
    Ok(rpc_health_result(
        config,
        status,
        chain_matches,
        Some(observed_chain_id),
        chain_matches,
        latest_block,
        duration_ms(started.elapsed()),
        error,
    ))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn rpc_health_result(
    config: &DesktopConfig,
    status: &str,
    healthy: bool,
    observed_chain_id: Option<String>,
    chain_matches: bool,
    latest_block: Option<String>,
    latency_ms: u64,
    error: Option<Value>,
) -> Value {
    json!({
        "operation": "rpc_health",
        "provider": "alchemy",
        "configured": config.rpc.is_some(),
        "healthy": healthy,
        "status": status,
        "expectedChainId": normalize_chain_id(&config.chain_id).ok(),
        "observedChainId": observed_chain_id,
        "chainMatches": chain_matches,
        "latestBlock": latest_block,
        "latencyMs": latency_ms,
        "checkedAtUnixMs": now_unix_ms(),
        "rpc": config.rpc.as_ref().map(DesktopRpcConfig::describe),
        "tokenExposed": false,
        "rpcUrlExposed": false,
        "error": error,
    })
}

pub(crate) fn rpc_health_error(scope: &str, error: &anyhow::Error) -> Value {
    let message = error
        .to_string()
        .chars()
        .filter(|ch| !ch.is_control())
        .collect::<String>();
    json!({
        "scope": scope,
        "message": truncate_for_event(&message, 160),
    })
}

pub(crate) fn wallet_assets_snapshot(state: &AppState, config: &DesktopConfig) -> Result<Value> {
    let address = state.require_connected_account_address()?;
    let mut errors = Vec::new();
    let watched_assets = state.watched_assets_for_chain(&config.chain_id)?;
    let wallet_ui_persistence = state.wallet_ui_state_persistence_snapshot()?;

    if config.rpc.is_none() {
        errors.push(json!({
            "scope": "rpc",
            "message": "Alchemy RPC is not configured",
        }));
        return Ok(wallet_assets_result(
            config,
            &address,
            None,
            None,
            PortfolioTokenDiscovery::empty("rpc_missing"),
            watched_assets,
            wallet_ui_persistence,
            errors,
        ));
    }

    let native_balance =
        match rpc_string_result(config, "eth_getBalance", json!([address, "latest"])) {
            Ok(balance) => Some(balance),
            Err(error) => {
                errors.push(json!({
                    "scope": "native",
                    "message": error.to_string(),
                }));
                None
            }
        };

    let block_number = match rpc_string_result(config, "eth_blockNumber", json!([])) {
        Ok(block) => Some(block),
        Err(error) => {
            errors.push(json!({
                "scope": "block",
                "message": error.to_string(),
            }));
            None
        }
    };

    let token_discovery = match alchemy_token_discovery(config, &address) {
        Ok(discovery) => discovery,
        Err(error) => {
            errors.push(json!({
                "scope": "tokens",
                "message": error.to_string(),
            }));
            PortfolioTokenDiscovery::empty("token_query_failed")
        }
    };

    Ok(wallet_assets_result(
        config,
        &address,
        native_balance,
        block_number,
        token_discovery,
        watched_assets,
        wallet_ui_persistence,
        errors,
    ))
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn wallet_assets_result(
    config: &DesktopConfig,
    address: &str,
    native_balance: Option<String>,
    block_number: Option<String>,
    token_discovery: PortfolioTokenDiscovery,
    watched_assets: Vec<WatchedAsset>,
    wallet_ui_persistence: WalletUiStatePersistenceStatus,
    errors: Vec<Value>,
) -> Value {
    let watched_count = watched_assets.len();
    let tokens = merge_watched_portfolio_tokens(token_discovery.tokens, watched_assets);
    json!({
        "address": address,
        "chainId": config.chain_id,
        "rpc": config.rpc.as_ref().map(DesktopRpcConfig::describe),
        "blockNumber": block_number,
        "native": {
            "assetKind": "native",
            "name": "Ether",
            "symbol": "ETH",
            "decimals": 18,
            "balance": native_balance,
        },
        "tokens": tokens,
        "tokenScan": {
            "provider": "alchemy_getTokenBalances",
            "status": token_discovery.status,
            "tokenSpec": "erc20",
            "maxCount": PORTFOLIO_TOKEN_BALANCE_MAX_COUNT,
            "returned": token_discovery.returned,
            "nonzero": token_discovery.nonzero,
            "shown": token_discovery.shown,
            "balanceErrors": token_discovery.balance_errors,
            "metadataQueried": token_discovery.metadata_queried,
            "metadataLimit": token_discovery.metadata_limit,
            "truncated": token_discovery.truncated,
            "nextPageKeyPresent": token_discovery.next_page_key_present,
            "watched": watched_count,
        },
        "walletState": {
            "persistence": wallet_ui_persistence,
            "watchedAssetLimit": WALLET_WATCHED_ASSET_LIMIT,
        },
        "errors": errors,
    })
}

pub(crate) fn merge_watched_portfolio_tokens(
    mut tokens: Vec<Value>,
    watched_assets: Vec<WatchedAsset>,
) -> Vec<Value> {
    let mut seen = BTreeMap::new();
    for (index, token) in tokens.iter().enumerate() {
        if let Some(contract) = token.get("contractAddress").and_then(Value::as_str) {
            seen.insert(contract.to_ascii_lowercase(), index);
        }
    }

    for asset in watched_assets {
        let key = asset.contract_address.to_ascii_lowercase();
        if let Some(index) = seen.get(&key).copied() {
            if let Some(object) = tokens[index].as_object_mut() {
                object.insert("watched".to_owned(), json!(true));
                object.insert("watchOrigin".to_owned(), json!(asset.origin.clone()));
                object.insert(
                    "watchedAtUnixMs".to_owned(),
                    json!(asset.watched_at_unix_ms),
                );
            }
            continue;
        }
        seen.insert(key, tokens.len());
        tokens.push(json!({
            "assetKind": "erc20",
            "contractAddress": asset.contract_address,
            "balance": "0x0",
            "metadata": {
                "name": Value::Null,
                "symbol": asset.symbol,
                "decimals": asset.decimals,
                "logoAvailable": false,
            },
            "metadataError": Value::Null,
            "watched": true,
            "watchOrigin": asset.origin,
            "watchedAtUnixMs": asset.watched_at_unix_ms,
            "source": "wallet_watchAsset",
        }));
    }

    tokens
}

pub(crate) fn send_native_transfer_from_trusted_ui(
    state: &AppState,
    config: &DesktopConfig,
    request: NativeTransferRequest,
) -> Result<Value> {
    let normalized = request.normalized(config)?;
    let provider_request = ProviderRequest {
        id: format!(
            "trusted-native-send-{}-{}",
            std::process::id(),
            now_unix_ms()
        ),
        method: "eth_sendTransaction".to_owned(),
        params: json!([
            {
                "to": normalized.to.clone(),
                "value": normalized.value.clone(),
                "data": "0x",
                "chainId": normalized.chain_id.clone(),
            }
        ]),
        origin: Some(TRUSTED_UI_ORIGIN.to_owned()),
    };
    let response = handle_send_transaction_request(state, config, &provider_request)?;
    let base = json!({
        "operation": "send_native_transfer",
        "chainId": normalized.chain_id,
        "to": normalized.to,
        "amount": normalized.amount,
        "value": normalized.value,
        "nativeSymbol": native_symbol_for_chain(&normalized.chain_id),
        "reviewOrigin": TRUSTED_UI_ORIGIN,
    });

    match response {
        ProviderResponse::Result(transaction_hash) => {
            let mut result = base;
            if let Value::Object(ref mut object) = result {
                object.insert("status".to_owned(), json!("broadcast"));
                object.insert("transactionHash".to_owned(), transaction_hash);
            }
            Ok(result)
        }
        ProviderResponse::Error(error) => {
            let mut result = base;
            if let Value::Object(ref mut object) = result {
                object.insert("status".to_owned(), json!("failed"));
                object.insert("providerError".to_owned(), json!(error));
            }
            Ok(result)
        }
    }
}

pub(crate) fn send_token_transfer_from_trusted_ui(
    state: &AppState,
    config: &DesktopConfig,
    request: TokenTransferRequest,
) -> Result<Value> {
    let normalized = request.normalized(config)?;
    let provider_request = ProviderRequest {
        id: format!(
            "trusted-token-send-{}-{}",
            std::process::id(),
            now_unix_ms()
        ),
        method: "eth_sendTransaction".to_owned(),
        params: json!([
            {
                "to": normalized.token_contract.clone(),
                "value": "0x0",
                "data": normalized.data.clone(),
                "chainId": normalized.chain_id.clone(),
            }
        ]),
        origin: Some(TRUSTED_UI_ORIGIN.to_owned()),
    };
    let response = handle_send_transaction_request(state, config, &provider_request)?;
    let base = json!({
        "operation": "send_token_transfer",
        "chainId": normalized.chain_id,
        "tokenContract": normalized.token_contract,
        "to": normalized.to,
        "amount": normalized.amount,
        "rawAmount": normalized.raw_amount,
        "decimals": normalized.decimals,
        "symbol": normalized.symbol,
        "reviewOrigin": TRUSTED_UI_ORIGIN,
    });

    match response {
        ProviderResponse::Result(transaction_hash) => {
            let mut result = base;
            if let Value::Object(ref mut object) = result {
                object.insert("status".to_owned(), json!("broadcast"));
                object.insert("transactionHash".to_owned(), transaction_hash);
            }
            Ok(result)
        }
        ProviderResponse::Error(error) => {
            let mut result = base;
            if let Value::Object(ref mut object) = result {
                object.insert("status".to_owned(), json!("failed"));
                object.insert("providerError".to_owned(), json!(error));
            }
            Ok(result)
        }
    }
}

pub(crate) fn native_symbol_for_chain(chain_id: &str) -> String {
    supported_alchemy_chain(chain_id)
        .map(|chain| chain.native_symbol.to_owned())
        .unwrap_or_else(|| "ETH".to_owned())
}

#[derive(Debug, Clone)]
pub(crate) struct PortfolioTokenDiscovery {
    pub(crate) status: &'static str,
    pub(crate) tokens: Vec<Value>,
    pub(crate) returned: usize,
    pub(crate) nonzero: usize,
    pub(crate) shown: usize,
    pub(crate) balance_errors: usize,
    pub(crate) metadata_queried: usize,
    pub(crate) metadata_limit: usize,
    pub(crate) truncated: bool,
    pub(crate) next_page_key_present: bool,
}

impl PortfolioTokenDiscovery {
    pub(crate) fn empty(status: &'static str) -> Self {
        Self {
            status,
            tokens: Vec::new(),
            returned: 0,
            nonzero: 0,
            shown: 0,
            balance_errors: 0,
            metadata_queried: 0,
            metadata_limit: PORTFOLIO_TOKEN_METADATA_LIMIT,
            truncated: false,
            next_page_key_present: false,
        }
    }
}

pub(crate) fn alchemy_token_discovery(
    config: &DesktopConfig,
    address: &str,
) -> Result<PortfolioTokenDiscovery> {
    let result = rpc_result(
        config,
        "alchemy_getTokenBalances",
        json!([
            address,
            "erc20",
            {
                "maxCount": PORTFOLIO_TOKEN_BALANCE_MAX_COUNT,
            }
        ]),
    )?;
    let balances = result
        .get("tokenBalances")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("alchemy_getTokenBalances result missing tokenBalances"))?;
    let returned = balances.len();
    let next_page_key_present = result.get("pageKey").and_then(Value::as_str).is_some();
    let balance_errors = balances
        .iter()
        .filter(|balance| balance.get("error").is_some())
        .count();

    let mut seen_contracts = BTreeSet::new();
    let mut nonzero_balances = Vec::new();
    for balance in balances {
        let Some(contract) = balance.get("contractAddress").and_then(Value::as_str) else {
            continue;
        };
        if contract.parse::<EvmAddress>().is_err() {
            continue;
        }
        if !seen_contracts.insert(contract.to_ascii_lowercase()) {
            continue;
        }
        let Some(token_balance) = balance.get("tokenBalance").and_then(Value::as_str) else {
            continue;
        };
        if !is_nonzero_hex_quantity(token_balance) {
            continue;
        }
        nonzero_balances.push((contract.to_owned(), token_balance.to_owned()));
    }

    let nonzero = nonzero_balances.len();
    let mut tokens = Vec::new();
    let mut metadata_queried = 0;
    for (index, (contract, balance)) in nonzero_balances.into_iter().enumerate() {
        let (metadata, metadata_error) = if index < PORTFOLIO_TOKEN_METADATA_LIMIT {
            metadata_queried += 1;
            match alchemy_token_metadata(config, &contract) {
                Ok(metadata) => (metadata, Value::Null),
                Err(error) => (
                    json!({
                        "name": Value::Null,
                        "symbol": Value::Null,
                        "decimals": Value::Null,
                        "logoAvailable": false,
                    }),
                    json!(truncate_for_event(&error.to_string(), 160)),
                ),
            }
        } else {
            (
                json!({
                    "name": Value::Null,
                    "symbol": Value::Null,
                    "decimals": Value::Null,
                    "logoAvailable": false,
                }),
                Value::Null,
            )
        };
        tokens.push(json!({
            "assetKind": "erc20",
            "contractAddress": contract,
            "balance": balance,
            "metadata": metadata,
            "metadataError": metadata_error,
        }));
    }

    Ok(PortfolioTokenDiscovery {
        status: "ok",
        shown: tokens.len(),
        tokens,
        returned,
        nonzero,
        balance_errors,
        metadata_queried,
        metadata_limit: PORTFOLIO_TOKEN_METADATA_LIMIT,
        truncated: nonzero > PORTFOLIO_TOKEN_METADATA_LIMIT || next_page_key_present,
        next_page_key_present,
    })
}

pub(crate) fn alchemy_token_metadata(config: &DesktopConfig, contract: &str) -> Result<Value> {
    let result = rpc_result(config, "alchemy_getTokenMetadata", json!([contract]))?;
    Ok(json!({
        "name": token_metadata_text(result.get("name"), 80),
        "symbol": token_metadata_text(result.get("symbol"), 24),
        "decimals": token_metadata_decimals(result.get("decimals")),
        "logoAvailable": result.get("logo").and_then(Value::as_str).is_some(),
    }))
}

pub(crate) fn token_metadata_text(value: Option<&Value>, max_chars: usize) -> Value {
    value
        .and_then(Value::as_str)
        .map(|text| json!(truncate_for_event(text, max_chars)))
        .unwrap_or(Value::Null)
}

pub(crate) fn token_metadata_decimals(value: Option<&Value>) -> Value {
    let Some(value) = value else {
        return Value::Null;
    };
    if let Some(decimals) = value.as_u64() {
        return json!(decimals.min(255));
    }
    if let Some(decimals) = value.as_f64()
        && decimals.is_finite()
        && decimals.fract() == 0.0
        && decimals >= 0.0
    {
        return json!((decimals as u64).min(255));
    }
    Value::Null
}

pub(crate) fn is_nonzero_hex_quantity(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("0x") else {
        return false;
    };
    !hex.is_empty()
        && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
        && hex.bytes().any(|byte| byte != b'0')
}

pub(crate) fn transaction_asset_context(
    config: &DesktopConfig,
    simulation: &TransactionSimulationReport,
) -> Option<Value> {
    let contracts = transaction_token_contracts(simulation);
    if contracts.is_empty() {
        return None;
    }

    if config.rpc.is_none() {
        return Some(json!({
            "status": "rpc_missing",
            "provider": "alchemy_getTokenMetadata",
            "metadataLimit": TRANSACTION_TOKEN_METADATA_LIMIT,
            "tokens": contracts
                .into_iter()
                .map(|(asset_kind, contract)| transaction_token_context_value(
                    asset_kind,
                    contract,
                    Value::Null,
                    Some("Alchemy RPC is not configured".to_owned()),
                ))
                .collect::<Vec<_>>(),
            "errors": [{
                "scope": "rpc",
                "message": "Alchemy RPC is not configured",
            }],
        }));
    }

    let mut tokens = Vec::new();
    let mut errors = Vec::new();
    for (index, (asset_kind, contract)) in contracts.into_iter().enumerate() {
        if index >= TRANSACTION_TOKEN_METADATA_LIMIT {
            errors.push(json!({
                "scope": "metadata",
                "contractAddress": contract,
                "message": "transaction token metadata limit reached",
            }));
            continue;
        }
        match alchemy_token_metadata(config, &contract) {
            Ok(metadata) => {
                tokens.push(transaction_token_context_value(
                    asset_kind, contract, metadata, None,
                ));
            }
            Err(error) => {
                let message = truncate_for_event(&error.to_string(), 160);
                errors.push(json!({
                    "scope": "metadata",
                    "contractAddress": contract,
                    "message": message,
                }));
                tokens.push(transaction_token_context_value(
                    asset_kind,
                    contract,
                    Value::Null,
                    Some(message),
                ));
            }
        }
    }

    let status = if errors.is_empty() {
        "ok"
    } else if tokens.is_empty() {
        "metadata_failed"
    } else {
        "metadata_partial"
    };
    Some(json!({
        "status": status,
        "provider": "alchemy_getTokenMetadata",
        "metadataLimit": TRANSACTION_TOKEN_METADATA_LIMIT,
        "tokens": tokens,
        "errors": errors,
    }))
}

pub(crate) fn transaction_token_contracts(
    simulation: &TransactionSimulationReport,
) -> Vec<(String, String)> {
    let mut seen = BTreeSet::new();
    let mut contracts = Vec::new();

    for approval in &simulation.approvals {
        if let Some(contract) = normalized_contract_address(approval.contract.as_deref()) {
            let key = contract.to_ascii_lowercase();
            if seen.insert(key) {
                contracts.push((approval.asset_kind.clone(), contract));
            }
        }
    }
    for transfer in &simulation.asset_transfers {
        if let Some(contract) = normalized_contract_address(transfer.contract.as_deref()) {
            let key = contract.to_ascii_lowercase();
            if seen.insert(key) {
                contracts.push((transfer.asset_kind.clone(), contract));
            }
        }
    }

    contracts
}

pub(crate) fn normalized_contract_address(value: Option<&str>) -> Option<String> {
    let address = value?.parse::<EvmAddress>().ok()?;
    Some(address.to_string())
}

pub(crate) fn transaction_token_context_value(
    asset_kind: String,
    contract_address: String,
    metadata: Value,
    metadata_error: Option<String>,
) -> Value {
    json!({
        "assetKind": asset_kind,
        "contractAddress": contract_address,
        "metadata": metadata,
        "metadataError": metadata_error,
    })
}

pub(crate) fn account_result(config: &DesktopConfig, account: DesktopAccount) -> Value {
    json!({
        "address": account.address,
        "chainId": config.chain_id,
        "wallet": account.wallet,
        "metadata": account.metadata,
        "keychain": account.keychain,
        "signerHelper": account.helper_report,
    })
}
