use std::{
    fs::{self},
    io::Write,
    path::{Path, PathBuf},
};

#[cfg(unix)]
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt, PermissionsExt};

use anyhow::{Context, Result};
use framkey_crypto::{decode_hex_array, encode_hex};
use framkey_gbxcart::GbaSaveType;

use crate::*;

pub(crate) fn config_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("FRAMKEY_DESKTOP_CONFIG") {
        return Ok(PathBuf::from(path));
    }
    let home = std::env::var_os("HOME")
        .ok_or_else(|| anyhow::anyhow!("HOME is required to locate FRAMKey desktop config"))?;
    Ok(PathBuf::from(home).join(".framkey/desktop.json"))
}

pub(crate) fn transaction_activity_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("FRAMKEY_DESKTOP_ACTIVITY_PATH") {
        return Ok(PathBuf::from(path));
    }
    let home = std::env::var_os("HOME").ok_or_else(|| {
        anyhow::anyhow!("HOME is required to locate FRAMKey transaction activity")
    })?;
    Ok(PathBuf::from(home).join(".framkey/transaction-activity.json"))
}

pub(crate) fn wallet_ui_state_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("FRAMKEY_DESKTOP_WALLET_STATE_PATH") {
        return Ok(PathBuf::from(path));
    }
    let home = std::env::var_os("HOME")
        .ok_or_else(|| anyhow::anyhow!("HOME is required to locate FRAMKey wallet UI state"))?;
    Ok(PathBuf::from(home).join(".framkey/wallet-state.json"))
}

pub(crate) fn recovery_ui_state_path() -> Result<PathBuf> {
    if let Ok(path) = std::env::var("FRAMKEY_DESKTOP_RECOVERY_STATE_PATH") {
        return Ok(PathBuf::from(path));
    }
    let home = std::env::var_os("HOME")
        .ok_or_else(|| anyhow::anyhow!("HOME is required to locate FRAMKey recovery state"))?;
    Ok(PathBuf::from(home).join(".framkey/recovery-state.json"))
}

pub(crate) fn write_json_atomically(path: &Path, bytes: &[u8]) -> Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        create_private_parent_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let tmp_path = path.with_extension("tmp");
    {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        options.mode(PRIVATE_FILE_MODE);
        let mut file = options
            .open(&tmp_path)
            .with_context(|| format!("failed to create {}", tmp_path.display()))?;
        set_private_file_permissions(&tmp_path)?;
        file.write_all(bytes)
            .with_context(|| format!("failed to write {}", tmp_path.display()))?;
        file.write_all(b"\n")
            .with_context(|| format!("failed to finish {}", tmp_path.display()))?;
        file.sync_all()
            .with_context(|| format!("failed to sync {}", tmp_path.display()))?;
    }
    fs::rename(&tmp_path, path).with_context(|| format!("failed to replace {}", path.display()))?;
    set_private_file_permissions(path)?;
    Ok(())
}

pub(crate) fn create_private_dir_all(path: &Path) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("failed to create {}", path.display()))?;
    set_private_dir_permissions(path)
}

pub(crate) fn create_private_dir(path: &Path) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        let mut builder = fs::DirBuilder::new();
        builder.mode(PRIVATE_DIR_MODE);
        builder.create(path)?;
    }
    #[cfg(not(unix))]
    {
        fs::create_dir(path)?;
    }
    set_private_dir_permissions(path).map_err(std::io::Error::other)?;
    Ok(())
}

pub(crate) fn create_private_parent_dir_all(path: &Path) -> Result<()> {
    if path.exists() {
        fs::create_dir_all(path).with_context(|| format!("failed to create {}", path.display()))?;
        return Ok(());
    }
    create_private_dir_all(path)
}

pub(crate) fn set_private_dir_permissions(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        fs::set_permissions(path, fs::Permissions::from_mode(PRIVATE_DIR_MODE))
            .with_context(|| format!("failed to set private permissions on {}", path.display()))?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

pub(crate) fn set_private_file_permissions(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        fs::set_permissions(path, fs::Permissions::from_mode(PRIVATE_FILE_MODE))
            .with_context(|| format!("failed to set private permissions on {}", path.display()))?;
    }
    #[cfg(not(unix))]
    {
        let _ = path;
    }
    Ok(())
}

pub(crate) fn default_signer_helper_path() -> Result<PathBuf> {
    let current_exe = std::env::current_exe()?;
    default_signer_helper_path_for_exe(&current_exe)
}

