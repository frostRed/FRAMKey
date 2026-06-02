use framkey_core::{FramkeyError, Result};
use framkey_crypto::SecretBytes;
use k256::ecdsa::{Signature, VerifyingKey};
use sha3::{Digest, Keccak256};

use crate::{
    EvmAddress, EvmPersonalSignature,
    keys::{address_from_verifying_key, signing_key_from_secret},
    signature::{eth_signature_bytes, recovery_id_from_eth_v},
};

pub fn personal_sign(secret: &SecretBytes<32>, message: &[u8]) -> Result<EvmPersonalSignature> {
    let signing_key = signing_key_from_secret(secret.expose())?;
    let digest = personal_sign_digest(message);
    let (signature, recovery_id) = signing_key
        .sign_digest_recoverable(Keccak256::new_with_prefix(personal_sign_payload(message)))
        .map_err(|_| FramkeyError::invalid_data("EVM personal_sign failed"))?;

    Ok(EvmPersonalSignature {
        address: address_from_verifying_key(signing_key.verifying_key()),
        message_hash: digest,
        signature: eth_signature_bytes(signature, recovery_id),
    })
}

pub fn recover_personal_signer(message: &[u8], signature: &[u8; 65]) -> Result<EvmAddress> {
    let signature_core = Signature::from_slice(&signature[..64])
        .map_err(|_| FramkeyError::invalid_data("invalid EVM signature bytes"))?;
    let recovery_id = recovery_id_from_eth_v(signature[64])?;
    let verifying_key = VerifyingKey::recover_from_digest(
        Keccak256::new_with_prefix(personal_sign_payload(message)),
        &signature_core,
        recovery_id,
    )
    .map_err(|_| FramkeyError::invalid_data("EVM signature recovery failed"))?;

    Ok(address_from_verifying_key(&verifying_key))
}

fn personal_sign_digest(message: &[u8]) -> [u8; 32] {
    let digest = Keccak256::digest(personal_sign_payload(message));
    let mut output = [0_u8; 32];
    output.copy_from_slice(&digest);
    output
}

fn personal_sign_payload(message: &[u8]) -> Vec<u8> {
    let mut payload = Vec::with_capacity(32 + message.len());
    payload.extend_from_slice(b"\x19Ethereum Signed Message:\n");
    payload.extend_from_slice(message.len().to_string().as_bytes());
    payload.extend_from_slice(message);
    payload
}
