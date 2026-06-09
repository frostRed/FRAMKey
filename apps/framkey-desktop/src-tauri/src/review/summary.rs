use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail};
use framkey_simulation::{
    PolicyBlocker, TransactionPolicyEvaluation, TransactionReviewReport, TransactionRiskSummary,
    known_protocol_counterparty, local_transaction_review,
};
use serde_json::{Map, Value, json};
use tauri::Url;

use super::*;

const MAX_PERMIT_DEADLINE_SECONDS_FROM_NOW: u64 = 90 * 24 * 60 * 60;
const MAX_PERMIT2_EXPIRATION_SECONDS_FROM_NOW: u64 = 90 * 24 * 60 * 60;
const MAX_SIWE_EXPIRATION_SECONDS_FROM_NOW: u64 = 30 * 60;
const MAX_SIWE_ISSUED_AT_AGE_SECONDS: u64 = 24 * 60 * 60;
const MAX_SIWE_CLOCK_SKEW_SECONDS: u64 = 5 * 60;
const SIWE_NONCE_MIN_LEN: usize = 8;
const SIWE_NONCE_MAX_LEN: usize = 96;
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
    origin: Option<&str>,
    transaction_review: Option<TransactionReviewReport>,
    transaction_asset_context: Option<Value>,
) -> Value {
    match kind {
        ReviewMethodKind::PersonalSign => summarize_personal_sign(params, origin, chain_id),
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
        ReviewMethodKind::BtcTransaction => summarize_btc_transaction(params),
    }
}

