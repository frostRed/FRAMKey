use anyhow::{Result, bail};
use framkey_simulation::{
    PolicyBlocker, TransactionPolicyEvaluation, TransactionReviewReport, TransactionRiskSummary,
    local_transaction_review,
};
use serde_json::{Map, Value, json};

use super::*;

pub(crate) fn summarize_review_request(
    kind: ReviewMethodKind,
    method: &str,
    params: &Value,
    chain_id: &str,
    transaction_review: Option<TransactionReviewReport>,
    transaction_asset_context: Option<Value>,
) -> Value {
    match kind {
        ReviewMethodKind::PersonalSign => summarize_personal_sign(params),
        ReviewMethodKind::AccountConnection => summarize_account_connection(method, params),
        ReviewMethodKind::NetworkSwitch => summarize_network_management(method, params, chain_id),
        ReviewMethodKind::WatchAsset => summarize_watch_asset(params, chain_id),
        ReviewMethodKind::EthSign => summarize_eth_sign(method, params),
        ReviewMethodKind::TypedData => summarize_typed_data(method, params),
        ReviewMethodKind::Transaction => summarize_transaction(
            method,
            params,
            chain_id,
            transaction_review,
            transaction_asset_context,
        ),
    }
}

pub(crate) fn summarize_network_management(
    method: &str,
    params: &Value,
    current_chain_id: &str,
) -> Value {
    let requested_chain_id = params
        .as_array()
        .and_then(|items| items.first())
        .and_then(|item| item.get("chainId"))
        .and_then(Value::as_str);
    let chain_name = params
        .as_array()
        .and_then(|items| items.first())
        .and_then(|item| item.get("chainName"))
        .and_then(Value::as_str);
    let rpc_urls = params
        .as_array()
        .and_then(|items| items.first())
        .and_then(|item| item.get("rpcUrls"))
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let block_explorers = params
        .as_array()
        .and_then(|items| items.first())
        .and_then(|item| item.get("blockExplorerUrls"))
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let intent = if method == "wallet_addEthereumChain" {
        "add_network"
    } else {
        "switch_network"
    };
    let rpc_source = if method == "wallet_addEthereumChain" {
        "trusted_alchemy_only"
    } else {
        "trusted_alchemy_session"
    };

    json!({
        "intent": intent,
        "currentChainId": current_chain_id,
        "requestedChainId": requested_chain_id,
        "providedChainName": chain_name,
        "providedRpcUrlCount": rpc_urls,
        "providedBlockExplorerUrlCount": block_explorers,
        "rpcSource": rpc_source,
        "simulation": "not_applicable",
        "decision": "requires_trusted_approval",
    })
}

pub(crate) fn summarize_watch_asset(params: &Value, current_chain_id: &str) -> Value {
    let request = watch_asset_request_object(params);
    let options = request
        .and_then(|request| request.get("options"))
        .and_then(Value::as_object);
    let asset_type = request
        .and_then(|request| request.get("type"))
        .and_then(Value::as_str);
    let address = options
        .and_then(|options| options.get("address"))
        .and_then(Value::as_str);
    let symbol = options
        .and_then(|options| options.get("symbol"))
        .and_then(Value::as_str);
    let decimals = options.and_then(|options| options.get("decimals"));
    let image = options
        .and_then(|options| options.get("image"))
        .and_then(Value::as_str);

    json!({
        "intent": "watch_asset",
        "assetType": asset_type,
        "chainId": current_chain_id,
        "contractAddress": address,
        "symbol": symbol.map(|symbol| preview_string(symbol, 24)),
        "decimals": decimals.map(|value| truncate_value(value, 0)),
        "imageProvided": image.is_some(),
        "source": "dapp_request",
        "decision": "requires_trusted_approval",
    })
}

pub(crate) fn watch_asset_request_object(params: &Value) -> Option<&Map<String, Value>> {
    params.as_object().or_else(|| {
        params
            .as_array()
            .and_then(|items| items.first())
            .and_then(Value::as_object)
    })
}

