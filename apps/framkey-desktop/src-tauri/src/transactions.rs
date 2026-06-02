use anyhow::{Context, Result};
use framkey_evm::{EvmAddress, EvmSignedTransaction, EvmTransaction};
use framkey_ipc::SignerSignTransactionResponse;
use framkey_simulation::{
    TransactionReviewReport, evaluate_transaction_impact, evaluate_transaction_policy,
    evaluate_transaction_risk, evaluate_transaction_trust, known_protocol_counterparty,
};
use serde_json::{Map, Value, json};

use crate::*;

const DEFAULT_PRIORITY_FEE_PER_GAS: &str = "0x3b9aca00";
const AAVE_GET_USER_ACCOUNT_DATA_SELECTOR: &str = "0xbf92857c";
const MAX_LEGACY_GAS_PRICE_WEI: u128 = 1_000_000_000_000;
const MAX_EIP1559_MAX_FEE_PER_GAS_WEI: u128 = 1_000_000_000_000;
const MAX_EIP1559_PRIORITY_FEE_PER_GAS_WEI: u128 = 100_000_000_000;

#[derive(Debug, Clone)]
pub(crate) struct PreparedTransaction {
    pub(crate) review_request: ProviderRequest,
    pub(crate) transaction: EvmTransaction,
    pub(crate) nonce_reservation: Option<PendingNonceReservation>,
}

#[derive(Debug, Clone)]
pub(crate) struct DesktopSignedTransaction {
    pub(crate) address: String,
    pub(crate) transaction_kind: String,
    pub(crate) transaction_hash: String,
    pub(crate) raw_transaction: String,
}

impl From<EvmSignedTransaction> for DesktopSignedTransaction {
    fn from(signed: EvmSignedTransaction) -> Self {
        Self {
            address: signed.address.to_string(),
            transaction_kind: format!("{:?}", signed.kind),
            transaction_hash: signed.transaction_hash_hex(),
            raw_transaction: signed.raw_transaction_hex(),
        }
    }
}

impl From<SignerSignTransactionResponse> for DesktopSignedTransaction {
    fn from(signed: SignerSignTransactionResponse) -> Self {
        Self {
            address: signed.address,
            transaction_kind: signed.transaction_kind,
            transaction_hash: signed.transaction_hash,
            raw_transaction: signed.raw_transaction,
        }
    }
}

