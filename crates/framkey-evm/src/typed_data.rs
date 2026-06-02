use std::collections::BTreeSet;

use framkey_core::{FramkeyError, Result};
use framkey_crypto::SecretBytes;
use k256::ecdsa::{Signature, VerifyingKey};
use serde_json::{Map, Value};

use crate::{
    EvmAddress, EvmTypedDataSignature,
    encoding::{decode_variable_hex, keccak256, minimal_integer_bytes, parse_data_bytes},
    keys::{address_from_verifying_key, signing_key_from_secret},
    signature::{eth_signature_bytes, recovery_id_from_eth_v},
};

pub fn sign_typed_data_v4(
    secret: &SecretBytes<32>,
    typed_data: &Value,
) -> Result<EvmTypedDataSignature> {
    let digest = typed_data_v4_hash(typed_data)?;
    let signing_key = signing_key_from_secret(secret.expose())?;
    let (signature, recovery_id) = signing_key
        .sign_prehash_recoverable(&digest)
        .map_err(|_| FramkeyError::invalid_data("EVM typed-data signing failed"))?;

    Ok(EvmTypedDataSignature {
        address: address_from_verifying_key(signing_key.verifying_key()),
        typed_data_hash: digest,
        signature: eth_signature_bytes(signature, recovery_id),
    })
}

pub fn recover_typed_data_signer(typed_data: &Value, signature: &[u8; 65]) -> Result<EvmAddress> {
    let digest = typed_data_v4_hash(typed_data)?;
    let signature_core = Signature::from_slice(&signature[..64])
        .map_err(|_| FramkeyError::invalid_data("invalid EVM signature bytes"))?;
    let recovery_id = recovery_id_from_eth_v(signature[64])?;
    let verifying_key =
        VerifyingKey::recover_from_prehash(&digest, &signature_core, recovery_id)
            .map_err(|_| FramkeyError::invalid_data("EVM typed-data signature recovery failed"))?;

    Ok(address_from_verifying_key(&verifying_key))
}

pub fn typed_data_v4_hash(typed_data: &Value) -> Result<[u8; 32]> {
    let typed_data = ParsedTypedData::parse(typed_data)?;
    let domain_separator = hash_typed_struct(
        "EIP712Domain",
        typed_data.domain,
        typed_data.types,
        "typed-data domain",
    )?;
    let message_hash = hash_typed_struct(
        typed_data.primary_type,
        typed_data.message,
        typed_data.types,
        "typed-data message",
    )?;

    let mut payload = Vec::with_capacity(66);
    payload.extend_from_slice(b"\x19\x01");
    payload.extend_from_slice(&domain_separator);
    payload.extend_from_slice(&message_hash);
    Ok(keccak256(&payload))
}

struct ParsedTypedData<'a> {
    types: &'a Map<String, Value>,
    primary_type: &'a str,
    domain: &'a Map<String, Value>,
    message: &'a Map<String, Value>,
}

impl<'a> ParsedTypedData<'a> {
    fn parse(value: &'a Value) -> Result<Self> {
        let object = value
            .as_object()
            .ok_or_else(|| FramkeyError::invalid_data("typed data must be a JSON object"))?;
        let types = object
            .get("types")
            .and_then(Value::as_object)
            .ok_or_else(|| FramkeyError::invalid_data("typed data types must be an object"))?;
        let primary_type = object
            .get("primaryType")
            .and_then(Value::as_str)
            .ok_or_else(|| FramkeyError::invalid_data("typed data primaryType must be a string"))?;
        validate_type_name(primary_type, "typed data primaryType")?;
        if !types.contains_key(primary_type) {
            return Err(FramkeyError::invalid_data(format!(
                "typed data primaryType {primary_type} is not declared"
            )));
        }
        if !types.contains_key("EIP712Domain") {
            return Err(FramkeyError::invalid_data(
                "typed data EIP712Domain type is not declared",
            ));
        }
        let domain = object
            .get("domain")
            .and_then(Value::as_object)
            .ok_or_else(|| FramkeyError::invalid_data("typed data domain must be an object"))?;
        let message = object
            .get("message")
            .and_then(Value::as_object)
            .ok_or_else(|| FramkeyError::invalid_data("typed data message must be an object"))?;
        Ok(Self {
            types,
            primary_type,
            domain,
            message,
        })
    }
}

