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
    assert_eq!(inspection.header_len, SAVE_IMAGE_HEADER_LEN);
    assert_eq!(inspection.format_version, SAVE_IMAGE_FORMAT_VERSION);
    assert_eq!(inspection.generation, 7);
    assert_eq!(inspection.data_shards, SAVE_RS_DATA_SHARDS);
    assert_eq!(inspection.parity_shards, SAVE_RS_PARITY_SHARDS);
    assert_eq!(inspection.shard_size, 2560);
    assert_eq!(inspection.valid_shard_count, SAVE_RS_TOTAL_SHARDS);
    assert_eq!(inspection.recovered_shard_count, 0);
    assert!(inspection.payload_hash_valid);
    assert!(inspection.superblocks.iter().all(|copy| copy.valid));
    assert!(inspection.shards.iter().all(|shard| shard.hash_valid));
}

#[test]
fn save_image_recovers_single_superblock_and_shard_corruption() {
    let mut image =
        build_test_save_image(DEFAULT_FRAM_SAVE_IMAGE_SIZE, Generation(1), "fixture").unwrap();
    let expected_payload = save_image_payload(&image).unwrap();
    let layout = SaveImageLayout::new(image.len()).unwrap();
    image[0] ^= 0x01;
    let shard_offset = layout.shard_byte_offset(0, 0).unwrap();
    image[shard_offset] ^= 0x01;

    let inspection = inspect_save_image(&image).unwrap();
    assert_eq!(
        inspection
            .superblocks
            .iter()
            .filter(|copy| copy.valid)
            .count(),
        SAVE_SUPERBLOCK_COPIES - 1
    );
    assert_eq!(inspection.valid_shard_count, SAVE_RS_TOTAL_SHARDS - 1);
    assert_eq!(inspection.recovered_shard_count, 1);
    assert_eq!(save_image_payload(&image).unwrap(), expected_payload);
}

#[test]
fn save_image_rejects_more_corrupt_shards_than_parity_can_recover() {
    let mut image =
        build_test_save_image(DEFAULT_FRAM_SAVE_IMAGE_SIZE, Generation(1), "fixture").unwrap();
    let layout = SaveImageLayout::new(image.len()).unwrap();
    for shard_index in 0..=SAVE_RS_PARITY_SHARDS {
        let offset = layout.shard_byte_offset(shard_index, 0).unwrap();
        image[offset] ^= 0x01;
    }

    let error = inspect_save_image(&image).unwrap_err().to_string();
    assert!(error.contains("valid Reed-Solomon shards"));
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
    assert_eq!(opened.shard_size, 2560);
    assert_eq!(opened.data_shards, SAVE_RS_DATA_SHARDS);
    assert_eq!(opened.parity_shards, SAVE_RS_PARITY_SHARDS);
    assert_eq!(opened.generation, 3);
    assert_eq!(opened.dev_wrapper_label, "fixture");
    assert_eq!(opened.wallet_id, built.metadata.wallet_id);
    assert_eq!(opened.wallet_secret_hash, built.metadata.wallet_secret_hash);
    assert!(opened.payload_hash_valid);
    assert_eq!(opened.recovered_shard_count, 0);

    let wrong_kek = SecretBytes::new([0x5A; 32]);
    assert!(open_dev_encrypted_save_image(&built.save_image, &wrong_kek).is_err());
}

