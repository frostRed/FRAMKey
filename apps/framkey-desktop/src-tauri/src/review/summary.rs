use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail};
use framkey_simulation::{
    PolicyBlocker, TransactionPolicyEvaluation, TransactionReviewReport, TransactionRiskSummary,
    known_protocol_counterparty, local_transaction_review,
};
use serde_json::{Map, Value, json};

use super::*;

const MAX_PERMIT_DEADLINE_SECONDS_FROM_NOW: u64 = 90 * 24 * 60 * 60;
const MAX_PERMIT2_EXPIRATION_SECONDS_FROM_NOW: u64 = 90 * 24 * 60 * 60;
const MAX_U256_DECIMAL: &str =
    "115792089237316195423570985008687907853269984665640564039457584007913129639935";
const MAX_U160_DECIMAL: &str = "1461501637330902918203684832716283019655932542975";
const MAX_U256_HEX: &str = "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
const MAX_U160_HEX: &str = "ffffffffffffffffffffffffffffffffffffffff";

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
        ReviewMethodKind::TypedData => summarize_typed_data(method, params, chain_id),
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

pub(crate) fn summarize_typed_data(method: &str, params: &Value, chain_id: &str) -> Value {
    let account = typed_data_signer_account(params).map(str::to_owned);
    let typed_data = typed_data_param(params);
    let typed_data = typed_data.map(parse_json_string_value);

    let summary = typed_data
        .as_ref()
        .map(|value| summarize_typed_data_value(method, params, value, chain_id));

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
        expected_address: typed_data_signer_account(params).map(str::to_owned),
    })
}

pub(crate) fn summarize_typed_data_value(
    method: &str,
    params: &Value,
    value: &Value,
    chain_id: &str,
) -> Value {
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
            "policy": typed_data_policy_value(method, params, chain_id, object),
        });
    }
    payload_summary(value)
}

pub(crate) fn typed_data_signer_account(params: &Value) -> Option<&str> {
    params
        .as_array()
        .and_then(|items| items.first())
        .and_then(Value::as_str)
        .filter(|account| looks_like_eth_address(account))
}

pub(crate) fn typed_data_policy_value(
    method: &str,
    params: &Value,
    current_chain_id: &str,
    object: &Map<String, Value>,
) -> Value {
    let mut blockers = Vec::new();
    if method != "eth_signTypedData_v4" {
        typed_data_blocker(
            &mut blockers,
            "unsupported_typed_data_method",
            "typed-data signing is enabled only for eth_signTypedData_v4",
        );
    }

    let account = typed_data_signer_account(params);
    if account.is_none() {
        typed_data_blocker(
            &mut blockers,
            "missing_typed_data_account",
            "typed-data params must start with the connected signer account",
        );
    }

    let primary_type = object.get("primaryType").and_then(Value::as_str);
    let intent = primary_type.and_then(typed_data_permit_intent);
    if intent.is_none() {
        typed_data_blocker(
            &mut blockers,
            "unrecognized_typed_data_intent",
            "typed-data signing is only enabled for recognized ERC-20 Permit and Permit2 requests",
        );
    }
    validate_typed_data_schema(&mut blockers, object, intent);

    let message = object.get("message").and_then(Value::as_object);
    if message.is_none() {
        typed_data_blocker(
            &mut blockers,
            "missing_typed_data_message",
            "typed-data payload must include a message object",
        );
    }

    let domain = object.get("domain").and_then(Value::as_object);
    validate_typed_data_domain(&mut blockers, current_chain_id, domain, intent);

    if let (Some(intent), Some(account), Some(message)) = (intent, account, message) {
        validate_typed_data_owner(&mut blockers, intent, account, message);
        validate_typed_data_spender(&mut blockers, current_chain_id, intent, message);
        validate_typed_data_tokens(&mut blockers, intent, object, message);
        validate_typed_data_deadlines(&mut blockers, intent, message);
        validate_typed_data_amounts(&mut blockers, intent, message);
    }

    json!({
        "decision": if blockers.is_empty() { "allowed" } else { "blocked" },
        "canSign": blockers.is_empty(),
        "blockers": blockers,
    })
}

