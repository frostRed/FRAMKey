use std::{
    collections::BTreeMap,
    fs::{self},
    path::Path,
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DappSessionState {
    pub(crate) open: bool,
    pub(crate) target_label: String,
    pub(crate) requested_url: Option<String>,
    pub(crate) current_url: Option<String>,
    pub(crate) origin: Option<String>,
    pub(crate) load_status: String,
    pub(crate) last_event: Option<String>,
    pub(crate) navigation_action: Option<String>,
    pub(crate) updated_at_unix_ms: u64,
}

impl DappSessionState {
    pub(crate) fn new() -> Self {
        Self {
            open: false,
            target_label: "No app open".to_owned(),
            requested_url: None,
            current_url: None,
            origin: None,
            load_status: "not_loaded".to_owned(),
            last_event: None,
            navigation_action: None,
            updated_at_unix_ms: now_unix_ms(),
        }
    }

    pub(crate) fn remember_open_request(&mut self, target: DappSessionTarget) {
        self.open = true;
        self.target_label = target.label;
        self.requested_url = target.url.clone();
        self.current_url = target.url;
        self.origin = target.origin;
        self.load_status = "opening".to_owned();
        self.last_event = Some("open_request".to_owned());
        self.navigation_action = None;
        self.updated_at_unix_ms = now_unix_ms();
    }

    pub(crate) fn remember_navigation_url(&mut self, location: DappSessionLocation) {
        self.open = true;
        self.current_url = location.url;
        self.origin = location.origin;
        self.load_status = "navigating".to_owned();
        self.last_event = Some("navigation".to_owned());
        self.navigation_action = None;
        self.updated_at_unix_ms = now_unix_ms();
    }

    pub(crate) fn remember_page_load(&mut self, event: &str, location: DappSessionLocation) {
        self.open = true;
        self.current_url = location.url;
        self.origin = location.origin;
        self.load_status = match event {
            "finished" => "loaded",
            "started" => "loading",
            _ => "unknown",
        }
        .to_owned();
        self.last_event = Some(format!("page_load_{event}"));
        if event == "finished" {
            self.navigation_action = None;
        }
        self.updated_at_unix_ms = now_unix_ms();
    }

    pub(crate) fn remember_navigation_action(&mut self, action: DappNavigationAction) {
        self.open = true;
        self.load_status = "navigation_requested".to_owned();
        self.last_event = Some("navigation_action".to_owned());
        self.navigation_action = Some(action.as_str().to_owned());
        self.updated_at_unix_ms = now_unix_ms();
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DappSessionTarget {
    pub(crate) label: String,
    pub(crate) url: Option<String>,
    pub(crate) origin: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DappSessionLocation {
    pub(crate) url: Option<String>,
    pub(crate) origin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WatchedAsset {
    pub(crate) chain_id: String,
    pub(crate) asset_type: String,
    pub(crate) contract_address: String,
    pub(crate) symbol: String,
    pub(crate) decimals: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) origin: Option<String>,
    pub(crate) watched_at_unix_ms: u64,
}

#[derive(Debug, Default)]
pub(crate) struct WatchedAssetStore {
    pub(crate) assets: BTreeMap<(String, String), WatchedAsset>,
}

impl WatchedAssetStore {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn read_from_path(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let bytes = fs::read(path)
            .with_context(|| format!("failed to read wallet UI state {}", path.display()))?;
        let persisted: PersistedWalletUiState = serde_json::from_slice(&bytes)
            .with_context(|| format!("failed to parse wallet UI state {}", path.display()))?;
        if persisted.version != WALLET_UI_STATE_VERSION {
            anyhow::bail!("unsupported wallet UI state version {}", persisted.version);
        }
        Self::from_assets(persisted.watched_assets)
    }

    pub(crate) fn from_assets(assets: Vec<WatchedAsset>) -> Result<Self> {
        let mut store = Self::new();
        for asset in assets.into_iter().take(WALLET_WATCHED_ASSET_LIMIT) {
            store.remember(normalize_watched_asset(asset)?);
        }
        Ok(store)
    }

    pub(crate) fn write_to_path(&self, path: &Path) -> Result<()> {
        let persisted = PersistedWalletUiState {
            version: WALLET_UI_STATE_VERSION,
            watched_assets: self.snapshot(),
        };
        let bytes =
            serde_json::to_vec_pretty(&persisted).context("failed to serialize wallet UI state")?;
        write_json_atomically(path, &bytes)
    }

    pub(crate) fn remember(&mut self, asset: WatchedAsset) {
        let key = (
            asset.chain_id.clone(),
            asset.contract_address.to_ascii_lowercase(),
        );
        self.assets.insert(key, asset);
        while self.assets.len() > WALLET_WATCHED_ASSET_LIMIT {
            let Some(key) = self.assets.keys().next().cloned() else {
                break;
            };
            self.assets.remove(&key);
        }
    }

    pub(crate) fn for_chain(&self, chain_id: &str) -> Vec<WatchedAsset> {
        let normalized = normalize_chain_id(chain_id).unwrap_or_else(|_| chain_id.to_owned());
        self.assets
            .values()
            .filter(|asset| asset.chain_id == normalized)
            .cloned()
            .collect()
    }

    pub(crate) fn snapshot(&self) -> Vec<WatchedAsset> {
        self.assets.values().cloned().collect()
    }

    pub(crate) fn len(&self) -> usize {
        self.assets.len()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PersistedWalletUiState {
    pub(crate) version: u32,
    pub(crate) watched_assets: Vec<WatchedAsset>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WalletUiStatePersistenceStatus {
    pub(crate) enabled: bool,
    pub(crate) restored: bool,
    pub(crate) watched_assets_restored: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) last_saved_at_unix_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) warning: Option<String>,
}

impl WalletUiStatePersistenceStatus {
    pub(crate) fn disabled() -> Self {
        Self {
            enabled: false,
            restored: false,
            watched_assets_restored: 0,
            last_saved_at_unix_ms: None,
            warning: None,
        }
    }

    pub(crate) fn unavailable(warning: String) -> Self {
        Self {
            enabled: false,
            restored: false,
            watched_assets_restored: 0,
            last_saved_at_unix_ms: None,
            warning: Some(warning),
        }
    }

    pub(crate) fn enabled() -> Self {
        Self {
            enabled: true,
            restored: false,
            watched_assets_restored: 0,
            last_saved_at_unix_ms: None,
            warning: None,
        }
    }

    pub(crate) fn mark_saved(&mut self) {
        self.enabled = true;
        self.restored = false;
        self.last_saved_at_unix_ms = Some(now_unix_ms());
        self.warning = None;
    }
}