pub(crate) fn prepare_transaction(
    state: &AppState,
    config: &DesktopConfig,
    request: &ProviderRequest,
    wallet_address: &str,
    allow_gas_fallback: bool,
) -> Result<PreparedTransaction> {
    let tx = transaction_params_object(&request.params)?;
    let from = optional_string_field(tx, "from")?.unwrap_or_else(|| wallet_address.to_owned());
    validate_address_matches(&from, wallet_address, "transaction from")?;

    let configured_chain_id = chain_id_u64(&config.chain_id)?;
    if let Some(chain_id) = optional_string_field(tx, "chainId")?
        && !chain_id.eq_ignore_ascii_case(&config.chain_id)
    {
        anyhow::bail!(
            "transaction chainId {} does not match configured {}",
            chain_id,
            config.chain_id
        );
    }
    validate_transaction_envelope_fields(tx)?;
    reject_access_list(tx)?;

    let to = optional_string_field(tx, "to")?;
    let value = optional_string_field(tx, "value")?.unwrap_or_else(|| "0x0".to_owned());
    let data = optional_string_field(tx, "data")?
        .or(optional_string_field(tx, "input")?)
        .unwrap_or_else(|| "0x".to_owned());

    let (nonce, nonce_reservation) = match optional_string_field(tx, "nonce")? {
        Some(nonce) => (nonce, None),
        None => rpc_string_result(
            config,
            "eth_getTransactionCount",
            json!([wallet_address, "pending"]),
        )
        .and_then(|rpc_nonce| {
            state
                .reserve_transaction_nonce(&config.chain_id, wallet_address, &rpc_nonce)
                .map(|reservation| (reservation.nonce.clone(), Some(reservation)))
        })?,
    };

    let gas_limit =
        match optional_string_field(tx, "gas")?.or(optional_string_field(tx, "gasLimit")?) {
            Some(gas) => gas,
            None => {
                let estimated = rpc_string_result(
                    config,
                    "eth_estimateGas",
                    json!([transaction_call_object(&from, to.as_deref(), &value, &data)]),
                );
                match (estimated, allow_gas_fallback) {
                    (Ok(estimate), _) => bump_hex_quantity(&estimate, 120, 100)?,
                    (Err(error), true) => {
                        eprintln!("mock eth_estimateGas failed, using fallback gas: {error}");
                        default_mock_gas_limit(&data).to_owned()
                    }
                    (Err(error), false) => return Err(error),
                }
            }
        };

    let (gas_price, max_fee_per_gas, max_priority_fee_per_gas) =
        transaction_fee_fields(config, tx)?;

    let transaction = EvmTransaction {
        chain_id: configured_chain_id,
        nonce,
        gas_limit,
        to: to.clone(),
        value: value.clone(),
        data: data.clone(),
        gas_price: gas_price.clone(),
        max_fee_per_gas: max_fee_per_gas.clone(),
        max_priority_fee_per_gas: max_priority_fee_per_gas.clone(),
    };

    let mut review_tx = Map::new();
    review_tx.insert("from".to_owned(), Value::String(from));
    review_tx.insert("chainId".to_owned(), Value::String(config.chain_id.clone()));
    if let Some(to) = to {
        review_tx.insert("to".to_owned(), Value::String(to));
    }
    review_tx.insert("value".to_owned(), Value::String(value));
    review_tx.insert("data".to_owned(), Value::String(data));
    review_tx.insert("nonce".to_owned(), Value::String(transaction.nonce.clone()));
    review_tx.insert(
        "gas".to_owned(),
        Value::String(transaction.gas_limit.clone()),
    );
    if let Some(gas_price) = gas_price {
        review_tx.insert("gasPrice".to_owned(), Value::String(gas_price));
    }
    if let Some(max_fee_per_gas) = max_fee_per_gas {
        review_tx.insert("maxFeePerGas".to_owned(), Value::String(max_fee_per_gas));
    }
    if let Some(max_priority_fee_per_gas) = max_priority_fee_per_gas {
        review_tx.insert(
            "maxPriorityFeePerGas".to_owned(),
            Value::String(max_priority_fee_per_gas),
        );
    }

    Ok(PreparedTransaction {
        review_request: ProviderRequest {
            id: request.id.clone(),
            method: request.method.clone(),
            params: Value::Array(vec![Value::Object(review_tx)]),
            origin: request.origin.clone(),
        },
        transaction,
        nonce_reservation,
    })
}

pub(crate) fn transaction_params_object(params: &Value) -> Result<&Map<String, Value>> {
    params
        .as_array()
        .and_then(|items| items.first())
        .and_then(Value::as_object)
        .ok_or_else(|| {
            anyhow::anyhow!("eth_sendTransaction params must contain one transaction object")
        })
}