pub(crate) fn summarize_account_connection(method: &str, params: &Value) -> Value {
    let requested = if method == "wallet_requestPermissions" {
        params
            .as_array()
            .and_then(|items| items.first())
            .and_then(Value::as_object)
            .map(|object| object.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default()
    } else {
        vec!["eth_accounts".to_owned()]
    };

    json!({
        "intent": "connect_account",
        "requestedPermissions": requested,
        "permission": "eth_accounts",
        "simulation": "not_applicable",
        "decision": "requires_trusted_approval",
    })
}

pub(crate) fn summarize_personal_sign(params: &Value) -> Value {
    let message = array_value(params, 0);
    let account = array_string(params, 1);
    json!({
        "intent": "personal_sign",
        "account": account,
        "message": message.map(payload_summary).unwrap_or(Value::Null),
        "simulation": "not_applicable",
        "decision": "blocked_before_approval",
    })
}

pub fn personal_sign_payload(params: &Value) -> Result<PersonalSignPayload> {
    let items = params
        .as_array()
        .ok_or_else(|| anyhow::anyhow!("personal_sign params must be an array"))?;
    let message = items
        .first()
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("personal_sign message must be a string"))?;
    let message = personal_sign_message_bytes(message)?;
    let expected_address = match items.get(1) {
        Some(Value::String(address)) => {
            if !looks_like_eth_address(address) {
                bail!("personal_sign account must be a 0x-prefixed EVM address");
            }
            Some(address.clone())
        }
        Some(Value::Null) | None => None,
        Some(_) => bail!("personal_sign account must be a string"),
    };

    Ok(PersonalSignPayload {
        message,
        expected_address,
    })
}

pub(crate) fn personal_sign_message_bytes(message: &str) -> Result<Vec<u8>> {
    let Some(hex) = message.strip_prefix("0x") else {
        return Ok(message.as_bytes().to_vec());
    };
    if hex.len() % 2 != 0 || !hex.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        bail!("personal_sign hex message is malformed");
    }

    let mut bytes = Vec::with_capacity(hex.len() / 2);
    for pair in hex.as_bytes().chunks(2) {
        let high = hex_nibble(pair[0]).expect("validated hex above");
        let low = hex_nibble(pair[1]).expect("validated hex above");
        bytes.push((high << 4) | low);
    }
    Ok(bytes)
}

pub(crate) fn summarize_eth_sign(method: &str, params: &Value) -> Value {
    let account = array_string(params, 0);
    let payload = array_value(params, 1);
    json!({
        "intent": method,
        "account": account,
        "payload": payload.map(payload_summary).unwrap_or(Value::Null),
        "simulation": "not_applicable",
        "decision": "blocked_before_approval",
    })
}

pub(crate) fn summarize_typed_data(method: &str, params: &Value) -> Value {
    let account = find_first_address(params);
    let typed_data = typed_data_param(params);
    let typed_data = typed_data.map(parse_json_string_value);

    let summary = typed_data.as_ref().map(summarize_typed_data_value);

    json!({
        "intent": method,
        "account": account,
        "typedData": summary.unwrap_or(Value::Null),
        "simulation": "not_applicable",
        "decision": "blocked_before_approval",
    })
}

pub fn typed_data_payload(params: &Value) -> Result<TypedDataPayload> {
    let typed_data = typed_data_param(params)
        .map(parse_json_string_value)
        .ok_or_else(|| anyhow::anyhow!("typed-data params must include a typed data object"))?;
    if !typed_data.is_object() {
        anyhow::bail!("typed-data payload must be a JSON object");
    }
    Ok(TypedDataPayload {
        typed_data,
        expected_address: find_first_address(params),
    })
}

