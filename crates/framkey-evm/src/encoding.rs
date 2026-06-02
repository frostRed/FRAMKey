use framkey_core::{FramkeyError, Result};
use framkey_crypto::{decode_hex_array, encode_hex};
use sha3::{Digest, Keccak256};

use crate::EvmAddress;

pub fn decode_signature_hex(input: &str) -> Result<[u8; 65]> {
    decode_hex_array::<65>(input)
}

pub fn encode_prefixed_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(2 + (bytes.len() * 2));
    out.push_str("0x");
    out.push_str(&encode_hex(bytes));
    out
}

pub(crate) fn keccak256(bytes: &[u8]) -> [u8; 32] {
    let digest = Keccak256::digest(bytes);
    let mut output = [0_u8; 32];
    output.copy_from_slice(&digest);
    output
}

pub(crate) fn parse_address_bytes(input: &str, label: &str) -> Result<[u8; 20]> {
    let address: EvmAddress = input.parse().map_err(|_| {
        FramkeyError::invalid_data(format!("{label} must be a 0x-prefixed address"))
    })?;
    Ok(address.0)
}

pub(crate) fn parse_quantity_bytes(input: &str, label: &str) -> Result<Vec<u8>> {
    let Some(hex) = input.strip_prefix("0x") else {
        return Err(FramkeyError::invalid_data(format!(
            "{label} must be a 0x-prefixed quantity"
        )));
    };
    if hex.is_empty() || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(FramkeyError::invalid_data(format!(
            "{label} must be a non-empty hex quantity"
        )));
    }
    let padded = if hex.len() % 2 == 0 {
        hex.to_owned()
    } else {
        format!("0{hex}")
    };
    let bytes = decode_variable_hex(&padded, label)?;
    Ok(minimal_integer_bytes(&bytes))
}

pub(crate) fn parse_data_bytes(input: &str, label: &str) -> Result<Vec<u8>> {
    let Some(hex) = input.strip_prefix("0x") else {
        return Err(FramkeyError::invalid_data(format!(
            "{label} must be 0x-prefixed hex data"
        )));
    };
    if hex.len() % 2 != 0 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(FramkeyError::invalid_data(format!(
            "{label} must contain an even number of hex digits"
        )));
    }
    decode_variable_hex(hex, label)
}

pub(crate) fn decode_variable_hex(hex: &str, label: &str) -> Result<Vec<u8>> {
    let mut output = Vec::with_capacity(hex.len() / 2);
    for index in (0..hex.len()).step_by(2) {
        let byte = u8::from_str_radix(&hex[index..index + 2], 16)
            .map_err(|_| FramkeyError::invalid_data(format!("{label} contains invalid hex")))?;
        output.push(byte);
    }
    Ok(output)
}

pub(crate) fn minimal_integer_bytes(bytes: &[u8]) -> Vec<u8> {
    let first_nonzero = bytes.iter().position(|byte| *byte != 0);
    match first_nonzero {
        Some(index) => bytes[index..].to_vec(),
        None => Vec::new(),
    }
}
