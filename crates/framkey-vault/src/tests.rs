use super::*;
use crate::save_image::SaveImageLayout;
use crate::util::{keychain_dek_wrapper_aad, recovery_dek_wrapper_aad};
use framkey_core::{Generation, PolicyId, WalletType};
use framkey_crypto::{SecretBytes, encode_hex};

#[test]
fn test_save_image_builds_and_inspects() {
    let image =
        build_test_save_image(DEFAULT_FRAM_SAVE_IMAGE_SIZE, Generation(7), "fixture").unwrap();
    let inspection = inspect_save_image(&image).unwrap();

    assert_eq!(inspection.image_size, DEFAULT_FRAM_SAVE_IMAGE_SIZE);
    assert_eq!(inspection.slot_size, 32704);
    assert_eq!(inspection.active_slot, SaveSlot::A);
    assert_eq!(inspection.latest_generation, 7);
    assert!(inspection.active_slot_hash_valid);
    assert_eq!(inspection.slots[0].generation, 7);
    assert!(inspection.slots[0].payload_hash_valid);
    assert!(inspection.slots[0].payload_preview.contains("redacted"));
    assert!(!inspection.slots[0].payload_preview.contains("fixture"));
}

#[test]
fn inspection_detects_payload_corruption() {
    let mut image =
        build_test_save_image(DEFAULT_FRAM_SAVE_IMAGE_SIZE, Generation(1), "fixture").unwrap();
    let layout = SaveImageLayout::new(image.len()).unwrap();
    let payload_offset = layout.slot_offset(SaveSlot::A) + SAVE_SLOT_HEADER_LEN;
    image[payload_offset] ^= 0x01;

    let inspection = inspect_save_image(&image).unwrap();
    assert!(!inspection.active_slot_hash_valid);
    assert!(!inspection.slots[0].payload_hash_valid);
}

#[test]
fn dev_encrypted_save_image_opens_with_matching_kek() {
    let kek = SecretBytes::new([0xA5; 32]);
    let built = build_dev_encrypted_save_image(
        DEFAULT_FRAM_SAVE_IMAGE_SIZE,
        Generation(3),
        "fixture",
        &kek,
    )
    .unwrap();

    let opened = open_dev_encrypted_save_image(&built.save_image, &kek).unwrap();

    assert_eq!(opened.image_size, DEFAULT_FRAM_SAVE_IMAGE_SIZE);
    assert_eq!(opened.slot_size, 32704);
    assert_eq!(opened.generation, 3);
    assert_eq!(opened.dev_wrapper_label, "fixture");
    assert_eq!(opened.wallet_id, built.metadata.wallet_id);
    assert_eq!(opened.wallet_secret_hash, built.metadata.wallet_secret_hash);
    assert!(opened.active_slot_hash_valid);
    assert!(opened.active_slot_payload_hash_valid);

    let wrong_kek = SecretBytes::new([0x5A; 32]);
    assert!(open_dev_encrypted_save_image(&built.save_image, &wrong_kek).is_err());
}

#[test]
fn keychain_encrypted_save_image_opens_with_matching_binding() {
    let kek = SecretBytes::new([0xC3; 32]);
    let device_id = [0x11; 16];
    let item_id = "io.framkey.kek:fixture";
    let built = build_keychain_encrypted_save_image(
        DEFAULT_FRAM_SAVE_IMAGE_SIZE,
        Generation(4),
        item_id,
        device_id,
        &kek,
    )
    .unwrap();

    let opened =
        open_keychain_encrypted_save_image(&built.save_image, item_id, device_id, &kek).unwrap();

    assert_eq!(opened.image_size, DEFAULT_FRAM_SAVE_IMAGE_SIZE);
    assert_eq!(opened.slot_size, 32704);
    assert_eq!(opened.generation, 4);
    assert_eq!(opened.keychain_item_id, item_id);
    assert_eq!(opened.device_id, encode_hex(&device_id));
    assert_eq!(opened.wallet_id, built.metadata.wallet_id);
    assert_eq!(opened.wallet_secret_hash, built.metadata.wallet_secret_hash);
    assert!(opened.active_slot_hash_valid);
    assert!(opened.active_slot_payload_hash_valid);

    let wrong_kek = SecretBytes::new([0x5A; 32]);
    assert!(
        open_keychain_encrypted_save_image(&built.save_image, item_id, device_id, &wrong_kek)
            .is_err()
    );

    let mut wrong_device_id = device_id;
    wrong_device_id[0] ^= 0x01;
    assert!(
        open_keychain_encrypted_save_image(&built.save_image, item_id, wrong_device_id, &kek)
            .is_err()
    );
}