pub(crate) fn default_signer_helper_path_for_exe(current_exe: &Path) -> Result<PathBuf> {
    let candidates = signer_helper_path_candidates(current_exe)?;
    for candidate in &candidates {
        if candidate.exists() {
            return candidate.canonicalize().with_context(|| {
                format!("failed to resolve signer helper {}", candidate.display())
            });
        }
    }
    candidates
        .into_iter()
        .next()
        .ok_or_else(|| anyhow::anyhow!("no signer helper path candidates available"))
}

pub(crate) fn signer_helper_path_candidates(current_exe: &Path) -> Result<Vec<PathBuf>> {
    let dir = current_exe
        .parent()
        .ok_or_else(|| anyhow::anyhow!("current executable has no parent directory"))?;
    let mut roots = vec![dir.to_path_buf()];
    if dir.file_name().and_then(|name| name.to_str()) == Some("MacOS")
        && let Some(contents) = dir.parent()
    {
        roots.push(contents.join("Resources"));
        roots.push(contents.join("Resources/binaries"));
    }

    let names = signer_helper_file_names();
    let mut candidates = Vec::new();
    for root in roots {
        for name in &names {
            candidates.push(root.join(name));
        }
    }
    Ok(candidates)
}

pub(crate) fn signer_helper_file_names() -> Vec<String> {
    let plain = format!("{SIGNER_HELPER_BASENAME}{}", std::env::consts::EXE_SUFFIX);
    let Some(target) = option_env!("FRAMKEY_BUILD_TARGET") else {
        return vec![plain];
    };
    let sidecar = format!(
        "{SIGNER_HELPER_BASENAME}-{target}{}",
        std::env::consts::EXE_SUFFIX
    );
    if plain == sidecar {
        vec![plain]
    } else {
        vec![plain, sidecar]
    }
}

pub(crate) fn default_helper_sandbox(allow_unsandboxed: bool) -> Result<SignerHelperSandbox> {
    if allow_unsandboxed {
        return Ok(SignerHelperSandbox::DisabledByConfig);
    }

    #[cfg(target_os = "macos")]
    {
        Ok(SignerHelperSandbox::MacosProcessIdentity)
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(SignerHelperSandbox::UnsupportedPlatform)
    }
}

pub(crate) fn macos_sandbox_exec_helper() -> Result<SignerHelperSandbox> {
    #[cfg(target_os = "macos")]
    {
        let sandbox_exec = Path::new("/usr/bin/sandbox-exec");
        if !sandbox_exec.exists() {
            anyhow::bail!(
                "macOS signer helper sandbox-exec mode was requested but /usr/bin/sandbox-exec was not found"
            );
        }
        Ok(SignerHelperSandbox::MacosSandboxExecNoNetwork)
    }

    #[cfg(not(target_os = "macos"))]
    {
        Ok(SignerHelperSandbox::UnsupportedPlatform)
    }
}

pub(crate) fn parse_wallet_mode(value: &str) -> Result<DesktopWalletConfig> {
    match value {
        "keychain_vault" | "keychain" | "vault" => Ok(DesktopWalletConfig::KeychainVault),
        "mock_in_memory" | "mock" | "dev_mock" => Ok(DesktopWalletConfig::MockInMemory),
        _ => anyhow::bail!("unsupported FRAMKey wallet mode {value}"),
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

pub(crate) fn env_save_type() -> Result<Option<GbaSaveType>> {
    std::env::var("FRAMKEY_GBA_SAVE_TYPE")
        .ok()
        .map(|value| parse_save_type(&value))
        .transpose()
}

pub(crate) fn env_usize(name: &str) -> Result<Option<usize>> {
    std::env::var(name)
        .ok()
        .map(|value| {
            value
                .parse::<usize>()
                .with_context(|| format!("failed to parse {name}={value} as usize"))
        })
        .transpose()
}

pub(crate) fn env_u64(name: &str) -> Result<Option<u64>> {
    std::env::var(name)
        .ok()
        .map(|value| {
            value
                .parse::<u64>()
                .with_context(|| format!("failed to parse {name} as u64"))
        })
        .transpose()
}

pub(crate) fn env_string(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .and_then(|value| non_empty_trimmed(value.as_str()))
        .or_else(|| dev_dotenv_string(name))
}

pub(crate) fn env_truthy(name: &str) -> bool {
    matches!(
        std::env::var(name).as_deref(),
        Ok("1") | Ok("true") | Ok("yes") | Ok("on")
    )
}

pub(crate) fn alchemy_token_from_env() -> Option<String> {
    env_string("FRAMKEY_ALCHEMY_TOKEN").or_else(|| env_string("ALCHEMY_TOKEN"))
}

pub(crate) fn alchemy_endpoint_from_token(network: &str, token: &str) -> Result<String> {
    validate_alchemy_network(network)?;
    validate_alchemy_token(token)?;
    Ok(format!("https://{network}.g.alchemy.com/v2/{token}"))
}

pub(crate) fn validate_alchemy_endpoint(url: &str) -> Result<()> {
    if url.trim() != url || url.is_empty() || url.chars().any(char::is_control) {
        anyhow::bail!("Alchemy RPC URL is malformed");
    }
    if !url.starts_with("https://") && !url.starts_with("http://") {
        anyhow::bail!("Alchemy RPC URL must start with http:// or https://");
    }
    Ok(())
}

pub(crate) fn validate_alchemy_network(network: &str) -> Result<()> {
    if network.is_empty()
        || network.len() > 64
        || !network
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        anyhow::bail!("Alchemy network must be a short alphanumeric slug");
    }
    Ok(())
}

pub(crate) fn validate_alchemy_token(token: &str) -> Result<()> {
    if token.is_empty()
        || token.chars().any(char::is_whitespace)
        || token.chars().any(char::is_control)
    {
        anyhow::bail!("Alchemy token is malformed");
    }
    Ok(())
}

pub(crate) fn is_alchemy_endpoint(url: &str) -> bool {
    url.starts_with("https://") && url.contains(".g.alchemy.com/")
}

pub(crate) fn validate_hex_quantity(value: &str, label: &str) -> Result<()> {
    let Some(hex) = value.strip_prefix("0x") else {
        anyhow::bail!("{label} must be a 0x-prefixed hex quantity");
    };
    if hex.is_empty() || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        anyhow::bail!("{label} must be a 0x-prefixed hex quantity");
    }
    Ok(())
}

