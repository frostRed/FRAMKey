use framkey_core::{FramkeyError, Generation, PolicyId, Result, WalletId, WalletType};
use framkey_crypto::{AeadBox, SecretBytes, encode_hex, random_array};
use framkey_recovery::{
    RecoveryBackupEntropy, RecoveryBackupFile, RecoveryBackupPack, reconstruct_recovery_root_key,
};
use zeroize::Zeroize;

use crate::{
    constants::{VAULT_FORMAT_VERSION, VAULT_MAGIC},
    save_image::{active_slot_payload, build_save_image_with_payload, inspect_save_image},
    types::{
        DekWrapper, KeychainEncryptedVaultImage, KeychainEncryptedVaultMetadata,
        KeychainVaultMetadata, RecoveryPolicyDescriptor, RecoveryRewrappedKeychainVaultImage,
        SaveSlot, VaultFile,
    },
    util::{
        current_unix_timestamp, hex_16, keychain_dek_wrapper_aad, random_wallet_secret,
        recovery_dek_wrapper_aad, validate_keychain_wrapper_binding,
        validate_recovery_files_for_vault, wallet_secret_aad,
    },
};

pub fn build_keychain_encrypted_save_image(
    image_size: usize,
    generation: Generation,
    keychain_item_id: &str,
    device_id: [u8; 16],
    keychain_kek: &SecretBytes<32>,
) -> Result<KeychainEncryptedVaultImage> {
    build_keychain_encrypted_save_image_inner(
        image_size,
        generation,
        keychain_item_id,
        device_id,
        keychain_kek,
        false,
    )
}

pub fn build_keychain_encrypted_save_image_with_recovery(
    image_size: usize,
    generation: Generation,
    keychain_item_id: &str,
    device_id: [u8; 16],
    keychain_kek: &SecretBytes<32>,
) -> Result<KeychainEncryptedVaultImage> {
    build_keychain_encrypted_save_image_inner(
        image_size,
        generation,
        keychain_item_id,
        device_id,
        keychain_kek,
        true,
    )
}

fn build_keychain_encrypted_save_image_inner(
    image_size: usize,
    generation: Generation,
    keychain_item_id: &str,
    device_id: [u8; 16],
    keychain_kek: &SecretBytes<32>,
    include_recovery: bool,
) -> Result<KeychainEncryptedVaultImage> {
    validate_keychain_wrapper_binding(keychain_item_id)?;

    let wallet_type = WalletType::EvmEoaSecp256k1;
    let wallet_secret = random_wallet_secret(wallet_type)?;
    let dek = SecretBytes::new(random_array::<32>()?);
    let wallet_id = WalletId(random_array::<16>()?);
    let timestamp = current_unix_timestamp();

    let secret_aad = wallet_secret_aad(wallet_id, generation, wallet_type);
    let encrypted_wallet_secret =
        AeadBox::encrypt(&dek, &secret_aad, wallet_secret.expose().as_slice())?;

    let wrapper_aad = keychain_dek_wrapper_aad(wallet_id, generation, device_id, keychain_item_id);
    let encrypted_dek = AeadBox::encrypt(keychain_kek, &wrapper_aad, dek.expose().as_slice())?;
    let recovery = if include_recovery {
        let recovery_root_key = SecretBytes::new(random_array::<32>()?);
        let policy_id = PolicyId(random_array::<16>()?);
        let recovery_wrapper_aad = recovery_dek_wrapper_aad(wallet_id, generation, policy_id);
        let encrypted_dek = AeadBox::encrypt(
            &recovery_root_key,
            &recovery_wrapper_aad,
            dek.expose().as_slice(),
        )?;
        let backup_pack = RecoveryBackupPack::standard(
            wallet_id.0,
            generation.0,
            policy_id.0,
            random_array::<16>()?,
            timestamp.0,
            recovery_root_key.expose(),
            RecoveryBackupEntropy {
                group_polynomial_coefficients: random_array::<32>()?,
                cloud_member_pad: random_array::<32>()?,
            },
        );
        Some((policy_id, encrypted_dek, backup_pack))
    } else {
        None
    };

    let mut dek_wrappers = vec![DekWrapper::MacKeychain {
        device_id,
        keychain_item_id: keychain_item_id.to_owned(),
        encrypted_dek,
    }];
    if let Some((policy_id, encrypted_dek, _backup_pack)) = &recovery {
        dek_wrappers.push(DekWrapper::Recovery {
            policy_id: *policy_id,
            encrypted_dek: encrypted_dek.clone(),
        });
    }
    let recovery_policy = recovery
        .as_ref()
        .map(
            |(policy_id, _encrypted_dek, _backup_pack)| RecoveryPolicyDescriptor {
                policy_id: *policy_id,
                label: "standard 2-of-3 grouped recovery".to_owned(),
            },
        )
        .unwrap_or_else(|| RecoveryPolicyDescriptor {
            policy_id: PolicyId::ZERO,
            label: "macOS Keychain local unlock".to_owned(),
        });
    let vault = VaultFile {
        magic: VAULT_MAGIC,
        format_version: VAULT_FORMAT_VERSION,
        wallet_id,
        generation,
        created_at: timestamp,
        updated_at: timestamp,
        wallet_type,
        public_address: None,
        encrypted_wallet_secret,
        dek_wrappers,
        recovery_policy,
    };
    vault.validate()?;

    let payload = serde_json::to_vec_pretty(&vault)
        .map_err(|error| FramkeyError::invalid_data(error.to_string()))?;
    let save_image = build_save_image_with_payload(image_size, SaveSlot::A, generation, &payload)?;
    let inspection = inspect_save_image(&save_image)?;

    Ok(KeychainEncryptedVaultImage {
        metadata: KeychainEncryptedVaultMetadata {
            image_size: save_image.len(),
            slot_size: inspection.slot_size,
            wallet_id: encode_hex(&wallet_id.0),
            generation: generation.0,
            wallet_type,
            keychain_item_id: keychain_item_id.to_owned(),
            device_id: encode_hex(&device_id),
            wallet_secret_hash: encode_hex(blake3::hash(wallet_secret.expose()).as_bytes()),
            active_slot_hash_valid: inspection.active_slot_hash_valid,
            active_slot_payload_hash_valid: inspection
                .slots
                .iter()
                .find(|slot| slot.slot == SaveSlot::A)
                .map(|slot| slot.payload_hash_valid)
                .unwrap_or(false),
        },
        save_image,
        recovery_backup_pack: recovery.map(|(_policy_id, _encrypted_dek, backup_pack)| backup_pack),
    })
}

