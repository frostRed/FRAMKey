use std::collections::{BTreeMap, BTreeSet};

use anyhow::Result;
use framkey_core::FramkeyError;
use framkey_ipc::SignerValidateRecoveryFilesResponse;
use framkey_recovery::{RecoveryBackupFile, reconstruct_recovery_root_key};

pub(crate) fn validate_recovery_files_drill(
    files: &[RecoveryBackupFile],
) -> Result<SignerValidateRecoveryFilesResponse> {
    let first = files.first().ok_or_else(|| {
        FramkeyError::invalid_data("at least one recovery share file is required")
    })?;

    let mut group_members: BTreeMap<String, BTreeSet<u8>> = BTreeMap::new();
    let mut group_thresholds: BTreeMap<String, u8> = BTreeMap::new();
    for file in files {
        if file.backup_set_id != first.backup_set_id
            || file.wallet_id != first.wallet_id
            || file.generation != first.generation
            || file.policy_id != first.policy_id
        {
            return Err(FramkeyError::invalid_data(
                "recovery share files do not belong to the same backup set",
            )
            .into());
        }
        let group = file.group_kind.as_str().to_owned();
        group_members
            .entry(group.clone())
            .or_default()
            .insert(file.member_index);
        group_thresholds
            .entry(group)
            .or_insert(file.member_threshold);
    }

    let satisfied_groups = group_members
        .iter()
        .filter_map(|(group, members)| {
            let threshold = group_thresholds.get(group).copied().unwrap_or(1);
            (members.len() >= usize::from(threshold)).then(|| group.clone())
        })
        .collect::<Vec<_>>();

    let recovery_result = reconstruct_recovery_root_key(files);
    Ok(SignerValidateRecoveryFilesResponse {
        backup_set_id: first.backup_set_id.clone(),
        wallet_id: first.wallet_id.clone(),
        generation: first.generation,
        policy_id: first.policy_id.clone(),
        recovery_share_file_count: files.len(),
        satisfied_groups,
        can_recover: recovery_result.is_ok(),
        failure_reason: recovery_result.err().map(|error| error.to_string()),
    })
}
