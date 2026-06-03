use framkey_core::{FramkeyError, Generation, PolicyId, Result, WalletId, WalletType};
use framkey_crypto::{AeadBox, SecretBytes, encode_hex, random_array};
use framkey_recovery::{
    RecoveryBackupEntropy, RecoveryBackupFile, RecoveryBackupPack,
    reconstruct_recovery_root_key_candidates,
};
use zeroize::Zeroize;

use crate::{
    constants::{VAULT_FORMAT_VERSION, VAULT_MAGIC},
    save_image::{build_save_image_with_payload, inspect_save_image, save_image_payload},
    types::{
        DekWrapper, KeychainEncryptedVaultImage, KeychainEncryptedVaultMetadata,
        KeychainVaultMetadata, RecoveryPolicyDescriptor, RecoveryRewrappedKeychainVaultImage,
        VaultFile,
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
    let save_image = build_save_image_with_payload(image_size, generation, &payload)?;
    let inspection = inspect_save_image(&save_image)?;

    Ok(KeychainEncryptedVaultImage {
        metadata: KeychainEncryptedVaultMetadata {
            image_size: save_image.len(),
            shard_size: inspection.shard_size,
            data_shards: inspection.data_shards,
            parity_shards: inspection.parity_shards,
            wallet_id: encode_hex(&wallet_id.0),
            generation: generation.0,
            wallet_type,
            keychain_item_id: keychain_item_id.to_owned(),
            device_id: encode_hex(&device_id),
            wallet_secret_hash: encode_hex(blake3::hash(wallet_secret.expose()).as_bytes()),
            payload_hash_valid: inspection.payload_hash_valid,
            recovered_shard_count: inspection.recovered_shard_count,
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
        shard_size: metadata.shard_size,
        data_shards: metadata.data_shards,
        parity_shards: metadata.parity_shards,
        wallet_id: metadata.wallet_id,
        generation: metadata.generation,
        wallet_type: metadata.wallet_type,
        keychain_item_id: metadata.keychain_item_id,
        device_id: metadata.device_id,
        wallet_secret_hash,
        payload_hash_valid: metadata.payload_hash_valid,
        recovered_shard_count: metadata.recovered_shard_count,
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

    let payload = save_image_payload(image)?;
    let mut vault: VaultFile = serde_json::from_slice(&payload)
        .map_err(|error| FramkeyError::invalid_data(error.to_string()))?;
    vault.validate()?;

    if vault.generation.0 != inspection.generation {
        return Err(FramkeyError::invalid_data(format!(
            "vault generation {} does not match save header generation {}",
            vault.generation.0, inspection.generation
        )));
    }

    validate_recovery_files_for_vault(&vault, recovery_files)?;
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
    let mut recovery_root_key_candidates =
        reconstruct_recovery_root_key_candidates(recovery_files)?;
    let mut dek = None;
    for candidate in &mut recovery_root_key_candidates {
        let recovery_root_key = SecretBytes::new(*candidate);
        candidate.zeroize();
        if let Ok(decrypted_dek) =
            recovery_encrypted_dek.decrypt_secret::<32>(&recovery_root_key, &recovery_aad)
        {
            dek = Some(decrypted_dek);
            break;
        }
    }
    recovery_root_key_candidates.zeroize();
    let dek = dek.ok_or_else(|| {
        FramkeyError::invalid_data(
            "recovery DEK wrapper could not be decrypted with supplied recovery files",
        )
    })?;

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
    let save_image = build_save_image_with_payload(image.len(), vault.generation, &payload)?;
    let inspection = inspect_save_image(&save_image)?;

    Ok(RecoveryRewrappedKeychainVaultImage {
        save_image,
        metadata: KeychainVaultMetadata {
            image_size: image.len(),
            shard_size: inspection.shard_size,
            data_shards: inspection.data_shards,
            parity_shards: inspection.parity_shards,
            wallet_id: encode_hex(&vault.wallet_id.0),
            generation: vault.generation.0,
            wallet_type: vault.wallet_type,
            keychain_item_id: keychain_item_id.to_owned(),
            device_id: encode_hex(&device_id),
            payload_hash_valid: inspection.payload_hash_valid,
            recovered_shard_count: inspection.recovered_shard_count,
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

    let payload = save_image_payload(image)?;
    let vault: VaultFile = serde_json::from_slice(&payload)
        .map_err(|error| FramkeyError::invalid_data(error.to_string()))?;
    vault.validate()?;

    if vault.generation.0 != inspection.generation {
        return Err(FramkeyError::invalid_data(format!(
            "vault generation {} does not match save header generation {}",
            vault.generation.0, inspection.generation
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
    let dek = encrypted_dek.decrypt_secret::<32>(keychain_kek, &wrapper_aad)?;

    let secret_aad = wallet_secret_aad(vault.wallet_id, vault.generation, vault.wallet_type);
    let wallet_secret = vault
        .encrypted_wallet_secret
        .decrypt_secret::<32>(&dek, &secret_aad)?;

    let metadata = KeychainVaultMetadata {
        image_size: image.len(),
        shard_size: inspection.shard_size,
        data_shards: inspection.data_shards,
        parity_shards: inspection.parity_shards,
        wallet_id: encode_hex(&vault.wallet_id.0),
        generation: vault.generation.0,
        wallet_type: vault.wallet_type,
        keychain_item_id: keychain_item_id.to_owned(),
        device_id: encode_hex(&device_id),
        payload_hash_valid: inspection.payload_hash_valid,
        recovered_shard_count: inspection.recovered_shard_count,
    };
    let result = use_secret(&metadata, &wallet_secret)?;

    Ok((metadata, result))
}