pub(crate) fn optional_string_field(
    map: &Map<String, Value>,
    name: &str,
) -> Result<Option<String>> {
    match map.get(name) {
        Some(Value::String(value)) => Ok(Some(value.clone())),
        Some(Value::Null) | None => Ok(None),
        Some(_) => anyhow::bail!("transaction field {name} must be a string"),
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TransactionTypeHint {
    Legacy,
    Eip1559,
}

pub(crate) fn validate_transaction_envelope_fields(map: &Map<String, Value>) -> Result<()> {
    let _ = transaction_type_hint(map)?;
    for field in [
        "maxFeePerBlobGas",
        "blobVersionedHashes",
        "blobs",
        "commitments",
        "proofs",
        "sidecars",
    ] {
        reject_non_null_field(map, field, "blob transaction fields are not supported")?;
    }
    reject_non_null_field(
        map,
        "authorizationList",
        "EIP-7702 authorizationList transactions are not supported",
    )
}

pub(crate) fn transaction_type_hint(
    map: &Map<String, Value>,
) -> Result<Option<TransactionTypeHint>> {
    let Some(value) = optional_string_field(map, "type")? else {
        return Ok(None);
    };
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "0" | "0x0" | "0x00" | "legacy" => Ok(Some(TransactionTypeHint::Legacy)),
        "2" | "0x2" | "0x02" | "eip1559" | "eip-1559" => Ok(Some(TransactionTypeHint::Eip1559)),
        "1" | "0x1" | "0x01" => {
            anyhow::bail!("transaction type 1 access-list envelopes are not supported")
        }
        "3" | "0x3" | "0x03" => {
            anyhow::bail!("transaction type 3 blob envelopes are not supported")
        }
        _ => anyhow::bail!("transaction type {value} is not supported"),
    }
}

fn reject_non_null_field(map: &Map<String, Value>, field: &str, message: &str) -> Result<()> {
    match map.get(field) {
        None | Some(Value::Null) => Ok(()),
        Some(_) => anyhow::bail!("transaction signing does not support {field}: {message}"),
    }
}

pub(crate) fn reject_access_list(map: &Map<String, Value>) -> Result<()> {
    match map.get("accessList") {
        None | Some(Value::Null) => Ok(()),
        Some(Value::Array(items)) if items.is_empty() => Ok(()),
        Some(_) => anyhow::bail!("transaction signing does not support non-empty accessList yet"),
    }
}

pub(crate) fn transaction_fee_fields(
    config: &DesktopConfig,
    tx: &Map<String, Value>,
) -> Result<(Option<String>, Option<String>, Option<String>)> {
    let tx_type = transaction_type_hint(tx)?;
    let gas_price = optional_string_field(tx, "gasPrice")?;
    let mut max_fee_per_gas = optional_string_field(tx, "maxFeePerGas")?;
    let mut max_priority_fee_per_gas = optional_string_field(tx, "maxPriorityFeePerGas")?;
    if tx_type == Some(TransactionTypeHint::Legacy)
        && (max_fee_per_gas.is_some() || max_priority_fee_per_gas.is_some())
    {
        anyhow::bail!("legacy transaction type cannot use EIP-1559 fee fields");
    }
    if tx_type == Some(TransactionTypeHint::Legacy) && gas_price.is_none() {
        anyhow::bail!("legacy transaction type requires gasPrice");
    }
    if tx_type == Some(TransactionTypeHint::Eip1559) && gas_price.is_some() {
        anyhow::bail!("EIP-1559 transaction type cannot use gasPrice");
    }
    if gas_price.is_some() && (max_fee_per_gas.is_some() || max_priority_fee_per_gas.is_some()) {
        anyhow::bail!("transaction fee fields cannot mix gasPrice with EIP-1559 fee fields");
    }
    if gas_price.is_some() {
        validate_legacy_fee_cap(gas_price.as_deref().expect("checked above"))?;
        return Ok((gas_price, None, None));
    }
    if max_fee_per_gas.is_some() && max_priority_fee_per_gas.is_some() {
        validate_eip1559_fee_caps(
            max_fee_per_gas.as_deref().expect("checked above"),
            max_priority_fee_per_gas.as_deref().expect("checked above"),
        )?;
        return Ok((None, max_fee_per_gas, max_priority_fee_per_gas));
    }

    match eip1559_fee_suggestion(config) {
        Ok(suggestion) => {
            if max_priority_fee_per_gas.is_none() {
                max_priority_fee_per_gas = Some(suggestion.max_priority_fee_per_gas);
            }
            if max_fee_per_gas.is_none() {
                let priority = max_priority_fee_per_gas
                    .as_deref()
                    .expect("priority fee set above");
                max_fee_per_gas = Some(max_fee_from_base_fee(
                    &suggestion.next_base_fee_per_gas,
                    priority,
                )?);
            }
            validate_eip1559_fee_caps(
                max_fee_per_gas.as_deref().expect("set above"),
                max_priority_fee_per_gas.as_deref().expect("set above"),
            )?;
            Ok((None, max_fee_per_gas, max_priority_fee_per_gas))
        }
        Err(error) if max_fee_per_gas.is_none() && max_priority_fee_per_gas.is_none() => {
            eprintln!("eth_feeHistory unavailable, falling back to legacy gasPrice: {error}");
            let gas_price = rpc_string_result(config, "eth_gasPrice", json!([]))?;
            validate_legacy_fee_cap(&gas_price)?;
            Ok((Some(gas_price), None, None))
        }
        Err(error) => Err(error).context("failed to complete EIP-1559 fee fields"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Eip1559FeeSuggestion {
    pub(crate) next_base_fee_per_gas: String,
    pub(crate) max_priority_fee_per_gas: String,
}

pub(crate) fn eip1559_fee_suggestion(config: &DesktopConfig) -> Result<Eip1559FeeSuggestion> {
    let result = rpc_result(config, "eth_feeHistory", json!(["0x1", "pending", [50]]))?;
    eip1559_fee_suggestion_from_fee_history(&result)
}

pub(crate) fn eip1559_fee_suggestion_from_fee_history(
    result: &Value,
) -> Result<Eip1559FeeSuggestion> {
    let base_fees = result
        .get("baseFeePerGas")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow::anyhow!("eth_feeHistory missing baseFeePerGas"))?;
    let next_base_fee_per_gas = base_fees
        .last()
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("eth_feeHistory missing next base fee"))?;
    hex_quantity_to_u128(next_base_fee_per_gas)
        .context("eth_feeHistory next base fee is malformed")?;

    let max_priority_fee_per_gas = result
        .get("reward")
        .and_then(Value::as_array)
        .and_then(|blocks| blocks.last())
        .and_then(Value::as_array)
        .and_then(|rewards| rewards.first())
        .and_then(Value::as_str)
        .filter(|reward| hex_quantity_to_u128(reward).is_ok_and(|value| value > 0))
        .unwrap_or(DEFAULT_PRIORITY_FEE_PER_GAS);

    Ok(Eip1559FeeSuggestion {
        next_base_fee_per_gas: next_base_fee_per_gas.to_owned(),
        max_priority_fee_per_gas: max_priority_fee_per_gas.to_owned(),
    })
}

pub(crate) fn max_fee_from_base_fee(base_fee: &str, priority_fee: &str) -> Result<String> {
    let base_fee = hex_quantity_to_u128(base_fee)?;
    let priority_fee = hex_quantity_to_u128(priority_fee)?;
    let max_fee = base_fee
        .checked_mul(2)
        .and_then(|base| base.checked_add(priority_fee))
        .ok_or_else(|| anyhow::anyhow!("EIP-1559 fee calculation overflowed"))?;
    Ok(format!("0x{max_fee:x}"))
}

fn validate_legacy_fee_cap(gas_price: &str) -> Result<()> {
    let gas_price = hex_quantity_to_u128(gas_price)?;
    if gas_price > MAX_LEGACY_GAS_PRICE_WEI {
        anyhow::bail!("transaction gasPrice exceeds FRAMKey safety cap");
    }
    Ok(())
}

fn validate_eip1559_fee_caps(max_fee_per_gas: &str, max_priority_fee_per_gas: &str) -> Result<()> {
    let max_fee = hex_quantity_to_u128(max_fee_per_gas)?;
    let priority_fee = hex_quantity_to_u128(max_priority_fee_per_gas)?;
    if max_fee > MAX_EIP1559_MAX_FEE_PER_GAS_WEI {
        anyhow::bail!("transaction maxFeePerGas exceeds FRAMKey safety cap");
    }
    if priority_fee > MAX_EIP1559_PRIORITY_FEE_PER_GAS_WEI {
        anyhow::bail!("transaction maxPriorityFeePerGas exceeds FRAMKey safety cap");
    }
    if priority_fee > max_fee {
        anyhow::bail!("transaction maxPriorityFeePerGas cannot exceed maxFeePerGas");
    }
    Ok(())
}

pub(crate) fn enrich_aave_account_evidence(
    config: &DesktopConfig,
    review: &mut TransactionReviewReport,
) {
    let Some(target) = aave_account_evidence_target(&review.simulation) else {
        return;
    };
    let evidence = if !target.known_pool {
        json!({
            "protocol": "Aave",
            "source": "eth_call:getUserAccountData",
            "status": "unrecognized_pool",
            "chainId": review.simulation.chain_id,
            "pool": target.pool,
            "account": target.account,
        })
    } else if config.rpc.is_none() {
        json!({
            "protocol": "Aave",
            "source": "eth_call:getUserAccountData",
            "status": "rpc_missing",
            "chainId": review.simulation.chain_id,
            "pool": target.pool,
            "account": target.account,
        })
    } else {
        match fetch_aave_user_account_data(config, &target.pool, &target.account) {
            Ok(mut evidence) => {
                evidence.insert("chainId".to_owned(), json!(review.simulation.chain_id));
                Value::Object(evidence)
            }
            Err(error) => json!({
                "protocol": "Aave",
                "source": "eth_call:getUserAccountData",
                "status": "rpc_error",
                "chainId": review.simulation.chain_id,
                "pool": target.pool,
                "account": target.account,
                "error": truncate_for_event(&error.to_string(), 180),
            }),
        }
    };

    let mut protocol_evidence = review
        .simulation
        .protocol_evidence
        .take()
        .and_then(|value| match value {
            Value::Object(map) => Some(map),
            _ => None,
        })
        .unwrap_or_default();
    protocol_evidence.insert("aave".to_owned(), evidence);
    review.simulation.protocol_evidence = Some(Value::Object(protocol_evidence));
    refresh_transaction_review_summaries(review);
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AaveAccountEvidenceTarget {
    pool: String,
    account: String,
    known_pool: bool,
}

fn aave_account_evidence_target(
    report: &framkey_simulation::TransactionSimulationReport,
) -> Option<AaveAccountEvidenceTarget> {
    let call = report.decoded_call.as_ref()?;
    if call.standard != "aave_v3_pool" || !aave_call_needs_account_evidence(call) {
        return None;
    }
    let pool = report
        .transaction
        .to
        .as_deref()
        .or(call.contract.as_deref())?
        .parse::<EvmAddress>()
        .ok()?
        .to_string();
    let account = aave_account_for_call(report, call)?
        .parse::<EvmAddress>()
        .ok()?
        .to_string();
    let known_pool = known_protocol_counterparty(&report.chain_id, &pool)
        .is_some_and(|known| known.protocol == "Aave");
    Some(AaveAccountEvidenceTarget {
        pool,
        account,
        known_pool,
    })
}

fn aave_call_needs_account_evidence(call: &framkey_simulation::DecodedCall) -> bool {
    match call.function.as_str() {
        "borrow(address,uint256,uint256,uint16,address)" => true,
        "withdraw(address,uint256,address)" => true,
        "setUserUseReserveAsCollateral(address,bool)" => {
            decoded_transaction_arg(call, "useAsCollateral") == Some("false")
        }
        _ => false,
    }
}

fn aave_account_for_call<'a>(
    report: &'a framkey_simulation::TransactionSimulationReport,
    call: &'a framkey_simulation::DecodedCall,
) -> Option<&'a str> {
    match call.function.as_str() {
        "borrow(address,uint256,uint256,uint16,address)" => {
            decoded_transaction_arg(call, "onBehalfOf")
        }
        "withdraw(address,uint256,address)" | "setUserUseReserveAsCollateral(address,bool)" => {
            report.transaction.from.as_deref()
        }
        _ => None,
    }
}

fn decoded_transaction_arg<'a>(
    call: &'a framkey_simulation::DecodedCall,
    name: &str,
) -> Option<&'a str> {
    call.arguments
        .iter()
        .find(|argument| argument.name == name)
        .map(|argument| argument.value.as_str())
}

