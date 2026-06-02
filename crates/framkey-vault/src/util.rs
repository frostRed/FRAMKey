use std::time::{SystemTime, UNIX_EPOCH};

use framkey_core::{
    FramkeyError, Generation, PolicyId, Result, UnixTimestamp, WalletId, WalletType,
};
use framkey_crypto::{SecretBytes, decode_hex_array, encode_hex, random_array};
use framkey_evm::validate_private_key_bytes;
use framkey_recovery::RecoveryBackupFile;

use crate::types::{DekWrapper, VaultFile};

pub(crate) fn current_unix_timestamp() -> UnixTimestamp {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0);
    UnixTimestamp(seconds)
}

pub(crate) fn random_wallet_secret(wallet_type: WalletType) -> Result<SecretBytes<32>> {
    match wallet_type {
        WalletType::EvmEoaSecp256k1 => loop {
            let candidate = random_array::<32>()?;
            if validate_private_key_bytes(&candidate).is_ok() {
                break Ok(SecretBytes::new(candidate));
            }
        },
        _ => Err(FramkeyError::unsupported("unsupported wallet type")),
    }
}

pub(crate) fn dev_kek_id(dev_kek: &SecretBytes<32>) -> [u8; 16] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"framkey:dev-kek-id:v1");
    hasher.update(dev_kek.expose());
    let hash = hasher.finalize();
    let mut key_id = [0_u8; 16];
    key_id.copy_from_slice(&hash.as_bytes()[..16]);
    key_id
}

pub(crate) fn wallet_secret_aad(
    wallet_id: WalletId,
    generation: Generation,
    wallet_type: WalletType,
) -> Vec<u8> {
    let mut aad = Vec::with_capacity(64);
    aad.extend_from_slice(b"framkey:wallet-secret:v1");
    aad.extend_from_slice(&wallet_id.0);
    aad.extend_from_slice(&generation.0.to_le_bytes());
    aad.extend_from_slice(wallet_type_name(wallet_type).as_bytes());
    aad
}

pub(crate) fn dev_dek_wrapper_aad(
    wallet_id: WalletId,
    generation: Generation,
    key_id: [u8; 16],
) -> Vec<u8> {
    let mut aad = Vec::with_capacity(64);
    aad.extend_from_slice(b"framkey:dev-dek-wrapper:v1");
    aad.extend_from_slice(&wallet_id.0);
    aad.extend_from_slice(&generation.0.to_le_bytes());
    aad.extend_from_slice(&key_id);
    aad
}

pub(crate) fn keychain_dek_wrapper_aad(
    wallet_id: WalletId,
    generation: Generation,
    device_id: [u8; 16],
    keychain_item_id: &str,
) -> Vec<u8> {
    let mut aad = Vec::with_capacity(88 + keychain_item_id.len());
    aad.extend_from_slice(b"framkey:mac-keychain-dek-wrapper:v1");
    aad.extend_from_slice(&wallet_id.0);
    aad.extend_from_slice(&generation.0.to_le_bytes());
    aad.extend_from_slice(&device_id);
    aad.extend_from_slice(&(keychain_item_id.len() as u64).to_le_bytes());
    aad.extend_from_slice(keychain_item_id.as_bytes());
    aad
}

pub(crate) fn recovery_dek_wrapper_aad(
    wallet_id: WalletId,
    generation: Generation,
    policy_id: PolicyId,
) -> Vec<u8> {
    let mut aad = Vec::with_capacity(64);
    aad.extend_from_slice(b"framkey:recovery-dek-wrapper:v1");
    aad.extend_from_slice(&wallet_id.0);
    aad.extend_from_slice(&generation.0.to_le_bytes());
    aad.extend_from_slice(&policy_id.0);
    aad
}

pub(crate) fn validate_recovery_files_for_vault(
    vault: &VaultFile,
    recovery_files: &[RecoveryBackupFile],
) -> Result<()> {
    let first = recovery_files
        .first()
        .ok_or_else(|| FramkeyError::invalid_data("no recovery backup files supplied"))?;
    if first.wallet_id != encode_hex(&vault.wallet_id.0) {
        return Err(FramkeyError::invalid_data(
            "recovery share wallet id does not match vault",
        ));
    }
    if first.generation != vault.generation.0 {
        return Err(FramkeyError::invalid_data(
            "recovery share generation does not match vault",
        ));
    }
    let policy_id = hex_16(&first.policy_id, "policy id")?;
    if !vault
        .dek_wrappers
        .iter()
        .any(|wrapper| matches!(wrapper, DekWrapper::Recovery { policy_id: wrapper_id, .. } if wrapper_id.0 == policy_id))
    {
        return Err(FramkeyError::invalid_data(
            "recovery share policy id does not match a vault recovery wrapper",
        ));
    }
    Ok(())
}

pub(crate) fn hex_16(value: &str, label: &str) -> Result<[u8; 16]> {
    decode_hex_array::<16>(value).map_err(|_| {
        FramkeyError::invalid_data(format!(
            "{label} must be 16 bytes encoded as 32 hex characters"
        ))
    })
}

pub(crate) fn validate_keychain_wrapper_binding(keychain_item_id: &str) -> Result<()> {
    if keychain_item_id.is_empty() {
        return Err(FramkeyError::invalid_data(
            "macOS Keychain item id must not be empty",
        ));
    }
    if keychain_item_id.contains('\0') {
        return Err(FramkeyError::invalid_data(
            "macOS Keychain item id must not contain NUL bytes",
        ));
    }
    Ok(())
}

pub(crate) fn wallet_type_name(wallet_type: WalletType) -> &'static str {
    match wallet_type {
        WalletType::EvmEoaSecp256k1 => "evm_eoa_secp256k1",
        _ => "unknown",
    }
}
