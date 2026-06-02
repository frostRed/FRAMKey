use anyhow::{Context, Result};
use framkey_evm::{EvmAddress, EvmSignedTransaction, EvmTransaction};
use framkey_ipc::SignerSignTransactionResponse;
use serde_json::{Map, Value, json};

use crate::*;

#[derive(Debug, Clone)]
pub(crate) struct PreparedTransaction {
    pub(crate) review_request: ProviderRequest,
    pub(crate) transaction: EvmTransaction,
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
    config: &DesktopConfig,
    request: &ProviderRequest,
    wallet_address: &str,
    allow_gas_fallback: bool,
) -> Result<PreparedTransaction> {
    let tx = transaction_params_object(&request.params)?;
    let from = optional_string_field(tx, "from")?.unwrap_or_else(|| wallet_address.to_owned());
    validate_address_matches(&from, wallet_address, "transaction from")?;

    let configured_chain_id = chain_id_u64(&config.chain_id)?;
    if let Some(chain_id) = optional_string_field(tx, "chainId")? {
        if !chain_id.eq_ignore_ascii_case(&config.chain_id) {
            anyhow::bail!(
                "transaction chainId {} does not match configured {}",
                chain_id,
                config.chain_id
            );
        }
    }
    reject_access_list(tx)?;

    let to = optional_string_field(tx, "to")?;
    let value = optional_string_field(tx, "value")?.unwrap_or_else(|| "0x0".to_owned());
    let data = optional_string_field(tx, "data")?
        .or(optional_string_field(tx, "input")?)
        .unwrap_or_else(|| "0x".to_owned());

    let nonce = match optional_string_field(tx, "nonce")? {
        Some(nonce) => nonce,
        None => rpc_string_result(
            config,
            "eth_getTransactionCount",
            json!([wallet_address, "pending"]),
        )?,
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

    let mut gas_price = optional_string_field(tx, "gasPrice")?;
    let max_fee_per_gas = optional_string_field(tx, "maxFeePerGas")?;
    let max_priority_fee_per_gas = optional_string_field(tx, "maxPriorityFeePerGas")?;
    if gas_price.is_none() && max_fee_per_gas.is_none() && max_priority_fee_per_gas.is_none() {
        gas_price = Some(rpc_string_result(config, "eth_gasPrice", json!([]))?);
    }

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

pub(crate) fn reject_access_list(map: &Map<String, Value>) -> Result<()> {
    match map.get("accessList") {
        None | Some(Value::Null) => Ok(()),
        Some(Value::Array(items)) if items.is_empty() => Ok(()),
        Some(_) => anyhow::bail!("mock transaction signing does not support accessList yet"),
    }
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