fn fetch_aave_user_account_data(
    config: &DesktopConfig,
    pool: &str,
    account: &str,
) -> Result<Map<String, Value>> {
    let result = rpc_string_result(
        config,
        "eth_call",
        json!([
            {
                "to": pool,
                "data": aave_user_account_data_call_data(account)?
            },
            "latest"
        ]),
    )?;
    let values = decode_aave_user_account_data_result(&result)?;
    let mut evidence = Map::new();
    evidence.insert("protocol".to_owned(), json!("Aave"));
    evidence.insert("source".to_owned(), json!("eth_call:getUserAccountData"));
    evidence.insert("status".to_owned(), json!("ok"));
    evidence.insert("pool".to_owned(), json!(pool));
    evidence.insert("account".to_owned(), json!(account));
    evidence.insert(
        "totalCollateralBase".to_owned(),
        json!(values.total_collateral_base),
    );
    evidence.insert("totalDebtBase".to_owned(), json!(values.total_debt_base));
    evidence.insert(
        "availableBorrowsBase".to_owned(),
        json!(values.available_borrows_base),
    );
    evidence.insert(
        "currentLiquidationThreshold".to_owned(),
        json!(values.current_liquidation_threshold),
    );
    evidence.insert("ltv".to_owned(), json!(values.ltv));
    evidence.insert("healthFactor".to_owned(), json!(values.health_factor));
    Ok(evidence)
}