#[test]
fn encrypted_vault_image_debug_redacts_save_image_bytes() {
    let kek = SecretBytes::new([0xC3; 32]);
    let device_id = [0x11; 16];
    let item_id = "io.framkey.kek:debug-fixture";
    let built = build_keychain_encrypted_save_image(
        DEFAULT_FRAM_SAVE_IMAGE_SIZE,
        Generation(4),
        item_id,
        device_id,
        &kek,
    )
    .unwrap();

    let debug = format!("{built:?}");
    assert!(debug.contains("save_image_len"));
    assert!(!debug.contains("save_image: ["));
    assert!(!debug.contains(&format!("{:?}", built.save_image)));
}

#[test]
fn keychain_vault_with_recovery_pack_wraps_same_dek() {
    let kek = SecretBytes::new([0xC3; 32]);
    let device_id = [0x33; 16];
    let item_id = "io.framkey.kek:recovery-fixture";
    let built = build_keychain_encrypted_save_image_with_recovery(
        DEFAULT_FRAM_SAVE_IMAGE_SIZE,
        Generation(6),
        item_id,
        device_id,
        &kek,
    )
    .unwrap();
    let backup_pack = built.recovery_backup_pack.as_ref().unwrap();
    assert_eq!(backup_pack.files.len(), 4);
    assert!(framkey_recovery::reconstruct_recovery_root_key(&backup_pack.files[0..2]).is_err());

    let recovery_files = vec![
        backup_pack.files[0].clone(),
        backup_pack.files[1].clone(),
        backup_pack.files[2].clone(),
    ];
    let recovery_root_key =
        SecretBytes::new(framkey_recovery::reconstruct_recovery_root_key(&recovery_files).unwrap());

    let payload = active_slot_payload(&built.save_image).unwrap();
    let vault: VaultFile = serde_json::from_slice(payload).unwrap();
    assert_eq!(
        vault.recovery_policy.label,
        "standard 2-of-3 grouped recovery"
    );

    let keychain_encrypted_dek = vault
        .dek_wrappers
        .iter()
        .find_map(|wrapper| match wrapper {
            DekWrapper::MacKeychain { encrypted_dek, .. } => Some(encrypted_dek),
            _ => None,
        })
        .unwrap();
    let recovery = vault
        .dek_wrappers
        .iter()
        .find_map(|wrapper| match wrapper {
            DekWrapper::Recovery {
                policy_id,
                encrypted_dek,
            } => Some((*policy_id, encrypted_dek)),
            _ => None,
        })
        .unwrap();

    let keychain_aad =
        keychain_dek_wrapper_aad(vault.wallet_id, vault.generation, device_id, item_id);
    let recovery_aad = recovery_dek_wrapper_aad(vault.wallet_id, vault.generation, recovery.0);
    let keychain_dek = keychain_encrypted_dek.decrypt(&kek, &keychain_aad).unwrap();
    let recovery_dek = recovery
        .1
        .decrypt(&recovery_root_key, &recovery_aad)
        .unwrap();
    assert_eq!(keychain_dek, recovery_dek);
}

#[test]
fn vault_validation_rejects_blank_keychain_wrapper_binding() {
    let kek = SecretBytes::new([0xC3; 32]);
    let device_id = [0x33; 16];
    let item_id = "io.framkey.kek:blank-wrapper-fixture";
    let built = build_keychain_encrypted_save_image(
        DEFAULT_FRAM_SAVE_IMAGE_SIZE,
        Generation(6),
        item_id,
        device_id,
        &kek,
    )
    .unwrap();
    let payload = active_slot_payload(&built.save_image).unwrap();
    let mut vault: VaultFile = serde_json::from_slice(payload).unwrap();
    for wrapper in &mut vault.dek_wrappers {
        if let DekWrapper::MacKeychain {
            keychain_item_id, ..
        } = wrapper
        {
            *keychain_item_id = " \t".to_owned();
        }
    }

    let error = vault.validate().unwrap_err();
    assert!(error.to_string().contains("must not be blank"));
}

