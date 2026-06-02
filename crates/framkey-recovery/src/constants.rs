pub const RECOVERY_BACKUP_MANIFEST_FORMAT: &str = "framkey.recovery_backup_manifest";
pub const RECOVERY_BACKUP_SHARE_FORMAT: &str = "framkey.recovery_backup_share";
pub const RECOVERY_BACKUP_BUNDLE_FORMAT: &str = "framkey.recovery_backup_bundle";
pub const RECOVERY_BACKUP_FORMAT_VERSION: u16 = 2;
pub const RECOVERY_ROOT_KEY_BYTES: usize = 32;

pub(crate) const RECOVERY_MEMBER_SHARE_ENCODING: &str = "masked_xor_hex_v1";
pub(crate) const RECOVERY_BUNDLE_VAULT_BACKUP_ENCODING: &str = "hex";
