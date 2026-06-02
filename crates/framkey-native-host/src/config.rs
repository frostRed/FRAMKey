use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use framkey_crypto::{decode_hex_array, encode_hex};
use framkey_gbxcart::GbaSaveType;
use framkey_ipc::SignerOpenKeychainVaultResponse;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::constants::{
    DEFAULT_CHAIN_ID, DEFAULT_GBXCART_PORT, DEFAULT_KEYCHAIN_ACCOUNT, DEFAULT_KEYCHAIN_SERVICE,
};

#[derive(Debug, Clone)]
pub(crate) struct NativeAccount {
    pub(crate) address: String,
    pub(crate) opened: SignerOpenKeychainVaultResponse,
    pub(crate) helper_report: Value,
}

#[derive(Debug, Clone)]
pub(crate) struct NativeHostConfig {
    pub(crate) chain_id: String,
    pub(crate) device: NativeDeviceConfig,
    pub(crate) keychain_service: String,
    pub(crate) keychain_account: String,
    pub(crate) helper: SignerHelperConfig,
}

impl NativeHostConfig {
    pub(crate) fn load() -> Result<Self> {
        let mut config = Self::default_for_repo()?;
        if let Some(file_config) = ConfigFile::load_optional()? {
            config.apply_file(file_config)?;
        }
        config.apply_env()?;
        config.validate()?;
        Ok(config)
    }

    fn default_for_repo() -> Result<Self> {
        Ok(Self {
            chain_id: DEFAULT_CHAIN_ID.to_owned(),
            device: NativeDeviceConfig::GbxCart {
                port: Some(DEFAULT_GBXCART_PORT.to_owned()),
                save_type: GbaSaveType::SramFram512Kbit,
                expected_save_size: None,
            },
            keychain_service: DEFAULT_KEYCHAIN_SERVICE.to_owned(),
            keychain_account: DEFAULT_KEYCHAIN_ACCOUNT.to_owned(),
            helper: SignerHelperConfig {
                path: default_signer_helper_path()?,
                expected_blake3: None,
                sandbox: default_helper_sandbox(false)?,
            },
        })
    }

    fn apply_file(&mut self, file: ConfigFile) -> Result<()> {
        if let Some(chain_id) = file.chain_id {
            self.chain_id = chain_id;
        }
        if let Some(device) = file.device {
            self.device = device.into_runtime()?;
        }
        if let Some(keychain) = file.keychain {
            if let Some(service) = keychain.service {
                self.keychain_service = service;
            }
            if let Some(account) = keychain.account {
                self.keychain_account = account;
            }
        }
        if let Some(helper) = file.signer_helper {
            if let Some(path) = helper.path {
                self.helper.path = path.canonicalize().with_context(|| {
                    format!("failed to resolve signer helper path {}", path.display())
                })?;
            }
            if let Some(expected) = helper.blake3 {
                self.helper.expected_blake3 = Some(normalize_blake3_hex(&expected)?);
            }
            if let Some(allow_unsandboxed) = helper.allow_unsandboxed {
                self.helper.sandbox = default_helper_sandbox(allow_unsandboxed)?;
            }
        }
        Ok(())
    }

    fn apply_env(&mut self) -> Result<()> {
        if let Ok(chain_id) = std::env::var("FRAMKEY_NATIVE_HOST_CHAIN_ID") {
            self.chain_id = chain_id;
        }
        if let Ok(path) = std::env::var("FRAMKEY_SAVE_IMAGE_PATH") {
            self.device = NativeDeviceConfig::File {
                path: PathBuf::from(path),
            };
        }
        if let Ok(port) = std::env::var("FRAMKEY_GBXCART_PORT") {
            self.device = NativeDeviceConfig::GbxCart {
                port: Some(port),
                save_type: env_save_type()?.unwrap_or(GbaSaveType::SramFram512Kbit),
                expected_save_size: env_usize("FRAMKEY_EXPECTED_SAVE_SIZE")?,
            };
        }
        if let Ok(save_type) = std::env::var("FRAMKEY_GBA_SAVE_TYPE")
            && let NativeDeviceConfig::GbxCart {
                save_type: current, ..
            } = &mut self.device
        {
            *current = parse_save_type(&save_type)?;
        }
        if let Ok(service) = std::env::var("FRAMKEY_KEYCHAIN_SERVICE") {
            self.keychain_service = service;
        }
        if let Ok(account) = std::env::var("FRAMKEY_KEYCHAIN_ACCOUNT") {
            self.keychain_account = account;
        }
        if let Ok(path) = std::env::var("FRAMKEY_SIGNER_HELPER") {
            let path = PathBuf::from(path);
            self.helper.path = path.canonicalize().with_context(|| {
                format!("failed to resolve signer helper path {}", path.display())
            })?;
        }
        if let Ok(expected) = std::env::var("FRAMKEY_SIGNER_HELPER_BLAKE3") {
            self.helper.expected_blake3 = Some(normalize_blake3_hex(&expected)?);
        }
        if env_truthy("FRAMKEY_NATIVE_HOST_ALLOW_UNSANDBOXED_HELPER") {
            self.helper.sandbox = default_helper_sandbox(true)?;
        }
        Ok(())
    }