pub(crate) fn validate_typed_data_schema(
    blockers: &mut Vec<Value>,
    object: &Map<String, Value>,
    intent: Option<&str>,
) {
    let Some(intent) = intent else {
        return;
    };
    let Some(types) = object.get("types").and_then(Value::as_object) else {
        typed_data_blocker(
            blockers,
            "missing_typed_data_types",
            "typed-data payload must include type definitions for recognized Permit signing",
        );
        return;
    };
    let valid = match intent {
        "erc20_permit" => type_fields_match(
            types,
            "Permit",
            &[
                ("owner", "address"),
                ("spender", "address"),
                ("value", "uint256"),
                ("nonce", "uint256"),
                ("deadline", "uint256"),
            ],
        ),
        "permit2_permit_single" => {
            permit2_details_schema_matches(types)
                && type_fields_match(
                    types,
                    "PermitSingle",
                    &[
                        ("details", "PermitDetails"),
                        ("spender", "address"),
                        ("sigDeadline", "uint256"),
                    ],
                )
        }
        "permit2_permit_batch" => {
            permit2_details_schema_matches(types)
                && type_fields_match(
                    types,
                    "PermitBatch",
                    &[
                        ("details", "PermitDetails[]"),
                        ("spender", "address"),
                        ("sigDeadline", "uint256"),
                    ],
                )
        }
        "permit2_transfer_from" => {
            permit2_token_permissions_schema_matches(types)
                && type_fields_match(
                    types,
                    "PermitTransferFrom",
                    &[
                        ("permitted", "TokenPermissions"),
                        ("spender", "address"),
                        ("nonce", "uint256"),
                        ("deadline", "uint256"),
                    ],
                )
        }
        "permit2_batch_transfer_from" => {
            permit2_token_permissions_schema_matches(types)
                && type_fields_match(
                    types,
                    "PermitBatchTransferFrom",
                    &[
                        ("permitted", "TokenPermissions[]"),
                        ("spender", "address"),
                        ("nonce", "uint256"),
                        ("deadline", "uint256"),
                    ],
                )
        }
        _ => false,
    };
    if !valid {
        typed_data_blocker(
            blockers,
            "typed_data_schema_mismatch",
            "typed-data type definitions must exactly match the recognized Permit schema",
        );
    }
}

fn permit2_details_schema_matches(types: &Map<String, Value>) -> bool {
    type_fields_match(
        types,
        "PermitDetails",
        &[
            ("token", "address"),
            ("amount", "uint160"),
            ("expiration", "uint48"),
            ("nonce", "uint48"),
        ],
    )
}

fn permit2_token_permissions_schema_matches(types: &Map<String, Value>) -> bool {
    type_fields_match(
        types,
        "TokenPermissions",
        &[("token", "address"), ("amount", "uint256")],
    )
}

fn type_fields_match(
    types: &Map<String, Value>,
    type_name: &str,
    expected: &[(&str, &str)],
) -> bool {
    let Some(fields) = types.get(type_name).and_then(Value::as_array) else {
        return false;
    };
    fields.len() == expected.len()
        && fields.iter().zip(expected.iter()).all(|(field, expected)| {
            let Some(field) = field.as_object() else {
                return false;
            };
            field.get("name").and_then(Value::as_str) == Some(expected.0)
                && field.get("type").and_then(Value::as_str) == Some(expected.1)
        })
}

pub(crate) fn typed_data_permit_intent(primary_type: &str) -> Option<&'static str> {
    match primary_type {
        "Permit" => Some("erc20_permit"),
        "PermitSingle" => Some("permit2_permit_single"),
        "PermitBatch" => Some("permit2_permit_batch"),
        "PermitTransferFrom" => Some("permit2_transfer_from"),
        "PermitBatchTransferFrom" => Some("permit2_batch_transfer_from"),
        _ => None,
    }
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