pub fn open_keychain_encrypted_save_image(
    image: &[u8],
    keychain_item_id: &str,
    device_id: [u8; 16],
    keychain_kek: &SecretBytes<32>,
) -> Result<KeychainEncryptedVaultMetadata> {
    let (metadata, wallet_secret_hash) = with_keychain_wallet_secret(
        image,
        keychain_item_id,
        device_id,
        keychain_kek,
        |_metadata, wallet_secret| Ok(encode_hex(blake3::hash(wallet_secret.expose()).as_bytes())),
    )?;

    Ok(KeychainEncryptedVaultMetadata {
        image_size: metadata.image_size,
        slot_size: metadata.slot_size,
        wallet_id: metadata.wallet_id,
        generation: metadata.generation,
        wallet_type: metadata.wallet_type,
        keychain_item_id: metadata.keychain_item_id,
        device_id: metadata.device_id,
        wallet_secret_hash,
        active_slot_hash_valid: metadata.active_slot_hash_valid,
        active_slot_payload_hash_valid: metadata.active_slot_payload_hash_valid,
    })
}

pub fn rewrap_keychain_vault_with_recovery(
    image: &[u8],
    recovery_files: &[RecoveryBackupFile],
    keychain_item_id: &str,
    device_id: [u8; 16],
    keychain_kek: &SecretBytes<32>,
) -> Result<RecoveryRewrappedKeychainVaultImage> {
    validate_keychain_wrapper_binding(keychain_item_id)?;

    let inspection = inspect_save_image(image)?;
    if !inspection.active_slot_hash_valid {
        return Err(FramkeyError::invalid_data("active slot hash is invalid"));
    }

    let payload = active_slot_payload(image)?;
    let mut vault: VaultFile = serde_json::from_slice(payload)
        .map_err(|error| FramkeyError::invalid_data(error.to_string()))?;
    vault.validate()?;

    if vault.generation.0 != inspection.latest_generation {
        return Err(FramkeyError::invalid_data(format!(
            "vault generation {} does not match save header generation {}",
            vault.generation.0, inspection.latest_generation
        )));
    }

    validate_recovery_files_for_vault(&vault, recovery_files)?;
    let recovery_root_key = SecretBytes::new(reconstruct_recovery_root_key(recovery_files)?);
    let recovery_policy_id = PolicyId(hex_16(&recovery_files[0].policy_id, "policy id")?);
    let recovery_encrypted_dek = vault
        .dek_wrappers
        .iter()
        .find_map(|wrapper| match wrapper {
            DekWrapper::Recovery {
                policy_id,
                encrypted_dek,
            } if *policy_id == recovery_policy_id => Some(encrypted_dek),
            _ => None,
        })
        .ok_or_else(|| FramkeyError::invalid_data("matching recovery DEK wrapper not found"))?;

    let recovery_aad =
        recovery_dek_wrapper_aad(vault.wallet_id, vault.generation, recovery_policy_id);
    let mut dek_plaintext = recovery_encrypted_dek.decrypt(&recovery_root_key, &recovery_aad)?;
    let dek = SecretBytes::<32>::from_slice(&dek_plaintext)?;
    dek_plaintext.zeroize();

    let keychain_aad = keychain_dek_wrapper_aad(
        vault.wallet_id,
        vault.generation,
        device_id,
        keychain_item_id,
    );
    let keychain_encrypted_dek =
        AeadBox::encrypt(keychain_kek, &keychain_aad, dek.expose().as_slice())?;

    vault.dek_wrappers.retain(|wrapper| {
        !matches!(
            wrapper,
            DekWrapper::MacKeychain {
                device_id: existing_device_id,
                keychain_item_id: existing_item_id,
                ..
            } if *existing_device_id == device_id && existing_item_id == keychain_item_id
        )
    });
    vault.dek_wrappers.push(DekWrapper::MacKeychain {
        device_id,
        keychain_item_id: keychain_item_id.to_owned(),
        encrypted_dek: keychain_encrypted_dek,
    });
    vault.updated_at = current_unix_timestamp();
    vault.validate()?;

    let payload = serde_json::to_vec_pretty(&vault)
        .map_err(|error| FramkeyError::invalid_data(error.to_string()))?;
    let save_image =
        build_save_image_with_payload(image.len(), SaveSlot::A, vault.generation, &payload)?;
    let inspection = inspect_save_image(&save_image)?;
    let active_slot_payload_hash_valid = inspection
        .slots
        .iter()
        .find(|slot| slot.slot == SaveSlot::A)
        .map(|slot| slot.payload_hash_valid)
        .unwrap_or(false);

    Ok(RecoveryRewrappedKeychainVaultImage {
        save_image,
        metadata: KeychainVaultMetadata {
            image_size: image.len(),
            slot_size: inspection.slot_size,
            wallet_id: encode_hex(&vault.wallet_id.0),
            generation: vault.generation.0,
            wallet_type: vault.wallet_type,
            keychain_item_id: keychain_item_id.to_owned(),
            device_id: encode_hex(&device_id),
            active_slot_hash_valid: inspection.active_slot_hash_valid,
            active_slot_payload_hash_valid,
        },
    })
}

