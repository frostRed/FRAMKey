use std::time::Duration;

use anyhow::{Context, Result};
use serde_json::{Value, json};

use crate::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ChainSwitchRpcProbe {
    VerifyEndpoint,
    #[cfg(test)]
    Skip,
}

pub(crate) fn handle_add_chain_request_with_probe(
    state: &AppState,
    config: &DesktopConfig,
    request: &ProviderRequest,
    alchemy_token: Option<String>,
    rpc_probe: ChainSwitchRpcProbe,
) -> Result<ProviderResponse> {
    let requested = match requested_chain_management_chain_id(&request.params, &request.method) {
        Ok(requested) => requested,
        Err(error) => {
            return Ok(ProviderResponse::Error(ProviderError {
                code: -32602,
                message: error.to_string(),
                data: None,
            }));
        }
    };
    let configured = normalize_chain_id(&config.chain_id)?;

    let Some(chain) = supported_chain(&requested) else {
        return Ok(ProviderResponse::Error(chain_management_provider_error(
            &request.method,
            &configured,
            &requested,
            "FRAMKey can only add known trusted chains",
        )));
    };

    if chain.requires_alchemy_token() && alchemy_token.is_none() {
        return Ok(ProviderResponse::Error(chain_management_provider_error(
            &request.method,
            &configured,
            &requested,
            "FRAMKey needs FRAMKEY_ALCHEMY_TOKEN or ALCHEMY_TOKEN to verify this Alchemy-backed chain",
        )));
    }

    let review = state.capture_review_request(config, request)?;
    let approved = state.wait_for_review_approval(&review.id)?;
    network_switch_authorization(&approved)?;

    if rpc_probe == ChainSwitchRpcProbe::VerifyEndpoint {
        let endpoint_url = trusted_chain_endpoint(chain, alchemy_token.as_deref())?;
        let timeout_ms = config
            .rpc
            .as_ref()
            .map(|rpc| rpc.timeout_ms)
            .unwrap_or(DEFAULT_RPC_TIMEOUT_MS);
        if let Err(error) = verify_supported_chain_endpoint(chain, &endpoint_url, timeout_ms) {
            let message =
                format!("FRAMKey could not verify the requested chain via trusted RPC: {error}");
            let _ = state.mark_review_sign_failed(&review.id, &message);
            return Ok(ProviderResponse::Error(chain_management_provider_error(
                &request.method,
                &configured,
                &requested,
                &message,
            )));
        }
    }

    state.mark_review_completed(&review.id, None)?;
    Ok(ProviderResponse::Result(Value::Null))
}

pub(crate) fn handle_switch_chain_request_with_probe(
    state: &AppState,
    config: &DesktopConfig,
    request: &ProviderRequest,
    alchemy_token: Option<String>,
    rpc_probe: ChainSwitchRpcProbe,
) -> Result<ProviderResponse> {
    let requested = match requested_chain_management_chain_id(&request.params, &request.method) {
        Ok(requested) => requested,
        Err(error) => {
            return Ok(ProviderResponse::Error(ProviderError {
                code: -32602,
                message: error.to_string(),
                data: None,
            }));
        }
    };

    let configured = normalize_chain_id(&config.chain_id)?;
    if requested == configured {
        return Ok(ProviderResponse::Result(Value::Null));
    }

    let Some(chain) = supported_chain(&requested) else {
        return Ok(ProviderResponse::Error(chain_management_provider_error(
            &request.method,
            &configured,
            &requested,
            "FRAMKey does not support session switching to the requested chain",
        )));
    };

    if chain.requires_alchemy_token() && alchemy_token.is_none() {
        return Ok(ProviderResponse::Error(chain_management_provider_error(
            &request.method,
            &configured,
            &requested,
            "FRAMKey needs FRAMKEY_ALCHEMY_TOKEN or ALCHEMY_TOKEN to derive a trusted Alchemy endpoint for this chain",
        )));
    }

    let review = state.capture_review_request(config, request)?;
    let approved = state.wait_for_review_approval(&review.id)?;
    network_switch_authorization(&approved)?;

    if rpc_probe == ChainSwitchRpcProbe::VerifyEndpoint {
        let endpoint_url = trusted_chain_endpoint(chain, alchemy_token.as_deref())?;
        let timeout_ms = config
            .rpc
            .as_ref()
            .map(|rpc| rpc.timeout_ms)
            .unwrap_or(DEFAULT_RPC_TIMEOUT_MS);
        if let Err(error) = verify_supported_chain_endpoint(chain, &endpoint_url, timeout_ms) {
            let message = format!(
                "FRAMKey could not verify a safe RPC endpoint for the requested chain: {error}"
            );
            let _ = state.mark_review_sign_failed(&review.id, &message);
            return Ok(ProviderResponse::Error(chain_management_provider_error(
                &request.method,
                &configured,
                &requested,
                &message,
            )));
        }
    }

    match state.switch_session_chain(chain, alchemy_token.as_deref()) {
        Ok(_) => {
            state.mark_review_completed(&review.id, None)?;
            Ok(ProviderResponse::Result(Value::Null))
        }
        Err(error) => {
            let message = error.to_string();
            let _ = state.mark_review_sign_failed(&review.id, &message);
            Err(error)
        }
    }
}

