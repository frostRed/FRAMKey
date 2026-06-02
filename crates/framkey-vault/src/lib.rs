mod constants;
mod dev_vault;
mod keychain_vault;
mod save_image;
mod test_image;
mod types;
mod util;

pub use constants::{
    DEFAULT_FRAM_SAVE_IMAGE_SIZE, SAVE_IMAGE_FORMAT_VERSION, SAVE_IMAGE_HEADER_LEN, SAVE_MAGIC,
    SAVE_SLOT_HEADER_LEN, SAVE_SLOT_MAGIC, VAULT_FORMAT_VERSION, VAULT_MAGIC,
};
pub use dev_vault::{build_dev_encrypted_save_image, open_dev_encrypted_save_image};
pub use keychain_vault::{
    build_keychain_encrypted_save_image, build_keychain_encrypted_save_image_with_recovery,
    open_keychain_encrypted_save_image, rewrap_keychain_vault_with_recovery,
    with_keychain_wallet_secret,
};
pub use save_image::{active_slot_payload, build_save_image_with_payload, inspect_save_image};
pub use test_image::build_test_save_image;
pub use types::{
    DekWrapper, DevEncryptedVaultImage, DevEncryptedVaultMetadata, KeychainEncryptedVaultImage,
    KeychainEncryptedVaultMetadata, KeychainVaultMetadata, RecoveryPolicyDescriptor,
    RecoveryRewrappedKeychainVaultImage, SaveImageHeader, SaveImageInspection, SaveSlot,
    SaveSlotInspection, VaultFile,
};

#[cfg(test)]
mod tests;
