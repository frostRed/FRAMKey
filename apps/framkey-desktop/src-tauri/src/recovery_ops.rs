use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use framkey_crypto::random_array;
use framkey_device::SaveImage;
use framkey_ipc::{
    SignerBuildKeychainVaultResponse, SignerRecoverKeychainVaultResponse,
    SignerValidateRecoveryFilesResponse,
};
use framkey_recovery::{
    RecoveryBackupEntropy, RecoveryBackupFile, RecoveryBackupPack, recovery_backup_file_name,
};
use serde_json::{Value, json};

use crate::*;

pub(crate) fn create_keychain_vault(
    config: &DesktopConfig,
    request: CreateKeychainVaultRequest,
) -> Result<Value> {
    request.validate()?;
    if !request.confirm_overwrite {
        anyhow::bail!(
            "creating a vault requires explicit configured-device overwrite confirmation"
        );
    }

    let recovery_parent_dir = recovery_out_dir_path(&request.recovery_out_dir)?;
    let image_size = config.device.vault_image_size()?;
    let response = build_keychain_vault_with_helper(config, image_size, request.generation)?;
    let SignerBuildKeychainVaultResponse {
        save_image,
        keychain_service,
        keychain_account,
        keychain_item_id,
        keychain_access_policy,
        device_id,
        kek_id,
        created_keychain_kek,
        metadata,
        recovery_backup_pack,
    } = response;

    let pack = recovery_backup_pack.ok_or_else(|| {
        anyhow::anyhow!("signer helper did not return requested recovery backups")
    })?;
    validate_recovery_backup_pack_targets(&pack)?;
    let image = SaveImage::new(save_image);
    let recovery_out_dir = recovery_backup_set_out_dir(&recovery_parent_dir, &pack)?;
    let recovery_backups =
        write_recovery_backup_pack(&recovery_out_dir, &pack, Some(image.as_bytes()))?;
    if let Err(error) = write_configured_save_image(config, &image) {
        let cleanup = cleanup_recovery_backup_pack(&recovery_out_dir, &pack);
        match cleanup {
            Ok(()) => anyhow::bail!(
                "vault image write failed after recovery backups were written; temporary recovery files were removed: {error}"
            ),
            Err(cleanup_error) => anyhow::bail!(
                "vault image write failed after recovery backups were written: {error}; failed to remove recovery files in {}: {cleanup_error}",
                recovery_out_dir.display()
            ),
        }
    }
    let signer_helper = helper_report(&config.helper)?;

    Ok(json!({
        "operation": "create_keychain_vault",
        "device": config.device.describe(),
        "saveSize": image.len(),
        "saveImageBlake3": image.blake3_hash().to_string(),
        "metadata": metadata,
        "keychain": {
            "service": keychain_service,
            "account": keychain_account,
            "itemId": keychain_item_id,
            "accessPolicy": keychain_access_policy,
            "deviceId": device_id,
            "kekId": kek_id,
            "createdKeychainKek": created_keychain_kek,
        },
        "plaintextSecretProcess": "framkey-signer-helper",
        "walletSecretPrinted": false,
        "recoveryBackups": recovery_backups,
        "signerHelper": signer_helper,
    }))
}

