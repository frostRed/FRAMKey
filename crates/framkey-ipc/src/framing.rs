use std::io::{Read, Write};

use framkey_core::{FramkeyError, Result};

use crate::MAX_NATIVE_MESSAGE_BYTES;

pub fn read_native_message<R: Read>(reader: &mut R) -> Result<Option<Vec<u8>>> {
    let mut header = [0_u8; 4];
    let mut read = 0;

    while read < header.len() {
        let n = reader.read(&mut header[read..])?;
        if n == 0 {
            return if read == 0 {
                Ok(None)
            } else {
                Err(FramkeyError::invalid_data(
                    "truncated native messaging header",
                ))
            };
        }
        read += n;
    }

    let len = u32::from_le_bytes(header) as usize;
    if len > MAX_NATIVE_MESSAGE_BYTES {
        return Err(FramkeyError::invalid_data(format!(
            "native message exceeds {} bytes",
            MAX_NATIVE_MESSAGE_BYTES
        )));
    }

    let mut payload = vec![0_u8; len];
    reader.read_exact(&mut payload)?;
    Ok(Some(payload))
}

pub fn write_native_message<W: Write>(writer: &mut W, payload: &[u8]) -> Result<()> {
    if payload.len() > MAX_NATIVE_MESSAGE_BYTES {
        return Err(FramkeyError::invalid_data(format!(
            "native message exceeds {} bytes",
            MAX_NATIVE_MESSAGE_BYTES
        )));
    }

    let len = payload.len() as u32;
    writer.write_all(&len.to_le_bytes())?;
    writer.write_all(payload)?;
    writer.flush()?;
    Ok(())
}