fn aave_user_account_data_call_data(account: &str) -> Result<String> {
    let account = account
        .parse::<EvmAddress>()
        .map_err(|_| anyhow::anyhow!("Aave account is not a valid EVM address"))?
        .to_string();
    let account_hex = account
        .strip_prefix("0x")
        .ok_or_else(|| anyhow::anyhow!("normalized Aave account address is missing 0x prefix"))?;
    Ok(format!(
        "{AAVE_GET_USER_ACCOUNT_DATA_SELECTOR}{account_hex:0>64}"
    ))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct AaveUserAccountData {
    total_collateral_base: String,
    total_debt_base: String,
    available_borrows_base: String,
    current_liquidation_threshold: String,
    ltv: String,
    health_factor: String,
}

fn decode_aave_user_account_data_result(result: &str) -> Result<AaveUserAccountData> {
    let hex = result
        .strip_prefix("0x")
        .ok_or_else(|| anyhow::anyhow!("Aave account data result must be 0x-prefixed"))?;
    if hex.len() != 64 * 6 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        anyhow::bail!("Aave account data result must contain six uint256 words");
    }
    Ok(AaveUserAccountData {
        total_collateral_base: hex_word_to_decimal(&hex[0..64])?,
        total_debt_base: hex_word_to_decimal(&hex[64..128])?,
        available_borrows_base: hex_word_to_decimal(&hex[128..192])?,
        current_liquidation_threshold: hex_word_to_decimal(&hex[192..256])?,
        ltv: hex_word_to_decimal(&hex[256..320])?,
        health_factor: hex_word_to_decimal(&hex[320..384])?,
    })
}