pub(crate) fn validate_typed_data_domain(
    blockers: &mut Vec<Value>,
    current_chain_id: &str,
    domain: Option<&Map<String, Value>>,
    intent: Option<&str>,
) {
    let Some(domain) = domain else {
        typed_data_blocker(
            blockers,
            "missing_typed_data_domain",
            "typed-data payload must include an EIP-712 domain",
        );
        return;
    };

    let domain_chain_id = domain.get("chainId").and_then(parse_u64_value);
    let current_chain_id = parse_chain_id_text(current_chain_id);
    match (domain_chain_id, current_chain_id) {
        (Some(domain_chain_id), Some(current_chain_id)) if domain_chain_id == current_chain_id => {}
        (Some(_), Some(_)) => typed_data_blocker(
            blockers,
            "typed_data_chain_mismatch",
            "typed-data domain chainId must match the active FRAMKey chain",
        ),
        _ => typed_data_blocker(
            blockers,
            "invalid_typed_data_chain",
            "typed-data domain chainId must be a valid integer chain id",
        ),
    }

    let Some(verifying_contract) = object_string(domain, "verifyingContract") else {
        typed_data_blocker(
            blockers,
            "missing_verifying_contract",
            "typed-data domain must include verifyingContract",
        );
        return;
    };
    if !looks_like_eth_address(verifying_contract) {
        typed_data_blocker(
            blockers,
            "invalid_verifying_contract",
            "typed-data domain verifyingContract must be a 0x-prefixed EVM address",
        );
        return;
    }

    if intent.is_some_and(|intent| intent.starts_with("permit2_")) {
        let is_permit2 = known_protocol_counterparty(
            &format_chain_id_hex(current_chain_id.unwrap_or_default()),
            verifying_contract,
        )
        .is_some_and(|counterparty| counterparty.label == "Permit2");
        if !is_permit2 {
            typed_data_blocker(
                blockers,
                "permit2_verifying_contract_mismatch",
                "Permit2 typed-data must be signed for the known Permit2 contract on the active chain",
            );
        }
        if let Some(name) = object_string(domain, "name")
            && name != "Permit2"
        {
            typed_data_blocker(
                blockers,
                "permit2_domain_name_mismatch",
                "Permit2 typed-data domain name must be Permit2",
            );
        }
    }
}

pub(crate) fn validate_typed_data_owner(
    blockers: &mut Vec<Value>,
    intent: &str,
    account: &str,
    message: &Map<String, Value>,
) {
    if intent == "erc20_permit" {
        let Some(owner) = object_string(message, "owner") else {
            typed_data_blocker(
                blockers,
                "missing_permit_owner",
                "ERC-20 Permit message must include owner",
            );
            return;
        };
        if !looks_like_eth_address(owner) {
            typed_data_blocker(
                blockers,
                "invalid_permit_owner",
                "ERC-20 Permit owner must be a 0x-prefixed EVM address",
            );
        } else if !owner.eq_ignore_ascii_case(account) {
            typed_data_blocker(
                blockers,
                "permit_owner_mismatch",
                "ERC-20 Permit owner must match the signer account",
            );
        }
        return;
    }

    if let Some(owner) = object_string(message, "owner") {
        if !looks_like_eth_address(owner) {
            typed_data_blocker(
                blockers,
                "invalid_permit_owner",
                "Permit2 owner must be a 0x-prefixed EVM address when present",
            );
        } else if !owner.eq_ignore_ascii_case(account) {
            typed_data_blocker(
                blockers,
                "permit_owner_mismatch",
                "Permit2 owner must match the signer account when present",
            );
        }
    }
}

pub(crate) fn validate_typed_data_spender(
    blockers: &mut Vec<Value>,
    current_chain_id: &str,
    intent: &str,
    message: &Map<String, Value>,
) {
    let Some(spender) = object_string(message, "spender") else {
        typed_data_blocker(
            blockers,
            "missing_permit_spender",
            "Permit message must include spender",
        );
        return;
    };
    if !looks_like_eth_address(spender) {
        typed_data_blocker(
            blockers,
            "invalid_permit_spender",
            "Permit spender must be a 0x-prefixed EVM address",
        );
        return;
    }
    if known_protocol_counterparty(current_chain_id, spender).is_none() {
        let message = if intent.starts_with("permit2_") {
            "Permit2 spender must be a known protocol counterparty on the active chain"
        } else {
            "ERC-20 Permit spender must be a known protocol counterparty on the active chain"
        };
        typed_data_blocker(blockers, "unknown_permit_spender", message);
    }
}