pub(crate) fn summarize_typed_data_value(value: &Value) -> Value {
    if let Some(object) = value.as_object() {
        let type_names = object
            .get("types")
            .and_then(Value::as_object)
            .map(|types| types.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        let primary_type = object.get("primaryType").and_then(Value::as_str);
        let permit =
            primary_type.and_then(|primary_type| typed_data_permit_summary(primary_type, object));
        let intent = permit
            .as_ref()
            .and_then(|permit| permit.get("kind"))
            .and_then(Value::as_str)
            .unwrap_or("typed_data");
        return json!({
            "intent": intent,
            "domain": object.get("domain").map(|value| truncate_value(value, 0)),
            "primaryType": primary_type,
            "typeNames": type_names,
            "messagePreview": object.get("message").map(|value| truncate_value(value, 0)),
            "permit": permit.unwrap_or(Value::Null),
        });
    }
    payload_summary(value)
}

pub(crate) fn typed_data_permit_summary(
    primary_type: &str,
    object: &Map<String, Value>,
) -> Option<Value> {
    let message = object.get("message")?.as_object()?;
    match primary_type {
        "Permit" => Some(json!({
            "kind": "erc20_permit",
            "owner": object_string(message, "owner"),
            "spender": object_string(message, "spender"),
            "token": domain_verifying_contract(object),
            "amount": object_display_value(message, "value"),
            "nonce": object_display_value(message, "nonce"),
            "deadline": object_display_value(message, "deadline"),
        })),
        "PermitSingle" => {
            let details = message.get("details").and_then(Value::as_object);
            Some(json!({
                "kind": "permit2_permit_single",
                "owner": object_string(message, "owner"),
                "spender": object_string(message, "spender"),
                "token": details.and_then(|details| object_string(details, "token")),
                "amount": details.and_then(|details| object_display_value(details, "amount")),
                "nonce": details.and_then(|details| object_display_value(details, "nonce")),
                "expiration": details.and_then(|details| object_display_value(details, "expiration")),
                "deadline": object_display_value(message, "sigDeadline"),
            }))
        }
        "PermitBatch" => {
            let tokens = message
                .get("details")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_object)
                        .take(8)
                        .map(permit2_details_summary)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            Some(json!({
                "kind": "permit2_permit_batch",
                "owner": object_string(message, "owner"),
                "spender": object_string(message, "spender"),
                "tokens": tokens,
                "tokenCount": message
                    .get("details")
                    .and_then(Value::as_array)
                    .map(Vec::len)
                    .unwrap_or(0),
                "deadline": object_display_value(message, "sigDeadline"),
            }))
        }
        "PermitTransferFrom" => {
            let permitted = message.get("permitted").and_then(Value::as_object);
            Some(json!({
                "kind": "permit2_transfer_from",
                "owner": object_string(message, "owner"),
                "spender": object_string(message, "spender"),
                "token": permitted.and_then(|permitted| object_string(permitted, "token")),
                "amount": permitted.and_then(|permitted| object_display_value(permitted, "amount")),
                "nonce": object_display_value(message, "nonce"),
                "deadline": object_display_value(message, "deadline"),
            }))
        }
        "PermitBatchTransferFrom" => {
            let tokens = message
                .get("permitted")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(Value::as_object)
                        .take(8)
                        .map(permit2_permitted_summary)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();
            Some(json!({
                "kind": "permit2_batch_transfer_from",
                "owner": object_string(message, "owner"),
                "spender": object_string(message, "spender"),
                "tokens": tokens,
                "tokenCount": message
                    .get("permitted")
                    .and_then(Value::as_array)
                    .map(Vec::len)
                    .unwrap_or(0),
                "nonce": object_display_value(message, "nonce"),
                "deadline": object_display_value(message, "deadline"),
            }))
        }
        _ => None,
    }
}

pub(crate) fn permit2_details_summary(details: &Map<String, Value>) -> Value {
    json!({
        "token": object_string(details, "token"),
        "amount": object_display_value(details, "amount"),
        "expiration": object_display_value(details, "expiration"),
        "nonce": object_display_value(details, "nonce"),
    })
}

pub(crate) fn permit2_permitted_summary(permitted: &Map<String, Value>) -> Value {
    json!({
        "token": object_string(permitted, "token"),
        "amount": object_display_value(permitted, "amount"),
    })
}

pub(crate) fn domain_verifying_contract(object: &Map<String, Value>) -> Option<&str> {
    object
        .get("domain")
        .and_then(Value::as_object)
        .and_then(|domain| object_string(domain, "verifyingContract"))
}

pub(crate) fn object_string<'a>(object: &'a Map<String, Value>, field: &str) -> Option<&'a str> {
    object.get(field).and_then(Value::as_str)
}

pub(crate) fn object_display_value(object: &Map<String, Value>, field: &str) -> Option<Value> {
    let value = object.get(field)?;
    Some(match value {
        Value::String(text) => json!(preview_string(text, 96)),
        Value::Number(number) => json!(number.to_string()),
        Value::Bool(value) => json!(value),
        Value::Null => Value::Null,
        other => truncate_value(other, 0),
    })
}

