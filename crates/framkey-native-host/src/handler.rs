use anyhow::Result;
use framkey_device::{FileImageDevice, VaultDevice};
use framkey_gbxcart::{GbxCartConfig, GbxCartDevice};
use framkey_ipc::{
    IpcError, IpcErrorCode, IpcRequest, IpcResponse, SignerHelperRequest, SignerHelperResult,
    SignerOpenKeychainVaultRequest, SignerOpenKeychainVaultResponse,
};
use serde_json::{Value, json};

use crate::{
    config::{NativeAccount, NativeDeviceConfig, NativeHostConfig},
    constants::DEFAULT_HOST_NAME,
    error::error_to_ipc,
    signer_helper::{helper_report, run_signer_helper},
};

#[derive(Debug, Default)]
pub(crate) struct NativeHostState {
    account: Option<NativeAccount>,
}

impl NativeHostState {
    fn connected_addresses(&self) -> Vec<String> {
        self.account
            .as_ref()
            .map(|account| vec![account.address.clone()])
            .unwrap_or_default()
    }

    fn remember_account(&mut self, account: NativeAccount) {
        self.account = Some(account);
    }
}

pub(crate) fn handle_request(
    config: &NativeHostConfig,
    state: &mut NativeHostState,
    request: IpcRequest,
) -> IpcResponse {
    match handle_request_result(config, state, &request) {
        Ok(result) => IpcResponse::Result {
            id: request.id,
            result,
        },
        Err(error) => IpcResponse::Error {
            id: request.id,
            error,
        },
    }
}

pub(crate) fn handle_request_result(
    config: &NativeHostConfig,
    state: &mut NativeHostState,
    request: &IpcRequest,
) -> std::result::Result<Value, IpcError> {
    match request.method.as_str() {
        "eth_chainId" => Ok(json!(config.chain_id)),
        "framkey_getStatus" | "wallet_getCapabilities" => Ok(status_result(config)),
        "framkey_getAccount" => {
            let account = load_account(config).map_err(error_to_ipc)?;
            let result = account_result(config, &account);
            state.remember_account(account);
            Ok(result)
        }
        "eth_requestAccounts" => {
            let account = load_account(config).map_err(error_to_ipc)?;
            let addresses = vec![account.address.clone()];
            state.remember_account(account);
            Ok(json!(addresses))
        }
        "eth_accounts" => Ok(json!(state.connected_addresses())),
        "eth_sendTransaction"
        | "eth_sign"
        | "eth_signTransaction"
        | "eth_signTypedData"
        | "eth_signTypedData_v1"
        | "eth_signTypedData_v3"
        | "eth_signTypedData_v4"
        | "personal_sign" => Err(IpcError {
            code: IpcErrorCode::DangerousSignatureBlocked,
            message: format!(
                "{} is blocked in the read-only browser bridge",
                request.method
            ),
        }),
        _ => Err(IpcError {
            code: IpcErrorCode::UnsupportedMethod,
            message: format!(
                "unsupported FRAMKey browser bridge method {}",
                request.method
            ),
        }),
    }
}

fn status_result(config: &NativeHostConfig) -> Value {
    json!({
        "host": DEFAULT_HOST_NAME,
        "version": env!("CARGO_PKG_VERSION"),
        "chainId": config.chain_id,
        "device": config.device.describe(),
        "keychain": {
            "configured": true,
        },
        "capabilities": {
            "readOnlyAccounts": true,
            "personalSign": false,
            "sendTransaction": false,
            "signTypedData": false,
            "simulation": false,
        }
    })
}

fn account_result(config: &NativeHostConfig, account: &NativeAccount) -> Value {
    json!({
        "address": account.address,
        "chainId": config.chain_id,
        "wallet": {
            "kind": "keychain_vault",
            "mock": false,
        },
        "keychain": {
            "configured": true,
            "accessPolicy": account.opened.keychain_access_policy,
        },
        "signerHelper": account.helper_report,
    })
}

fn load_account(config: &NativeHostConfig) -> Result<NativeAccount> {
    let save_image = read_configured_save_image(config)?;
    let opened = open_keychain_vault_with_helper(config, save_image)?;
    let address = opened
        .address
        .clone()
        .ok_or_else(|| anyhow::anyhow!("Keychain vault did not expose an EVM address"))?;
    let helper_report = helper_report(&config.helper)?;

    Ok(NativeAccount {
        address,
        opened,
        helper_report,
    })
}

fn read_configured_save_image(config: &NativeHostConfig) -> Result<Vec<u8>> {
    let device: Box<dyn VaultDevice> = match &config.device {
        NativeDeviceConfig::File { path } => Box::new(FileImageDevice::new(path.clone())),
        NativeDeviceConfig::GbxCart {
            port,
            save_type,
            expected_save_size,
        } => Box::new(GbxCartDevice::new(GbxCartConfig {
            port_hint: port.clone(),
            expected_save_size: *expected_save_size,
            save_type: Some(*save_type),
        })),
    };

    Ok(device.read_save_image()?.as_bytes().to_vec())
}

fn open_keychain_vault_with_helper(
    config: &NativeHostConfig,
    save_image: Vec<u8>,
) -> Result<SignerOpenKeychainVaultResponse> {
    let response = run_signer_helper(
        &config.helper,
        &SignerHelperRequest::OpenKeychainVault(SignerOpenKeychainVaultRequest {
            save_image,
            keychain_service: config.keychain_service.clone(),
            keychain_account: config.keychain_account.clone(),
        }),
    )?;

    match response.into_result() {
        Ok(SignerHelperResult::OpenKeychainVault(result)) => Ok(result),
        Ok(result) => {
            anyhow::bail!("unexpected signer helper result: {result:?}")
        }
        Err(error) => {
            anyhow::bail!("signer helper failed: {:?}: {}", error.code, error.message)
        }
    }
}
