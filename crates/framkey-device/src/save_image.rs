use std::fmt;

use framkey_core::{FramkeyError, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveImage {
    bytes: Vec<u8>,
}

impl SaveImage {
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub fn blake3_hash(&self) -> SaveImageHash {
        SaveImageHash(*blake3::hash(&self.bytes).as_bytes())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SaveImageHash(pub [u8; 32]);

impl SaveImageHash {
    pub fn from_hex(input: &str) -> Result<Self> {
        let input = input.strip_prefix("0x").unwrap_or(input);
        if input.len() != 64 {
            return Err(FramkeyError::invalid_data(
                "BLAKE3 hash must be 32 bytes encoded as 64 hex characters",
            ));
        }

        let mut bytes = [0_u8; 32];
        for (index, slot) in bytes.iter_mut().enumerate() {
            let start = index * 2;
            *slot = u8::from_str_radix(&input[start..start + 2], 16)
                .map_err(|_| FramkeyError::invalid_data("invalid BLAKE3 hash hex"))?;
        }

        Ok(Self(bytes))
    }
}

impl fmt::Display for SaveImageHash {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(formatter, "{byte:02x}")?;
        }
        Ok(())
    }
}
