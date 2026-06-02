use serde_json::{Map, Value, json};

use crate::{
    clients::AlchemyRpcSimulationConfig,
    decoder::{
        decimal_digits_to_hex_quantity, decode_hex_quantity, hex_bytes_to_decimal,
        looks_like_eth_address, normalize_decimal_digits, normalize_hex_quantity, string_field,
        warning,
    },
    model::{
        ApprovalChange, AssetTransfer, SimulationStatus, TokenAmount, TransactionSimulationReport,
        WarningSeverity,
    },
};

pub(crate) fn alchemy_rpc_payload(params: &Value, config: &AlchemyRpcSimulationConfig) -> Value {
    let tx = params
        .as_array()
        .and_then(|items| items.first())
        .and_then(Value::as_object);
    let mut call = Map::new();
    if let Some(tx) = tx {
        copy_string_field(tx, &mut call, "from", "from");
        copy_string_field(tx, &mut call, "to", "to");
        copy_string_field(tx, &mut call, "value", "value");
        copy_string_field(tx, &mut call, "gasPrice", "gasPrice");
        copy_string_field(tx, &mut call, "maxFeePerGas", "maxFeePerGas");
        copy_string_field(
            tx,
            &mut call,
            "maxPriorityFeePerGas",
            "maxPriorityFeePerGas",
        );
        copy_string_field(tx, &mut call, "nonce", "nonce");
        if let Some(gas) = string_field(tx, "gas").or_else(|| string_field(tx, "gasLimit")) {
            call.insert("gas".to_owned(), Value::String(gas.to_owned()));
        } else {
            call.insert("gas".to_owned(), Value::String(config.default_gas.clone()));
        }
        if let Some(data) = string_field(tx, "data").or_else(|| string_field(tx, "input")) {
            call.insert("data".to_owned(), Value::String(data.to_owned()));
        } else {
            call.insert("data".to_owned(), Value::String("0x".to_owned()));
        }
    }

    json!({
        "id": 1,
        "jsonrpc": "2.0",
        "method": "alchemy_simulateAssetChanges",
        "params": [
            Value::Object(call),
        ],
    })
}

pub(crate) fn alchemy_transport_error_message(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        "Alchemy RPC request timed out".to_owned()
    } else if error.is_connect() {
        "Alchemy RPC connection failed".to_owned()
    } else {
        "Alchemy RPC request failed".to_owned()
    }
}

pub(crate) fn alchemy_result_error(response_body: &Value) -> Option<String> {
    let error = response_body.get("result")?.get("error")?;
    match error {
        Value::Null => None,
        Value::String(message) if message.is_empty() => None,
        Value::String(message) => Some(message.clone()),
        value => Some(value.to_string()),
    }
}

pub(crate) fn alchemy_response_evidence(response_body: &Value, http_status: u16) -> Value {
    let result = response_body.get("result").and_then(Value::as_object);
    let change_count = result
        .and_then(|result| result.get("changes"))
        .and_then(Value::as_array)
        .map(Vec::len);
    let result_error =
        result
            .and_then(|result| result.get("error"))
            .is_some_and(|error| match error {
                Value::Null => false,
                Value::String(message) => !message.is_empty(),
                _ => true,
            });
    let json_rpc_error_code = response_body
        .get("error")
        .and_then(Value::as_object)
        .and_then(|error| error.get("code"))
        .filter(|code| code.is_number() || code.is_string())
        .cloned();

    json!({
        "provider": "alchemy_simulateAssetChanges",
        "httpStatus": http_status,
        "jsonRpcError": response_body.get("error").is_some(),
        "jsonRpcErrorCode": json_rpc_error_code,
        "resultError": result_error,
        "changeCount": change_count,
    })
}

