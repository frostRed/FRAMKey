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

pub(crate) fn handle_request(config: &NativeHostConfig, request: IpcRequest) -> IpcResponse {
    match handle_request_result(config, &request) {
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
    request: &IpcRequest,
) -> std::result::Result<Value, IpcError> {
    match request.method.as_str() {
        "eth_chainId" => Ok(json!(config.chain_id)),
        "framkey_getStatus" | "wallet_getCapabilities" => Ok(status_result(config)),
        "framkey_getAccount" => {
            let account = load_account(config).map_err(error_to_ipc)?;
            Ok(account_result(config, account))
        }
        "eth_requestAccounts" => {
            let account = load_account(config).map_err(error_to_ipc)?;
            Ok(json!([account.address]))
        }
        "eth_accounts" => {
            let account = load_account(config).map_err(error_to_ipc)?;
            Ok(json!([account.address]))
        }
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
            "service": config.keychain_service,
            "account": config.keychain_account,
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

fn account_result(config: &NativeHostConfig, account: NativeAccount) -> Value {
    json!({
        "address": account.address,
        "chainId": config.chain_id,
        "metadata": account.opened.metadata,
        "keychain": {
            "service": account.opened.keychain_service,
            "account": account.opened.keychain_account,
            "itemId": account.opened.keychain_item_id,
            "deviceId": account.opened.device_id,
            "kekId": account.opened.kek_id,
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
