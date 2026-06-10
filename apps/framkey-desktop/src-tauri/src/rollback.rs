use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result};
use framkey_ipc::SignerVaultMetadata;
use framkey_vault::{KeychainVaultMetadata, inspect_keychain_vault_metadata};
use serde::{Deserialize, Serialize};

use crate::*;

const VAULT_GENERATION_STATE_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VaultGenerationCheckpoint {
    pub(crate) wallet_id: String,
    pub(crate) generation: u64,
    pub(crate) keychain_item_id: String,
    pub(crate) device_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedVaultGenerationState {
    version: u32,
    entries: Vec<VaultGenerationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct VaultGenerationEntry {
    wallet_id: String,
    highest_generation: u64,
    keychain_item_id: String,
    device_id: String,
    updated_at_unix_ms: u64,
}

#[derive(Debug, Clone, Default)]
struct VaultGenerationState {
    entries: BTreeMap<String, VaultGenerationEntry>,
}

impl VaultGenerationCheckpoint {
    fn normalized(
        wallet_id: String,
        generation: u64,
        keychain_item_id: String,
        device_id: String,
    ) -> Result<Self> {
        let wallet_id = normalize_metadata_id("wallet id", wallet_id)?;
        let device_id = normalize_metadata_id("device id", device_id)?;
        if generation == 0 {
            anyhow::bail!("vault generation must be at least 1");
        }
        if keychain_item_id.trim().is_empty() || keychain_item_id.chars().any(char::is_control) {
            anyhow::bail!("vault Keychain item id is malformed");
        }
        Ok(Self {
            wallet_id,
            generation,
            keychain_item_id,
            device_id,
        })
    }
}

impl VaultGenerationState {
    fn read_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let bytes = fs::read(path)
            .with_context(|| format!("failed to read vault generation state {}", path.display()))?;
        let persisted: PersistedVaultGenerationState = serde_json::from_slice(&bytes)
            .with_context(|| {
                format!("failed to parse vault generation state {}", path.display())
            })?;
        if persisted.version != VAULT_GENERATION_STATE_VERSION {
            anyhow::bail!(
                "unsupported vault generation state version {}",
                persisted.version
            );
        }
        let mut entries = BTreeMap::new();
        for entry in persisted.entries {
            if entry.wallet_id.trim().is_empty() {
                continue;
            }
            entries.insert(entry.wallet_id.clone(), entry);
        }
        Ok(Self { entries })
    }

    fn write_to_path(&self, path: &Path) -> Result<()> {
        let persisted = PersistedVaultGenerationState {
            version: VAULT_GENERATION_STATE_VERSION,
            entries: self.entries.values().cloned().collect(),
        };
        let bytes = serde_json::to_vec_pretty(&persisted)
            .context("failed to serialize vault generation state")?;
        write_json_atomically(path, &bytes)
    }

    fn enforce(&self, checkpoint: &VaultGenerationCheckpoint) -> Result<()> {
        let Some(entry) = self.entries.get(&checkpoint.wallet_id) else {
            return Ok(());
        };
        if checkpoint.generation < entry.highest_generation {
            anyhow::bail!(
                "configured vault rollback detected for wallet {}: image generation {} is older than local high-water generation {}",
                checkpoint.wallet_id,
                checkpoint.generation,
                entry.highest_generation
            );
        }
        Ok(())
    }

    fn remember(&mut self, checkpoint: VaultGenerationCheckpoint) -> Result<bool> {
        let updated_at_unix_ms = now_unix_ms();
        match self.entries.get_mut(&checkpoint.wallet_id) {
            Some(entry) if checkpoint.generation < entry.highest_generation => {
                anyhow::bail!(
                    "refusing to lower local vault high-water generation for wallet {} from {} to {}",
                    checkpoint.wallet_id,
                    entry.highest_generation,
                    checkpoint.generation
                );
            }
            Some(entry) => {
                let changed = entry.highest_generation != checkpoint.generation
                    || entry.keychain_item_id != checkpoint.keychain_item_id
                    || entry.device_id != checkpoint.device_id;
                entry.highest_generation = checkpoint.generation;
                entry.keychain_item_id = checkpoint.keychain_item_id;
                entry.device_id = checkpoint.device_id;
                entry.updated_at_unix_ms = updated_at_unix_ms;
                Ok(changed)
            }
            None => {
                self.entries.insert(
                    checkpoint.wallet_id.clone(),
                    VaultGenerationEntry {
                        wallet_id: checkpoint.wallet_id,
                        highest_generation: checkpoint.generation,
                        keychain_item_id: checkpoint.keychain_item_id,
                        device_id: checkpoint.device_id,
                        updated_at_unix_ms,
                    },
                );
                Ok(true)
            }
        }
    }
}

pub(crate) fn inspect_keychain_vault_checkpoint(
    save_image: &[u8],
) -> Result<VaultGenerationCheckpoint> {
    checkpoint_from_vault_metadata(inspect_keychain_vault_metadata(save_image)?)
}

pub(crate) fn checkpoint_from_signer_metadata(
    metadata: &SignerVaultMetadata,
    keychain_item_id: &str,
    device_id: &str,
) -> Result<VaultGenerationCheckpoint> {
    VaultGenerationCheckpoint::normalized(
        metadata.wallet_id.clone(),
        metadata.generation,
        keychain_item_id.to_owned(),
        device_id.to_owned(),
    )
}

pub(crate) fn enforce_configured_vault_high_water(
    checkpoint: &VaultGenerationCheckpoint,
) -> Result<()> {
    let path = vault_generation_state_path()?;
    let state = VaultGenerationState::read_from_path(&path)?;
    state.enforce(checkpoint)
}

pub(crate) fn remember_configured_vault_generation(
    checkpoint: VaultGenerationCheckpoint,
) -> Result<()> {
    let path = vault_generation_state_path()?;
    let mut state = VaultGenerationState::read_from_path(&path)?;
    if state.remember(checkpoint)? {
        state.write_to_path(&path)?;
    }
    Ok(())
}

fn checkpoint_from_vault_metadata(
    metadata: KeychainVaultMetadata,
) -> Result<VaultGenerationCheckpoint> {
    VaultGenerationCheckpoint::normalized(
        metadata.wallet_id,
        metadata.generation,
        metadata.keychain_item_id,
        metadata.device_id,
    )
}

fn normalize_metadata_id(label: &str, value: String) -> Result<String> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() || normalized.chars().any(char::is_control) {
        anyhow::bail!("vault {label} is malformed");
    }
    if !normalized.chars().all(|ch| ch.is_ascii_hexdigit()) {
        anyhow::bail!("vault {label} must be hex");
    }
    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checkpoint(generation: u64) -> VaultGenerationCheckpoint {
        VaultGenerationCheckpoint::normalized(
            "aa".repeat(16),
            generation,
            "io.framkey.local-kek:default".to_owned(),
            "bb".repeat(16),
        )
        .unwrap()
    }

    #[test]
    fn vault_generation_state_rejects_rollback() {
        let mut state = VaultGenerationState::default();
        assert!(state.remember(checkpoint(7)).unwrap());

        let error = state.enforce(&checkpoint(6)).unwrap_err().to_string();

        assert!(error.contains("rollback detected"));
        assert!(error.contains("generation 6"));
        assert!(error.contains("generation 7"));
    }

    #[test]
    fn vault_generation_state_allows_equal_or_newer_generation() {
        let mut state = VaultGenerationState::default();
        assert!(state.remember(checkpoint(7)).unwrap());

        state.enforce(&checkpoint(7)).unwrap();
        state.enforce(&checkpoint(8)).unwrap();
        assert!(state.remember(checkpoint(8)).unwrap());
    }

    #[test]
    fn vault_generation_state_refuses_to_remember_lower_generation() {
        let mut state = VaultGenerationState::default();
        assert!(state.remember(checkpoint(7)).unwrap());

        let error = state.remember(checkpoint(6)).unwrap_err().to_string();

        assert!(error.contains("refusing to lower"));
    }
}