pub(crate) fn validate_typed_data_tokens(
    blockers: &mut Vec<Value>,
    intent: &str,
    object: &Map<String, Value>,
    message: &Map<String, Value>,
) {
    match intent {
        "erc20_permit" => {
            if domain_verifying_contract(object).is_none_or(|token| !looks_like_eth_address(token))
            {
                typed_data_blocker(
                    blockers,
                    "invalid_permit_token",
                    "ERC-20 Permit token must be the domain verifyingContract",
                );
            }
        }
        "permit2_permit_single" => {
            validate_permit2_details_token(blockers, message.get("details"));
        }
        "permit2_permit_batch" => {
            validate_permit2_details_array_tokens(blockers, message.get("details"));
        }
        "permit2_transfer_from" => {
            validate_permit2_details_token(blockers, message.get("permitted"));
        }
        "permit2_batch_transfer_from" => {
            validate_permit2_details_array_tokens(blockers, message.get("permitted"));
        }
        _ => {}
    }
}

pub(crate) fn validate_typed_data_deadlines(
    blockers: &mut Vec<Value>,
    intent: &str,
    message: &Map<String, Value>,
) {
    let now = current_unix_seconds();
    let deadline_field = if intent.starts_with("permit2_permit") {
        "sigDeadline"
    } else {
        "deadline"
    };
    validate_deadline_field(
        blockers,
        message,
        deadline_field,
        "permit_deadline_invalid",
        "Permit signature deadline must be a valid future unix timestamp",
        now,
        MAX_PERMIT_DEADLINE_SECONDS_FROM_NOW,
    );

    match intent {
        "permit2_permit_single" => {
            if let Some(details) = message.get("details").and_then(Value::as_object) {
                validate_deadline_field(
                    blockers,
                    details,
                    "expiration",
                    "permit2_expiration_invalid",
                    "Permit2 allowance expiration must be a valid future unix timestamp",
                    now,
                    MAX_PERMIT2_EXPIRATION_SECONDS_FROM_NOW,
                );
            }
        }
        "permit2_permit_batch" => {
            if let Some(items) = message.get("details").and_then(Value::as_array) {
                for details in items.iter().filter_map(Value::as_object) {
                    validate_deadline_field(
                        blockers,
                        details,
                        "expiration",
                        "permit2_expiration_invalid",
                        "Permit2 allowance expiration must be a valid future unix timestamp",
                        now,
                        MAX_PERMIT2_EXPIRATION_SECONDS_FROM_NOW,
                    );
                }
            }
        }
        _ => {}
    }
}

pub(crate) fn validate_typed_data_amounts(
    blockers: &mut Vec<Value>,
    intent: &str,
    message: &Map<String, Value>,
) {
    match intent {
        "erc20_permit" => {
            if message
                .get("value")
                .is_some_and(|value| is_unbounded_uint(value, MAX_U256_DECIMAL))
            {
                typed_data_blocker(
                    blockers,
                    "typed_data_unlimited_permit",
                    "ERC-20 Permit value must not be the maximum uint256 allowance",
                );
            }
        }
        "permit2_permit_single" => {
            if let Some(details) = message.get("details").and_then(Value::as_object) {
                validate_permit2_amount(blockers, details);
            }
        }
        "permit2_permit_batch" => {
            if let Some(items) = message.get("details").and_then(Value::as_array) {
                for details in items.iter().filter_map(Value::as_object) {
                    validate_permit2_amount(blockers, details);
                }
            }
        }
        "permit2_transfer_from" => {
            if let Some(permitted) = message.get("permitted").and_then(Value::as_object) {
                validate_permit2_amount(blockers, permitted);
            }
        }
        "permit2_batch_transfer_from" => {
            if let Some(items) = message.get("permitted").and_then(Value::as_array) {
                for permitted in items.iter().filter_map(Value::as_object) {
                    validate_permit2_amount(blockers, permitted);
                }
            }
        }
        _ => {}
    }
}

pub(crate) fn validate_permit2_details_token(blockers: &mut Vec<Value>, details: Option<&Value>) {
    let Some(details) = details.and_then(Value::as_object) else {
        typed_data_blocker(
            blockers,
            "missing_permit2_token",
            "Permit2 details must include a token address",
        );
        return;
    };
    if details
        .get("token")
        .and_then(Value::as_str)
        .is_none_or(|token| !looks_like_eth_address(token))
    {
        typed_data_blocker(
            blockers,
            "invalid_permit2_token",
            "Permit2 token must be a 0x-prefixed EVM address",
        );
    }
}

