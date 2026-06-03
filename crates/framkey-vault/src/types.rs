use std::fmt;

use framkey_core::{
    FramkeyError, Generation, PolicyId, Result, UnixTimestamp, WalletId, WalletType,
};
use framkey_crypto::AeadBox;
use framkey_recovery::RecoveryBackupPack;
use serde::{Deserialize, Serialize};

use crate::constants::{VAULT_FORMAT_VERSION, VAULT_MAGIC};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VaultFile {
    pub magic: [u8; 8],
    pub format_version: u16,
    pub wallet_id: WalletId,
    pub generation: Generation,
    pub created_at: UnixTimestamp,
    pub updated_at: UnixTimestamp,
    pub wallet_type: WalletType,
    pub public_address: Option<[u8; 20]>,
    pub encrypted_wallet_secret: AeadBox,
    pub dek_wrappers: Vec<DekWrapper>,
    pub recovery_policy: RecoveryPolicyDescriptor,
}

impl VaultFile {
    pub fn validate(&self) -> Result<()> {
        if self.magic != VAULT_MAGIC {
            return Err(FramkeyError::invalid_data("vault magic mismatch"));
        }

        if self.format_version != VAULT_FORMAT_VERSION {
            return Err(FramkeyError::unsupported(format!(
                "vault format version {}",
                self.format_version
            )));
        }

        if self.dek_wrappers.is_empty() {
            return Err(FramkeyError::invalid_data(
                "vault must contain at least one DEK wrapper",
            ));
        }

        for wrapper in &self.dek_wrappers {
            wrapper.validate()?;
        }

        self.validate_recovery_policy()?;

        Ok(())
    }