pub(crate) fn recover_keychain_vault(
    config: &DesktopConfig,
    request: RecoverKeychainVaultRequest,
) -> Result<Value> {
    request.validate()?;
    if !request.confirm_overwrite {
        anyhow::bail!(
            "recovering a vault requires explicit configured-device overwrite confirmation"
        );
    }

    let vault_backup_path = request.vault_backup_path()?;
    let recovery_paths = request.recovery_file_paths()?;
    let recovery_files = read_recovery_backup_files(&recovery_paths)?;
    let vault_backup = SaveImage::new(read_encrypted_vault_backup_from_bundle(&vault_backup_path)?);
    let vault_backup_blake3 = vault_backup.blake3_hash().to_string();
    let response = recover_keychain_vault_with_helper(
        config,
        vault_backup.as_bytes().to_vec(),
        recovery_files,
    )?;
    let SignerRecoverKeychainVaultResponse {
        save_image,
        keychain_service,
        keychain_account,
        keychain_item_id,
        keychain_access_policy,
        device_id,
        kek_id,
        created_keychain_kek,
        metadata,
        recovery_share_file_count,
    } = response;

    let image = SaveImage::new(save_image);
    write_configured_save_image(config, &image)?;
    let signer_helper = helper_report(&config.helper)?;

    Ok(json!({
        "operation": "recover_keychain_vault",
        "device": config.device.describe(),
        "vaultBackupPath": vault_backup_path.display().to_string(),
        "vaultBackupBlake3": vault_backup_blake3,
        "saveSize": image.len(),
        "saveImageBlake3": image.blake3_hash().to_string(),
        "metadata": metadata,
        "keychain": {
            "service": keychain_service,
            "account": keychain_account,
            "itemId": keychain_item_id,
            "accessPolicy": keychain_access_policy,
            "deviceId": device_id,
            "kekId": kek_id,
            "createdKeychainKek": created_keychain_kek,
        },
        "recoveryFiles": recovery_paths
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>(),
        "recoveryShareFileCount": recovery_share_file_count,
        "walletSecretTouched": false,
        "recoveryShareBytesPrinted": false,
        "plaintextSecretProcess": "not_required_for_rewrap",
        "signerHelper": signer_helper,
    }))
}

pub(crate) fn validate_recovery_set(
    config: &DesktopConfig,
    request: ValidateRecoverySetRequest,
) -> Result<Value> {
    request.validate()?;
    let recovery_paths = request.recovery_file_paths()?;
    let recovery_files = read_recovery_backup_files(&recovery_paths)?;
    let response = validate_recovery_files_with_helper(config, recovery_files)?;
    let SignerValidateRecoveryFilesResponse {
        backup_set_id,
        wallet_id,
        generation,
        policy_id,
        recovery_share_file_count,
        satisfied_groups,
        can_recover,
        failure_reason,
    } = response;
    let signer_helper = helper_report(&config.helper)?;

    Ok(json!({
        "operation": "validate_recovery_set",
        "backupSetId": backup_set_id,
        "walletId": wallet_id,
        "generation": generation,
        "policyId": policy_id,
        "recoveryFiles": recovery_paths
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>(),
        "recoveryShareFileCount": recovery_share_file_count,
        "satisfiedGroups": satisfied_groups,
        "canRecover": can_recover,
        "failureReason": failure_reason,
        "walletSecretTouched": false,
        "recoveryRootKeyPrinted": false,
        "recoveryShareBytesPrinted": false,
        "configuredVaultDeviceTouched": false,
        "plaintextSecretProcess": "not_required_for_drill",
        "signerHelper": signer_helper,
    }))
}

