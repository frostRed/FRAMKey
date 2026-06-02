use std::fmt;

use chacha20poly1305::{
    XChaCha20Poly1305, XNonce,
    aead::{Aead, KeyInit, Payload},
};
use framkey_core::{FramkeyError, Result};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

use crate::{SecretBytes, random_array};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[non_exhaustive]
pub enum AeadAlg {
    XChaCha20Poly1305,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AeadBox {
    pub alg: AeadAlg,
    pub nonce: [u8; 24],
    pub aad_hash: [u8; 32],
    pub ciphertext: Vec<u8>,
}

impl fmt::Debug for AeadBox {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AeadBox")
            .field("alg", &self.alg)
            .field("ciphertext_len", &self.ciphertext.len())
            .finish_non_exhaustive()
    }
}

impl AeadBox {
    pub fn encrypt(key: &SecretBytes<32>, aad: &[u8], plaintext: &[u8]) -> Result<Self> {
        let nonce = random_array::<24>()?;
        Self::encrypt_with_nonce(key, nonce, aad, plaintext)
    }

    pub fn encrypt_with_nonce(
        key: &SecretBytes<32>,
        nonce: [u8; 24],
        aad: &[u8],
        plaintext: &[u8],
    ) -> Result<Self> {
        let cipher = XChaCha20Poly1305::new_from_slice(key.expose())
            .map_err(|_| FramkeyError::invalid_data("invalid AEAD key length"))?;
        let ciphertext = cipher
            .encrypt(
                XNonce::from_slice(&nonce),
                Payload {
                    msg: plaintext,
                    aad,
                },
            )
            .map_err(|_| FramkeyError::invalid_data("AEAD encryption failed"))?;

        Ok(Self {
            alg: AeadAlg::XChaCha20Poly1305,
            nonce,
            aad_hash: blake3_hash(aad),
            ciphertext,
        })
    }

    pub fn decrypt(&self, key: &SecretBytes<32>, aad: &[u8]) -> Result<Vec<u8>> {
        if self.alg != AeadAlg::XChaCha20Poly1305 {
            return Err(FramkeyError::unsupported("unsupported AEAD algorithm"));
        }

        let aad_hash = blake3_hash(aad);
        if self.aad_hash != aad_hash {
            return Err(FramkeyError::invalid_data("AEAD AAD hash mismatch"));
        }

        let cipher = XChaCha20Poly1305::new_from_slice(key.expose())
            .map_err(|_| FramkeyError::invalid_data("invalid AEAD key length"))?;
        cipher
            .decrypt(
                XNonce::from_slice(&self.nonce),
                Payload {
                    msg: &self.ciphertext,
                    aad,
                },
            )
            .map_err(|_| FramkeyError::invalid_data("AEAD decryption failed"))
    }

    /// Decrypt fixed-size secret material and wipe the intermediate plaintext buffer after copying.
    pub fn decrypt_secret<const N: usize>(
        &self,
        key: &SecretBytes<32>,
        aad: &[u8],
    ) -> Result<SecretBytes<N>> {
        let mut plaintext = self.decrypt(key, aad)?;
        let secret = SecretBytes::from_slice(&plaintext);
        plaintext.zeroize();
        secret
    }
}

fn blake3_hash(bytes: &[u8]) -> [u8; 32] {
    *blake3::hash(bytes).as_bytes()
}
