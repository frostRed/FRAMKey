use std::{
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Result;
use framkey_crypto::encode_hex;
use framkey_recovery::{
    RecoveryBackupBundle, RecoveryBackupFile, RecoveryBackupPack, RecoveryPolicy,
    parse_recovery_backup_bundle, recovery_backup_file_name,
};
use serde_json::json;

use crate::args::RecoveryCommand;

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
    std::fs::create_dir_all(out_dir)?;

    let mut files = Vec::new();
    for file in &pack.files {
        let path = out_dir.join(recovery_backup_file_name(file));
        let bundle = RecoveryBackupBundle::new(file.clone(), encrypted_vault_backup);
        let bytes = serde_json::to_vec(&bundle)?;
        write_new_file(&path, &bytes)?;
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

pub(crate) fn write_new_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| anyhow::anyhow!("failed to create {}: {error}", path.display()))?;
    file.write_all(bytes)?;
    file.flush()?;
    Ok(())
}