pub(crate) fn recovery_smoke_pack_with_validator(
    config: &DesktopConfig,
    request: RecoverySmokePackRequest,
    validate: impl Fn(
        &DesktopConfig,
        Vec<RecoveryBackupFile>,
    ) -> Result<SignerValidateRecoveryFilesResponse>,
) -> Result<Value> {
    request.validate()?;
    let out_dir = request.out_dir_path()?;
    let generation = request.generation.unwrap_or(1);
    let recovery_root_key = random_array::<32>()?;
    let pack = RecoveryBackupPack::standard(
        random_array::<16>()?,
        generation,
        random_array::<16>()?,
        random_array::<16>()?,
        now_unix_ms() / 1_000,
        &recovery_root_key,
        RecoveryBackupEntropy {
            group_polynomial_coefficients: random_array::<32>()?,
            cloud_member_pad: random_array::<32>()?,
        },
    );
    let smoke_vault_backup = recovery_smoke_encrypted_vault_backup(&pack, generation);
    let recovery_backups = write_recovery_backup_pack(&out_dir, &pack, Some(&smoke_vault_backup))?;
    let cloud_paths = recovery_paths_for_group(&out_dir, &pack, "cloud");
    let recommended_paths = recommended_recovery_paths(&out_dir, &pack)?;
    let cloud_only_drill = recovery_drill_value(
        config,
        &cloud_paths,
        validate_recovery_paths(config, &cloud_paths, &validate)?,
    );
    let recommended_drill = recovery_drill_value(
        config,
        &recommended_paths,
        validate_recovery_paths(config, &recommended_paths, &validate)?,
    );
    let signer_helper = helper_report(&config.helper).ok();

    Ok(json!({
        "operation": "recovery_smoke_pack",
        "developmentOnly": true,
        "outDir": out_dir.display().to_string(),
        "generation": generation,
        "recoveryBackups": recovery_backups,
        "cloudOnlyDrill": cloud_only_drill,
        "recommendedDrill": recommended_drill,
        "keychainTouched": false,
        "configuredVaultDeviceTouched": false,
        "walletSecretTouched": false,
        "walletSecretPrinted": false,
        "recoveryRootKeyPrinted": false,
        "recoveryShareBytesPrinted": false,
        "plaintextSecretProcess": "not_required_for_recovery_smoke",
        "signerHelper": signer_helper,
    }))
}

pub(crate) fn recovery_paths_for_group(
    out_dir: &Path,
    pack: &RecoveryBackupPack,
    group: &str,
) -> Vec<PathBuf> {
    pack.files
        .iter()
        .filter(|file| file.group_kind.as_str() == group)
        .map(|file| out_dir.join(recovery_backup_file_name(file)))
        .collect()
}

pub(crate) fn recommended_recovery_paths(
    out_dir: &Path,
    pack: &RecoveryBackupPack,
) -> Result<Vec<PathBuf>> {
    let mut paths = recovery_paths_for_group(out_dir, pack, "cloud");
    let physical = pack
        .files
        .iter()
        .find(|file| file.group_kind.as_str() == "local_physical")
        .or_else(|| {
            pack.files
                .iter()
                .find(|file| file.group_kind.as_str() == "remote_physical")
        })
        .ok_or_else(|| anyhow::anyhow!("recovery pack is missing a physical share"))?;
    paths.push(out_dir.join(recovery_backup_file_name(physical)));
    Ok(paths)
}

pub(crate) fn validate_recovery_paths(
    config: &DesktopConfig,
    paths: &[PathBuf],
    validate: &impl Fn(
        &DesktopConfig,
        Vec<RecoveryBackupFile>,
    ) -> Result<SignerValidateRecoveryFilesResponse>,
) -> Result<SignerValidateRecoveryFilesResponse> {
    let recovery_files = read_recovery_backup_files(paths)?;
    validate(config, recovery_files)
}

pub(crate) fn recovery_drill_value(
    config: &DesktopConfig,
    paths: &[PathBuf],
    response: SignerValidateRecoveryFilesResponse,
) -> Value {
    json!({
        "operation": "validate_recovery_set",
        "backupSetId": response.backup_set_id,
        "walletId": response.wallet_id,
        "generation": response.generation,
        "policyId": response.policy_id,
        "recoveryFiles": paths
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>(),
        "recoveryShareFileCount": response.recovery_share_file_count,
        "satisfiedGroups": response.satisfied_groups,
        "canRecover": response.can_recover,
        "failureReason": response.failure_reason,
        "walletSecretTouched": false,
        "recoveryRootKeyPrinted": false,
        "recoveryShareBytesPrinted": false,
        "configuredVaultDeviceTouched": false,
        "plaintextSecretProcess": "not_required_for_drill",
        "signerHelper": helper_report(&config.helper).ok(),
    })
}