#[test]
fn keychain_encrypted_save_image_opens_with_matching_binding() {
    let kek = SecretBytes::new([0xC3; 32]);
    let device_id = [0x11; 16];
    let item_id = "io.framkey.local-kek:fixture";
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
    assert_eq!(opened.shard_size, 2560);
    assert_eq!(opened.data_shards, SAVE_RS_DATA_SHARDS);
    assert_eq!(opened.parity_shards, SAVE_RS_PARITY_SHARDS);
    assert_eq!(opened.generation, 4);
    assert_eq!(opened.keychain_item_id, item_id);
    assert_eq!(opened.device_id, encode_hex(&device_id));
    assert_eq!(opened.wallet_id, built.metadata.wallet_id);
    assert_eq!(opened.wallet_secret_hash, built.metadata.wallet_secret_hash);
    assert!(opened.payload_hash_valid);
    assert_eq!(opened.recovered_shard_count, 0);

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
fn encrypted_vault_image_debug_redacts_save_image_and_recovery_material() {
    let kek = SecretBytes::new([0xC3; 32]);
    let device_id = [0x11; 16];
    let item_id = "io.framkey.local-kek:debug-fixture";
    let built = build_keychain_encrypted_save_image_with_recovery(
        DEFAULT_FRAM_SAVE_IMAGE_SIZE,
        Generation(4),
        item_id,
        device_id,
        &kek,
    )
    .unwrap();

    let debug = format!("{built:?}");
    assert!(debug.contains("save_image_len"));
    assert!(debug.contains("recovery_backup_file_count"));
    assert!(!debug.contains("save_image: ["));
    assert!(!debug.contains(&format!("{:?}", built.save_image)));
    let share_hex = &built.recovery_backup_pack.as_ref().unwrap().files[0].share_hex;
    assert!(!debug.contains(share_hex));
}

#[test]
fn keychain_vault_with_recovery_pack_wraps_same_dek() {
    let kek = SecretBytes::new([0xC3; 32]);
    let device_id = [0x33; 16];
    let item_id = "io.framkey.local-kek:recovery-fixture";
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

    let payload = save_image_payload(&built.save_image).unwrap();
    let vault: VaultFile = serde_json::from_slice(&payload).unwrap();
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
fn vault_validation_rejects_ambiguous_keychain_wrapper_binding() {
    let kek = SecretBytes::new([0xC3; 32]);
    let device_id = [0x33; 16];
    let item_id = "io.framkey.local-kek:blank-wrapper-fixture";
    let built = build_keychain_encrypted_save_image(
        DEFAULT_FRAM_SAVE_IMAGE_SIZE,
        Generation(6),
        item_id,
        device_id,
        &kek,
    )
    .unwrap();
    let payload = save_image_payload(&built.save_image).unwrap();
    let mut vault: VaultFile = serde_json::from_slice(&payload).unwrap();
    for invalid in [
        (" \t", "must not be blank"),
        (
            " io.framkey.local-kek:blank-wrapper-fixture",
            "leading or trailing whitespace",
        ),
        (
            "io.framkey.local-kek:blank-wrapper-fixture\n",
            "leading or trailing whitespace",
        ),
        (
            "io.framkey.local-kek:blank-wrapper\u{7f}-fixture",
            "control characters",
        ),
    ] {
        for wrapper in &mut vault.dek_wrappers {
            if let DekWrapper::MacKeychain {
                keychain_item_id, ..
            } = wrapper
            {
                *keychain_item_id = invalid.0.to_owned();
            }
        }

        let error = vault.validate().unwrap_err();
        assert!(error.to_string().contains(invalid.1));
    }
}

#[test]
fn vault_validation_rejects_inconsistent_recovery_policy() {
    let kek = SecretBytes::new([0xC3; 32]);
    let device_id = [0x33; 16];
    let item_id = "io.framkey.local-kek:policy-fixture";
    let built = build_keychain_encrypted_save_image_with_recovery(
        DEFAULT_FRAM_SAVE_IMAGE_SIZE,
        Generation(6),
        item_id,
        device_id,
        &kek,
    )
    .unwrap();
    let payload = save_image_payload(&built.save_image).unwrap();
    let mut vault: VaultFile = serde_json::from_slice(&payload).unwrap();
    vault.recovery_policy.policy_id = PolicyId::ZERO;

    let error = vault.validate().unwrap_err();
    assert!(error.to_string().contains("recovery wrapper"));
}

#[test]
fn recovery_rewrap_binds_vault_to_new_keychain_without_wallet_secret() {
    let source_kek = SecretBytes::new([0xC3; 32]);
    let source_device_id = [0x33; 16];
    let source_item_id = "io.framkey.local-kek:source-recovery-fixture";
    let built = build_keychain_encrypted_save_image_with_recovery(
        DEFAULT_FRAM_SAVE_IMAGE_SIZE,
        Generation(7),
        source_item_id,
        source_device_id,
        &source_kek,
    )
    .unwrap();
    let backup_pack = built.recovery_backup_pack.as_ref().unwrap();
    let recovery_files = vec![
        backup_pack.files[0].clone(),
        backup_pack.files[1].clone(),
        backup_pack.files[2].clone(),
    ];
    let target_kek = SecretBytes::new([0xA5; 32]);
    let target_device_id = [0x44; 16];
    let target_item_id = "io.framkey.local-kek:target-recovery-fixture";

    let recovered = rewrap_keychain_vault_with_recovery(
        &built.save_image,
        &recovery_files,
        target_item_id,
        target_device_id,
        &target_kek,
    )
    .unwrap();

    assert_eq!(recovered.metadata.image_size, DEFAULT_FRAM_SAVE_IMAGE_SIZE);
    assert_eq!(recovered.metadata.generation, 7);
    assert_eq!(recovered.metadata.wallet_id, built.metadata.wallet_id);
    assert_eq!(recovered.metadata.keychain_item_id, target_item_id);
    assert_eq!(recovered.metadata.device_id, encode_hex(&target_device_id));
    assert!(recovered.metadata.payload_hash_valid);
    assert_eq!(recovered.metadata.recovered_shard_count, 0);

    let opened = open_keychain_encrypted_save_image(
        &recovered.save_image,
        target_item_id,
        target_device_id,
        &target_kek,
    )
    .unwrap();
    assert_eq!(opened.wallet_secret_hash, built.metadata.wallet_secret_hash);

    assert!(
        rewrap_keychain_vault_with_recovery(
            &built.save_image,
            &backup_pack.files[0..2],
            target_item_id,
            target_device_id,
            &target_kek,
        )
        .is_err()
    );
}

#[test]
fn recovery_rewrap_tries_valid_group_pair_when_another_group_is_corrupted() {
    let source_kek = SecretBytes::new([0xC3; 32]);
    let source_device_id = [0x33; 16];
    let source_item_id = "io.framkey.local-kek:source-corrupt-recovery-fixture";
    let built = build_keychain_encrypted_save_image_with_recovery(
        DEFAULT_FRAM_SAVE_IMAGE_SIZE,
        Generation(8),
        source_item_id,
        source_device_id,
        &source_kek,
    )
    .unwrap();
    let backup_pack = built.recovery_backup_pack.as_ref().unwrap();
    let mut corrupted_local = backup_pack.files[2].clone();
    corrupted_local.share_hex.replace_range(0..2, "00");
    let recovery_files = vec![
        backup_pack.files[0].clone(),
        backup_pack.files[1].clone(),
        corrupted_local,
        backup_pack.files[3].clone(),
    ];
    let target_kek = SecretBytes::new([0xA5; 32]);
    let target_device_id = [0x44; 16];
    let target_item_id = "io.framkey.local-kek:target-corrupt-recovery-fixture";

    let recovered = rewrap_keychain_vault_with_recovery(
        &built.save_image,
        &recovery_files,
        target_item_id,
        target_device_id,
        &target_kek,
    )
    .unwrap();
    let opened = open_keychain_encrypted_save_image(
        &recovered.save_image,
        target_item_id,
        target_device_id,
        &target_kek,
    )
    .unwrap();

    assert_eq!(opened.wallet_secret_hash, built.metadata.wallet_secret_hash);
}

#[test]
fn keychain_wallet_secret_can_sign_and_recover_personal_message() {
    use framkey_evm::{personal_sign, recover_personal_signer};

    let kek = SecretBytes::new([0xC3; 32]);
    let device_id = [0x22; 16];
    let item_id = "io.framkey.local-kek:signer-fixture";
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
            assert_eq!(metadata.wallet_type, WalletType::Secp256k1SingleKey);
            personal_sign(secret, message)
        },
    )
    .unwrap();
    let recovered = recover_personal_signer(message, &signed.signature).unwrap();

    assert_eq!(metadata.wallet_id, built.metadata.wallet_id);
    assert_eq!(recovered, signed.address);
}
