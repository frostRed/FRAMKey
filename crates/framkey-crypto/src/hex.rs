use framkey_core::{FramkeyError, Result};

pub fn encode_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

pub fn decode_hex_array<const N: usize>(input: &str) -> Result<[u8; N]> {
    let hex = input.strip_prefix("0x").unwrap_or(input);
    if hex.len() != N * 2 {
        return Err(FramkeyError::invalid_data(format!(
            "hex value must encode {N} bytes"
        )));
    }

    let mut output = [0_u8; N];
    for (index, slot) in output.iter_mut().enumerate() {
        let start = index * 2;
        *slot = u8::from_str_radix(&hex[start..start + 2], 16)
            .map_err(|_| FramkeyError::invalid_data("invalid hex value"))?;
    }
    Ok(output)
}