pub(crate) fn validate_permit2_details_array_tokens(
    blockers: &mut Vec<Value>,
    details: Option<&Value>,
) {
    let Some(items) = details.and_then(Value::as_array) else {
        typed_data_blocker(
            blockers,
            "missing_permit2_token",
            "Permit2 batch must include token details",
        );
        return;
    };
    if items.is_empty() {
        typed_data_blocker(
            blockers,
            "missing_permit2_token",
            "Permit2 batch must include at least one token",
        );
    }
    for item in items {
        validate_permit2_details_token(blockers, Some(item));
    }
}

pub(crate) fn validate_permit2_amount(blockers: &mut Vec<Value>, details: &Map<String, Value>) {
    if details
        .get("amount")
        .is_some_and(|amount| is_unbounded_uint(amount, MAX_U160_DECIMAL))
    {
        typed_data_blocker(
            blockers,
            "typed_data_unlimited_permit",
            "Permit2 amount must not be the maximum uint160 allowance",
        );
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn validate_deadline_field(
    blockers: &mut Vec<Value>,
    object: &Map<String, Value>,
    field: &str,
    code: &str,
    message: &str,
    now: u64,
    max_seconds_from_now: u64,
) {
    let Some(value) = object.get(field).and_then(parse_u64_value) else {
        typed_data_blocker(blockers, code, message);
        return;
    };
    if value <= now {
        typed_data_blocker(blockers, code, message);
        return;
    }
    if value.saturating_sub(now) > max_seconds_from_now {
        typed_data_blocker(
            blockers,
            "permit_deadline_too_far",
            "Permit deadline or expiration is too far in the future for ordinary signing",
        );
    }
}

pub(crate) fn typed_data_blocker(blockers: &mut Vec<Value>, code: &str, message: &str) {
    if blockers
        .iter()
        .any(|blocker| blocker.get("code").and_then(Value::as_str) == Some(code))
    {
        return;
    }
    blockers.push(json!({
        "code": code,
        "message": message,
    }));
}

pub(crate) fn parse_u64_value(value: &Value) -> Option<u64> {
    match value {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => parse_chain_id_text(text),
        _ => None,
    }
}

pub(crate) fn parse_chain_id_text(value: &str) -> Option<u64> {
    let value = value.trim();
    if let Some(hex) = value.strip_prefix("0x") {
        u64::from_str_radix(hex, 16).ok()
    } else {
        value.parse::<u64>().ok()
    }
}

pub(crate) fn format_chain_id_hex(value: u64) -> String {
    format!("0x{value:x}")
}

pub(crate) fn current_unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

pub(crate) fn is_unbounded_uint(value: &Value, max_decimal: &str) -> bool {
    match value {
        Value::String(text) => {
            let text = text.trim();
            if text.starts_with("0x") {
                normalized_hex_uint(text)
                    .as_deref()
                    .is_some_and(|normalized| {
                        max_hex_for_decimal(max_decimal).is_some_and(|max| normalized == max)
                    })
            } else {
                normalized_decimal_uint(text)
                    .as_deref()
                    .is_some_and(|normalized| normalized == max_decimal)
            }
        }
        Value::Number(number) => number.to_string() == max_decimal,
        _ => false,
    }
}

pub(crate) fn max_hex_for_decimal(max_decimal: &str) -> Option<&'static str> {
    match max_decimal {
        MAX_U256_DECIMAL => Some(MAX_U256_HEX),
        MAX_U160_DECIMAL => Some(MAX_U160_HEX),
        _ => None,
    }
}

pub(crate) fn normalized_decimal_uint(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty() || value.starts_with('-') {
        return None;
    }
    if !value.as_bytes().iter().all(u8::is_ascii_digit) {
        return None;
    }
    let trimmed = value.trim_start_matches('0');
    Some(if trimmed.is_empty() {
        "0".to_owned()
    } else {
        trimmed.to_owned()
    })
}

pub(crate) fn normalized_hex_uint(value: &str) -> Option<String> {
    let hex = value.strip_prefix("0x")?;
    if hex.is_empty() || !hex.as_bytes().iter().all(u8::is_ascii_hexdigit) {
        return None;
    }
    let trimmed = hex.trim_start_matches('0');
    Some(if trimmed.is_empty() {
        "0".to_owned()
    } else {
        trimmed.to_ascii_lowercase()
    })
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
