use framkey_core::{FramkeyError, Result};
use framkey_crypto::SecretBytes;
use k256::ecdsa::{SigningKey, VerifyingKey};
use sha3::{Digest, Keccak256};

use crate::EvmAddress;

pub fn validate_private_key_bytes(bytes: &[u8; 32]) -> Result<()> {
    signing_key_from_secret(bytes)?;
    Ok(())
}

pub fn address_from_secret(secret: &SecretBytes<32>) -> Result<EvmAddress> {
    let signing_key = signing_key_from_secret(secret.expose())?;
    Ok(address_from_verifying_key(signing_key.verifying_key()))
}

pub(crate) fn signing_key_from_secret(bytes: &[u8; 32]) -> Result<SigningKey> {
    SigningKey::from_slice(bytes)
        .map_err(|_| FramkeyError::invalid_data("invalid secp256k1 private key"))
}

pub(crate) fn address_from_verifying_key(verifying_key: &VerifyingKey) -> EvmAddress {
    let encoded = verifying_key.to_encoded_point(false);
    let public_key = encoded.as_bytes();
    debug_assert_eq!(public_key.len(), 65);
    debug_assert_eq!(public_key[0], 0x04);

    let hash = Keccak256::digest(&public_key[1..]);
    let mut address = [0_u8; 20];
    address.copy_from_slice(&hash[12..32]);
    EvmAddress(address)
}
