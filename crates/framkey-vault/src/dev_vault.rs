use framkey_core::{FramkeyError, Generation, PolicyId, Result, WalletId, WalletType};
use framkey_crypto::{AeadBox, SecretBytes, encode_hex, random_array};

use crate::{
    constants::{VAULT_FORMAT_VERSION, VAULT_MAGIC},
    save_image::{active_slot_payload, build_save_image_with_payload, inspect_save_image},
    types::{
        DekWrapper, DevEncryptedVaultImage, DevEncryptedVaultMetadata, RecoveryPolicyDescriptor,
        SaveSlot, VaultFile,
    },
    util::{
        current_unix_timestamp, dev_dek_wrapper_aad, dev_kek_id, random_wallet_secret,
        wallet_secret_aad,
    },
};

pub fn build_dev_encrypted_save_image(
    image_size: usize,
    generation: Generation,
    label: &str,
    dev_kek: &SecretBytes<32>,
) -> Result<DevEncryptedVaultImage> {
    let wallet_type = WalletType::EvmEoaSecp256k1;
    let wallet_secret = random_wallet_secret(wallet_type)?;
    let dek = SecretBytes::new(random_array::<32>()?);
    let wallet_id = WalletId(random_array::<16>()?);
    let dev_key_id = dev_kek_id(dev_kek);
    let timestamp = current_unix_timestamp();

    let secret_aad = wallet_secret_aad(wallet_id, generation, wallet_type);
    let encrypted_wallet_secret =
        AeadBox::encrypt(&dek, &secret_aad, wallet_secret.expose().as_slice())?;

    let wrapper_aad = dev_dek_wrapper_aad(wallet_id, generation, dev_key_id);
    let encrypted_dek = AeadBox::encrypt(dev_kek, &wrapper_aad, dek.expose().as_slice())?;

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
        dek_wrappers: vec![DekWrapper::DevTest {
            label: label.to_owned(),
            key_id: dev_key_id,
            encrypted_dek,
        }],
        recovery_policy: RecoveryPolicyDescriptor {
            policy_id: PolicyId::ZERO,
            label: "dev/test only".to_owned(),
        },
    };
    vault.validate()?;

    let payload = serde_json::to_vec_pretty(&vault)
        .map_err(|error| FramkeyError::invalid_data(error.to_string()))?;
    let save_image = build_save_image_with_payload(image_size, SaveSlot::A, generation, &payload)?;
    let inspection = inspect_save_image(&save_image)?;

    Ok(DevEncryptedVaultImage {
        metadata: DevEncryptedVaultMetadata {
            image_size: save_image.len(),
            slot_size: inspection.slot_size,
            wallet_id: encode_hex(&wallet_id.0),
            generation: generation.0,
            wallet_type,
            dev_wrapper_label: label.to_owned(),
            dev_key_id: encode_hex(&dev_key_id),
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
    })
}

pub fn open_dev_encrypted_save_image(
    image: &[u8],
    dev_kek: &SecretBytes<32>,
) -> Result<DevEncryptedVaultMetadata> {
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

    let expected_key_id = dev_kek_id(dev_kek);
    let (label, encrypted_dek) = vault
        .dek_wrappers
        .iter()
        .find_map(|wrapper| match wrapper {
            DekWrapper::DevTest {
                label,
                key_id,
                encrypted_dek,
            } if *key_id == expected_key_id => Some((label, encrypted_dek)),
            _ => None,
        })
        .ok_or_else(|| FramkeyError::invalid_data("matching dev/test DEK wrapper not found"))?;

    let wrapper_aad = dev_dek_wrapper_aad(vault.wallet_id, vault.generation, expected_key_id);
    let dek = encrypted_dek.decrypt_secret::<32>(dev_kek, &wrapper_aad)?;

    let secret_aad = wallet_secret_aad(vault.wallet_id, vault.generation, vault.wallet_type);
    let wallet_secret = vault
        .encrypted_wallet_secret
        .decrypt_secret::<32>(&dek, &secret_aad)?;
    let wallet_secret_hash = encode_hex(blake3::hash(wallet_secret.expose()).as_bytes());

    let active_slot_payload_hash_valid = inspection
        .slots
        .iter()
        .find(|slot| slot.slot == inspection.active_slot)
        .map(|slot| slot.payload_hash_valid)
        .unwrap_or(false);

    Ok(DevEncryptedVaultMetadata {
        image_size: image.len(),
        slot_size: inspection.slot_size,
        wallet_id: encode_hex(&vault.wallet_id.0),
        generation: vault.generation.0,
        wallet_type: vault.wallet_type,
        dev_wrapper_label: label.clone(),
        dev_key_id: encode_hex(&expected_key_id),
        wallet_secret_hash,
        active_slot_hash_valid: inspection.active_slot_hash_valid,
        active_slot_payload_hash_valid,
    })
}