    fn validate_recovery_policy(&self) -> Result<()> {
        if self.recovery_policy.label.trim().is_empty() {
            return Err(FramkeyError::invalid_data(
                "vault recovery policy label must not be blank",
            ));
        }

        let matching_recovery_wrapper = self.dek_wrappers.iter().any(|wrapper| {
            matches!(
                wrapper,
                DekWrapper::Recovery { policy_id, .. } if *policy_id == self.recovery_policy.policy_id
            )
        });
        let contains_recovery_wrapper = self
            .dek_wrappers
            .iter()
            .any(|wrapper| matches!(wrapper, DekWrapper::Recovery { .. }));

        if self.recovery_policy.policy_id == PolicyId::ZERO {
            if contains_recovery_wrapper {
                return Err(FramkeyError::invalid_data(
                    "vault has recovery wrapper but no recovery policy id",
                ));
            }
        } else if !matching_recovery_wrapper {
            return Err(FramkeyError::invalid_data(
                "vault recovery policy id does not match a recovery DEK wrapper",
            ));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
#[non_exhaustive]
pub enum DekWrapper {
    MacKeychain {
        device_id: [u8; 16],
        keychain_item_id: String,
        encrypted_dek: AeadBox,
    },
    DevTest {
        label: String,
        key_id: [u8; 16],
        encrypted_dek: AeadBox,
    },
    Recovery {
        policy_id: PolicyId,
        encrypted_dek: AeadBox,
    },
}

impl DekWrapper {
    fn validate(&self) -> Result<()> {
        match self {
            Self::MacKeychain {
                keychain_item_id, ..
            } => validate_keychain_item_id(keychain_item_id),
            Self::DevTest { label, .. } => {
                if label.trim().is_empty() {
                    return Err(FramkeyError::invalid_data(
                        "dev/test DEK wrapper label must not be blank",
                    ));
                }
                Ok(())
            }
            Self::Recovery { .. } => Ok(()),
        }
    }
}

pub(crate) fn validate_keychain_item_id(keychain_item_id: &str) -> Result<()> {
    if keychain_item_id.trim().is_empty() {
        return Err(FramkeyError::invalid_data(
            "macOS Keychain item id must not be blank",
        ));
    }
    if keychain_item_id.trim() != keychain_item_id {
        return Err(FramkeyError::invalid_data(
            "macOS Keychain item id must not have leading or trailing whitespace",
        ));
    }
    if keychain_item_id.chars().any(char::is_control) {
        return Err(FramkeyError::invalid_data(
            "macOS Keychain item id must not contain control characters",
        ));
    }
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryPolicyDescriptor {
    pub policy_id: PolicyId,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveImageInspection {
    pub image_size: usize,
    pub header_len: usize,
    pub format_version: u16,
    pub generation: u64,
    pub payload_len: usize,
    pub payload_hash: String,
    pub payload_hash_valid: bool,
    pub data_shards: usize,
    pub parity_shards: usize,
    pub shard_size: usize,
    pub valid_shard_count: usize,
    pub recovered_shard_count: usize,
    pub superblocks: Vec<SaveSuperblockInspection>,
    pub shards: Vec<SaveShardInspection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveSuperblockInspection {
    pub copy_index: usize,
    pub valid: bool,
    pub generation: Option<u64>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveShardInspection {
    pub shard_index: usize,
    pub is_data_shard: bool,
    pub hash: String,
    pub hash_valid: bool,
    pub recovered: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct TestVaultPayload<'a> {
    pub(crate) kind: &'static str,
    pub(crate) label: &'a str,
    pub(crate) generation: u64,
    pub(crate) note: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevEncryptedVaultMetadata {
    pub image_size: usize,
    pub shard_size: usize,
    pub data_shards: usize,
    pub parity_shards: usize,
    pub wallet_id: String,
    pub generation: u64,
    pub wallet_type: WalletType,
    pub dev_wrapper_label: String,
    pub dev_key_id: String,
    pub wallet_secret_hash: String,
    pub payload_hash_valid: bool,
    pub recovered_shard_count: usize,
}

#[derive(Clone, PartialEq, Eq)]
pub struct DevEncryptedVaultImage {
    pub save_image: Vec<u8>,
    pub metadata: DevEncryptedVaultMetadata,
}

impl fmt::Debug for DevEncryptedVaultImage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DevEncryptedVaultImage")
            .field("save_image_len", &self.save_image.len())
            .field("metadata", &self.metadata)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeychainEncryptedVaultMetadata {
    pub image_size: usize,
    pub shard_size: usize,
    pub data_shards: usize,
    pub parity_shards: usize,
    pub wallet_id: String,
    pub generation: u64,
    pub wallet_type: WalletType,
    pub keychain_item_id: String,
    pub device_id: String,
    pub wallet_secret_hash: String,
    pub payload_hash_valid: bool,
    pub recovered_shard_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeychainVaultMetadata {
    pub image_size: usize,
    pub shard_size: usize,
    pub data_shards: usize,
    pub parity_shards: usize,
    pub wallet_id: String,
    pub generation: u64,
    pub wallet_type: WalletType,
    pub keychain_item_id: String,
    pub device_id: String,
    pub payload_hash_valid: bool,
    pub recovered_shard_count: usize,
}

#[derive(Clone, PartialEq, Eq)]
pub struct KeychainEncryptedVaultImage {
    pub save_image: Vec<u8>,
    pub metadata: KeychainEncryptedVaultMetadata,
    pub recovery_backup_pack: Option<RecoveryBackupPack>,
}

impl fmt::Debug for KeychainEncryptedVaultImage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let recovery_backup_file_count = self
            .recovery_backup_pack
            .as_ref()
            .map(|pack| pack.files.len());
        formatter
            .debug_struct("KeychainEncryptedVaultImage")
            .field("save_image_len", &self.save_image.len())
            .field("metadata", &self.metadata)
            .field("recovery_backup_file_count", &recovery_backup_file_count)
            .finish()
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct RecoveryRewrappedKeychainVaultImage {
    pub save_image: Vec<u8>,
    pub metadata: KeychainVaultMetadata,
}

impl fmt::Debug for RecoveryRewrappedKeychainVaultImage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RecoveryRewrappedKeychainVaultImage")
            .field("save_image_len", &self.save_image.len())
            .field("metadata", &self.metadata)
            .finish()
    }
}