    pub(crate) fn validate(&self) -> Result<()> {
        validate_chain_id(&self.chain_id)?;
        validate_keychain_name("service", &self.keychain_service)?;
        validate_keychain_name("account", &self.keychain_account)?;
        match &self.device {
            NativeDeviceConfig::File { path } if path.as_os_str().is_empty() => {
                anyhow::bail!("native host file save-image path must not be blank");
            }
            NativeDeviceConfig::GbxCart {
                port: Some(port), ..
            } => {
                validate_device_hint("GBxCart port", port)?;
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) enum NativeDeviceConfig {
    File {
        path: PathBuf,
    },
    GbxCart {
        port: Option<String>,
        save_type: GbaSaveType,
        expected_save_size: Option<usize>,
    },
}

impl NativeDeviceConfig {
    pub(crate) fn describe(&self) -> Value {
        match self {
            Self::File { path } => json!({
                "kind": "file",
                "pathConfigured": !path.as_os_str().is_empty(),
            }),
            Self::GbxCart {
                port,
                save_type,
                expected_save_size,
            } => json!({
                "kind": "gbx_cart",
                "portConfigured": port.is_some(),
                "saveType": save_type_name(*save_type),
                "expectedSaveSize": expected_save_size,
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SignerHelperConfig {
    pub(crate) path: PathBuf,
    pub(crate) expected_blake3: Option<String>,
    pub(crate) sandbox: SignerHelperSandbox,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SignerHelperSandbox {
    MacosSandboxExecNoNetwork,
    DisabledByConfig,
    #[cfg(not(target_os = "macos"))]
    UnsupportedPlatform,
}

impl SignerHelperSandbox {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::MacosSandboxExecNoNetwork => "macos_sandbox_exec_no_network",
            Self::DisabledByConfig => "disabled_by_config",
            #[cfg(not(target_os = "macos"))]
            Self::UnsupportedPlatform => "unsupported_platform",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigFile {
    #[serde(default)]
    pub(crate) chain_id: Option<String>,
    #[serde(default)]
    pub(crate) device: Option<ConfigDevice>,
    #[serde(default)]
    keychain: Option<ConfigKeychain>,
    #[serde(default)]
    signer_helper: Option<ConfigSignerHelper>,
}

impl ConfigFile {
    pub(crate) fn load_optional() -> Result<Option<Self>> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(&path)
            .with_context(|| format!("failed to read native host config {}", path.display()))?;
        let config = serde_json::from_slice(&bytes)
            .with_context(|| format!("failed to parse native host config {}", path.display()))?;
        Ok(Some(config))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
enum ConfigDevice {
    File {
        path: PathBuf,
    },
    GbxCart {
        #[serde(default)]
        port: Option<String>,
        save_type: String,
        #[serde(default)]
        expected_save_size: Option<usize>,
    },
}

impl ConfigDevice {
    fn into_runtime(self) -> Result<NativeDeviceConfig> {
        match self {
            Self::File { path } => Ok(NativeDeviceConfig::File { path }),
            Self::GbxCart {
                port,
                save_type,
                expected_save_size,
            } => Ok(NativeDeviceConfig::GbxCart {
                port,
                save_type: parse_save_type(&save_type)?,
                expected_save_size,
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigKeychain {
    #[serde(default)]
    service: Option<String>,
    #[serde(default)]
    account: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ConfigSignerHelper {
    #[serde(default)]
    pub(crate) path: Option<PathBuf>,
    #[serde(default)]
    blake3: Option<String>,
    #[serde(default)]
    allow_unsandboxed: Option<bool>,
}

fn config_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("FRAMKEY_NATIVE_HOST_CONFIG") {
        return Ok(PathBuf::from(path));
    }
    let home = std::env::var_os("HOME")
        .ok_or_else(|| anyhow::anyhow!("HOME is required to locate FRAMKey native host config"))?;
    Ok(PathBuf::from(home).join(".framkey/native-host.json"))
}

pub(crate) fn default_signer_helper_path() -> Result<PathBuf> {
    let current_exe = std::env::current_exe()?;
    let dir = current_exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("current executable has no parent directory"))?;
    dir.join("framkey-signer-helper")
        .canonicalize()
        .context("failed to resolve default signer helper next to native host")
}

pub(crate) fn default_helper_sandbox(allow_unsandboxed: bool) -> Result<SignerHelperSandbox> {
    if allow_unsandboxed {
        return Ok(SignerHelperSandbox::DisabledByConfig);
    }

    #[cfg(target_os = "macos")]
    {
        let sandbox_exec = Path::new("/usr/bin/sandbox-exec");
        if !sandbox_exec.exists() {
            anyhow::bail!(
                "macOS signer helper sandbox is required but /usr/bin/sandbox-exec was not found"
            );
        }
        Ok(SignerHelperSandbox::MacosSandboxExecNoNetwork)
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(SignerHelperSandbox::UnsupportedPlatform)
    }
}

pub(crate) fn parse_save_type(value: &str) -> Result<GbaSaveType> {
    match value {
        "gba-eeprom-64k" | "gba-eeprom64k" => Ok(GbaSaveType::Eeprom64k),
        "gba-sram-fram-256k" | "gba-sram-256k" | "gba-fram-256k" => Ok(GbaSaveType::SramFram256k),
        "gba-sram-fram-512kbit"
        | "gba-sram-fram-512k"
        | "gba-sram-fram-64kib"
        | "gba-sram-512k"
        | "gba-fram-512k" => Ok(GbaSaveType::SramFram512Kbit),
        "gba-sram-fram-1mbit"
        | "gba-sram-fram-1m"
        | "gba-sram-fram-128k"
        | "gba-sram-1m"
        | "gba-fram-1m" => Ok(GbaSaveType::SramFram1Mbit),
        _ => anyhow::bail!("unsupported GBA save type {value}"),
    }
}

pub(crate) fn save_type_name(save_type: GbaSaveType) -> &'static str {
    match save_type {
        GbaSaveType::Eeprom64k => "gba-eeprom-64k",
        GbaSaveType::SramFram256k => "gba-sram-fram-256k",
        GbaSaveType::SramFram512Kbit => "gba-sram-fram-512kbit",
        GbaSaveType::SramFram1Mbit => "gba-sram-fram-1mbit",
        _ => "unknown",
    }
}

fn env_save_type() -> Result<Option<GbaSaveType>> {
    std::env::var("FRAMKEY_GBA_SAVE_TYPE")
        .ok()
        .map(|value| parse_save_type(&value))
        .transpose()
}

fn env_usize(name: &str) -> Result<Option<usize>> {
    std::env::var(name)
        .ok()
        .map(|value| {
            value
                .parse::<usize>()
                .with_context(|| format!("failed to parse {name}={value} as usize"))
        })
        .transpose()
}

fn env_truthy(name: &str) -> bool {
    matches!(
        std::env::var(name).as_deref(),
        Ok("1") | Ok("true") | Ok("yes") | Ok("on")
    )
}

fn normalize_blake3_hex(value: &str) -> Result<String> {
    let value = value.trim();
    let value = value.strip_prefix("0x").unwrap_or(value);
    let bytes = decode_hex_array::<32>(value)?;
    Ok(encode_hex(&bytes))
}

pub(crate) fn validate_chain_id(chain_id: &str) -> Result<()> {
    let Some(hex) = chain_id.strip_prefix("0x") else {
        anyhow::bail!("chain id must be 0x-prefixed hex");
    };
    if hex.is_empty() || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        anyhow::bail!("chain id must be 0x-prefixed hex");
    }
    Ok(())
}

fn validate_keychain_name(label: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("native host Keychain {label} must not be blank");
    }
    if value.trim() != value {
        anyhow::bail!("native host Keychain {label} must not have leading or trailing whitespace");
    }
    if value.chars().any(char::is_control) {
        anyhow::bail!("native host Keychain {label} must not contain control characters");
    }
    Ok(())
}

fn validate_device_hint(label: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("native host {label} must not be blank");
    }
    if value.trim() != value {
        anyhow::bail!("native host {label} must not have leading or trailing whitespace");
    }
    if value.chars().any(char::is_control) {
        anyhow::bail!("native host {label} must not contain control characters");
    }
    Ok(())
}