pub(crate) fn verify_supported_chain_endpoint(
    chain: SupportedChain,
    endpoint_url: &str,
    timeout_ms: u64,
) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(timeout_ms))
        .build()
        .map_err(|_| anyhow::anyhow!("failed to build chain switch RPC probe client"))?;
    let payload = json!({
        "jsonrpc": "2.0",
        "id": "framkey_chain_switch_probe",
        "method": "eth_chainId",
        "params": [],
    });
    let response = client
        .post(endpoint_url)
        .json(&payload)
        .send()
        .map_err(|_| anyhow::anyhow!("chain switch RPC probe request failed"))?;
    let status = response.status();
    let body = response
        .text()
        .map_err(|_| anyhow::anyhow!("failed to read chain switch RPC probe response"))?;
    if !status.is_success() {
        anyhow::bail!("chain switch RPC probe returned HTTP {}", status.as_u16());
    }
    let body: Value = serde_json::from_str(&body)
        .map_err(|_| anyhow::anyhow!("chain switch RPC probe returned non-JSON response"))?;
    if let Some(error) = body.get("error") {
        let code = error
            .get("code")
            .map(Value::to_string)
            .unwrap_or_else(|| "unknown".to_owned());
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .map(|message| message.chars().take(120).collect::<String>())
            .unwrap_or_else(|| "read RPC returned an error".to_owned());
        anyhow::bail!("chain switch RPC probe returned JSON-RPC error {code}: {message}");
    }
    let observed = body
        .get("result")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("chain switch RPC probe did not return a chain id"))?;
    let observed = normalize_chain_id(observed)?;
    if observed != chain.chain_id {
        anyhow::bail!(
            "chain switch RPC probe reported {observed}, expected {}",
            chain.chain_id
        );
    }
    Ok(())
}

pub(crate) fn switch_session_chain_from_trusted_ui(
    state: &AppState,
    config: &DesktopConfig,
    request: SwitchSessionChainRequest,
    alchemy_token: Option<String>,
    rpc_probe: ChainSwitchRpcProbe,
) -> Result<Value> {
    request.validate()?;
    let requested = request.normalized_chain_id()?;
    let configured = normalize_chain_id(&config.chain_id)?;
    if requested == configured {
        return Ok(json!({
            "operation": "switch_session_chain",
            "switched": false,
            "chainId": configured,
            "network": active_chain_value(&configured),
            "status": status_result(config),
        }));
    }

    let chain = supported_chain(&requested).ok_or_else(|| {
        anyhow::anyhow!("FRAMKey does not support session switching to the requested chain")
    })?;
    if chain.requires_alchemy_token() && alchemy_token.is_none() {
        anyhow::bail!(
            "FRAMKey needs FRAMKEY_ALCHEMY_TOKEN or ALCHEMY_TOKEN to derive a trusted Alchemy endpoint for this chain"
        );
    }

    if rpc_probe == ChainSwitchRpcProbe::VerifyEndpoint {
        let endpoint_url = trusted_chain_endpoint(chain, alchemy_token.as_deref())?;
        let timeout_ms = config
            .rpc
            .as_ref()
            .map(|rpc| rpc.timeout_ms)
            .unwrap_or(DEFAULT_RPC_TIMEOUT_MS);
        verify_supported_chain_endpoint(chain, &endpoint_url, timeout_ms)
            .with_context(|| "failed to verify target chain RPC before switching session")?;
    }

    let updated = state.switch_session_chain(chain, alchemy_token.as_deref())?;
    Ok(json!({
        "operation": "switch_session_chain",
        "switched": true,
        "chainId": updated.chain_id,
        "network": active_chain_value(&updated.chain_id),
        "status": status_result(&updated),
    }))
}