#[test]
fn vault_validation_rejects_inconsistent_recovery_policy() {
    let kek = SecretBytes::new([0xC3; 32]);
    let device_id = [0x33; 16];
    let item_id = "io.framkey.kek:policy-fixture";
    let built = build_keychain_encrypted_save_image_with_recovery(
        DEFAULT_FRAM_SAVE_IMAGE_SIZE,
        Generation(6),
        item_id,
        device_id,
        &kek,
    )
    .unwrap();
    let payload = active_slot_payload(&built.save_image).unwrap();
    let mut vault: VaultFile = serde_json::from_slice(payload).unwrap();
    vault.recovery_policy.policy_id = PolicyId::ZERO;

    let error = vault.validate().unwrap_err();
    assert!(error.to_string().contains("recovery wrapper"));
}

#[test]
fn recovery_rewrap_binds_vault_to_new_keychain_without_wallet_secret() {
    let old_kek = SecretBytes::new([0xC3; 32]);
    let old_device_id = [0x33; 16];
    let old_item_id = "io.framkey.kek:old-recovery-fixture";
    let built = build_keychain_encrypted_save_image_with_recovery(
        DEFAULT_FRAM_SAVE_IMAGE_SIZE,
        Generation(7),
        old_item_id,
        old_device_id,
        &old_kek,
    )
    .unwrap();
    let backup_pack = built.recovery_backup_pack.as_ref().unwrap();
    let recovery_files = vec![
        backup_pack.files[0].clone(),
        backup_pack.files[1].clone(),
        backup_pack.files[2].clone(),
    ];
    let new_kek = SecretBytes::new([0xA5; 32]);
    let new_device_id = [0x44; 16];
    let new_item_id = "io.framkey.kek:new-recovery-fixture";

    let recovered = rewrap_keychain_vault_with_recovery(
        &built.save_image,
        &recovery_files,
        new_item_id,
        new_device_id,
        &new_kek,
    )
    .unwrap();

    assert_eq!(recovered.metadata.image_size, DEFAULT_FRAM_SAVE_IMAGE_SIZE);
    assert_eq!(recovered.metadata.generation, 7);
    assert_eq!(recovered.metadata.wallet_id, built.metadata.wallet_id);
    assert_eq!(recovered.metadata.keychain_item_id, new_item_id);
    assert_eq!(recovered.metadata.device_id, encode_hex(&new_device_id));
    assert!(recovered.metadata.active_slot_hash_valid);
    assert!(recovered.metadata.active_slot_payload_hash_valid);

    let opened = open_keychain_encrypted_save_image(
        &recovered.save_image,
        new_item_id,
        new_device_id,
        &new_kek,
    )
    .unwrap();
    assert_eq!(opened.wallet_secret_hash, built.metadata.wallet_secret_hash);

    assert!(
        rewrap_keychain_vault_with_recovery(
            &built.save_image,
            &backup_pack.files[0..2],
            new_item_id,
            new_device_id,
            &new_kek,
        )
        .is_err()
    );
}

#[test]
fn keychain_wallet_secret_can_sign_and_recover_personal_message() {
    use framkey_evm::{personal_sign, recover_personal_signer};

    let kek = SecretBytes::new([0xC3; 32]);
    let device_id = [0x22; 16];
    let item_id = "io.framkey.kek:signer-fixture";
    let built = build_keychain_encrypted_save_image(
        DEFAULT_FRAM_SAVE_IMAGE_SIZE,
        Generation(5),
        item_id,
        device_id,
        &kek,
    )
    .unwrap();
    let message = b"FRAMKey signer helper test";

    let (metadata, signed) = with_keychain_wallet_secret(
        &built.save_image,
        item_id,
        device_id,
        &kek,
        |metadata, secret| {
            assert_eq!(metadata.wallet_type, WalletType::EvmEoaSecp256k1);
            personal_sign(secret, message)
        },
    )
    .unwrap();
    let recovered = recover_personal_signer(message, &signed.signature).unwrap();

    assert_eq!(metadata.wallet_id, built.metadata.wallet_id);
    assert_eq!(recovered, signed.address);
}