#[cfg(debug_assertions)]
pub(crate) fn dev_dotenv_string(name: &str) -> Option<String> {
    for path in dev_dotenv_paths() {
        if let Some(value) = read_dotenv_value(&path, name) {
            return Some(value);
        }
    }
    None
}

#[cfg(not(debug_assertions))]
pub(crate) fn dev_dotenv_string(_name: &str) -> Option<String> {
    None
}

#[cfg(debug_assertions)]
pub(crate) fn dev_dotenv_paths() -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Ok(cwd) = std::env::current_dir() {
        paths.push(cwd.join(".env"));
    }
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if let Some(repo_root) = manifest_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
    {
        paths.push(repo_root.join(".env"));
    }
    paths
}

#[cfg(debug_assertions)]
pub(crate) fn read_dotenv_value(path: &Path, name: &str) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if key.trim() != name {
            continue;
        }
        let value = strip_dotenv_quotes(value.trim());
        if let Some(value) = non_empty_trimmed(value) {
            return Some(value);
        }
    }
    None
}

#[cfg(debug_assertions)]
pub(crate) fn strip_dotenv_quotes(value: &str) -> &str {
    if value.len() >= 2 {
        let bytes = value.as_bytes();
        if (bytes[0] == b'"' && bytes[value.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[value.len() - 1] == b'\'')
        {
            return &value[1..value.len() - 1];
        }
    }
    value
}

pub(crate) fn non_empty_trimmed(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

pub(crate) fn normalize_blake3_hex(value: &str) -> Result<String> {
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

pub(crate) fn validate_desktop_device_config(device: &DeviceConfig) -> Result<()> {
    match device {
        DeviceConfig::File { path } => validate_desktop_path("file save-image path", path),
        DeviceConfig::GbxCart {
            port: Some(port), ..
        } => validate_desktop_text("GBxCart port", port),
        DeviceConfig::GbxCart { .. } => Ok(()),
    }
}

pub(crate) fn validate_desktop_keychain_name(label: &str, value: &str) -> Result<()> {
    validate_desktop_text(&format!("Keychain {label}"), value)
}

pub(crate) fn validate_desktop_text(label: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        anyhow::bail!("desktop {label} must not be blank");
    }
    if value.trim() != value {
        anyhow::bail!("desktop {label} must not have leading or trailing whitespace");
    }
    if value.chars().any(char::is_control) {
        anyhow::bail!("desktop {label} must not contain control characters");
    }
    Ok(())
}

pub(crate) fn validate_desktop_path(label: &str, path: &Path) -> Result<()> {
    if path.as_os_str().is_empty() {
        anyhow::bail!("desktop {label} must not be blank");
    }
    let path_text = path.display().to_string();
    if path_text.chars().any(char::is_control) {
        anyhow::bail!("desktop {label} must not contain control characters");
    }
    Ok(())
}
