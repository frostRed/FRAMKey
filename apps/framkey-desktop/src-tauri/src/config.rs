use std::{
    path::PathBuf,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use framkey_crypto::SecretBytes;
use framkey_device::{FileImageDevice, VaultDevice};
use framkey_evm::EvmAddress;
use framkey_gbxcart::{GbaSaveType, GbxCartConfig, GbxCartDevice};
use framkey_ipc::{IpcError, IpcErrorCode};
use framkey_simulation::{
    AlchemyRpcSimulationClient, AlchemyRpcSimulationConfig, TransactionReviewReport,
    TransactionSimulationRequest, local_transaction_review, simulate_transaction_review,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tauri::Url;

use crate::*;

#[derive(Debug)]

pub(crate) struct MockWalletSnapshot {
    pub(crate) secret: SecretBytes<32>,
    pub(crate) address: String,
    pub(crate) secret_hash: String,
}

pub(crate) enum ProviderResponse {
    Result(Value),
    Error(ProviderError),
}

#[derive(Debug, Clone)]
pub(crate) struct DesktopConfig {
    pub(crate) chain_id: String,
    pub(crate) device: DeviceConfig,
    pub(crate) wallet: DesktopWalletConfig,
    pub(crate) keychain_service: String,
    pub(crate) keychain_account: String,
    pub(crate) helper: SignerHelperConfig,
    pub(crate) simulation: DesktopSimulationConfig,
    pub(crate) rpc: Option<DesktopRpcConfig>,
}

impl DesktopConfig {
    pub(crate) fn load() -> Result<Self> {
        let mut config = Self::default_for_repo()?;
        let mut file_simulation_explicit = false;
        if let Some(file_config) = ConfigFile::load_optional()? {
            file_simulation_explicit = file_config.simulation.is_some();
            config.apply_file(file_config)?;
        }
        config.apply_env(file_simulation_explicit)?;
        config.validate()?;
        Ok(config)
    }

    pub(crate) fn default_for_repo() -> Result<Self> {
        Ok(Self {
            chain_id: DEFAULT_CHAIN_ID.to_owned(),
            device: DeviceConfig::GbxCart {
                port: Some(DEFAULT_GBXCART_PORT.to_owned()),
                save_type: GbaSaveType::SramFram512Kbit,
                expected_save_size: None,
            },
            wallet: DesktopWalletConfig::KeychainVault,
            keychain_service: DEFAULT_KEYCHAIN_SERVICE.to_owned(),
            keychain_account: DEFAULT_KEYCHAIN_ACCOUNT.to_owned(),
            helper: SignerHelperConfig {
                path: default_signer_helper_path()?,
                expected_blake3: None,
                sandbox: default_helper_sandbox(false)?,
            },
            simulation: DesktopSimulationConfig::LocalDecoderOnly,
            rpc: None,
        })
    }

    pub(crate) fn apply_file(&mut self, file: ConfigFile) -> Result<()> {
        if let Some(chain_id) = file.chain_id {
            self.chain_id = chain_id;
        }
        if let Some(device) = file.device {
            self.device = device.into_runtime()?;
        }
        if let Some(wallet) = file.wallet {
            self.wallet = wallet.into_runtime();
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
        if let Some(simulation) = file.simulation {
            self.simulation = simulation.into_runtime()?;
        }
        if let Some(rpc) = file.rpc {
            self.rpc = Some(rpc.into_runtime()?);
        }
        Ok(())
    }

    pub(crate) fn apply_env(&mut self, file_simulation_explicit: bool) -> Result<()> {
        if let Ok(chain_id) = std::env::var("FRAMKEY_DESKTOP_CHAIN_ID") {
            self.chain_id = chain_id;
        }
        if let Ok(path) = std::env::var("FRAMKEY_SAVE_IMAGE_PATH") {
            self.device = DeviceConfig::File {
                path: PathBuf::from(path),
            };
        }
        if let Ok(port) = std::env::var("FRAMKEY_GBXCART_PORT") {
            self.device = DeviceConfig::GbxCart {
                port: Some(port),
                save_type: env_save_type()?.unwrap_or(GbaSaveType::SramFram512Kbit),
                expected_save_size: env_usize("FRAMKEY_EXPECTED_SAVE_SIZE")?,
            };
        }
        if let Ok(save_type) = std::env::var("FRAMKEY_GBA_SAVE_TYPE")
            && let DeviceConfig::GbxCart {
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
        if env_truthy("FRAMKEY_DESKTOP_ALLOW_UNSANDBOXED_HELPER") {
            self.helper.sandbox = default_helper_sandbox(true)?;
        }
        if env_truthy("FRAMKEY_DESKTOP_EXPERIMENTAL_SANDBOX_EXEC_HELPER") {
            self.helper.sandbox = macos_sandbox_exec_helper()?;
        }
        if let Some(wallet_mode) =
            env_string("FRAMKEY_WALLET_MODE").or_else(|| env_string("FRAMKEY_DESKTOP_WALLET_MODE"))
        {
            self.wallet = parse_wallet_mode(&wallet_mode)?;
        }
        self.apply_rpc_env()?;
        self.apply_simulation_env(file_simulation_explicit)?;
        Ok(())
    }

    pub(crate) fn validate(&self) -> Result<()> {
        validate_chain_id(&self.chain_id)?;
        validate_desktop_device_config(&self.device)?;
        validate_desktop_keychain_name("service", &self.keychain_service)?;
        validate_desktop_keychain_name("account", &self.keychain_account)?;
        validate_desktop_path("signer helper path", &self.helper.path)?;
        self.simulation.validate()?;
        if let Some(rpc) = &self.rpc {
            rpc.validate()?;
        }
        Ok(())
    }

    pub(crate) fn switch_to_alchemy_chain(
        &mut self,
        chain: SupportedAlchemyChain,
        alchemy_token: &str,
    ) -> Result<()> {
        let timeout_ms = self
            .rpc
            .as_ref()
            .map(|rpc| rpc.timeout_ms)
            .unwrap_or(DEFAULT_RPC_TIMEOUT_MS);
        let endpoint_url = alchemy_endpoint_from_token(chain.alchemy_network, alchemy_token)?;
        self.chain_id = chain.chain_id.to_owned();
        self.rpc = Some(DesktopRpcConfig {
            endpoint_url: endpoint_url.clone(),
            network: Some(chain.alchemy_network.to_owned()),
            timeout_ms,
        });
        if let DesktopSimulationConfig::AlchemyAssetChanges {
            endpoint_url: simulation_endpoint,
            network,
            ..
        } = &mut self.simulation
        {
            *simulation_endpoint = endpoint_url;
            *network = Some(chain.alchemy_network.to_owned());
        }
        self.validate()
    }

    pub(crate) fn apply_simulation_env(&mut self, file_simulation_explicit: bool) -> Result<()> {
        let provider = env_string("FRAMKEY_SIMULATION_PROVIDER");
        let default_rpc = if provider.is_none() && !file_simulation_explicit {
            self.rpc
                .as_ref()
                .filter(|rpc| is_alchemy_endpoint(&rpc.endpoint_url))
        } else {
            None
        };
        let rpc_url = env_string("FRAMKEY_ALCHEMY_RPC_URL")
            .or_else(|| env_string("ALCHEMY_RPC_URL"))
            .or_else(|| default_rpc.map(|rpc| rpc.endpoint_url.clone()));
        let token = env_string("FRAMKEY_ALCHEMY_TOKEN").or_else(|| env_string("ALCHEMY_TOKEN"));
        let network = env_string("FRAMKEY_ALCHEMY_NETWORK")
            .or_else(|| default_rpc.and_then(|rpc| rpc.network.clone()));
        self.simulation = simulation_config_from_env(
            &self.simulation,
            provider.as_deref(),
            rpc_url,
            token,
            network,
            env_u64("FRAMKEY_SIMULATION_TIMEOUT_MS")?,
            env_string("FRAMKEY_SIMULATION_DEFAULT_GAS"),
            !file_simulation_explicit,
        )?;
        Ok(())
    }

    pub(crate) fn apply_rpc_env(&mut self) -> Result<()> {
        let rpc_url = env_string("FRAMKEY_RPC_URL")
            .or_else(|| env_string("FRAMKEY_ALCHEMY_RPC_URL"))
            .or_else(|| env_string("ALCHEMY_RPC_URL"));
        let token = env_string("FRAMKEY_ALCHEMY_TOKEN").or_else(|| env_string("ALCHEMY_TOKEN"));
        self.rpc = rpc_config_from_env(
            self.rpc.as_ref(),
            rpc_url,
            token,
            env_string("FRAMKEY_ALCHEMY_NETWORK"),
            env_u64("FRAMKEY_RPC_TIMEOUT_MS")?,
        )?;
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn simulation_config_from_env(
    current: &DesktopSimulationConfig,
    provider: Option<&str>,
    rpc_url: Option<String>,
    token: Option<String>,
    network: Option<String>,
    timeout_ms: Option<u64>,
    default_gas: Option<String>,
    allow_default_alchemy: bool,
) -> Result<DesktopSimulationConfig> {
    let has_alchemy_inputs = rpc_url.is_some() || token.is_some();
    match provider {
        Some("local_decoder_only") => Ok(DesktopSimulationConfig::LocalDecoderOnly),
        Some("alchemy_asset_changes") => alchemy_simulation_config_from_inputs(
            current,
            rpc_url,
            token,
            network,
            timeout_ms,
            default_gas,
            true,
        ),
        Some(other) => anyhow::bail!("unsupported FRAMKEY_SIMULATION_PROVIDER {other}"),
        None => match current {
            DesktopSimulationConfig::LocalDecoderOnly
                if allow_default_alchemy && has_alchemy_inputs =>
            {
                alchemy_simulation_config_from_inputs(
                    current,
                    rpc_url,
                    token,
                    network,
                    timeout_ms,
                    default_gas,
                    false,
                )
            }
            DesktopSimulationConfig::LocalDecoderOnly => {
                Ok(DesktopSimulationConfig::LocalDecoderOnly)
            }
            DesktopSimulationConfig::AlchemyAssetChanges { .. } => {
                alchemy_simulation_config_from_inputs(
                    current,
                    rpc_url,
                    token,
                    network,
                    timeout_ms,
                    default_gas,
                    false,
                )
            }
        },
    }
}

pub(crate) fn alchemy_simulation_config_from_inputs(
    current: &DesktopSimulationConfig,
    rpc_url: Option<String>,
    token: Option<String>,
    network: Option<String>,
    timeout_ms: Option<u64>,
    default_gas: Option<String>,
    require_endpoint: bool,
) -> Result<DesktopSimulationConfig> {
    let current_endpoint = match current {
        DesktopSimulationConfig::AlchemyAssetChanges { endpoint_url, .. } => {
            Some(endpoint_url.clone())
        }
        DesktopSimulationConfig::LocalDecoderOnly => None,
    };
    let current_network = match current {
        DesktopSimulationConfig::AlchemyAssetChanges { network, .. } => network.clone(),
        DesktopSimulationConfig::LocalDecoderOnly => None,
    };
    let network = network
        .or(current_network)
        .unwrap_or_else(|| DEFAULT_ALCHEMY_NETWORK.to_owned());
    let endpoint_url = alchemy_endpoint_from_inputs(
        &network,
        rpc_url,
        token,
        current_endpoint,
        require_endpoint,
        "FRAMKEY_SIMULATION_PROVIDER=alchemy_asset_changes requires FRAMKEY_ALCHEMY_RPC_URL or ALCHEMY_TOKEN",
    )?;
    let timeout_ms = timeout_ms
        .or_else(|| current.timeout_ms())
        .unwrap_or(DEFAULT_SIMULATION_TIMEOUT_MS);
    let default_gas = default_gas
        .or_else(|| current.default_gas().map(str::to_owned))
        .unwrap_or_else(|| DEFAULT_SIMULATION_DEFAULT_GAS.to_owned());
    Ok(DesktopSimulationConfig::AlchemyAssetChanges {
        endpoint_url,
        network: Some(network),
        timeout_ms,
        default_gas,
    })
}

pub(crate) fn rpc_config_from_env(
    current: Option<&DesktopRpcConfig>,
    rpc_url: Option<String>,
    token: Option<String>,
    network: Option<String>,
    timeout_ms: Option<u64>,
) -> Result<Option<DesktopRpcConfig>> {
    let timeout_ms = timeout_ms
        .or_else(|| current.map(|rpc| rpc.timeout_ms))
        .unwrap_or(DEFAULT_RPC_TIMEOUT_MS);

    if let Some(endpoint_url) = rpc_url {
        let network = network.unwrap_or_else(|| DEFAULT_ALCHEMY_NETWORK.to_owned());
        return Ok(Some(DesktopRpcConfig {
            endpoint_url,
            network: Some(network),
            timeout_ms,
        }));
    }

    if let Some(token) = token {
        let network = network.unwrap_or_else(|| DEFAULT_ALCHEMY_NETWORK.to_owned());
        return Ok(Some(DesktopRpcConfig {
            endpoint_url: alchemy_endpoint_from_token(&network, &token)?,
            network: Some(network),
            timeout_ms,
        }));
    }

    Ok(current.map(|rpc| DesktopRpcConfig {
        endpoint_url: rpc.endpoint_url.clone(),
        network: rpc.network.clone(),
        timeout_ms,
    }))
}

pub(crate) fn alchemy_endpoint_from_inputs(
    network: &str,
    rpc_url: Option<String>,
    token: Option<String>,
    current_endpoint: Option<String>,
    require_endpoint: bool,
    missing_message: &str,
) -> Result<String> {
    if let Some(endpoint_url) = rpc_url {
        return Ok(endpoint_url);
    }
    if let Some(token) = token {
        return alchemy_endpoint_from_token(network, &token);
    }
    if let Some(endpoint_url) = current_endpoint {
        return Ok(endpoint_url);
    }
    if require_endpoint {
        anyhow::bail!("{missing_message}");
    }
    anyhow::bail!("Alchemy simulation endpoint is not configured")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DesktopWalletConfig {
    KeychainVault,
    MockInMemory,
}

impl DesktopWalletConfig {
    pub(crate) fn send_transaction_capability(self) -> &'static str {
        match self {
            Self::KeychainVault => "signer_helper_approval_required",
            Self::MockInMemory => "mock_approval_required",
        }
    }

    pub(crate) fn describe(self) -> Value {
        match self {
            Self::KeychainVault => json!({
                "kind": "keychain_vault",
                "mock": false,
                "plaintextSecretOwner": "signer_helper",
            }),
            Self::MockInMemory => json!({
                "kind": "mock_in_memory",
                "mock": true,
                "lifetime": "process",
                "plaintextSecretOwner": "desktop_process",
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DesktopRpcConfig {
    pub(crate) endpoint_url: String,
    pub(crate) network: Option<String>,
    pub(crate) timeout_ms: u64,
}

impl DesktopRpcConfig {
    pub(crate) fn describe(&self) -> Value {
        json!({
            "kind": "alchemy_rpc",
            "configured": true,
            "network": self.network,
            "timeoutMs": self.timeout_ms,
        })
    }

    pub(crate) fn validate(&self) -> Result<()> {
        validate_alchemy_endpoint(&self.endpoint_url)?;
        if let Some(network) = &self.network {
            validate_alchemy_network(network)?;
        }
        if self.timeout_ms == 0 || self.timeout_ms > 30_000 {
            anyhow::bail!("read RPC timeout must be between 1 and 30000 ms");
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub(crate) enum DeviceConfig {
    File {
        path: PathBuf,
    },
    GbxCart {
        port: Option<String>,
        save_type: GbaSaveType,
        expected_save_size: Option<usize>,
    },
}

impl DeviceConfig {
    pub(crate) fn open_device(&self) -> Box<dyn VaultDevice> {
        match self {
            Self::File { path } => Box::new(FileImageDevice::new(path.clone())),
            Self::GbxCart {
                port,
                save_type,
                expected_save_size,
            } => Box::new(GbxCartDevice::new(GbxCartConfig {
                port_hint: port.clone(),
                expected_save_size: *expected_save_size,
                save_type: Some(*save_type),
            })),
        }
    }

    pub(crate) fn vault_image_size(&self) -> Result<usize> {
        let size = match self {
            Self::File { path } => match std::fs::metadata(path) {
                Ok(metadata) => {
                    if !metadata.is_file() {
                        anyhow::bail!(
                            "configured save image path is not a file: {}",
                            path.display()
                        );
                    }
                    metadata.len() as usize
                }
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    GbaSaveType::SramFram512Kbit.save_size()
                }
                Err(error) => {
                    return Err(error)
                        .with_context(|| format!("failed to inspect {}", path.display()));
                }
            },
            Self::GbxCart {
                save_type,
                expected_save_size,
                ..
            } => {
                let size = save_type.save_size();
                if let Some(expected) = expected_save_size
                    && *expected != size
                {
                    anyhow::bail!(
                        "expected save size {} does not match {} ({})",
                        expected,
                        save_type.label(),
                        size
                    );
                }
                size
            }
        };

        validate_vault_image_size(size)?;
        Ok(size)
    }

    pub(crate) fn describe(&self) -> Value {
        match self {
            Self::File { path } => json!({
                "kind": "file",
                "path": path.display().to_string(),
            }),
            Self::GbxCart {
                port,
                save_type,
                expected_save_size,
            } => json!({
                "kind": "gbx_cart",
                "port": port,
                "saveType": save_type_name(*save_type),
                "expectedSaveSize": expected_save_size,
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum DesktopSimulationConfig {
    LocalDecoderOnly,
    AlchemyAssetChanges {
        endpoint_url: String,
        network: Option<String>,
        timeout_ms: u64,
        default_gas: String,
    },
}

impl DesktopSimulationConfig {
    pub(crate) fn capability_value(&self) -> &'static str {
        match self {
            Self::LocalDecoderOnly => "local_decoder_only",
            Self::AlchemyAssetChanges { .. } => "alchemy_asset_changes",
        }
    }

    pub(crate) fn describe(&self) -> Value {
        match self {
            Self::LocalDecoderOnly => json!({
                "kind": "local_decoder_only",
                "liveProvider": false,
            }),
            Self::AlchemyAssetChanges {
                network,
                timeout_ms,
                default_gas,
                ..
            } => json!({
                "kind": "alchemy_asset_changes",
                "liveProvider": true,
                "rpcUrlConfigured": true,
                "network": network,
                "timeoutMs": timeout_ms,
                "defaultGas": default_gas,
            }),
        }
    }

    pub(crate) fn transaction_review(
        &self,
        method: &str,
        params: &Value,
        chain_id: &str,
    ) -> TransactionReviewReport {
        match self {
            Self::LocalDecoderOnly => local_transaction_review(method, params, chain_id),
            Self::AlchemyAssetChanges {
                endpoint_url,
                timeout_ms,
                default_gas,
                ..
            } => {
                let client = AlchemyRpcSimulationClient::new(AlchemyRpcSimulationConfig {
                    endpoint_url: endpoint_url.clone(),
                    timeout_ms: *timeout_ms,
                    default_gas: default_gas.clone(),
                });
                simulate_transaction_review(
                    &client,
                    TransactionSimulationRequest {
                        method,
                        params,
                        default_chain_id: chain_id,
                    },
                )
            }
        }
    }

    pub(crate) fn timeout_ms(&self) -> Option<u64> {
        match self {
            Self::AlchemyAssetChanges { timeout_ms, .. } => Some(*timeout_ms),
            Self::LocalDecoderOnly => None,
        }
    }

    pub(crate) fn default_gas(&self) -> Option<&str> {
        match self {
            Self::AlchemyAssetChanges { default_gas, .. } => Some(default_gas),
            Self::LocalDecoderOnly => None,
        }
    }

    pub(crate) fn validate(&self) -> Result<()> {
        match self {
            Self::LocalDecoderOnly => Ok(()),
            Self::AlchemyAssetChanges {
                endpoint_url,
                network,
                timeout_ms,
                default_gas,
            } => {
                validate_alchemy_endpoint(endpoint_url)?;
                if let Some(network) = network {
                    validate_alchemy_network(network)?;
                }
                if *timeout_ms == 0 || *timeout_ms > 30_000 {
                    anyhow::bail!("simulation timeout must be between 1 and 30000 ms");
                }
                validate_hex_quantity(default_gas, "simulation default gas")?;
                Ok(())
            }
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
    MacosProcessIdentity,
    MacosSandboxExecNoNetwork,
    DisabledByConfig,
    #[cfg(not(target_os = "macos"))]
    UnsupportedPlatform,
}

impl SignerHelperSandbox {
    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::MacosProcessIdentity => "macos_process_identity",
            Self::MacosSandboxExecNoNetwork => "macos_sandbox_exec_no_network",
            Self::DisabledByConfig => "disabled_by_config",
            #[cfg(not(target_os = "macos"))]
            Self::UnsupportedPlatform => "unsupported_platform",
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct DesktopAccount {
    pub(crate) address: String,
    pub(crate) wallet: Value,
    pub(crate) metadata: Value,
    pub(crate) keychain: Option<Value>,
    pub(crate) helper_report: Option<Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ConfigFile {
    #[serde(default)]
    pub(crate) chain_id: Option<String>,
    #[serde(default)]
    pub(crate) device: Option<ConfigDevice>,
    #[serde(default)]
    pub(crate) wallet: Option<ConfigWallet>,
    #[serde(default)]
    pub(crate) keychain: Option<ConfigKeychain>,
    #[serde(default)]
    pub(crate) signer_helper: Option<ConfigSignerHelper>,
    #[serde(default)]
    pub(crate) simulation: Option<ConfigSimulation>,
    #[serde(default)]
    pub(crate) rpc: Option<ConfigRpc>,
}

impl ConfigFile {
    pub(crate) fn load_optional() -> Result<Option<Self>> {
        let path = config_path()?;
        if !path.exists() {
            return Ok(None);
        }
        let bytes = std::fs::read(&path)
            .with_context(|| format!("failed to read desktop config {}", path.display()))?;
        let config = serde_json::from_slice(&bytes)
            .with_context(|| format!("failed to parse desktop config {}", path.display()))?;
        Ok(Some(config))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub(crate) enum ConfigDevice {
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
    pub(crate) fn into_runtime(self) -> Result<DeviceConfig> {
        match self {
            Self::File { path } => Ok(DeviceConfig::File { path }),
            Self::GbxCart {
                port,
                save_type,
                expected_save_size,
            } => Ok(DeviceConfig::GbxCart {
                port,
                save_type: parse_save_type(&save_type)?,
                expected_save_size,
            }),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub(crate) enum ConfigWallet {
    KeychainVault,
    MockInMemory,
}

impl ConfigWallet {
    pub(crate) fn into_runtime(self) -> DesktopWalletConfig {
        match self {
            Self::KeychainVault => DesktopWalletConfig::KeychainVault,
            Self::MockInMemory => DesktopWalletConfig::MockInMemory,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ConfigKeychain {
    #[serde(default)]
    pub(crate) service: Option<String>,
    #[serde(default)]
    pub(crate) account: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ConfigSignerHelper {
    #[serde(default)]
    pub(crate) path: Option<PathBuf>,
    #[serde(default)]
    pub(crate) blake3: Option<String>,
    #[serde(default)]
    pub(crate) allow_unsandboxed: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub(crate) enum ConfigSimulation {
    LocalDecoderOnly,
    AlchemyAssetChanges {
        rpc_url: String,
        #[serde(default)]
        network: Option<String>,
        #[serde(default)]
        timeout_ms: Option<u64>,
        #[serde(default)]
        default_gas: Option<String>,
    },
}

impl ConfigSimulation {
    pub(crate) fn into_runtime(self) -> Result<DesktopSimulationConfig> {
        match self {
            Self::LocalDecoderOnly => Ok(DesktopSimulationConfig::LocalDecoderOnly),
            Self::AlchemyAssetChanges {
                rpc_url,
                network,
                timeout_ms,
                default_gas,
            } => Ok(DesktopSimulationConfig::AlchemyAssetChanges {
                endpoint_url: rpc_url,
                network,
                timeout_ms: timeout_ms.unwrap_or(DEFAULT_SIMULATION_TIMEOUT_MS),
                default_gas: default_gas
                    .unwrap_or_else(|| DEFAULT_SIMULATION_DEFAULT_GAS.to_owned()),
            }),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub(crate) enum ConfigRpc {
    Alchemy {
        rpc_url: String,
        #[serde(default)]
        network: Option<String>,
        #[serde(default)]
        timeout_ms: Option<u64>,
    },
}

impl ConfigRpc {
    pub(crate) fn into_runtime(self) -> Result<DesktopRpcConfig> {
        match self {
            Self::Alchemy {
                rpc_url,
                network,
                timeout_ms,
            } => Ok(DesktopRpcConfig {
                endpoint_url: rpc_url,
                network,
                timeout_ms: timeout_ms.unwrap_or(DEFAULT_RPC_TIMEOUT_MS),
            }),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SwitchSessionChainRequest {
    pub(crate) chain_id: String,
}

impl SwitchSessionChainRequest {
    pub(crate) fn validate(&self) -> Result<()> {
        self.normalized_chain_id().map(|_| ())
    }

    pub(crate) fn normalized_chain_id(&self) -> Result<String> {
        normalize_chain_id(self.chain_id.trim())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RecoverySmokePackRequest {
    #[serde(default)]
    pub(crate) out_dir: Option<String>,
    #[serde(default)]
    pub(crate) generation: Option<u64>,
}

impl RecoverySmokePackRequest {
    pub(crate) fn validate(&self) -> Result<()> {
        if let Some(generation) = self.generation {
            if generation == 0 {
                anyhow::bail!("recovery smoke generation must be at least 1");
            }
            if generation > 1_000_000 {
                anyhow::bail!("recovery smoke generation is unexpectedly large");
            }
        }
        if let Some(out_dir) = &self.out_dir
            && (out_dir.trim().is_empty() || out_dir.chars().any(char::is_control))
        {
            anyhow::bail!("recovery smoke output directory is malformed");
        }
        Ok(())
    }

    pub(crate) fn out_dir_path(&self) -> Result<PathBuf> {
        match self
            .out_dir
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            Some(path) => user_path(path),
            None => Ok(std::env::temp_dir().join(format!(
                "framkey-recovery-smoke-{}-{}",
                std::process::id(),
                now_unix_ms()
            ))),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CreateKeychainVaultRequest {
    pub(crate) generation: u64,
    pub(crate) recovery_out_dir: String,
    #[serde(default)]
    pub(crate) confirm_overwrite: bool,
}

impl CreateKeychainVaultRequest {
    pub(crate) fn validate(&self) -> Result<()> {
        if self.generation == 0 {
            anyhow::bail!("vault generation must be at least 1");
        }
        if self.generation > 1_000_000 {
            anyhow::bail!("vault generation is unexpectedly large");
        }
        if self.recovery_out_dir.trim().is_empty()
            || self.recovery_out_dir.chars().any(char::is_control)
        {
            anyhow::bail!("recovery output directory is malformed");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RecoverKeychainVaultRequest {
    pub(crate) vault_backup_path: String,
    pub(crate) recovery_files: Vec<String>,
    #[serde(default)]
    pub(crate) confirm_overwrite: bool,
}

impl RecoverKeychainVaultRequest {
    pub(crate) fn validate(&self) -> Result<()> {
        validate_vault_backup_path(&self.vault_backup_path)?;
        validate_recovery_file_path_list(&self.recovery_files)
    }

    pub(crate) fn vault_backup_path(&self) -> Result<PathBuf> {
        vault_backup_path(&self.vault_backup_path)
    }

    pub(crate) fn recovery_file_paths(&self) -> Result<Vec<PathBuf>> {
        recovery_file_paths(&self.recovery_files)
    }
}

pub(crate) fn validate_vault_backup_path(path: &str) -> Result<()> {
    if path.trim().is_empty() || path.chars().any(char::is_control) {
        anyhow::bail!("recovery backup file path is malformed");
    }
    Ok(())
}

pub(crate) fn vault_backup_path(path: &str) -> Result<PathBuf> {
    validate_vault_backup_path(path)?;
    user_path(path.trim())
}

pub(crate) fn validate_recovery_file_path_list(recovery_files: &[String]) -> Result<()> {
    if recovery_files.is_empty() {
        anyhow::bail!("at least one recovery backup file path is required");
    }
    if recovery_files.len() > 4 {
        anyhow::bail!("standard recovery accepts at most four backup files");
    }
    for path in recovery_files {
        if path.trim().is_empty() || path.chars().any(char::is_control) {
            anyhow::bail!("recovery backup file path is malformed");
        }
    }
    Ok(())
}

pub(crate) fn recovery_file_paths(recovery_files: &[String]) -> Result<Vec<PathBuf>> {
    recovery_files
        .iter()
        .map(|path| user_path(path.trim()))
        .collect()
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ValidateRecoverySetRequest {
    pub(crate) recovery_files: Vec<String>,
}

impl ValidateRecoverySetRequest {
    pub(crate) fn validate(&self) -> Result<()> {
        validate_recovery_file_path_list(&self.recovery_files)
    }

    pub(crate) fn recovery_file_paths(&self) -> Result<Vec<PathBuf>> {
        recovery_file_paths(&self.recovery_files)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct RevealPathRequest {
    pub(crate) path: String,
}

impl RevealPathRequest {
    pub(crate) fn path(&self) -> Result<PathBuf> {
        let path = self.path.trim();
        if path.is_empty() || path.chars().any(char::is_control) {
            anyhow::bail!("path is malformed");
        }
        user_path(path)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TransactionActivityRequest {
    #[serde(default)]
    pub(crate) refresh_receipts: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NativeTransferRequest {
    pub(crate) to: String,
    pub(crate) amount: String,
    #[serde(default)]
    pub(crate) chain_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NormalizedNativeTransferRequest {
    pub(crate) to: String,
    pub(crate) value: String,
    pub(crate) amount: String,
    pub(crate) chain_id: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TokenTransferRequest {
    pub(crate) token_contract: String,
    pub(crate) to: String,
    pub(crate) amount: String,
    pub(crate) decimals: Option<u64>,
    #[serde(default)]
    pub(crate) symbol: Option<String>,
    #[serde(default)]
    pub(crate) chain_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NormalizedTokenTransferRequest {
    pub(crate) token_contract: String,
    pub(crate) to: String,
    pub(crate) raw_amount: String,
    pub(crate) amount: String,
    pub(crate) decimals: u8,
    pub(crate) symbol: Option<String>,
    pub(crate) data: String,
    pub(crate) chain_id: String,
}

impl NativeTransferRequest {
    pub(crate) fn normalized(
        &self,
        config: &DesktopConfig,
    ) -> Result<NormalizedNativeTransferRequest> {
        if config.rpc.is_none() {
            anyhow::bail!("Alchemy RPC is required before sending a native transfer");
        }
        let to = self
            .to
            .trim()
            .parse::<EvmAddress>()
            .map_err(|_| anyhow::anyhow!("native transfer recipient is not a valid EVM address"))?
            .to_string();
        let value = native_amount_decimal_to_wei_hex(&self.amount)?;
        let chain_id = normalize_chain_id(&config.chain_id)?;
        if let Some(request_chain_id) = self
            .chain_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let request_chain_id = normalize_chain_id(request_chain_id)?;
            if request_chain_id != chain_id {
                anyhow::bail!(
                    "native transfer chainId {} does not match active {}",
                    request_chain_id,
                    chain_id
                );
            }
        }
        Ok(NormalizedNativeTransferRequest {
            to,
            value,
            amount: self.amount.trim().to_owned(),
            chain_id,
        })
    }
}

impl TokenTransferRequest {
    pub(crate) fn normalized(
        &self,
        config: &DesktopConfig,
    ) -> Result<NormalizedTokenTransferRequest> {
        if config.rpc.is_none() {
            anyhow::bail!("Alchemy RPC is required before sending a token transfer");
        }
        let token_contract = self
            .token_contract
            .trim()
            .parse::<EvmAddress>()
            .map_err(|_| anyhow::anyhow!("token contract is not a valid EVM address"))?
            .to_string();
        let to = self
            .to
            .trim()
            .parse::<EvmAddress>()
            .map_err(|_| anyhow::anyhow!("token transfer recipient is not a valid EVM address"))?
            .to_string();
        let decimals = self
            .decimals
            .ok_or_else(|| anyhow::anyhow!("token decimals are required before sending"))?;
        if decimals > u64::from(u8::MAX) {
            anyhow::bail!("token decimals must be between 0 and 255");
        }
        let decimals_u8 = u8::try_from(decimals).context("token decimals are invalid")?;
        let raw_amount = token_amount_decimal_to_raw_hex(
            &self.amount,
            usize::from(decimals_u8),
            "token transfer",
        )?;
        let data = erc20_transfer_calldata(&to, &raw_amount)?;
        let chain_id = normalize_chain_id(&config.chain_id)?;
        if let Some(request_chain_id) = self
            .chain_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            let request_chain_id = normalize_chain_id(request_chain_id)?;
            if request_chain_id != chain_id {
                anyhow::bail!(
                    "token transfer chainId {} does not match active {}",
                    request_chain_id,
                    chain_id
                );
            }
        }
        Ok(NormalizedTokenTransferRequest {
            token_contract,
            to,
            raw_amount,
            amount: self.amount.trim().to_owned(),
            decimals: decimals_u8,
            symbol: sanitized_optional_token_symbol(self.symbol.as_deref()),
            data,
            chain_id,
        })
    }
}

pub(crate) fn sanitized_optional_token_symbol(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    if value.is_empty() || value.chars().any(char::is_control) {
        return None;
    }
    Some(truncate_for_event(value, 24))
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DappCompatibilityCheckRequest {
    #[serde(default)]
    pub(crate) mode: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NormalizedDappCompatibilityCheckRequest {
    pub(crate) mode: &'static str,
}

impl DappCompatibilityCheckRequest {
    pub(crate) fn normalized(&self) -> Result<NormalizedDappCompatibilityCheckRequest> {
        let mode = match self
            .mode
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("read")
            .to_ascii_lowercase()
            .as_str()
        {
            "read" | "readonly" | "read_only" => "read",
            "0" | "false" | "no" | "off" => {
                anyhow::bail!("dApp compatibility check mode cannot be disabled")
            }
            _ => {
                anyhow::bail!("dApp compatibility check only supports read mode from the UI")
            }
        };
        Ok(NormalizedDappCompatibilityCheckRequest { mode })
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DappNavigationRequest {
    pub(crate) action: String,
}

impl DappNavigationRequest {
    pub(crate) fn action(&self) -> Result<DappNavigationAction> {
        DappNavigationAction::parse(&self.action)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DappNavigationAction {
    Back,
    Forward,
    Reload,
    Home,
}

impl DappNavigationAction {
    pub(crate) fn parse(value: &str) -> Result<Self> {
        match value.trim() {
            "back" => Ok(Self::Back),
            "forward" => Ok(Self::Forward),
            "reload" => Ok(Self::Reload),
            "home" => Ok(Self::Home),
            _ => anyhow::bail!("unsupported dApp navigation action"),
        }
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::Back => "back",
            Self::Forward => "forward",
            Self::Reload => "reload",
            Self::Home => "home",
        }
    }

    pub(crate) fn script(self) -> &'static str {
        match self {
            Self::Back => "window.history.back();",
            Self::Forward => "window.history.forward();",
            Self::Reload => "window.location.reload();",
            Self::Home => "",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SmokeEvent {
    pub(crate) stage: String,
    #[serde(default)]
    pub(crate) detail: Value,
}

#[derive(Debug, Clone)]
pub(crate) struct RecoveryBackupArtifactSummary {
    pub(crate) kind: &'static str,
    pub(crate) path: String,
    pub(crate) blake3: String,
    pub(crate) group: Option<String>,
    pub(crate) member: Option<String>,
    pub(crate) destination: String,
    pub(crate) contains_secret_bytes: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderTelemetryEvent {
    pub(crate) event: String,
    #[serde(default)]
    pub(crate) origin: Option<String>,
    #[serde(default)]
    pub(crate) url: Option<String>,
    #[serde(default)]
    pub(crate) detail: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub(crate) struct ProviderRequest {
    pub(crate) id: String,
    pub(crate) method: String,
    #[serde(default)]
    pub(crate) params: Value,
    #[serde(default)]
    pub(crate) origin: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ProviderEvent {
    pub(crate) sequence: u64,
    pub(crate) unix_ms: u64,
    pub(crate) kind: String,
    pub(crate) status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) window: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) request_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) origin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) result_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) result_preview: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error_code: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) detail: Option<Value>,
}

impl ProviderEvent {
    pub(crate) fn from_provider_request(
        request: &ProviderRequest,
        envelope: &ProviderEnvelope,
        duration: Duration,
    ) -> Self {
        let (status, result_kind, result_preview, error_code, error_message) =
            match (&envelope.result, &envelope.error) {
                (Some(result), None) => (
                    "ok".to_owned(),
                    Some(value_kind(result).to_owned()),
                    provider_result_preview(result),
                    None,
                    None,
                ),
                (_, Some(error)) => (
                    "error".to_owned(),
                    None,
                    None,
                    Some(error.code),
                    Some(truncate_for_event(&error.message, 240)),
                ),
                _ => ("unknown".to_owned(), None, None, None, None),
            };

        Self {
            sequence: 0,
            unix_ms: now_unix_ms(),
            kind: "provider_request".to_owned(),
            status,
            window: None,
            request_id: Some(truncate_for_event(&request.id, 120)),
            method: Some(truncate_for_event(&request.method, 120)),
            origin: request
                .origin
                .as_deref()
                .map(|origin| truncate_for_event(origin, 512)),
            url: None,
            duration_ms: Some(duration_ms(duration)),
            result_kind,
            result_preview,
            error_code,
            error_message,
            detail: None,
        }
    }

    pub(crate) fn from_telemetry(
        window_label: &str,
        event: ProviderTelemetryEvent,
    ) -> Result<Self> {
        let event_name = validate_provider_event_name(&event.event)?;
        let origin = event
            .origin
            .as_deref()
            .map(validate_provider_event_text)
            .transpose()?;
        let url = event
            .url
            .as_deref()
            .map(sanitize_provider_event_url)
            .transpose()?;
        Ok(Self {
            sequence: 0,
            unix_ms: now_unix_ms(),
            kind: event_name,
            status: "recorded".to_owned(),
            window: Some(truncate_for_event(window_label, 64)),
            request_id: None,
            method: None,
            origin,
            url,
            duration_ms: None,
            result_kind: None,
            result_preview: None,
            error_code: None,
            error_message: None,
            detail: sanitized_provider_event_detail(event.detail)?,
        })
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProviderEnvelope {
    pub(crate) id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) error: Option<ProviderError>,
}

impl ProviderEnvelope {
    pub(crate) fn result(id: impl Into<String>, result: Value) -> Self {
        Self {
            id: id.into(),
            result: Some(result),
            error: None,
        }
    }

    pub(crate) fn error(id: impl Into<String>, error: ProviderError) -> Self {
        Self {
            id: id.into(),
            result: None,
            error: Some(error),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ProviderError {
    pub(crate) code: i64,
    pub(crate) message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) data: Option<Value>,
}

pub(crate) fn now_unix_ms() -> u64 {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    millis.min(u128::from(u64::MAX)) as u64
}

pub(crate) fn duration_ms(duration: Duration) -> u64 {
    duration.as_millis().min(u128::from(u64::MAX)) as u64
}

pub(crate) fn value_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

pub(crate) fn provider_result_preview(value: &Value) -> Option<String> {
    match value {
        Value::Null => Some("null".to_owned()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        Value::String(value) => Some(truncate_for_event(value, 48)),
        Value::Array(items) => Some(format!("items={}", items.len())),
        Value::Object(map) => {
            let keys = map.keys().take(6).cloned().collect::<Vec<_>>().join(", ");
            Some(format!("keys={keys}"))
        }
    }
}

pub(crate) fn truncate_for_event(value: &str, max_chars: usize) -> String {
    let mut output = String::new();
    for (index, ch) in value.chars().enumerate() {
        if index >= max_chars {
            output.push_str("...");
            return output;
        }
        output.push(ch);
    }
    output
}

pub(crate) fn validate_provider_event_name(value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.'))
    {
        anyhow::bail!("provider telemetry event name is malformed");
    }
    Ok(value.to_owned())
}

pub(crate) fn validate_provider_event_text(value: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() || value.len() > 2048 || value.chars().any(char::is_control) {
        anyhow::bail!("provider telemetry text is malformed");
    }
    Ok(truncate_for_event(value, 1024))
}

pub(crate) fn sanitize_provider_event_url(value: &str) -> Result<String> {
    let value = validate_provider_event_text(value)?;
    if let Ok(mut url) = Url::parse(&value) {
        url.set_query(None);
        url.set_fragment(None);
        return Ok(truncate_for_event(url.as_str(), 1024));
    }
    Ok(value)
}

pub(crate) fn sanitized_provider_event_detail(detail: Value) -> Result<Option<Value>> {
    if detail.is_null() {
        return Ok(None);
    }
    let len = serde_json::to_vec(&detail)
        .context("failed to encode provider telemetry detail")?
        .len();
    if len > 4096 {
        return Ok(Some(json!({
            "omitted": "detail too large",
            "bytes": len,
        })));
    }
    Ok(Some(detail))
}

pub(crate) fn error_to_provider_error(error: anyhow::Error) -> ProviderError {
    let message = error.to_string();
    let code = if message.contains("blocked") || message.contains("unsupported FRAMKey provider") {
        4200
    } else if message.contains("account mismatch") {
        4100
    } else if message.contains("local unlock binding changed")
        || message.contains("RecoveryRequired")
    {
        4900
    } else if message.contains("rejected")
        || message.contains("expired before approval")
        || message.contains("superseded by a newer connect or disconnect request")
        || message.contains("Touch ID")
        || message.contains("LocalAuthentication")
    {
        4001
    } else {
        4900
    };

    ProviderError {
        code,
        message,
        data: None,
    }
}

pub(crate) fn _ipc_error_to_provider_error(error: IpcError) -> ProviderError {
    let code = match error.code {
        IpcErrorCode::UserRejected | IpcErrorCode::TouchIdFailed => 4001,
        IpcErrorCode::UnsupportedMethod | IpcErrorCode::DangerousSignatureBlocked => 4200,
        IpcErrorCode::UnsupportedChain => 4901,
        _ => 4900,
    };
    ProviderError {
        code,
        message: error.message,
        data: Some(json!({"ipcCode": error.code})),
    }
}
