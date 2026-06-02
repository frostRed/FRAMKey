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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryPolicyDescriptor {
    pub policy_id: PolicyId,
    pub label: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveImageHeader {
    pub magic: [u8; 8],
    pub format_version: u16,
    pub active_slot: SaveSlot,
    pub latest_generation: Generation,
    pub save_image_size: u32,
    pub checksum: [u8; 32],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SaveSlot {
    A,
    B,
}

impl SaveSlot {
    pub(crate) fn index(self) -> u8 {
        match self {
            Self::A => 0,
            Self::B => 1,
        }
    }

    pub(crate) fn from_index(index: u8) -> Result<Self> {
        match index {
            0 => Ok(Self::A),
            1 => Ok(Self::B),
            _ => Err(FramkeyError::invalid_data(format!(
                "invalid save slot index {index}"
            ))),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveImageInspection {
    pub image_size: usize,
    pub header_len: usize,
    pub slot_size: usize,
    pub active_slot: SaveSlot,
    pub latest_generation: u64,
    pub active_slot_hash: String,
    pub active_slot_hash_valid: bool,
    pub slots: Vec<SaveSlotInspection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SaveSlotInspection {
    pub slot: SaveSlot,
    pub generation: u64,
    pub payload_len: usize,
    pub payload_hash: String,
    pub payload_hash_valid: bool,
    pub payload_preview: String,
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
    pub slot_size: usize,
    pub wallet_id: String,
    pub generation: u64,
    pub wallet_type: WalletType,
    pub dev_wrapper_label: String,
    pub dev_key_id: String,
    pub wallet_secret_hash: String,
    pub active_slot_hash_valid: bool,
    pub active_slot_payload_hash_valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DevEncryptedVaultImage {
    pub save_image: Vec<u8>,
    pub metadata: DevEncryptedVaultMetadata,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeychainEncryptedVaultMetadata {
    pub image_size: usize,
    pub slot_size: usize,
    pub wallet_id: String,
    pub generation: u64,
    pub wallet_type: WalletType,
    pub keychain_item_id: String,
    pub device_id: String,
    pub wallet_secret_hash: String,
    pub active_slot_hash_valid: bool,
    pub active_slot_payload_hash_valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeychainVaultMetadata {
    pub image_size: usize,
    pub slot_size: usize,
    pub wallet_id: String,
    pub generation: u64,
    pub wallet_type: WalletType,
    pub keychain_item_id: String,
    pub device_id: String,
    pub active_slot_hash_valid: bool,
    pub active_slot_payload_hash_valid: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeychainEncryptedVaultImage {
    pub save_image: Vec<u8>,
    pub metadata: KeychainEncryptedVaultMetadata,
    pub recovery_backup_pack: Option<RecoveryBackupPack>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryRewrappedKeychainVaultImage {
    pub save_image: Vec<u8>,
    pub metadata: KeychainVaultMetadata,
}