fn hex_word_to_decimal(hex: &str) -> Result<String> {
    if hex.is_empty() || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        anyhow::bail!("hex word must contain hex digits");
    }
    let mut digits = vec![0_u8];
    for byte in hex.bytes() {
        let nibble = hex_nibble_value(byte)
            .ok_or_else(|| anyhow::anyhow!("hex word contains non-hex digit"))?;
        decimal_digits_mul_add(&mut digits, 16, nibble);
    }
    let value = digits
        .into_iter()
        .skip_while(|digit| *digit == 0)
        .map(|digit| char::from(b'0' + digit))
        .collect::<String>();
    Ok(if value.is_empty() {
        "0".to_owned()
    } else {
        value
    })
}

fn decimal_digits_mul_add(digits: &mut Vec<u8>, multiplier: u8, addend: u8) {
    let mut carry = u16::from(addend);
    for digit in digits.iter_mut().rev() {
        let value = u16::from(*digit) * u16::from(multiplier) + carry;
        *digit = u8::try_from(value % 10).expect("decimal digit fits");
        carry = value / 10;
    }
    while carry > 0 {
        digits.insert(0, u8::try_from(carry % 10).expect("decimal digit fits"));
        carry /= 10;
    }
}

fn hex_nibble_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn refresh_transaction_review_summaries(review: &mut TransactionReviewReport) {
    review.policy = evaluate_transaction_policy(&review.simulation);
    review.risk = evaluate_transaction_risk(&review.simulation, &review.policy);
    review.impact = evaluate_transaction_impact(&review.simulation);
    review.trust = evaluate_transaction_trust(&review.simulation);
}

