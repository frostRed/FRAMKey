use std::collections::BTreeSet;

use framkey_core::{FramkeyError, Result};

use crate::{
    RecoveryBackupFile, RecoveryGroupKind,
    constants::{
        RECOVERY_BACKUP_FORMAT_VERSION, RECOVERY_BACKUP_SHARE_FORMAT,
        RECOVERY_MEMBER_SHARE_ENCODING, RECOVERY_ROOT_KEY_BYTES,
    },
    encoding::decode_hex,
};

pub fn reconstruct_recovery_root_key(
    files: &[RecoveryBackupFile],
) -> Result<[u8; RECOVERY_ROOT_KEY_BYTES]> {
    reconstruct_recovery_root_key_candidates(files)?
        .into_iter()
        .next()
        .ok_or_else(|| FramkeyError::invalid_data("not enough satisfied recovery groups"))
}

pub fn reconstruct_recovery_root_key_candidates(
    files: &[RecoveryBackupFile],
) -> Result<Vec<[u8; RECOVERY_ROOT_KEY_BYTES]>> {
    if files.is_empty() {
        return Err(FramkeyError::invalid_data(
            "no recovery backup files supplied",
        ));
    }

    validate_same_backup_set(files)?;
    let group_threshold = files[0].group_threshold;
    if group_threshold != 2 {
        return Err(FramkeyError::unsupported(format!(
            "unsupported group threshold {group_threshold}"
        )));
    }

    let mut group_shares = Vec::new();
    for group_kind in [
        RecoveryGroupKind::Cloud,
        RecoveryGroupKind::LocalPhysical,
        RecoveryGroupKind::RemotePhysical,
    ] {
        if let Some((index, share)) = reconstruct_group_share(files, group_kind)? {
            group_shares.push((index, share));
        }
    }

    if group_shares.len() < usize::from(group_threshold) {
        return Err(FramkeyError::invalid_data(
            "not enough satisfied recovery groups",
        ));
    }

    let mut candidates = Vec::new();
    for left in 0..group_shares.len() {
        for right in (left + 1)..group_shares.len() {
            candidates.push(interpolate_root_key(&[
                group_shares[left],
                group_shares[right],
            ])?);
        }
    }
    Ok(candidates)
}

fn interpolate_root_key(
    selected: &[(u8, [u8; RECOVERY_ROOT_KEY_BYTES])],
) -> Result<[u8; RECOVERY_ROOT_KEY_BYTES]> {
    let mut root = [0_u8; RECOVERY_ROOT_KEY_BYTES];
    for byte_index in 0..RECOVERY_ROOT_KEY_BYTES {
        let points = selected
            .iter()
            .map(|(x, share)| (*x, share[byte_index]))
            .collect::<Vec<_>>();
        root[byte_index] = interpolate_at_zero(&points)?;
    }
    Ok(root)
}

fn reconstruct_group_share(
    files: &[RecoveryBackupFile],
    group_kind: RecoveryGroupKind,
) -> Result<Option<(u8, [u8; RECOVERY_ROOT_KEY_BYTES])>> {
    let group_files = files
        .iter()
        .filter(|file| file.group_kind == group_kind)
        .collect::<Vec<_>>();
    let Some(first) = group_files.first() else {
        return Ok(None);
    };

    validate_group_files(&group_files)?;

    if group_files.len() < usize::from(first.member_threshold) {
        return Ok(None);
    }

    match first.member_threshold {
        1 => Ok(Some((first.group_share_index, decode_member_share(first)?))),
        2 => {
            let mut output = [0_u8; RECOVERY_ROOT_KEY_BYTES];
            for file in group_files.iter().take(usize::from(first.member_threshold)) {
                let share = decode_member_share(file)?;
                for (left, right) in output.iter_mut().zip(share) {
                    *left ^= right;
                }
            }
            Ok(Some((first.group_share_index, output)))
        }
        threshold => Err(FramkeyError::unsupported(format!(
            "unsupported member threshold {threshold}"
        ))),
    }
}