pub(crate) fn reveal_path(path: &Path) -> Result<Value> {
    let metadata = std::fs::metadata(path)
        .with_context(|| format!("path does not exist: {}", path.display()))?;
    let mut command = Command::new("open");
    if metadata.is_dir() {
        command.arg(path);
    } else {
        command.arg("-R").arg(path);
    }
    let output = command
        .output()
        .with_context(|| format!("failed to launch Finder for {}", path.display()))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!(
            "Finder reveal failed for {}: {}",
            path.display(),
            stderr.trim()
        );
    }
    Ok(json!({
        "operation": "reveal_path",
        "path": path.display().to_string(),
        "kind": if metadata.is_dir() { "directory" } else { "file" },
        "opened": true,
    }))
}

pub(crate) fn pick_recovery_files() -> Result<Value> {
    #[cfg(target_os = "macos")]
    {
        let output = run_macos_osascript(&[
            "set chosenFiles to choose file with prompt \"Select backup files\" with multiple selections allowed",
            "set output to \"\"",
            "repeat with chosenFile in chosenFiles",
            "set output to output & POSIX path of chosenFile & linefeed",
            "end repeat",
            "return output",
        ])?;
        recovery_file_picker_result("pick_recovery_files", output, true)
    }

    #[cfg(not(target_os = "macos"))]
    {
        anyhow::bail!("native recovery file picker is currently supported on macOS only");
    }
}

pub(crate) fn pick_vault_backup_file() -> Result<Value> {
    #[cfg(target_os = "macos")]
    {
        let output = run_macos_osascript(&[
            "set chosenFile to choose file with prompt \"Select backup file\"",
            "return POSIX path of chosenFile",
        ])?;
        recovery_file_picker_result("pick_vault_backup_file", output, false)
    }

    #[cfg(not(target_os = "macos"))]
    {
        anyhow::bail!("native backup file picker is currently supported on macOS only");
    }
}

pub(crate) fn pick_recovery_out_dir() -> Result<Value> {
    #[cfg(target_os = "macos")]
    {
        let output = run_macos_osascript(&[
            "set chosenFolder to choose folder with prompt \"Select FRAMKey recovery output directory\"",
            "return POSIX path of chosenFolder",
        ])?;
        recovery_file_picker_result("pick_recovery_out_dir", output, false)
    }

    #[cfg(not(target_os = "macos"))]
    {
        anyhow::bail!(
            "native recovery output directory picker is currently supported on macOS only"
        );
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn run_macos_osascript(lines: &[&str]) -> Result<std::process::Output> {
    let mut command = Command::new("/usr/bin/osascript");
    for line in lines {
        command.arg("-e").arg(line);
    }
    command
        .output()
        .context("failed to launch macOS recovery picker")
}

pub(crate) fn recovery_file_picker_result(
    operation: &str,
    output: std::process::Output,
    multiple: bool,
) -> Result<Value> {
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        if is_macos_user_cancelled(&stderr) {
            return Ok(json!({
                "operation": operation,
                "cancelled": true,
                "paths": [],
                "count": 0,
            }));
        }
        anyhow::bail!("recovery picker failed: {}", sanitize_picker_error(&stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let paths = parse_picker_paths(&stdout)?;
    if !multiple && paths.len() > 1 {
        anyhow::bail!("recovery picker returned more than one path");
    }
    Ok(json!({
        "operation": operation,
        "cancelled": false,
        "paths": paths,
        "count": paths.len(),
    }))
}

pub(crate) fn parse_picker_paths(stdout: &str) -> Result<Vec<String>> {
    let mut paths = Vec::new();
    for line in stdout.lines() {
        let path = line.strip_suffix('\r').unwrap_or(line);
        if path.is_empty() {
            continue;
        }
        if path.chars().any(char::is_control) {
            anyhow::bail!("recovery picker returned a malformed path");
        }
        paths.push(path.to_owned());
    }
    Ok(paths)
}

pub(crate) fn is_macos_user_cancelled(stderr: &str) -> bool {
    stderr.contains("User canceled") || stderr.contains("User cancelled")
}

pub(crate) fn sanitize_picker_error(stderr: &str) -> String {
    let message = stderr
        .chars()
        .filter(|ch| !ch.is_control())
        .collect::<String>();
    truncate_for_event(message.trim(), 200)
}