pub(crate) fn validate_address_matches(actual: &str, expected: &str, label: &str) -> Result<()> {
    let actual: EvmAddress = actual
        .parse()
        .with_context(|| format!("{label} is not a valid EVM address"))?;
    let expected: EvmAddress = expected
        .parse()
        .with_context(|| format!("configured wallet address is not valid: {expected}"))?;
    if actual != expected {
        anyhow::bail!("{label} {actual} does not match wallet {expected}");
    }
    Ok(())
}

pub(crate) fn transaction_call_object(
    from: &str,
    to: Option<&str>,
    value: &str,
    data: &str,
) -> Value {
    let mut tx = Map::new();
    tx.insert("from".to_owned(), Value::String(from.to_owned()));
    if let Some(to) = to {
        tx.insert("to".to_owned(), Value::String(to.to_owned()));
    }
    tx.insert("value".to_owned(), Value::String(value.to_owned()));
    tx.insert("data".to_owned(), Value::String(data.to_owned()));
    Value::Object(tx)
}

pub(crate) fn default_mock_gas_limit(data: &str) -> &'static str {
    if data == "0x" || data == "0x0" {
        DEFAULT_MOCK_NATIVE_TRANSFER_GAS
    } else {
        DEFAULT_MOCK_CONTRACT_CALL_GAS
    }
}

pub(crate) fn rpc_string_result(
    config: &DesktopConfig,
    method: &str,
    params: Value,
) -> Result<String> {
    let result = rpc_result(config, method, params)?;
    result
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| anyhow::anyhow!("{method} returned a non-string result"))
}

pub(crate) fn bump_hex_quantity(
    quantity: &str,
    numerator: u128,
    denominator: u128,
) -> Result<String> {
    if denominator == 0 {
        anyhow::bail!("quantity bump denominator must not be zero");
    }
    let value = hex_quantity_to_u128(quantity)?;
    let bumped = value.saturating_mul(numerator).div_ceil(denominator);
    Ok(format!("0x{bumped:x}"))
}

pub(crate) fn hex_quantity_to_u128(quantity: &str) -> Result<u128> {
    let hex = quantity
        .strip_prefix("0x")
        .ok_or_else(|| anyhow::anyhow!("quantity must be 0x-prefixed"))?;
    if hex.is_empty() || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        anyhow::bail!("quantity must contain hex digits");
    }
    u128::from_str_radix(hex, 16).with_context(|| format!("failed to parse quantity {quantity}"))
}

pub(crate) fn native_amount_decimal_to_wei_hex(amount: &str) -> Result<String> {
    pub(crate) const WEI_DECIMALS: usize = 18;

    token_amount_decimal_to_raw_hex(amount, WEI_DECIMALS, "native transfer")
}

pub(crate) fn token_amount_decimal_to_raw_hex(
    amount: &str,
    decimals: usize,
    label: &str,
) -> Result<String> {
    let digits = token_amount_decimal_to_raw_decimal_digits(amount, decimals, label)?;
    raw_decimal_digits_to_hex(&digits).with_context(|| format!("{label} amount is too large"))
}