pub(crate) fn requested_chain_management_chain_id(params: &Value, method: &str) -> Result<String> {
    let requested = params
        .as_array()
        .and_then(|items| items.first())
        .and_then(|item| item.get("chainId"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("{method} requires params[0].chainId"))?;
    normalize_chain_id(requested)
}

pub(crate) fn chain_management_provider_error(
    method: &str,
    configured_chain_id: &str,
    requested_chain_id: &str,
    message: &str,
) -> ProviderError {
    ProviderError {
        code: 4902,
        message: message.to_owned(),
        data: Some(json!({
            "method": method,
            "configuredChainId": configured_chain_id,
            "requestedChainId": requested_chain_id,
            "supportedChains": supported_chains_value(),
        })),
    }
}

pub(crate) fn is_read_only_rpc_method(method: &str) -> bool {
    matches!(
        method,
        "eth_blockNumber"
            | "eth_call"
            | "eth_estimateGas"
            | "eth_feeHistory"
            | "eth_gasPrice"
            | "eth_getBalance"
            | "eth_getBlockByHash"
            | "eth_getBlockByNumber"
            | "eth_getCode"
            | "eth_getLogs"
            | "eth_getProof"
            | "eth_getStorageAt"
            | "eth_getTransactionByHash"
            | "eth_getTransactionCount"
            | "eth_getTransactionReceipt"
            | "eth_maxPriorityFeePerGas"
            | "eth_syncing"
    )
}

pub(crate) fn proxy_read_rpc(
    config: &DesktopConfig,
    request: &ProviderRequest,
) -> Result<ProviderResponse> {
    rpc_provider_request(config, &request.id, &request.method, request.params.clone())
}

pub(crate) fn rpc_provider_request(
    config: &DesktopConfig,
    id: &str,
    method: &str,
    params: Value,
) -> Result<ProviderResponse> {
    let Some(rpc) = &config.rpc else {
        return Ok(ProviderResponse::Error(ProviderError {
            code: 4900,
            message: "read RPC is not configured; set ALCHEMY_TOKEN or FRAMKEY_RPC_URL".to_owned(),
            data: Some(json!({
                "method": method,
                "provider": "alchemy",
            })),
        }));
    };

    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_millis(rpc.timeout_ms))
        .build()
        .map_err(|_| anyhow::anyhow!("failed to build read RPC client"))?;

    let payload = json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": params,
    });

    let response = client
        .post(&rpc.endpoint_url)
        .json(&payload)
        .send()
        .map_err(|_| anyhow::anyhow!("read RPC request failed"))?;
    let status = response.status();
    let body: Value = response
        .json()
        .map_err(|_| anyhow::anyhow!("failed to decode read RPC response"))?;

    if !status.is_success() {
        return Ok(ProviderResponse::Error(ProviderError {
            code: -32000,
            message: format!("read RPC returned HTTP {}", status.as_u16()),
            data: Some(sanitized_rpc_error_data(body)),
        }));
    }

    if let Some(error) = body.get("error") {
        return Ok(ProviderResponse::Error(provider_error_from_rpc_error(
            error,
        )));
    }

    let result = body
        .get("result")
        .cloned()
        .ok_or_else(|| anyhow::anyhow!("read RPC response did not include result"))?;
    Ok(ProviderResponse::Result(result))
}