pub(crate) fn summarize_btc_transaction(params: &Value) -> Value {
    let object = params.as_object();
    let policy = object
        .and_then(|object| object.get("policy"))
        .cloned()
        .unwrap_or_else(|| {
            json!({
                "canSign": false,
                "blockers": [{
                    "code": "missing_btc_policy",
                    "message": "BTC transaction review is missing a trusted PSBT policy summary",
                }],
            })
        });
    let can_sign = policy
        .get("canSign")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    json!({
        "intent": "btc_transaction",
        "network": object.and_then(|object| object.get("network")).cloned().unwrap_or(Value::Null),
        "fromAddress": object.and_then(|object| object.get("fromAddress")).cloned().unwrap_or(Value::Null),
        "toAddress": object.and_then(|object| object.get("toAddress")).cloned().unwrap_or(Value::Null),
        "amountSat": object.and_then(|object| object.get("amountSat")).cloned().unwrap_or(Value::Null),
        "feeSat": object.and_then(|object| object.get("feeSat")).cloned().unwrap_or(Value::Null),
        "feeRateSatVb": object.and_then(|object| object.get("feeRateSatVb")).cloned().unwrap_or(Value::Null),
        "inputValueSat": object.and_then(|object| object.get("inputValueSat")).cloned().unwrap_or(Value::Null),
        "changeSat": object.and_then(|object| object.get("changeSat")).cloned().unwrap_or(Value::Null),
        "estimatedVbytes": object.and_then(|object| object.get("estimatedVbytes")).cloned().unwrap_or(Value::Null),
        "inputCount": object
            .and_then(|object| object.get("selectedUtxos"))
            .and_then(Value::as_array)
            .map(|items| json!(items.len()))
            .unwrap_or(Value::Null),
        "outputs": object.and_then(|object| object.get("outputs")).cloned().unwrap_or(Value::Null),
        "policy": policy,
        "simulation": "not_applicable",
        "decision": if can_sign {
            "requires_trusted_approval"
        } else {
            "blocked_before_approval"
        },
    })
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
        "trusted_chain_endpoint"
    } else {
        "trusted_chain_session"
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

pub(crate) fn summarize_personal_sign(
    params: &Value,
    origin: Option<&str>,
    current_chain_id: &str,
) -> Value {
    let message = array_value(params, 0);
    let account = array_string(params, 1).filter(|account| looks_like_eth_address(account));
    let siwe = decode_personal_sign_text(message).and_then(|text| parse_siwe_message(&text));
    let mut blockers = Vec::new();

    validate_personal_sign_policy(
        &mut blockers,
        params,
        origin,
        current_chain_id,
        account,
        siwe.as_ref().ok(),
    );
    if let Err(error) = &siwe {
        personal_sign_blocker(&mut blockers, error.code, error.message);
    }

    json!({
        "intent": "personal_sign",
        "account": account,
        "message": message.map(payload_summary).unwrap_or(Value::Null),
        "siwe": personal_sign_siwe_summary(siwe.as_ref()),
        "policy": {
            "decision": if blockers.is_empty() { "allowed" } else { "blocked" },
            "canSign": blockers.is_empty(),
            "blockers": blockers,
        },
        "simulation": "not_applicable",
        "decision": if blockers.is_empty() {
            "requires_trusted_approval"
        } else {
            "blocked_before_approval"
        },
    })
}

fn decode_personal_sign_text(
    message: Option<&Value>,
) -> std::result::Result<String, SiweParseError> {
    let raw = message.and_then(Value::as_str).ok_or_else(|| {
        siwe_parse_error(
            "invalid_personal_sign_params",
            "personal_sign params must include a string message",
        )
    })?;
    let bytes = personal_sign_message_bytes(raw).map_err(|_| {
        siwe_parse_error(
            "invalid_personal_sign_message",
            "personal_sign message must be valid text or hex-encoded text",
        )
    })?;
    String::from_utf8(bytes).map_err(|_| {
        siwe_parse_error(
            "unsupported_personal_sign_message",
            "personal_sign signing is enabled only for UTF-8 SIWE messages",
        )
    })
}

fn validate_personal_sign_policy(
    blockers: &mut Vec<Value>,
    params: &Value,
    origin: Option<&str>,
    current_chain_id: &str,
    account: Option<&str>,
    siwe: Option<&SiweMessage>,
) {
    if params.as_array().is_none() {
        personal_sign_blocker(
            blockers,
            "invalid_personal_sign_params",
            "personal_sign params must be an array",
        );
    }
    match array_string(params, 1) {
        Some(address) if looks_like_eth_address(address) => {}
        Some(_) => personal_sign_blocker(
            blockers,
            "invalid_personal_sign_account",
            "personal_sign params must include a 0x-prefixed EVM account",
        ),
        None => personal_sign_blocker(
            blockers,
            "missing_personal_sign_account",
            "personal_sign params must include the connected signer account",
        ),
    }

    let Some(siwe) = siwe else {
        return;
    };

    let origin_authority = origin.and_then(url_authority);
    if origin_authority.is_none() {
        personal_sign_blocker(
            blockers,
            "missing_personal_sign_origin",
            "SIWE personal_sign requires a trusted dApp origin",
        );
    }

    let siwe_domain_authority = siwe_domain_authority(&siwe.domain);
    match (
        origin_authority.as_deref(),
        siwe_domain_authority.as_deref(),
    ) {
        (Some(origin), Some(domain)) if origin.eq_ignore_ascii_case(domain) => {}
        (Some(_), Some(_)) => personal_sign_blocker(
            blockers,
            "siwe_domain_origin_mismatch",
            "SIWE domain must match the requesting dApp origin",
        ),
        _ => personal_sign_blocker(
            blockers,
            "invalid_siwe_domain",
            "SIWE domain must be a valid host or host:port",
        ),
    }

    match (
        origin_authority.as_deref(),
        url_authority(&siwe.uri).as_deref(),
    ) {
        (Some(origin), Some(uri)) if origin.eq_ignore_ascii_case(uri) => {}
        (Some(_), Some(_)) => personal_sign_blocker(
            blockers,
            "siwe_uri_origin_mismatch",
            "SIWE URI must match the requesting dApp origin",
        ),
        _ => personal_sign_blocker(
            blockers,
            "invalid_siwe_uri",
            "SIWE URI must be an absolute URI for the requesting dApp origin",
        ),
    }

    if !looks_like_eth_address(&siwe.address) {
        personal_sign_blocker(
            blockers,
            "invalid_siwe_address",
            "SIWE address must be a 0x-prefixed EVM address",
        );
    } else if let Some(account) = account
        && !siwe.address.eq_ignore_ascii_case(account)
    {
        personal_sign_blocker(
            blockers,
            "siwe_account_mismatch",
            "SIWE address must match the requested signer account",
        );
    }

    if siwe.version != "1" {
        personal_sign_blocker(
            blockers,
            "unsupported_siwe_version",
            "SIWE version must be 1",
        );
    }

    match (
        siwe.chain_id.parse::<u64>().ok(),
        parse_chain_id_text(current_chain_id),
    ) {
        (Some(siwe_chain_id), Some(active_chain_id)) if siwe_chain_id == active_chain_id => {}
        (Some(_), Some(_)) => personal_sign_blocker(
            blockers,
            "siwe_chain_mismatch",
            "SIWE Chain ID must match the active FRAMKey chain",
        ),
        _ => personal_sign_blocker(
            blockers,
            "invalid_siwe_chain",
            "SIWE Chain ID must be a valid decimal chain id",
        ),
    }

    if siwe.nonce.len() < SIWE_NONCE_MIN_LEN
        || siwe.nonce.len() > SIWE_NONCE_MAX_LEN
        || !siwe.nonce.as_bytes().iter().all(u8::is_ascii_alphanumeric)
    {
        personal_sign_blocker(
            blockers,
            "invalid_siwe_nonce",
            "SIWE nonce must be 8-96 alphanumeric characters",
        );
    }

    validate_siwe_time_fields(blockers, siwe);

    if !siwe.resources.is_empty() {
        personal_sign_blocker(
            blockers,
            "siwe_resources_not_supported",
            "SIWE Resources are not supported for ordinary personal_sign signing",
        );
    }
}

fn validate_siwe_time_fields(blockers: &mut Vec<Value>, siwe: &SiweMessage) {
    let now = current_unix_seconds();
    match parse_rfc3339_seconds(&siwe.issued_at) {
        Some(issued_at) if issued_at > now.saturating_add(MAX_SIWE_CLOCK_SKEW_SECONDS) => {
            personal_sign_blocker(
                blockers,
                "siwe_issued_at_invalid",
                "SIWE Issued At must not be in the future",
            );
        }
        Some(issued_at) if now.saturating_sub(issued_at) > MAX_SIWE_ISSUED_AT_AGE_SECONDS => {
            personal_sign_blocker(
                blockers,
                "siwe_issued_at_too_old",
                "SIWE Issued At is too old for ordinary signing",
            );
        }
        Some(_) => {}
        None => personal_sign_blocker(
            blockers,
            "siwe_issued_at_invalid",
            "SIWE Issued At must be a valid RFC3339 UTC timestamp",
        ),
    }

    match siwe
        .expiration_time
        .as_deref()
        .and_then(parse_rfc3339_seconds)
    {
        Some(expiration_time) if expiration_time <= now => {
            personal_sign_blocker(
                blockers,
                "siwe_expiration_invalid",
                "SIWE Expiration Time must be in the future",
            );
        }
        Some(expiration_time)
            if expiration_time.saturating_sub(now) > MAX_SIWE_EXPIRATION_SECONDS_FROM_NOW =>
        {
            personal_sign_blocker(
                blockers,
                "siwe_expiration_too_far",
                "SIWE Expiration Time must be no more than 30 minutes in the future",
            );
        }
        Some(_) => {}
        None => personal_sign_blocker(
            blockers,
            "siwe_expiration_missing",
            "SIWE personal_sign requires a short Expiration Time",
        ),
    }

    if let Some(not_before) = &siwe.not_before {
        match parse_rfc3339_seconds(not_before) {
            Some(not_before) if not_before > now => personal_sign_blocker(
                blockers,
                "siwe_not_before_invalid",
                "SIWE Not Before must not be in the future",
            ),
            Some(_) => {}
            None => personal_sign_blocker(
                blockers,
                "siwe_not_before_invalid",
                "SIWE Not Before must be a valid RFC3339 UTC timestamp when present",
            ),
        }
    }
}

fn personal_sign_siwe_summary(siwe: std::result::Result<&SiweMessage, &SiweParseError>) -> Value {
    match siwe {
        Ok(siwe) => json!({
            "status": "ok",
            "domain": siwe.domain,
            "address": siwe.address,
            "statement": siwe.statement.as_deref().map(|statement| preview_string(statement, 160)),
            "uri": siwe.uri,
            "version": siwe.version,
            "chainId": siwe.chain_id,
            "nonce": preview_string(&siwe.nonce, 32),
            "issuedAt": siwe.issued_at,
            "expirationTime": siwe.expiration_time,
            "notBefore": siwe.not_before,
            "requestId": siwe.request_id.as_deref().map(|request_id| preview_string(request_id, 80)),
            "resourceCount": siwe.resources.len(),
        }),
        Err(error) => json!({
            "status": "unrecognized",
            "error": {
                "code": error.code,
                "message": error.message,
            },
        }),
    }
}

fn personal_sign_blocker(blockers: &mut Vec<Value>, code: &str, message: &str) {
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

#[derive(Debug, Clone)]
struct SiweMessage {
    domain: String,
    address: String,
    statement: Option<String>,
    uri: String,
    version: String,
    chain_id: String,
    nonce: String,
    issued_at: String,
    expiration_time: Option<String>,
    not_before: Option<String>,
    request_id: Option<String>,
    resources: Vec<String>,
}

#[derive(Debug, Clone)]
struct SiweParseError {
    code: &'static str,
    message: &'static str,
}

fn siwe_parse_error(code: &'static str, message: &'static str) -> SiweParseError {
    SiweParseError { code, message }
}

fn parse_siwe_message(text: &str) -> std::result::Result<SiweMessage, SiweParseError> {
    if text
        .chars()
        .any(|char| char.is_control() && !matches!(char, '\n' | '\r' | '\t'))
    {
        return Err(siwe_parse_error(
            "unsupported_personal_sign_message",
            "personal_sign signing is enabled only for plain-text SIWE messages",
        ));
    }

    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let lines = normalized.lines().collect::<Vec<_>>();
    let Some(first_line) = lines.first() else {
        return Err(siwe_parse_error(
            "unrecognized_personal_sign_message",
            "personal_sign signing is enabled only for SIWE messages",
        ));
    };
    let Some(domain) = first_line.strip_suffix(" wants you to sign in with your Ethereum account:")
    else {
        return Err(siwe_parse_error(
            "unrecognized_personal_sign_message",
            "personal_sign signing is enabled only for SIWE messages",
        ));
    };
    let domain = domain.trim();
    if domain.is_empty() {
        return Err(siwe_parse_error(
            "invalid_siwe_domain",
            "SIWE domain must not be empty",
        ));
    }

    let address = lines
        .get(1)
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .ok_or_else(|| {
            siwe_parse_error(
                "invalid_siwe_address",
                "SIWE message must include an Ethereum account address",
            )
        })?;
    if lines.get(2) != Some(&"") {
        return Err(siwe_parse_error(
            "unrecognized_personal_sign_message",
            "SIWE message must include the standard blank line after the address",
        ));
    }

    let mut index = 3;
    let mut statement = None;
    if let Some(line) = lines.get(index)
        && !line.starts_with("URI: ")
    {
        if line.is_empty() {
            index += 1;
        } else {
            let mut statement_lines = Vec::new();
            while let Some(line) = lines.get(index) {
                if line.is_empty() {
                    index += 1;
                    break;
                }
                statement_lines.push(*line);
                index += 1;
            }
            statement = Some(statement_lines.join("\n"));
        }
    }

    let mut uri = None;
    let mut version = None;
    let mut chain_id = None;
    let mut nonce = None;
    let mut issued_at = None;
    let mut expiration_time = None;
    let mut not_before = None;
    let mut request_id = None;
    let mut resources = Vec::new();

    while let Some(line) = lines.get(index) {
        if let Some(value) = line.strip_prefix("URI: ") {
            set_siwe_field(&mut uri, value)?;
        } else if let Some(value) = line.strip_prefix("Version: ") {
            set_siwe_field(&mut version, value)?;
        } else if let Some(value) = line.strip_prefix("Chain ID: ") {
            set_siwe_field(&mut chain_id, value)?;
        } else if let Some(value) = line.strip_prefix("Nonce: ") {
            set_siwe_field(&mut nonce, value)?;
        } else if let Some(value) = line.strip_prefix("Issued At: ") {
            set_siwe_field(&mut issued_at, value)?;
        } else if let Some(value) = line.strip_prefix("Expiration Time: ") {
            set_siwe_field(&mut expiration_time, value)?;
        } else if let Some(value) = line.strip_prefix("Not Before: ") {
            set_siwe_field(&mut not_before, value)?;
        } else if let Some(value) = line.strip_prefix("Request ID: ") {
            set_siwe_field(&mut request_id, value)?;
        } else if *line == "Resources:" {
            index += 1;
            while let Some(resource_line) = lines.get(index) {
                let Some(resource) = resource_line.strip_prefix("- ") else {
                    return Err(siwe_parse_error(
                        "invalid_siwe_resources",
                        "SIWE Resources entries must start with '- '",
                    ));
                };
                resources.push(resource.trim().to_owned());
                index += 1;
            }
            break;
        } else if line.trim().is_empty() {
            return Err(siwe_parse_error(
                "unrecognized_personal_sign_message",
                "SIWE fields must use the standard EIP-4361 format",
            ));
        } else {
            return Err(siwe_parse_error(
                "unrecognized_personal_sign_message",
                "personal_sign signing is enabled only for SIWE messages",
            ));
        }
        index += 1;
    }

    Ok(SiweMessage {
        domain: domain.to_owned(),
        address: address.to_owned(),
        statement,
        uri: required_siwe_field(uri, "URI")?,
        version: required_siwe_field(version, "Version")?,
        chain_id: required_siwe_field(chain_id, "Chain ID")?,
        nonce: required_siwe_field(nonce, "Nonce")?,
        issued_at: required_siwe_field(issued_at, "Issued At")?,
        expiration_time,
        not_before,
        request_id,
        resources,
    })
}

fn set_siwe_field(
    field: &mut Option<String>,
    value: &str,
) -> std::result::Result<(), SiweParseError> {
    if field.is_some() {
        return Err(siwe_parse_error(
            "invalid_siwe_message",
            "SIWE message must not repeat fields",
        ));
    }
    let value = value.trim();
    if value.is_empty() {
        return Err(siwe_parse_error(
            "invalid_siwe_message",
            "SIWE message fields must not be empty",
        ));
    }
    *field = Some(value.to_owned());
    Ok(())
}

fn required_siwe_field(
    field: Option<String>,
    label: &'static str,
) -> std::result::Result<String, SiweParseError> {
    field.ok_or_else(|| {
        siwe_parse_error(
            "invalid_siwe_message",
            match label {
                "URI" => "SIWE message must include URI",
                "Version" => "SIWE message must include Version",
                "Chain ID" => "SIWE message must include Chain ID",
                "Nonce" => "SIWE message must include Nonce",
                "Issued At" => "SIWE message must include Issued At",
                _ => "SIWE message is missing a required field",
            },
        )
    })
}

fn url_authority(value: &str) -> Option<String> {
    let url = Url::parse(value).ok()?;
    let host = url.host_str()?;
    Some(match url.port() {
        Some(port) => format!("{}:{port}", host.to_ascii_lowercase()),
        None => host.to_ascii_lowercase(),
    })
}

fn siwe_domain_authority(value: &str) -> Option<String> {
    let value = value.trim();
    if value.is_empty()
        || value.contains("://")
        || value.contains('/')
        || value.contains('@')
        || value.chars().any(char::is_control)
    {
        return None;
    }
    Some(value.trim_end_matches('.').to_ascii_lowercase())
}

fn parse_rfc3339_seconds(value: &str) -> Option<u64> {
    let value = value.trim();
    let without_z = value.strip_suffix('Z')?;
    let (base, fraction) = without_z
        .split_once('.')
        .map(|(base, fraction)| (base, Some(fraction)))
        .unwrap_or((without_z, None));
    if let Some(fraction) = fraction
        && (fraction.is_empty() || !fraction.as_bytes().iter().all(u8::is_ascii_digit))
    {
        return None;
    }
    if base.len() != 19
        || base.as_bytes().get(4) != Some(&b'-')
        || base.as_bytes().get(7) != Some(&b'-')
        || base.as_bytes().get(10) != Some(&b'T')
        || base.as_bytes().get(13) != Some(&b':')
        || base.as_bytes().get(16) != Some(&b':')
    {
        return None;
    }
    let year = parse_fixed_i64(base, 0, 4)?;
    let month = parse_fixed_u32(base, 5, 2)?;
    let day = parse_fixed_u32(base, 8, 2)?;
    let hour = parse_fixed_u32(base, 11, 2)?;
    let minute = parse_fixed_u32(base, 14, 2)?;
    let second = parse_fixed_u32(base, 17, 2)?;
    if !(1..=12).contains(&month)
        || day == 0
        || day > days_in_month(year, month)
        || hour > 23
        || minute > 59
        || second > 59
    {
        return None;
    }
    let days = days_from_civil(year, month, day);
    if days < 0 {
        return None;
    }
    Some(
        (days as u64)
            .saturating_mul(86_400)
            .saturating_add((hour as u64) * 3_600)
            .saturating_add((minute as u64) * 60)
            .saturating_add(second as u64),
    )
}

fn parse_fixed_i64(value: &str, start: usize, len: usize) -> Option<i64> {
    value.get(start..start + len)?.parse().ok()
}

fn parse_fixed_u32(value: &str, start: usize, len: usize) -> Option<u32> {
    value.get(start..start + len)?.parse().ok()
}

fn days_in_month(year: i64, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let month = month as i64;
    let day = day as i64;
    let year = year - i64::from(month <= 2);
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let month_prime = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * month_prime + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
pub(crate) fn format_rfc3339_seconds(seconds: u64) -> String {
    let days = (seconds / 86_400) as i64;
    let seconds_of_day = seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = seconds_of_day / 3_600;
    let minute = (seconds_of_day % 3_600) / 60;
    let second = seconds_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

#[cfg(test)]
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let days = days + 719_468;
    let era = if days >= 0 { days } else { days - 146_096 } / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    (year, month as u32, day as u32)
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
    let live_simulation_present = risk
        .reasons
        .iter()
        .any(|reason| reason.code == "live_simulation_present");
    let message = if policy.can_sign && live_simulation_present {
        "Live simulation succeeded and policy found no blockers.".to_owned()
    } else if policy.can_sign {
        "Local decoder matched a supported transfer or curated protocol action.".to_owned()
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
