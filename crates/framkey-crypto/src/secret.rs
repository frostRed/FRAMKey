use std::fmt;

use framkey_core::{FramkeyError, Result};
use zeroize::Zeroize;

pub struct SecretBytes<const N: usize> {
    bytes: [u8; N],
}

impl<const N: usize> SecretBytes<N> {
    pub fn new(bytes: [u8; N]) -> Self {
        Self { bytes }
    }

    pub fn from_slice(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != N {
            return Err(FramkeyError::invalid_data(format!(
                "secret must be {N} bytes, got {}",
                bytes.len()
            )));
        }

        let mut output = [0_u8; N];
        output.copy_from_slice(bytes);
        Ok(Self::new(output))
    }

    pub fn expose(&self) -> &[u8; N] {
        &self.bytes
    }
}

impl<const N: usize> Drop for SecretBytes<N> {
    fn drop(&mut self) {
        self.bytes.zeroize();
    }
}

impl<const N: usize> fmt::Debug for SecretBytes<N> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SecretBytes")
            .field("len", &N)
            .finish_non_exhaustive()
    }
}

impl<const N: usize> From<[u8; N]> for SecretBytes<N> {
    fn from(bytes: [u8; N]) -> Self {
        Self::new(bytes)
    }
}
