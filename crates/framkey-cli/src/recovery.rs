use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use anyhow::Result;
use framkey_crypto::encode_hex;
use framkey_recovery::{
    RecoveryBackupBundle, RecoveryBackupFile, RecoveryBackupPack, RecoveryPolicy,
    parse_recovery_backup_bundle, recovery_backup_file_name,
};
use serde_json::json;

use crate::{
    args::RecoveryCommand,
    files::{create_private_dir_all, write_new_file},
};

pub(crate) fn run_recovery(command: RecoveryCommand) -> Result<()> {
    match command {
        RecoveryCommand::Policy => {
            let policy = RecoveryPolicy::standard_cloud_plus_physical();
            println!("{}", serde_json::to_string_pretty(&policy)?);
        }
    }

    Ok(())
}

pub(crate) fn write_recovery_backup_pack(
    out_dir: &Path,
    pack: &RecoveryBackupPack,
    encrypted_vault_backup: &[u8],
) -> Result<serde_json::Value> {
    validate_recovery_backup_pack_targets(pack)?;
    create_private_dir_all(out_dir)?;

    let mut created_paths = Vec::new();
    match write_recovery_backup_pack_files(
        out_dir,
        pack,
        encrypted_vault_backup,
        &mut created_paths,
    ) {
        Ok(summary) => Ok(summary),
        Err(error) => {
            cleanup_paths(&created_paths)?;
            Err(error)
        }
    }
}

fn write_recovery_backup_pack_files(
    out_dir: &Path,
    pack: &RecoveryBackupPack,
    encrypted_vault_backup: &[u8],
    created_paths: &mut Vec<PathBuf>,
) -> Result<serde_json::Value> {
    let mut files = Vec::new();
    for file in &pack.files {
        let path = out_dir.join(recovery_backup_file_name(file));
        let bundle = RecoveryBackupBundle::new(file.clone(), encrypted_vault_backup);
        let bytes = serde_json::to_vec(&bundle)?;
        write_new_file(&path, &bytes)?;
        created_paths.push(path.clone());
        files.push(json!({
            "kind": "bundle",
            "path": path.display().to_string(),
            "blake3": encode_hex(blake3::hash(&bytes).as_bytes()),
            "group": file.group_kind.as_str(),
            "member": file.member_label,
            "share_bytes_printed": false,
            "encrypted_vault_data": "embedded",
        }));
    }

    Ok(json!({
        "out_dir": out_dir.display().to_string(),
        "backup_set_id": pack.manifest.backup_set_id,
        "policy_id": pack.manifest.policy_id,
        "wallet_id": pack.manifest.wallet_id,
        "generation": pack.manifest.generation,
        "share_file_count": pack.files.len(),
        "backup_file_count": pack.files.len(),
        "embedded_vault_backup_count": pack.files.len(),
        "files": files,
        "cloud_alone_recovers": false,
    }))
}

fn validate_recovery_backup_pack_targets(pack: &RecoveryBackupPack) -> Result<()> {
    if pack.files.len() != 4 {
        anyhow::bail!(
            "signer helper returned {} recovery backup files, but this CLI requires exactly four; rebuild the signer helper to match the CLI",
            pack.files.len()
        );
    }

    let mut names = BTreeSet::new();
    for file in &pack.files {
        let name = recovery_backup_file_name(file);
        if !names.insert(name.clone()) {
            anyhow::bail!(
                "signer helper returned a recovery pack that maps multiple backup files to {name}; rebuild the signer helper to match the CLI"
            );
        }
    }

    Ok(())
}

fn cleanup_paths(paths: &[PathBuf]) -> Result<()> {
    for path in paths.iter().rev() {
        match std::fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                anyhow::bail!(
                    "failed to remove partial recovery backup {}: {error}",
                    path.display()
                );
            }
        }
    }
    Ok(())
}

pub(crate) fn read_recovery_backup_files(paths: &[PathBuf]) -> Result<Vec<RecoveryBackupFile>> {
    if paths.is_empty() {
        anyhow::bail!("pass at least one --recovery-file");
    }
    if paths.len() > 4 {
        anyhow::bail!("standard recovery accepts at most four --recovery-file values");
    }

    paths
        .iter()
        .map(|path| {
            let bytes = std::fs::read(path)
                .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", path.display()))?;
            parse_recovery_backup_bundle(&bytes)
                .map(|bundle| bundle.recovery_file)
                .map_err(|error| {
                    anyhow::anyhow!(
                        "failed to parse recovery backup {}: {error}",
                        path.display()
                    )
                })
        })
        .collect()
}

pub(crate) fn read_encrypted_vault_backup_from_bundle(path: &Path) -> Result<Vec<u8>> {
    let bytes = std::fs::read(path)
        .map_err(|error| anyhow::anyhow!("failed to read {}: {error}", path.display()))?;
    let bundle = parse_recovery_backup_bundle(&bytes).map_err(|error| {
        anyhow::anyhow!(
            "failed to parse recovery backup {}: {error}",
            path.display()
        )
    })?;
    bundle.encrypted_vault_backup_bytes().map_err(|error| {
        anyhow::anyhow!(
            "failed to read encrypted vault data from {}: {error}",
            path.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use framkey_recovery::{RecoveryBackupEntropy, RecoveryGroupKind};

    fn test_pack() -> RecoveryBackupPack {
        RecoveryBackupPack::standard(
            [1_u8; 16],
            1,
            [2_u8; 16],
            [3_u8; 16],
            1_700_000_000,
            &[4_u8; 32],
            RecoveryBackupEntropy {
                group_polynomial_coefficients: [5_u8; 32],
                cloud_member_pad: [6_u8; 32],
            },
        )
    }

    fn test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "framkey-cli-recovery-{name}-{}",
            std::process::id()
        ))
    }

    #[test]
    fn recovery_backup_pack_targets_reject_duplicate_names_before_writing() {
        let mut pack = test_pack();
        pack.files[2].group_kind = RecoveryGroupKind::RemotePhysical;

        let error = validate_recovery_backup_pack_targets(&pack).unwrap_err();

        assert!(error.to_string().contains("maps multiple backup files"));
    }

    #[test]
    fn recovery_backup_pack_failure_removes_partial_new_files() {
        let pack = test_pack();
        let dir = test_dir("cleanup");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("backup-02.dat"), b"existing").unwrap();

        let error = write_recovery_backup_pack(&dir, &pack, b"encrypted vault backup").unwrap_err();

        assert!(error.to_string().contains("failed to create"));
        assert!(!dir.join("backup-01.dat").exists());
        assert_eq!(
            std::fs::read(dir.join("backup-02.dat")).unwrap(),
            b"existing"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }
}
