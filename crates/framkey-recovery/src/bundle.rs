use std::fmt;

use framkey_core::{FramkeyError, Result};
use serde::{Deserialize, Serialize};

use crate::{
    constants::{
        RECOVERY_BACKUP_BUNDLE_FORMAT, RECOVERY_BACKUP_FORMAT_VERSION,
        RECOVERY_BACKUP_MANIFEST_FORMAT, RECOVERY_BACKUP_SHARE_FORMAT,
        RECOVERY_BUNDLE_VAULT_BACKUP_ENCODING, RECOVERY_MEMBER_SHARE_ENCODING,
        RECOVERY_ROOT_KEY_BYTES,
    },
    encoding::{decode_hex, encode_hex, share_hash},
    policy::{RecoveryGroupKind, RecoveryPolicy},
    shares::{
        MemberShareMaskInput, encode_member_share, group_share, validate_recovery_backup_file,
        xor_32,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryBackupPack {
    pub manifest: RecoveryBackupManifest,
    pub files: Vec<RecoveryBackupFile>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryBackupManifest {
    pub format: String,
    pub format_version: u16,
    pub backup_set_id: String,
    pub wallet_id: String,
    pub generation: u64,
    pub policy_id: String,
    pub created_at_unix: u64,
    pub policy: RecoveryPolicy,
    pub files: Vec<RecoveryBackupFileDescriptor>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryBackupFileDescriptor {
    pub file_name: String,
    pub group_kind: RecoveryGroupKind,
    pub group_label: String,
    pub member_index: u8,
    pub member_label: String,
    pub share_hash: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryBackupFile {
    pub format: String,
    pub format_version: u16,
    pub backup_set_id: String,
    pub wallet_id: String,
    pub generation: u64,
    pub policy_id: String,
    pub group_kind: RecoveryGroupKind,
    pub group_label: String,
    pub group_share_index: u8,
    pub group_threshold: u8,
    pub member_index: u8,
    pub member_threshold: u8,
    pub member_count: u8,
    pub member_label: String,
    pub share_encoding: String,
    pub share_hex: String,
    pub instructions: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryBackupBundle {
    pub format: String,
    pub format_version: u16,
    pub backup_set_id: String,
    pub wallet_id: String,
    pub generation: u64,
    pub policy_id: String,
    pub recovery_file: RecoveryBackupFile,
    pub encrypted_vault_backup: RecoveryBackupVaultBackup,
    pub instructions: String,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecoveryBackupVaultBackup {
    pub encoding: String,
    pub byte_count: u64,
    pub blake3: String,
    pub bytes_hex: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct RecoveryBackupEntropy {
    pub group_polynomial_coefficients: [u8; RECOVERY_ROOT_KEY_BYTES],
    pub cloud_member_pad: [u8; RECOVERY_ROOT_KEY_BYTES],
}

impl fmt::Debug for RecoveryBackupFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RecoveryBackupFile")
            .field("format", &self.format)
            .field("format_version", &self.format_version)
            .field("backup_set_id", &self.backup_set_id)
            .field("wallet_id", &self.wallet_id)
            .field("generation", &self.generation)
            .field("policy_id", &self.policy_id)
            .field("group_kind", &self.group_kind)
            .field("group_label", &self.group_label)
            .field("group_share_index", &self.group_share_index)
            .field("group_threshold", &self.group_threshold)
            .field("member_index", &self.member_index)
            .field("member_threshold", &self.member_threshold)
            .field("member_count", &self.member_count)
            .field("member_label", &self.member_label)
            .field("share_encoding", &self.share_encoding)
            .field("share_hash", &share_hash(&self.share_hex))
            .field("instructions", &self.instructions)
            .finish()
    }
}

impl fmt::Debug for RecoveryBackupVaultBackup {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RecoveryBackupVaultBackup")
            .field("encoding", &self.encoding)
            .field("byte_count", &self.byte_count)
            .field("blake3", &self.blake3)
            .finish_non_exhaustive()
    }
}

impl fmt::Debug for RecoveryBackupEntropy {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RecoveryBackupEntropy")
            .field("root_key_bytes", &RECOVERY_ROOT_KEY_BYTES)
            .finish_non_exhaustive()
    }
}

impl RecoveryBackupBundle {
    pub fn new(recovery_file: RecoveryBackupFile, encrypted_vault_backup: &[u8]) -> Self {
        Self {
            format: RECOVERY_BACKUP_BUNDLE_FORMAT.to_owned(),
            format_version: RECOVERY_BACKUP_FORMAT_VERSION,
            backup_set_id: recovery_file.backup_set_id.clone(),
            wallet_id: recovery_file.wallet_id.clone(),
            generation: recovery_file.generation,
            policy_id: recovery_file.policy_id.clone(),
            instructions: bundle_instructions(&recovery_file),
            recovery_file,
            encrypted_vault_backup: RecoveryBackupVaultBackup::new(encrypted_vault_backup),
        }
    }

    pub fn validate(&self) -> Result<()> {
        if self.format != RECOVERY_BACKUP_BUNDLE_FORMAT {
            return Err(FramkeyError::invalid_data(
                "recovery backup bundle format mismatch",
            ));
        }
        if self.format_version != RECOVERY_BACKUP_FORMAT_VERSION {
            return Err(FramkeyError::unsupported(format!(
                "recovery backup bundle format version {}",
                self.format_version
            )));
        }
        if self.backup_set_id != self.recovery_file.backup_set_id
            || self.wallet_id != self.recovery_file.wallet_id
            || self.generation != self.recovery_file.generation
            || self.policy_id != self.recovery_file.policy_id
        {
            return Err(FramkeyError::invalid_data(
                "recovery backup bundle metadata mismatch",
            ));
        }
        validate_recovery_backup_file(&self.recovery_file)?;
        self.encrypted_vault_backup.bytes()?;
        Ok(())
    }

    pub fn recovery_file(&self) -> Result<RecoveryBackupFile> {
        self.validate()?;
        Ok(self.recovery_file.clone())
    }

    pub fn encrypted_vault_backup_bytes(&self) -> Result<Vec<u8>> {
        self.validate()?;
        self.encrypted_vault_backup.bytes()
    }
}

pub fn parse_recovery_backup_bundle(bytes: &[u8]) -> Result<RecoveryBackupBundle> {
    let bundle: RecoveryBackupBundle = serde_json::from_slice(bytes)
        .map_err(|error| FramkeyError::invalid_data(format!("recovery backup bundle: {error}")))?;
    bundle.validate()?;
    Ok(bundle)
}

impl RecoveryBackupVaultBackup {
    pub fn new(bytes: &[u8]) -> Self {
        Self {
            encoding: RECOVERY_BUNDLE_VAULT_BACKUP_ENCODING.to_owned(),
            byte_count: bytes.len() as u64,
            blake3: encode_hex(blake3::hash(bytes).as_bytes()),
            bytes_hex: encode_hex(bytes),
        }
    }

    pub fn bytes(&self) -> Result<Vec<u8>> {
        if self.encoding != RECOVERY_BUNDLE_VAULT_BACKUP_ENCODING {
            return Err(FramkeyError::unsupported(format!(
                "unsupported encrypted vault backup encoding {}",
                self.encoding
            )));
        }
        let bytes = decode_hex(&self.bytes_hex)?;
        if bytes.len() as u64 != self.byte_count {
            return Err(FramkeyError::invalid_data(
                "encrypted vault backup byte count mismatch",
            ));
        }
        let blake3 = encode_hex(blake3::hash(&bytes).as_bytes());
        if blake3 != self.blake3 {
            return Err(FramkeyError::invalid_data(
                "encrypted vault backup hash mismatch",
            ));
        }
        Ok(bytes)
    }
}

impl RecoveryBackupPack {
    pub fn standard(
        wallet_id: [u8; 16],
        generation: u64,
        policy_id: [u8; 16],
        backup_set_id: [u8; 16],
        created_at_unix: u64,
        recovery_root_key: &[u8; RECOVERY_ROOT_KEY_BYTES],
        entropy: RecoveryBackupEntropy,
    ) -> Self {
        let policy = RecoveryPolicy::standard_cloud_plus_physical();
        let wallet_id = encode_hex(&wallet_id);
        let policy_id = encode_hex(&policy_id);
        let backup_set_id = encode_hex(&backup_set_id);

        let cloud_group_share =
            group_share(recovery_root_key, &entropy.group_polynomial_coefficients, 1);
        let local_group_share =
            group_share(recovery_root_key, &entropy.group_polynomial_coefficients, 2);
        let remote_group_share =
            group_share(recovery_root_key, &entropy.group_polynomial_coefficients, 3);

        let cloud_a = entropy.cloud_member_pad;
        let cloud_b = xor_32(&cloud_group_share, &cloud_a);

        let files = vec![
            share_file(ShareFileSpec {
                backup_set_id: &backup_set_id,
                wallet_id: &wallet_id,
                generation,
                policy_id: &policy_id,
                group_kind: RecoveryGroupKind::Cloud,
                group_label: "Cloud",
                group_share_index: 1,
                group_threshold: 2,
                member_index: 1,
                member_threshold: 2,
                member_count: 2,
                member_label: "iCloud",
                share: cloud_a,
                instructions: "Upload this file to iCloud. It is not sufficient without the Google Drive cloud file and one physical recovery group.",
            }),
            share_file(ShareFileSpec {
                backup_set_id: &backup_set_id,
                wallet_id: &wallet_id,
                generation,
                policy_id: &policy_id,
                group_kind: RecoveryGroupKind::Cloud,
                group_label: "Cloud",
                group_share_index: 1,
                group_threshold: 2,
                member_index: 2,
                member_threshold: 2,
                member_count: 2,
                member_label: "Google Drive",
                share: cloud_b,
                instructions: "Upload this file to Google Drive. It is not sufficient without the iCloud cloud file and one physical recovery group.",
            }),
            share_file(ShareFileSpec {
                backup_set_id: &backup_set_id,
                wallet_id: &wallet_id,
                generation,
                policy_id: &policy_id,
                group_kind: RecoveryGroupKind::LocalPhysical,
                group_label: "Local Physical",
                group_share_index: 2,
                group_threshold: 2,
                member_index: 1,
                member_threshold: 1,
                member_count: 1,
                member_label: "Local Physical",
                share: local_group_share,
                instructions: "Store this file on local physical storage. It is sufficient only with the cloud pair or the off-site physical backup.",
            }),
            share_file(ShareFileSpec {
                backup_set_id: &backup_set_id,
                wallet_id: &wallet_id,
                generation,
                policy_id: &policy_id,
                group_kind: RecoveryGroupKind::RemotePhysical,
                group_label: "Remote Physical",
                group_share_index: 3,
                group_threshold: 2,
                member_index: 1,
                member_threshold: 1,
                member_count: 1,
                member_label: "Off-site Physical",
                share: remote_group_share,
                instructions: "Store this file away from the main Mac and GBA card. It is sufficient only with the cloud pair or the local physical backup.",
            }),
        ];

        let descriptors = files
            .iter()
            .map(|file| RecoveryBackupFileDescriptor {
                file_name: recovery_backup_file_name(file),
                group_kind: file.group_kind,
                group_label: file.group_label.clone(),
                member_index: file.member_index,
                member_label: file.member_label.clone(),
                share_hash: share_hash(&file.share_hex),
            })
            .collect();

        Self {
            manifest: RecoveryBackupManifest {
                format: RECOVERY_BACKUP_MANIFEST_FORMAT.to_owned(),
                format_version: RECOVERY_BACKUP_FORMAT_VERSION,
                backup_set_id,
                wallet_id,
                generation,
                policy_id,
                created_at_unix,
                policy,
                files: descriptors,
            },
            files,
        }
    }
}

pub fn recovery_backup_file_name(file: &RecoveryBackupFile) -> String {
    match file.group_kind {
        RecoveryGroupKind::Cloud if file.member_index == 1 => "backup-01.dat".to_owned(),
        RecoveryGroupKind::Cloud if file.member_index == 2 => "backup-02.dat".to_owned(),
        RecoveryGroupKind::LocalPhysical => "backup-03.dat".to_owned(),
        RecoveryGroupKind::RemotePhysical => "backup-04.dat".to_owned(),
        _ => format!("backup-{:02}.dat", file.member_index),
    }
}

struct ShareFileSpec<'a> {
    backup_set_id: &'a str,
    wallet_id: &'a str,
    generation: u64,
    policy_id: &'a str,
    group_kind: RecoveryGroupKind,
    group_label: &'a str,
    group_share_index: u8,
    group_threshold: u8,
    member_index: u8,
    member_threshold: u8,
    member_count: u8,
    member_label: &'a str,
    share: [u8; RECOVERY_ROOT_KEY_BYTES],
    instructions: &'a str,
}

fn share_file(spec: ShareFileSpec<'_>) -> RecoveryBackupFile {
    let share = encode_member_share(
        &spec.share,
        MemberShareMaskInput {
            backup_set_id: spec.backup_set_id,
            wallet_id: spec.wallet_id,
            generation: spec.generation,
            policy_id: spec.policy_id,
            group_kind: spec.group_kind,
            group_share_index: spec.group_share_index,
            group_threshold: spec.group_threshold,
            member_index: spec.member_index,
            member_threshold: spec.member_threshold,
            member_count: spec.member_count,
            member_label: spec.member_label,
        },
    );
    RecoveryBackupFile {
        format: RECOVERY_BACKUP_SHARE_FORMAT.to_owned(),
        format_version: RECOVERY_BACKUP_FORMAT_VERSION,
        backup_set_id: spec.backup_set_id.to_owned(),
        wallet_id: spec.wallet_id.to_owned(),
        generation: spec.generation,
        policy_id: spec.policy_id.to_owned(),
        group_kind: spec.group_kind,
        group_label: spec.group_label.to_owned(),
        group_share_index: spec.group_share_index,
        group_threshold: spec.group_threshold,
        member_index: spec.member_index,
        member_threshold: spec.member_threshold,
        member_count: spec.member_count,
        member_label: spec.member_label.to_owned(),
        share_encoding: RECOVERY_MEMBER_SHARE_ENCODING.to_owned(),
        share_hex: encode_hex(&share),
        instructions: spec.instructions.to_owned(),
    }
}

fn bundle_instructions(file: &RecoveryBackupFile) -> String {
    match file.group_kind {
        RecoveryGroupKind::Cloud if file.member_index == 1 => {
            "Put this FRAMKey backup file in iCloud Drive. It contains encrypted vault data and Cloud recovery share 1; it cannot recover the wallet by itself.".to_owned()
        }
        RecoveryGroupKind::Cloud if file.member_index == 2 => {
            "Put this FRAMKey backup file in Google Drive. It contains encrypted vault data and Cloud recovery share 2; it cannot recover the wallet by itself.".to_owned()
        }
        RecoveryGroupKind::LocalPhysical => {
            "Copy this FRAMKey backup file to local physical storage. It contains encrypted vault data and the Local recovery share.".to_owned()
        }
        RecoveryGroupKind::RemotePhysical => {
            "Store this FRAMKey backup file off-site. It contains encrypted vault data and the Off-site recovery share.".to_owned()
        }
        _ => file.instructions.clone(),
    }
}