fn validate_same_backup_set(files: &[RecoveryBackupFile]) -> Result<()> {
    let first = &files[0];
    for file in files {
        validate_recovery_backup_file(file)?;
        if file.backup_set_id != first.backup_set_id
            || file.wallet_id != first.wallet_id
            || file.generation != first.generation
            || file.policy_id != first.policy_id
            || file.group_threshold != first.group_threshold
        {
            return Err(FramkeyError::invalid_data(
                "recovery share files are from different backup sets",
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_recovery_backup_file(file: &RecoveryBackupFile) -> Result<()> {
    if file.format != RECOVERY_BACKUP_SHARE_FORMAT {
        return Err(FramkeyError::invalid_data("recovery share format mismatch"));
    }
    if file.format_version != RECOVERY_BACKUP_FORMAT_VERSION {
        return Err(FramkeyError::unsupported(format!(
            "recovery share format version {}",
            file.format_version
        )));
    }
    if file.group_threshold != 2 {
        return Err(FramkeyError::unsupported(format!(
            "unsupported group threshold {}",
            file.group_threshold
        )));
    }
    if file.member_index == 0 || file.member_index > file.member_count {
        return Err(FramkeyError::invalid_data(
            "recovery share member index is outside the group",
        ));
    }
    if file.member_threshold == 0 || file.member_threshold > file.member_count {
        return Err(FramkeyError::invalid_data(
            "recovery share member threshold is invalid",
        ));
    }
    decode_member_share(file)?;
    Ok(())
}

fn validate_group_files(files: &[&RecoveryBackupFile]) -> Result<()> {
    let first = files[0];
    let mut member_indices = BTreeSet::new();
    for file in files {
        if file.group_share_index != first.group_share_index
            || file.group_threshold != first.group_threshold
            || file.member_threshold != first.member_threshold
            || file.member_count != first.member_count
        {
            return Err(FramkeyError::invalid_data(
                "recovery share group metadata mismatch",
            ));
        }
        if file.member_index == 0 || file.member_index > file.member_count {
            return Err(FramkeyError::invalid_data(
                "recovery share member index is outside the group",
            ));
        }
        if file.member_threshold == 0 || file.member_threshold > file.member_count {
            return Err(FramkeyError::invalid_data(
                "recovery share member threshold is invalid",
            ));
        }
        if !member_indices.insert(file.member_index) {
            return Err(FramkeyError::invalid_data(
                "duplicate recovery group member share",
            ));
        }
    }
    Ok(())
}

pub(crate) fn group_share(
    root_key: &[u8; RECOVERY_ROOT_KEY_BYTES],
    coefficients: &[u8; RECOVERY_ROOT_KEY_BYTES],
    x: u8,
) -> [u8; RECOVERY_ROOT_KEY_BYTES] {
    let mut share = [0_u8; RECOVERY_ROOT_KEY_BYTES];
    for index in 0..RECOVERY_ROOT_KEY_BYTES {
        share[index] = root_key[index] ^ gf_mul(coefficients[index], x);
    }
    share
}

pub(crate) fn interpolate_at_zero(points: &[(u8, u8)]) -> Result<u8> {
    let mut output = 0_u8;
    for (i, (x_i, y_i)) in points.iter().copied().enumerate() {
        if x_i == 0 {
            return Err(FramkeyError::invalid_data(
                "recovery share x must be nonzero",
            ));
        }
        let mut numerator = 1_u8;
        let mut denominator = 1_u8;
        for (j, (x_j, _)) in points.iter().copied().enumerate() {
            if i == j {
                continue;
            }
            if x_i == x_j {
                return Err(FramkeyError::invalid_data("duplicate recovery group share"));
            }
            numerator = gf_mul(numerator, x_j);
            denominator = gf_mul(denominator, x_i ^ x_j);
        }
        output ^= gf_mul(y_i, gf_div(numerator, denominator)?);
    }
    Ok(output)
}

fn gf_div(left: u8, right: u8) -> Result<u8> {
    if right == 0 {
        return Err(FramkeyError::invalid_data("GF(256) division by zero"));
    }
    Ok(gf_mul(left, gf_pow(right, 254)))
}

fn gf_pow(mut value: u8, mut exponent: u8) -> u8 {
    let mut output = 1_u8;
    while exponent > 0 {
        if exponent & 1 == 1 {
            output = gf_mul(output, value);
        }
        value = gf_mul(value, value);
        exponent >>= 1;
    }
    output
}

fn gf_mul(mut left: u8, mut right: u8) -> u8 {
    let mut output = 0_u8;
    while right != 0 {
        if right & 1 != 0 {
            output ^= left;
        }
        let carry = left & 0x80;
        left <<= 1;
        if carry != 0 {
            left ^= 0x1b;
        }
        right >>= 1;
    }
    output
}

pub(crate) fn xor_32(
    left: &[u8; RECOVERY_ROOT_KEY_BYTES],
    right: &[u8; RECOVERY_ROOT_KEY_BYTES],
) -> [u8; RECOVERY_ROOT_KEY_BYTES] {
    let mut output = [0_u8; RECOVERY_ROOT_KEY_BYTES];
    for index in 0..RECOVERY_ROOT_KEY_BYTES {
        output[index] = left[index] ^ right[index];
    }
    output
}

fn decode_share_hex(value: &str) -> Result<[u8; RECOVERY_ROOT_KEY_BYTES]> {
    let bytes = decode_hex(value)?;
    if bytes.len() != RECOVERY_ROOT_KEY_BYTES {
        return Err(FramkeyError::invalid_data(format!(
            "recovery share must be {RECOVERY_ROOT_KEY_BYTES} bytes"
        )));
    }
    let mut output = [0_u8; RECOVERY_ROOT_KEY_BYTES];
    output.copy_from_slice(&bytes);
    Ok(output)
}

pub(crate) fn encode_member_share(
    share: &[u8; RECOVERY_ROOT_KEY_BYTES],
    input: MemberShareMaskInput<'_>,
) -> [u8; RECOVERY_ROOT_KEY_BYTES] {
    xor_32(share, &member_share_mask(input))
}

fn decode_member_share(file: &RecoveryBackupFile) -> Result<[u8; RECOVERY_ROOT_KEY_BYTES]> {
    let share = decode_share_hex(&file.share_hex)?;
    match file.share_encoding.as_str() {
        RECOVERY_MEMBER_SHARE_ENCODING => Ok(xor_32(
            &share,
            &member_share_mask(MemberShareMaskInput {
                backup_set_id: &file.backup_set_id,
                wallet_id: &file.wallet_id,
                generation: file.generation,
                policy_id: &file.policy_id,
                group_kind: file.group_kind,
                group_share_index: file.group_share_index,
                group_threshold: file.group_threshold,
                member_index: file.member_index,
                member_threshold: file.member_threshold,
                member_count: file.member_count,
                member_label: &file.member_label,
            }),
        )),
        encoding => Err(FramkeyError::unsupported(format!(
            "unsupported recovery share encoding {encoding}"
        ))),
    }
}

pub(crate) struct MemberShareMaskInput<'a> {
    pub(crate) backup_set_id: &'a str,
    pub(crate) wallet_id: &'a str,
    pub(crate) generation: u64,
    pub(crate) policy_id: &'a str,
    pub(crate) group_kind: RecoveryGroupKind,
    pub(crate) group_share_index: u8,
    pub(crate) group_threshold: u8,
    pub(crate) member_index: u8,
    pub(crate) member_threshold: u8,
    pub(crate) member_count: u8,
    pub(crate) member_label: &'a str,
}

fn member_share_mask(input: MemberShareMaskInput<'_>) -> [u8; RECOVERY_ROOT_KEY_BYTES] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"framkey:recovery-member-share-mask:v1");
    hasher.update(input.backup_set_id.as_bytes());
    hasher.update(&[0]);
    hasher.update(input.wallet_id.as_bytes());
    hasher.update(&[0]);
    hasher.update(&input.generation.to_le_bytes());
    hasher.update(input.policy_id.as_bytes());
    hasher.update(&[0]);
    hasher.update(input.group_kind.as_str().as_bytes());
    hasher.update(&[0]);
    hasher.update(&[input.group_share_index]);
    hasher.update(&[input.group_threshold]);
    hasher.update(&[input.member_index]);
    hasher.update(&[input.member_threshold]);
    hasher.update(&[input.member_count]);
    hasher.update(input.member_label.as_bytes());

    let mut output = [0_u8; RECOVERY_ROOT_KEY_BYTES];
    output.copy_from_slice(hasher.finalize().as_bytes());
    output
}