#[derive(Debug, Clone)]
struct TypedField {
    name: String,
    type_name: String,
}

fn hash_typed_struct(
    type_name: &str,
    value: &Map<String, Value>,
    types: &Map<String, Value>,
    label: &str,
) -> Result<[u8; 32]> {
    let type_hash = keccak256(encode_type(type_name, types)?.as_bytes());
    let fields = typed_fields(type_name, types)?;
    let mut encoded = Vec::with_capacity(32 * (fields.len() + 1));
    encoded.extend_from_slice(&type_hash);
    for field in fields {
        let field_value = value.get(&field.name).ok_or_else(|| {
            FramkeyError::invalid_data(format!("{label} missing field {}", field.name))
        })?;
        encoded.extend_from_slice(&encode_typed_value(&field.type_name, field_value, types)?);
    }
    Ok(keccak256(&encoded))
}

fn encode_type(type_name: &str, types: &Map<String, Value>) -> Result<String> {
    validate_type_name(type_name, "typed-data type")?;
    let mut dependencies = BTreeSet::new();
    collect_type_dependencies(type_name, types, &mut dependencies)?;

    let mut output = encode_type_single(type_name, types)?;
    for dependency in dependencies {
        output.push_str(&encode_type_single(&dependency, types)?);
    }
    Ok(output)
}

fn collect_type_dependencies(
    type_name: &str,
    types: &Map<String, Value>,
    dependencies: &mut BTreeSet<String>,
) -> Result<()> {
    for field in typed_fields(type_name, types)? {
        let base_type = base_eip712_type(&field.type_name)?;
        if base_type == type_name || !types.contains_key(base_type) {
            continue;
        }
        if dependencies.insert(base_type.to_owned()) {
            collect_type_dependencies(base_type, types, dependencies)?;
        }
    }
    Ok(())
}

fn encode_type_single(type_name: &str, types: &Map<String, Value>) -> Result<String> {
    let fields = typed_fields(type_name, types)?;
    let encoded_fields = fields
        .iter()
        .map(|field| format!("{} {}", field.type_name, field.name))
        .collect::<Vec<_>>()
        .join(",");
    Ok(format!("{type_name}({encoded_fields})"))
}

fn typed_fields(type_name: &str, types: &Map<String, Value>) -> Result<Vec<TypedField>> {
    validate_type_name(type_name, "typed-data type")?;
    let fields = types
        .get(type_name)
        .and_then(Value::as_array)
        .ok_or_else(|| {
            FramkeyError::invalid_data(format!("typed data type {type_name} must be an array"))
        })?;
    let mut parsed = Vec::with_capacity(fields.len());
    for field in fields {
        let field = field.as_object().ok_or_else(|| {
            FramkeyError::invalid_data(format!("typed data type {type_name} has malformed field"))
        })?;
        let name = field.get("name").and_then(Value::as_str).ok_or_else(|| {
            FramkeyError::invalid_data(format!("typed data type {type_name} field is missing name"))
        })?;
        validate_field_name(name, "typed-data field")?;
        let field_type = field.get("type").and_then(Value::as_str).ok_or_else(|| {
            FramkeyError::invalid_data(format!("typed data type {type_name} field is missing type"))
        })?;
        validate_field_type(field_type)?;
        parsed.push(TypedField {
            name: name.to_owned(),
            type_name: field_type.to_owned(),
        });
    }
    Ok(parsed)
}

fn encode_typed_value(
    type_name: &str,
    value: &Value,
    types: &Map<String, Value>,
) -> Result<[u8; 32]> {
    if let Some(element_type) = array_element_type(type_name)? {
        let items = value.as_array().ok_or_else(|| {
            FramkeyError::invalid_data(format!("typed data field {type_name} must be an array"))
        })?;
        let mut encoded = Vec::with_capacity(items.len() * 32);
        for item in items {
            encoded.extend_from_slice(&encode_typed_value(element_type, item, types)?);
        }
        return Ok(keccak256(&encoded));
    }

    if types.contains_key(type_name) {
        let object = value.as_object().ok_or_else(|| {
            FramkeyError::invalid_data(format!(
                "typed data field {type_name} must be a struct object"
            ))
        })?;
        return hash_typed_struct(type_name, object, types, "typed-data struct");
    }

    match type_name {
        "address" => address_word(value),
        "bool" => bool_word(value),
        "string" => value
            .as_str()
            .map(|text| keccak256(text.as_bytes()))
            .ok_or_else(|| FramkeyError::invalid_data("typed data string field must be a string")),
        "bytes" => {
            let text = value.as_str().ok_or_else(|| {
                FramkeyError::invalid_data("typed data bytes field must be a 0x string")
            })?;
            Ok(keccak256(&parse_data_bytes(text, "typed data bytes")?))
        }
        _ => {
            if let Some(bits) = uint_bits(type_name)? {
                return uint_word(value, bits, type_name);
            }
            if let Some(len) = fixed_bytes_len(type_name)? {
                return fixed_bytes_word(value, len);
            }
            Err(FramkeyError::unsupported(format!(
                "typed data field type {type_name} is not supported"
            )))
        }
    }
}

