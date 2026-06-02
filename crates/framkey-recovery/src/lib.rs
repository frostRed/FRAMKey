mod bundle;
mod constants;
mod encoding;
mod policy;
mod shares;

pub use bundle::{
    RecoveryBackupBundle, RecoveryBackupEntropy, RecoveryBackupFile, RecoveryBackupFileDescriptor,
    RecoveryBackupManifest, RecoveryBackupPack, RecoveryBackupVaultBackup,
    parse_recovery_backup_bundle, recovery_backup_file_name,
};
pub use constants::{
    RECOVERY_BACKUP_BUNDLE_FORMAT, RECOVERY_BACKUP_FORMAT_VERSION, RECOVERY_BACKUP_MANIFEST_FORMAT,
    RECOVERY_BACKUP_SHARE_FORMAT, RECOVERY_ROOT_KEY_BYTES,
};
pub use policy::{RecoveryGroupKind, RecoveryGroupPolicy, RecoveryPolicy};
pub use shares::{reconstruct_recovery_root_key, reconstruct_recovery_root_key_candidates};

#[cfg(test)]
mod tests;