pub(crate) fn token_amount_decimal_to_raw_decimal_digits(
    amount: &str,
    decimals: usize,
    label: &str,
) -> Result<String> {
    let amount = amount.trim();
    if amount.is_empty() || amount.chars().any(char::is_control) {
        anyhow::bail!("{label} amount is malformed");
    }
    if amount.starts_with(['+', '-'])
        || amount.contains(['e', 'E', '_', ','])
        || amount.matches('.').count() > 1
    {
        anyhow::bail!("{label} amount must be a plain decimal value");
    }

    let (whole, fractional) = amount.split_once('.').unwrap_or((amount, ""));
    if whole.is_empty() && fractional.is_empty() {
        anyhow::bail!("{label} amount is malformed");
    }
    if !whole.bytes().all(|byte| byte.is_ascii_digit())
        || !fractional.bytes().all(|byte| byte.is_ascii_digit())
    {
        anyhow::bail!("{label} amount must contain only decimal digits");
    }
    if fractional.len() > decimals {
        anyhow::bail!("{label} amount has more than {decimals} decimal places");
    }

    let whole = whole.trim_start_matches('0');
    let mut digits = String::with_capacity(whole.len() + decimals);
    digits.push_str(whole);
    digits.push_str(fractional);
    digits.extend(std::iter::repeat_n('0', decimals - fractional.len()));
    let digits = digits.trim_start_matches('0').to_owned();
    if digits.is_empty() {
        anyhow::bail!("{label} amount must be greater than zero");
    }
    Ok(digits)
}

pub(crate) fn raw_decimal_digits_to_hex(digits: &str) -> Result<String> {
    pub(crate) const MAX_U256_DECIMAL: &str =
        "115792089237316195423570985008687907853269984665640564039457584007913129639935";

    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        anyhow::bail!("raw token amount must contain decimal digits");
    }
    let digits = digits.trim_start_matches('0');
    if digits.is_empty() {
        anyhow::bail!("raw token amount must be greater than zero");
    }
    if digits.len() > MAX_U256_DECIMAL.len()
        || (digits.len() == MAX_U256_DECIMAL.len() && digits > MAX_U256_DECIMAL)
    {
        anyhow::bail!("raw token amount exceeds uint256");
    }

    let mut decimal = digits.as_bytes().to_vec();
    let mut hex = String::new();
    while !(decimal.len() == 1 && decimal[0] == b'0') {
        let mut quotient = Vec::with_capacity(decimal.len());
        let mut remainder = 0_u32;
        for digit in &decimal {
            let value = remainder * 10 + u32::from(digit - b'0');
            let q = value / 16;
            remainder = value % 16;
            if !quotient.is_empty() || q != 0 {
                quotient.push(b'0' + u8::try_from(q).expect("quotient digit fits"));
            }
        }
        hex.push(char::from_digit(remainder, 16).expect("remainder is hex digit"));
        decimal = if quotient.is_empty() {
            vec![b'0']
        } else {
            quotient
        };
    }

    let hex = hex.chars().rev().collect::<String>();
    Ok(format!("0x{hex}"))
}

pub(crate) fn erc20_transfer_calldata(to: &str, raw_amount_hex: &str) -> Result<String> {
    let to = to
        .parse::<EvmAddress>()
        .map_err(|_| anyhow::anyhow!("token transfer recipient is not a valid EVM address"))?
        .to_string();
    let address_hex = to
        .strip_prefix("0x")
        .ok_or_else(|| anyhow::anyhow!("normalized address is missing 0x prefix"))?;
    let amount_hex = raw_amount_hex
        .strip_prefix("0x")
        .ok_or_else(|| anyhow::anyhow!("token transfer raw amount is missing 0x prefix"))?;
    if amount_hex.is_empty()
        || amount_hex.len() > 64
        || !amount_hex.bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        anyhow::bail!("token transfer raw amount does not fit uint256");
    }
    Ok(format!("0xa9059cbb{address_hex:0>64}{amount_hex:0>64}"))
}