fn array_element_type(type_name: &str) -> Result<Option<&str>> {
    if let Some(element) = type_name.strip_suffix("[]") {
        if element.is_empty() {
            return Err(FramkeyError::invalid_data("typed data array type is empty"));
        }
        return Ok(Some(element));
    }
    if type_name.contains('[') || type_name.contains(']') {
        return Err(FramkeyError::unsupported(
            "typed data fixed-size arrays are not supported yet",
        ));
    }
    Ok(None)
}

fn base_eip712_type(type_name: &str) -> Result<&str> {
    Ok(array_element_type(type_name)?.unwrap_or(type_name))
}

fn address_word(value: &Value) -> Result<[u8; 32]> {
    let address: EvmAddress = value
        .as_str()
        .ok_or_else(|| FramkeyError::invalid_data("typed data address field must be a string"))?
        .parse()?;
    let mut word = [0_u8; 32];
    word[12..32].copy_from_slice(&address.0);
    Ok(word)
}

fn bool_word(value: &Value) -> Result<[u8; 32]> {
    let value = value
        .as_bool()
        .ok_or_else(|| FramkeyError::invalid_data("typed data bool field must be a boolean"))?;
    let mut word = [0_u8; 32];
    word[31] = u8::from(value);
    Ok(word)
}

fn uint_bits(type_name: &str) -> Result<Option<usize>> {
    let Some(bits) = type_name.strip_prefix("uint") else {
        if type_name.starts_with("int") {
            return Err(FramkeyError::unsupported(
                "typed data signed integer fields are not supported yet",
            ));
        }
        return Ok(None);
    };
    if bits.is_empty() {
        return Ok(Some(256));
    }
    let bits = bits.parse::<usize>().map_err(|_| {
        FramkeyError::invalid_data(format!("typed data integer type {type_name} is malformed"))
    })?;
    if bits == 0 || bits > 256 || bits % 8 != 0 {
        return Err(FramkeyError::invalid_data(format!(
            "typed data integer type {type_name} is unsupported"
        )));
    }
    Ok(Some(bits))
}

fn uint_word(value: &Value, bits: usize, type_name: &str) -> Result<[u8; 32]> {
    match value {
        Value::String(text) if text.starts_with("0x") || text.starts_with("0X") => {
            hex_uint_word(text, bits, type_name)
        }
        Value::String(text) => decimal_uint_word(text, bits, type_name),
        Value::Number(number) => {
            let Some(value) = number.as_u64() else {
                return Err(FramkeyError::invalid_data(format!(
                    "typed data {type_name} number must be an unsigned integer"
                )));
            };
            let mut word = [0_u8; 32];
            word[24..32].copy_from_slice(&value.to_be_bytes());
            enforce_uint_bits(&word, bits, type_name)?;
            Ok(word)
        }
        _ => Err(FramkeyError::invalid_data(format!(
            "typed data {type_name} field must be a string or number"
        ))),
    }
}

fn hex_uint_word(input: &str, bits: usize, label: &str) -> Result<[u8; 32]> {
    let hex = input
        .strip_prefix("0x")
        .or_else(|| input.strip_prefix("0X"))
        .ok_or_else(|| FramkeyError::invalid_data(format!("{label} must be 0x-prefixed")))?;
    if hex.is_empty() || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(FramkeyError::invalid_data(format!(
            "{label} must contain hex digits"
        )));
    }
    let padded = if hex.len() % 2 == 0 {
        hex.to_owned()
    } else {
        format!("0{hex}")
    };
    let bytes = minimal_integer_bytes(&decode_variable_hex(&padded, label)?);
    uint_word_from_minimal_bytes(&bytes, bits, label)
}