pub(crate) fn summarize_transaction(
    method: &str,
    params: &Value,
    default_chain_id: &str,
    review: Option<TransactionReviewReport>,
    asset_context: Option<Value>,
) -> Value {
    let review =
        review.unwrap_or_else(|| local_transaction_review(method, params, default_chain_id));
    let tx = array_value(params, 0).and_then(Value::as_object);
    let data = tx.and_then(|tx| tx.get("data").or_else(|| tx.get("input")));
    let data_len = data.and_then(Value::as_str).and_then(hex_data_byte_len);

    json!({
        "intent": method,
        "chainId": review.simulation.chain_id,
        "from": tx.and_then(|tx| tx.get("from")).and_then(Value::as_str),
        "to": tx.and_then(|tx| tx.get("to")).and_then(Value::as_str),
        "value": tx.and_then(|tx| tx.get("value")).and_then(Value::as_str),
        "dataBytes": data_len,
        "hasData": data_len.unwrap_or(0) > 0,
        "gas": tx.and_then(|tx| tx.get("gas")).or_else(|| tx.and_then(|tx| tx.get("gasLimit"))),
        "gasPrice": tx.and_then(|tx| tx.get("gasPrice")),
        "maxFeePerGas": tx.and_then(|tx| tx.get("maxFeePerGas")),
        "maxPriorityFeePerGas": tx.and_then(|tx| tx.get("maxPriorityFeePerGas")),
        "nonce": tx.and_then(|tx| tx.get("nonce")),
        "simulation": review.simulation,
        "policy": review.policy,
        "risk": review.risk,
        "guidance": transaction_guidance_value(&review.policy, &review.risk),
        "impact": review.impact,
        "trust": review.trust,
        "assetContext": asset_context.unwrap_or(Value::Null),
        "decision": "requires_trusted_policy_approval",
    })
}

pub(crate) fn transaction_guidance_value(
    policy: &TransactionPolicyEvaluation,
    risk: &TransactionRiskSummary,
) -> Value {
    let first_blocker = policy
        .blockers
        .iter()
        .find(|blocker| !blocker.overrideable)
        .or_else(|| policy.blockers.first());
    let status = if policy.can_sign {
        "ready"
    } else if policy.override_allowed {
        "high_risk"
    } else {
        "blocked"
    };
    let tone = if policy.can_sign {
        "good"
    } else if policy.override_allowed {
        "warn"
    } else {
        "bad"
    };
    let primary_action = if policy.can_sign {
        "Approve Transaction"
    } else if policy.override_allowed {
        "Approve High Risk"
    } else {
        "Cannot Sign"
    };
    let title = if policy.can_sign {
        "Ready to approve"
    } else if policy.override_allowed {
        "High-risk confirmation required"
    } else {
        "Cannot sign this transaction"
    };
    let message = if policy.can_sign {
        "Live simulation succeeded and policy found no blockers.".to_owned()
    } else if policy.override_allowed {
        risk.message.clone()
    } else {
        first_blocker
            .map(blocked_guidance_message)
            .unwrap_or_else(|| "Policy does not allow this request to reach signing.".to_owned())
    };
    let next_step = if policy.can_sign {
        "Review the simulated impact, then approve only if it matches your intent."
    } else if policy.override_allowed {
        "Continue only if you understand every warning; otherwise reject the request."
    } else {
        blocked_guidance_next_step(first_blocker)
    };

    json!({
        "status": status,
        "tone": tone,
        "title": title,
        "message": message,
        "primaryAction": primary_action,
        "nextStep": next_step,
        "canApprove": policy.can_sign || policy.override_allowed,
        "requiresHighRisk": !policy.can_sign && policy.override_allowed,
        "blocked": !policy.can_sign && !policy.override_allowed,
        "reasonCode": first_blocker.map(|blocker| blocker.code.as_str()),
    })
}

pub(crate) fn blocked_guidance_message(blocker: &PolicyBlocker) -> String {
    match blocker.code.as_str() {
        "simulation_provider_failed" => {
            "Live simulation did not return a safe result, so signing is disabled.".to_owned()
        }
        "invalid_transaction_request" => {
            "The transaction request is malformed, so signing is disabled.".to_owned()
        }
        _ => blocker.message.clone(),
    }
}

pub(crate) fn blocked_guidance_next_step(blocker: Option<&PolicyBlocker>) -> &'static str {
    match blocker.map(|blocker| blocker.code.as_str()) {
        Some("simulation_provider_failed") => {
            "Check RPC health or retry after the dApp can be simulated successfully."
        }
        Some("invalid_transaction_request") => {
            "Reject and retry from the dApp with a valid request."
        }
        _ => "Reject this request; signing is unavailable for the current policy state.",
    }
}