pub(crate) fn apply_alchemy_asset_changes(
    report: &mut TransactionSimulationReport,
    response_body: &Value,
) -> Result<(), String> {
    const MAX_PROVIDER_CHANGES: usize = 64;

    let result = response_body
        .get("result")
        .and_then(Value::as_object)
        .ok_or_else(|| "Alchemy simulation response missing result object".to_owned())?;
    let changes = result
        .get("changes")
        .and_then(Value::as_array)
        .ok_or_else(|| "Alchemy simulation result missing changes array".to_owned())?;

    let mut transfers = Vec::new();
    let mut approvals = Vec::new();
    for change in changes.iter().take(MAX_PROVIDER_CHANGES) {
        let Some(change) = change.as_object() else {
            report.warnings.push(warning(
                WarningSeverity::Warning,
                "provider_asset_change_ignored",
                "Alchemy asset change was not an object",
            ));
            continue;
        };
        match string_field(change, "changeType")
            .unwrap_or_default()
            .to_ascii_uppercase()
            .as_str()
        {
            "TRANSFER" => {
                transfers.push(alchemy_transfer_change(change));
            }
            "APPROVAL" => {
                approvals.push(alchemy_approval_change(change));
            }
            _ => report.warnings.push(warning(
                WarningSeverity::Info,
                "provider_asset_change_ignored",
                "Alchemy asset change type is not currently normalized",
            )),
        }
    }

    if changes.len() > MAX_PROVIDER_CHANGES {
        report.warnings.push(warning(
            WarningSeverity::Warning,
            "provider_asset_changes_truncated",
            format!("Alchemy returned more than {MAX_PROVIDER_CHANGES} asset changes"),
        ));
    }
    if !changes.is_empty() && transfers.is_empty() && approvals.is_empty() {
        report.warnings.push(warning(
            WarningSeverity::Warning,
            "provider_asset_changes_unrecognized",
            "Alchemy asset changes were present but none could be normalized",
        ));
    }

    report.asset_transfers = transfers;
    report.approvals = approvals;
    Ok(())
}

fn alchemy_transfer_change(change: &Map<String, Value>) -> AssetTransfer {
    AssetTransfer {
        asset_kind: alchemy_asset_kind(change),
        contract: alchemy_address_field(change, "contractAddress"),
        from: alchemy_address_field(change, "from"),
        to: alchemy_address_field(change, "to"),
        amount: alchemy_amount(change),
        token_id: change.get("tokenId").and_then(token_amount_from_value),
    }
}

fn alchemy_approval_change(change: &Map<String, Value>) -> ApprovalChange {
    let amount = alchemy_amount(change);
    ApprovalChange {
        asset_kind: alchemy_asset_kind(change),
        contract: alchemy_address_field(change, "contractAddress"),
        owner: alchemy_address_field(change, "owner")
            .or_else(|| alchemy_address_field(change, "from")),
        spender: alchemy_address_field(change, "spender")
            .or_else(|| alchemy_address_field(change, "to")),
        operator: alchemy_address_field(change, "operator"),
        approved: change
            .get("approved")
            .and_then(Value::as_bool)
            .or_else(|| amount.as_ref().map(|amount| amount.decimal != "0")),
        amount,
    }
}

fn alchemy_asset_kind(change: &Map<String, Value>) -> String {
    string_field(change, "assetType")
        .map(|value| value.to_ascii_lowercase().replace('-', "_"))
        .unwrap_or_else(|| "unknown".to_owned())
}

fn alchemy_address_field(change: &Map<String, Value>, field: &str) -> Option<String> {
    let value = string_field(change, field)?;
    looks_like_eth_address(value).then(|| value.to_ascii_lowercase())
}

fn alchemy_amount(change: &Map<String, Value>) -> Option<TokenAmount> {
    change.get("rawAmount").and_then(token_amount_from_value)
}

fn token_amount_from_value(value: &Value) -> Option<TokenAmount> {
    let text = match value {
        Value::String(text) => text.as_str(),
        Value::Number(number) => return decimal_token_amount(&number.to_string()),
        Value::Null => return None,
        _ => return None,
    };
    if text.starts_with("0x") {
        let bytes = decode_hex_quantity(text).ok()?;
        return Some(TokenAmount {
            hex: normalize_hex_quantity(&bytes),
            decimal: hex_bytes_to_decimal(&bytes),
        });
    }
    decimal_token_amount(text)
}

fn decimal_token_amount(value: &str) -> Option<TokenAmount> {
    let decimal = value.trim();
    if decimal.is_empty() || !decimal.as_bytes().iter().all(u8::is_ascii_digit) {
        return None;
    }
    Some(TokenAmount {
        hex: decimal_digits_to_hex_quantity(decimal)?,
        decimal: normalize_decimal_digits(decimal),
    })
}

fn copy_string_field(
    source: &Map<String, Value>,
    target: &mut Map<String, Value>,
    from: &str,
    to: &str,
) {
    if let Some(value) = string_field(source, from) {
        target.insert(to.to_owned(), Value::String(value.to_owned()));
    }
}

pub(crate) fn mark_provider_failed(
    report: &mut TransactionSimulationReport,
    code: impl Into<String>,
    message: impl Into<String>,
    provider_evidence: Option<Value>,
) {
    report.status = SimulationStatus::ProviderFailed;
    if let Some(evidence) = provider_evidence {
        report.provider_evidence = Some(evidence);
    }
    report
        .warnings
        .push(warning(WarningSeverity::Error, code, message));
}