fn decimal_uint_word(input: &str, bits: usize, label: &str) -> Result<[u8; 32]> {
    if input.is_empty() || !input.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(FramkeyError::invalid_data(format!(
            "{label} decimal value is malformed"
        )));
    }
    let mut word = [0_u8; 32];
    for digit in input.bytes().map(|byte| byte - b'0') {
        multiply_word_small(&mut word, 10, label)?;
        add_word_small(&mut word, digit, label)?;
    }
    enforce_uint_bits(&word, bits, label)?;
    Ok(word)
}

fn uint_word_from_minimal_bytes(bytes: &[u8], bits: usize, label: &str) -> Result<[u8; 32]> {
    if bytes.len() > 32 {
        return Err(FramkeyError::invalid_data(format!(
            "{label} exceeds 256 bits"
        )));
    }
    let mut word = [0_u8; 32];
    word[32 - bytes.len()..32].copy_from_slice(bytes);
    enforce_uint_bits(&word, bits, label)?;
    Ok(word)
}

fn enforce_uint_bits(word: &[u8; 32], bits: usize, label: &str) -> Result<()> {
    let allowed_bytes = bits / 8;
    if word[..32 - allowed_bytes].iter().any(|byte| *byte != 0) {
        return Err(FramkeyError::invalid_data(format!(
            "{label} exceeds {bits} bits"
        )));
    }
    Ok(())
}

fn multiply_word_small(word: &mut [u8; 32], multiplier: u8, label: &str) -> Result<()> {
    let mut carry = 0_u16;
    for byte in word.iter_mut().rev() {
        let value = u16::from(*byte) * u16::from(multiplier) + carry;
        *byte = value as u8;
        carry = value >> 8;
    }
    if carry != 0 {
        return Err(FramkeyError::invalid_data(format!(
            "{label} exceeds 256 bits"
        )));
    }
    Ok(())
}

fn add_word_small(word: &mut [u8; 32], addend: u8, label: &str) -> Result<()> {
    let mut carry = u16::from(addend);
    for byte in word.iter_mut().rev() {
        let value = u16::from(*byte) + carry;
        *byte = value as u8;
        carry = value >> 8;
        if carry == 0 {
            return Ok(());
        }
    }
    Err(FramkeyError::invalid_data(format!(
        "{label} exceeds 256 bits"
    )))
}

fn fixed_bytes_len(type_name: &str) -> Result<Option<usize>> {
    let Some(len) = type_name.strip_prefix("bytes") else {
        return Ok(None);
    };
    if len.is_empty() {
        return Ok(None);
    }
    let len = len.parse::<usize>().map_err(|_| {
        FramkeyError::invalid_data(format!(
            "typed data fixed bytes type {type_name} is malformed"
        ))
    })?;
    if !(1..=32).contains(&len) {
        return Err(FramkeyError::invalid_data(format!(
            "typed data fixed bytes type {type_name} is unsupported"
        )));
    }
    Ok(Some(len))
}

fn fixed_bytes_word(value: &Value, len: usize) -> Result<[u8; 32]> {
    let text = value.as_str().ok_or_else(|| {
        FramkeyError::invalid_data("typed data fixed bytes field must be a 0x string")
    })?;
    let bytes = parse_data_bytes(text, "typed data fixed bytes")?;
    if bytes.len() != len {
        return Err(FramkeyError::invalid_data(format!(
            "typed data fixed bytes field must be {len} bytes"
        )));
    }
    let mut word = [0_u8; 32];
    word[..len].copy_from_slice(&bytes);
    Ok(word)
}

fn validate_type_name(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(FramkeyError::invalid_data(format!("{label} is malformed")));
    }
    Ok(())
}

fn validate_field_name(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    {
        return Err(FramkeyError::invalid_data(format!("{label} is malformed")));
    }
    Ok(())
}

fn validate_field_type(value: &str) -> Result<()> {
    let base = base_eip712_type(value)?;
    if base.is_empty() || base.len() > 128 {
        return Err(FramkeyError::invalid_data(
            "typed data field type is malformed",
        ));
    }
    if base.starts_with("uint")
        || base.starts_with("int")
        || base.starts_with("bytes")
        || matches!(base, "address" | "bool" | "string")
    {
        return Ok(());
    }
    validate_type_name(base, "typed-data custom field type")
}