pub(crate) fn rpc_result(config: &DesktopConfig, method: &str, params: Value) -> Result<Value> {
    match rpc_provider_request(config, "framkey_internal", method, params)? {
        ProviderResponse::Result(result) => Ok(result),
        ProviderResponse::Error(error) => Err(anyhow::anyhow!("{}: {}", error.code, error.message)),
    }
}

pub(crate) fn provider_error_from_rpc_error(error: &Value) -> ProviderError {
    let code = error.get("code").and_then(Value::as_i64).unwrap_or(-32000);
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .unwrap_or("read RPC returned an error")
        .to_owned();
    let data = error.get("data").cloned();
    ProviderError {
        code,
        message,
        data,
    }
}

pub(crate) fn sanitized_rpc_error_data(body: Value) -> Value {
    match body {
        Value::Object(mut map) => {
            map.remove("id");
            Value::Object(map)
        }
        other => json!({ "body": other }),
    }
}

pub(crate) fn normalize_chain_id(chain_id: &str) -> Result<String> {
    validate_chain_id(chain_id)?;
    Ok(format!("0x{:x}", chain_id_u64(chain_id)?))
}

pub(crate) fn supported_chain(chain_id: &str) -> Option<SupportedChain> {
    let normalized = normalize_chain_id(chain_id).ok()?;
    SUPPORTED_CHAINS
        .iter()
        .copied()
        .find(|chain| chain.chain_id == normalized)
}

pub(crate) fn active_chain_value(chain_id: &str) -> Value {
    if let Some(chain) = supported_chain(chain_id) {
        return supported_chain_value(chain);
    }
    json!({
        "chainId": chain_id,
        "name": "Custom",
        "alchemyNetwork": Value::Null,
        "rpcProvider": Value::Null,
        "rpcNetwork": Value::Null,
        "rpcKind": Value::Null,
        "nativeName": Value::Null,
        "nativeSymbol": Value::Null,
        "blockExplorerUrl": Value::Null,
        "switchable": false,
    })
}

pub(crate) fn supported_chains_value() -> Value {
    Value::Array(
        SUPPORTED_CHAINS
            .iter()
            .copied()
            .map(supported_chain_value)
            .collect(),
    )
}

pub(crate) fn supported_chain_value(chain: SupportedChain) -> Value {
    json!({
        "chainId": chain.chain_id,
        "name": chain.name,
        "alchemyNetwork": chain.alchemy_network(),
        "rpcProvider": chain.rpc_provider(),
        "rpcNetwork": chain.rpc_network(),
        "rpcKind": chain.rpc_kind(),
        "nativeName": chain.native_name,
        "nativeSymbol": chain.native_symbol,
        "blockExplorerUrl": chain.block_explorer_url,
        "switchable": true,
        "capabilities": {
            "trustedRpcEndpoint": true,
            "alchemyTokenApi": chain.supports_alchemy_token_api(),
        },
    })
}

pub(crate) fn trusted_chain_endpoint(
    chain: SupportedChain,
    alchemy_token: Option<&str>,
) -> Result<String> {
    match chain.rpc {
        SupportedChainRpc::Alchemy { network } => {
            let token = alchemy_token.ok_or_else(|| {
                anyhow::anyhow!("trusted Alchemy endpoint requires an Alchemy token")
            })?;
            alchemy_endpoint_from_token(network, token)
        }
        SupportedChainRpc::StaticJsonRpc { endpoint_url, .. } => Ok(endpoint_url.to_owned()),
    }
}

pub(crate) fn chain_id_decimal(chain_id: &str) -> Result<String> {
    let hex = chain_id
        .strip_prefix("0x")
        .ok_or_else(|| anyhow::anyhow!("chain id must be 0x-prefixed hex"))?;
    let value = u64::from_str_radix(hex, 16)
        .with_context(|| format!("failed to parse chain id {chain_id}"))?;
    Ok(value.to_string())
}

pub(crate) fn chain_id_u64(chain_id: &str) -> Result<u64> {
    let hex = chain_id
        .strip_prefix("0x")
        .ok_or_else(|| anyhow::anyhow!("chain id must be 0x-prefixed hex"))?;
    u64::from_str_radix(hex, 16).with_context(|| format!("failed to parse chain id {chain_id}"))
}
