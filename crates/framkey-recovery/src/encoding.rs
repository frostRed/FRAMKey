use framkey_core::{FramkeyError, Result};

pub(crate) fn share_hash(share_hex: &str) -> String {
    encode_hex(blake3::hash(share_hex.as_bytes()).as_bytes())
}

pub(crate) fn encode_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

pub(crate) fn decode_hex(input: &str) -> Result<Vec<u8>> {
    let hex = input.strip_prefix("0x").unwrap_or(input);
    if !hex.len().is_multiple_of(2) || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(FramkeyError::invalid_data("invalid hex value"));
    }
    let mut output = Vec::with_capacity(hex.len() / 2);
    for index in (0..hex.len()).step_by(2) {
        output.push(
            u8::from_str_radix(&hex[index..index + 2], 16)
                .map_err(|_| FramkeyError::invalid_data("invalid hex value"))?,
        );
    }
    Ok(output)
}
