use super::*;
use crate::{constants::RECOVERY_MEMBER_SHARE_ENCODING, shares::interpolate_at_zero};

#[test]
fn cloud_group_alone_cannot_recover() {
    let policy = RecoveryPolicy::standard_cloud_plus_physical();
    assert!(!policy.can_recover_groups([RecoveryGroupKind::Cloud]));
}

#[test]
fn cloud_plus_local_can_recover() {
    let policy = RecoveryPolicy::standard_cloud_plus_physical();
    assert!(
        policy.can_recover_groups([RecoveryGroupKind::Cloud, RecoveryGroupKind::LocalPhysical,])
    );
}

#[test]
fn local_plus_remote_can_recover() {
    let policy = RecoveryPolicy::standard_cloud_plus_physical();
    assert!(policy.can_recover_groups([
        RecoveryGroupKind::LocalPhysical,
        RecoveryGroupKind::RemotePhysical,
    ]));
}

#[test]
fn backup_pack_matches_policy_matrix() {
    let root_key = [0x42_u8; RECOVERY_ROOT_KEY_BYTES];
    let pack = RecoveryBackupPack::standard(
        [0x11; 16],
        7,
        [0x22; 16],
        [0x33; 16],
        1234,
        &root_key,
        RecoveryBackupEntropy {
            group_polynomial_coefficients: [0x55; RECOVERY_ROOT_KEY_BYTES],
            cloud_member_pad: [0xAA; RECOVERY_ROOT_KEY_BYTES],
        },
    );

    assert_eq!(pack.files.len(), 4);
    assert_eq!(pack.manifest.files.len(), 4);
    assert!(
        pack.files
            .iter()
            .all(|file| file.share_encoding == RECOVERY_MEMBER_SHARE_ENCODING)
    );
    assert_eq!(recovery_backup_file_name(&pack.files[0]), "backup-01.dat");
    assert_eq!(recovery_backup_file_name(&pack.files[1]), "backup-02.dat");
    assert_eq!(recovery_backup_file_name(&pack.files[2]), "backup-03.dat");
    assert_eq!(recovery_backup_file_name(&pack.files[3]), "backup-04.dat");
    assert!(reconstruct_recovery_root_key(&pack.files[0..2]).is_err());
    assert!(reconstruct_recovery_root_key(&pack.files[2..3]).is_err());

    let cloud_plus_local = vec![
        pack.files[0].clone(),
        pack.files[1].clone(),
        pack.files[2].clone(),
    ];
    let cloud_plus_remote = vec![
        pack.files[0].clone(),
        pack.files[1].clone(),
        pack.files[3].clone(),
    ];
    let local_plus_remote = vec![pack.files[2].clone(), pack.files[3].clone()];

    assert_eq!(
        reconstruct_recovery_root_key(&cloud_plus_local).unwrap(),
        root_key
    );
    assert_eq!(
        reconstruct_recovery_root_key(&cloud_plus_remote).unwrap(),
        root_key
    );
    assert_eq!(
        reconstruct_recovery_root_key(&local_plus_remote).unwrap(),
        root_key
    );

    let bundle = RecoveryBackupBundle::new(pack.files[0].clone(), b"encrypted vault");
    let bundle_bytes = serde_json::to_vec(&bundle).unwrap();
    let parsed = parse_recovery_backup_bundle(&bundle_bytes).unwrap();
    assert_eq!(parsed.recovery_file().unwrap(), pack.files[0]);
    assert_eq!(
        parsed.encrypted_vault_backup_bytes().unwrap(),
        b"encrypted vault"
    );
}

#[test]
fn debug_output_redacts_recovery_material() {
    let root_key = [0x42_u8; RECOVERY_ROOT_KEY_BYTES];
    let entropy = RecoveryBackupEntropy {
        group_polynomial_coefficients: [0x55; RECOVERY_ROOT_KEY_BYTES],
        cloud_member_pad: [0xAA; RECOVERY_ROOT_KEY_BYTES],
    };
    let pack = RecoveryBackupPack::standard(
        [0x11; 16], 7, [0x22; 16], [0x33; 16], 1234, &root_key, entropy,
    );
    let share_hex = pack.files[0].share_hex.clone();
    let bundle = RecoveryBackupBundle::new(pack.files[0].clone(), b"encrypted vault");

    let pack_debug = format!("{pack:?}");
    let bundle_debug = format!("{bundle:?}");
    let entropy_debug = format!("{entropy:?}");

    assert!(!pack_debug.contains(&share_hex));
    assert!(!bundle_debug.contains(&share_hex));
    assert!(!bundle_debug.contains("656e63727970746564207661756c74"));
    assert!(!entropy_debug.contains("55555555"));
    assert!(!entropy_debug.contains("aaaaaaaa"));
}

#[test]
fn duplicate_group_shares_are_rejected() {
    let error = interpolate_at_zero(&[(1, 2), (1, 3)]).unwrap_err();
    assert!(error.to_string().contains("duplicate"));
}

#[test]
fn duplicate_member_shares_are_rejected() {
    let root_key = [0x42_u8; RECOVERY_ROOT_KEY_BYTES];
    let pack = RecoveryBackupPack::standard(
        [0x11; 16],
        7,
        [0x22; 16],
        [0x33; 16],
        1234,
        &root_key,
        RecoveryBackupEntropy {
            group_polynomial_coefficients: [0x55; RECOVERY_ROOT_KEY_BYTES],
            cloud_member_pad: [0xAA; RECOVERY_ROOT_KEY_BYTES],
        },
    );
    let duplicate_local_member = vec![
        pack.files[0].clone(),
        pack.files[1].clone(),
        pack.files[2].clone(),
        pack.files[2].clone(),
    ];

    let error = reconstruct_recovery_root_key(&duplicate_local_member).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("duplicate recovery group member")
    );
}

#[test]
fn root_key_candidates_include_valid_pair_when_one_satisfied_group_is_bad() {
    let root_key = [0x42_u8; RECOVERY_ROOT_KEY_BYTES];
    let pack = RecoveryBackupPack::standard(
        [0x11; 16],
        7,
        [0x22; 16],
        [0x33; 16],
        1234,
        &root_key,
        RecoveryBackupEntropy {
            group_polynomial_coefficients: [0x55; RECOVERY_ROOT_KEY_BYTES],
            cloud_member_pad: [0xAA; RECOVERY_ROOT_KEY_BYTES],
        },
    );
    let mut bad_local = pack.files[2].clone();
    bad_local.share_hex.replace_range(0..2, "00");
    let files = vec![
        pack.files[0].clone(),
        pack.files[1].clone(),
        bad_local,
        pack.files[3].clone(),
    ];

    let candidates = reconstruct_recovery_root_key_candidates(&files).unwrap();

    assert_eq!(candidates.len(), 3);
    assert!(candidates.contains(&root_key));
}