pub fn with_keychain_wallet_secret<R>(
    image: &[u8],
    keychain_item_id: &str,
    device_id: [u8; 16],
    keychain_kek: &SecretBytes<32>,
    use_secret: impl FnOnce(&KeychainVaultMetadata, &SecretBytes<32>) -> Result<R>,
) -> Result<(KeychainVaultMetadata, R)> {
    validate_keychain_wrapper_binding(keychain_item_id)?;

    let inspection = inspect_save_image(image)?;
    if !inspection.active_slot_hash_valid {
        return Err(FramkeyError::invalid_data("active slot hash is invalid"));
    }

    let payload = active_slot_payload(image)?;
    let vault: VaultFile = serde_json::from_slice(payload)
        .map_err(|error| FramkeyError::invalid_data(error.to_string()))?;
    vault.validate()?;

    if vault.generation.0 != inspection.latest_generation {
        return Err(FramkeyError::invalid_data(format!(
            "vault generation {} does not match save header generation {}",
            vault.generation.0, inspection.latest_generation
        )));
    }

    let encrypted_dek = vault
        .dek_wrappers
        .iter()
        .find_map(|wrapper| match wrapper {
            DekWrapper::MacKeychain {
                device_id: wrapper_device_id,
                keychain_item_id: wrapper_item_id,
                encrypted_dek,
            } if *wrapper_device_id == device_id && wrapper_item_id == keychain_item_id => {
                Some(encrypted_dek)
            }
            _ => None,
        })
        .ok_or_else(|| {
            FramkeyError::invalid_data("matching macOS Keychain DEK wrapper not found")
        })?;

    let wrapper_aad = keychain_dek_wrapper_aad(
        vault.wallet_id,
        vault.generation,
        device_id,
        keychain_item_id,
    );
    let mut dek_plaintext = encrypted_dek.decrypt(keychain_kek, &wrapper_aad)?;
    let dek = SecretBytes::<32>::from_slice(&dek_plaintext)?;
    dek_plaintext.zeroize();

    let secret_aad = wallet_secret_aad(vault.wallet_id, vault.generation, vault.wallet_type);
    let mut wallet_secret_plaintext = vault.encrypted_wallet_secret.decrypt(&dek, &secret_aad)?;
    if wallet_secret_plaintext.len() != 32 {
        wallet_secret_plaintext.zeroize();
        return Err(FramkeyError::invalid_data(format!(
            "wallet secret must be 32 bytes, got {}",
            wallet_secret_plaintext.len()
        )));
    }

    let active_slot_payload_hash_valid = inspection
        .slots
        .iter()
        .find(|slot| slot.slot == inspection.active_slot)
        .map(|slot| slot.payload_hash_valid)
        .unwrap_or(false);

    let wallet_secret = SecretBytes::<32>::from_slice(&wallet_secret_plaintext)?;
    wallet_secret_plaintext.zeroize();

    let metadata = KeychainVaultMetadata {
        image_size: image.len(),
        slot_size: inspection.slot_size,
        wallet_id: encode_hex(&vault.wallet_id.0),
        generation: vault.generation.0,
        wallet_type: vault.wallet_type,
        keychain_item_id: keychain_item_id.to_owned(),
        device_id: encode_hex(&device_id),
        active_slot_hash_valid: inspection.active_slot_hash_valid,
        active_slot_payload_hash_valid,
    };
    let result = use_secret(&metadata, &wallet_secret)?;

    Ok((metadata, result))
}
