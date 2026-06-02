use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::Result;
use framkey_core::Generation;
use framkey_crypto::{decode_hex_array, encode_hex};
use framkey_ipc::{
    SignerBuildKeychainVaultRequest, SignerBuildKeychainVaultResponse, SignerHelperRequest,
    SignerHelperResponse, SignerHelperResult, SignerOpenKeychainVaultRequest,
    SignerOpenKeychainVaultResponse, SignerPersonalSignRequest, SignerPersonalSignResponse,
    SignerRecoverKeychainVaultRequest, SignerRecoverKeychainVaultResponse,
};
use framkey_recovery::RecoveryBackupFile;
use serde_json::json;

use crate::{
    args::{KeychainItemArgs, SignerHelperArgs},
    constants::{FRAMKEY_SIGNER_HELPER_BLAKE3_ENV, MACOS_NO_NETWORK_SANDBOX_PROFILE},
};

pub(crate) fn helper_build_keychain_vault(
    helper: &SignerHelperArgs,
    keychain: &KeychainItemArgs,
    image_size: usize,
    generation: Generation,
    recovery_backups: bool,
) -> Result<(SignerBuildKeychainVaultResponse, SignerHelperExecution)> {
    let invocation = run_signer_helper(
        helper,
        &SignerHelperRequest::BuildKeychainVault(SignerBuildKeychainVaultRequest {
            image_size,
            generation: generation.0,
            keychain_service: keychain.service.clone(),
            keychain_account: keychain.account.clone(),
            recovery_backups,
        }),
    )?;
    let SignerHelperInvocation {
        response,
        execution,
    } = invocation;

    match response {
        SignerHelperResponse::Ok {
            result: SignerHelperResult::BuildKeychainVault(result),
        } => Ok((result, execution)),
        SignerHelperResponse::Ok { result } => {
            anyhow::bail!("unexpected signer helper result: {result:?}")
        }
        SignerHelperResponse::Error { error } => {
            anyhow::bail!("signer helper failed: {:?}: {}", error.code, error.message)
        }
    }
}

pub(crate) fn helper_open_keychain_vault(
    helper: &SignerHelperArgs,
    keychain: &KeychainItemArgs,
    save_image: Vec<u8>,
) -> Result<(SignerOpenKeychainVaultResponse, SignerHelperExecution)> {
    let invocation = run_signer_helper(
        helper,
        &SignerHelperRequest::OpenKeychainVault(SignerOpenKeychainVaultRequest {
            save_image,
            keychain_service: keychain.service.clone(),
            keychain_account: keychain.account.clone(),
        }),
    )?;
    let SignerHelperInvocation {
        response,
        execution,
    } = invocation;

    match response {
        SignerHelperResponse::Ok {
            result: SignerHelperResult::OpenKeychainVault(result),
        } => Ok((result, execution)),
        SignerHelperResponse::Ok { result } => {
            anyhow::bail!("unexpected signer helper result: {result:?}")
        }
        SignerHelperResponse::Error { error } => {
            anyhow::bail!("signer helper failed: {:?}: {}", error.code, error.message)
        }
    }
}

pub(crate) fn helper_recover_keychain_vault(
    helper: &SignerHelperArgs,
    keychain: &KeychainItemArgs,
    save_image: Vec<u8>,
    recovery_files: Vec<RecoveryBackupFile>,
) -> Result<(SignerRecoverKeychainVaultResponse, SignerHelperExecution)> {
    let invocation = run_signer_helper(
        helper,
        &SignerHelperRequest::RecoverKeychainVault(SignerRecoverKeychainVaultRequest {
            save_image,
            keychain_service: keychain.service.clone(),
            keychain_account: keychain.account.clone(),
            recovery_files,
        }),
    )?;
    let SignerHelperInvocation {
        response,
        execution,
    } = invocation;

    match response {
        SignerHelperResponse::Ok {
            result: SignerHelperResult::RecoverKeychainVault(result),
        } => Ok((result, execution)),
        SignerHelperResponse::Ok { result } => {
            anyhow::bail!("unexpected signer helper result: {result:?}")
        }
        SignerHelperResponse::Error { error } => {
            anyhow::bail!("signer helper failed: {:?}: {}", error.code, error.message)
        }
    }
}

pub(crate) fn helper_personal_sign(
    helper: &SignerHelperArgs,
    keychain: &KeychainItemArgs,
    save_image: Vec<u8>,
    message: Vec<u8>,
) -> Result<(SignerPersonalSignResponse, SignerHelperExecution)> {
    let invocation = run_signer_helper(
        helper,
        &SignerHelperRequest::PersonalSign(SignerPersonalSignRequest {
            save_image,
            keychain_service: keychain.service.clone(),
            keychain_account: keychain.account.clone(),
            message,
            expected_address: None,
        }),
    )?;
    let SignerHelperInvocation {
        response,
        execution,
    } = invocation;

    match response {
        SignerHelperResponse::Ok {
            result: SignerHelperResult::PersonalSign(result),
        } => Ok((result, execution)),
        SignerHelperResponse::Ok { result } => {
            anyhow::bail!("unexpected signer helper result: {result:?}")
        }
        SignerHelperResponse::Error { error } => {
            anyhow::bail!("signer helper failed: {:?}: {}", error.code, error.message)
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct SignerHelperInvocation {
    response: SignerHelperResponse,
    execution: SignerHelperExecution,
}

#[derive(Debug, Clone)]
pub(crate) struct SignerHelperExecution {
    path: PathBuf,
    blake3: String,
    sandbox: SignerHelperSandbox,
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum SignerHelperSandbox {
    MacosProcessIdentity,
    MacosSandboxExecNoNetwork,
    DisabledByUser,
    #[cfg(not(target_os = "macos"))]
    UnsupportedPlatform,
}

impl SignerHelperSandbox {
    fn as_str(self) -> &'static str {
        match self {
            Self::MacosProcessIdentity => "macos_process_identity",
            Self::MacosSandboxExecNoNetwork => "macos_sandbox_exec_no_network",
            Self::DisabledByUser => "disabled_by_user",
            #[cfg(not(target_os = "macos"))]
            Self::UnsupportedPlatform => "unsupported_platform",
        }
    }
}

fn run_signer_helper(
    helper: &SignerHelperArgs,
    request: &SignerHelperRequest,
) -> Result<SignerHelperInvocation> {
    let execution = inspect_signer_helper(helper)?;
    let mut command = signer_helper_command(&execution);
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            anyhow::anyhow!(
                "failed to start signer helper {}: {error}",
                execution.path.display()
            )
        })?;

    {
        let mut stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow::anyhow!("signer helper stdin was not available"))?;
        serde_json::to_writer(&mut stdin, request)?;
        stdin.write_all(b"\n")?;
    }

    let output = child.wait_with_output()?;
    if output.stdout.is_empty() && !output.status.success() {
        anyhow::bail!(
            "signer helper exited with {} before returning JSON; stderr: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let response: SignerHelperResponse =
        serde_json::from_slice(&output.stdout).map_err(|error| {
            anyhow::anyhow!(
                "failed to parse signer helper response: {error}; stderr: {}",
                String::from_utf8_lossy(&output.stderr)
            )
        })?;

    if !output.status.success() {
        match &response {
            SignerHelperResponse::Error { error } => {
                anyhow::bail!("signer helper failed: {:?}: {}", error.code, error.message);
            }
            SignerHelperResponse::Ok { .. } => {
                anyhow::bail!("signer helper exited with {}", output.status);
            }
        }
    }

    Ok(SignerHelperInvocation {
        response,
        execution,
    })
}

fn inspect_signer_helper(helper: &SignerHelperArgs) -> Result<SignerHelperExecution> {
    let path = signer_helper_path(helper)?;
    let blake3 = hash_file_blake3(&path)?;
    if let Some(expected) = expected_signer_helper_blake3(helper)? {
        if expected != blake3 {
            anyhow::bail!(
                "signer helper BLAKE3 mismatch: expected {}, got {} for {}",
                expected,
                blake3,
                path.display()
            );
        }
    }

    Ok(SignerHelperExecution {
        path,
        blake3,
        sandbox: signer_helper_sandbox(helper)?,
    })
}

fn signer_helper_path(helper: &SignerHelperArgs) -> Result<PathBuf> {
    let path = if let Some(path) = &helper.signer_helper {
        path.clone()
    } else {
        let current_exe = std::env::current_exe()?;
        let dir = current_exe
            .parent()
            .ok_or_else(|| anyhow::anyhow!("current executable has no parent directory"))?;
        dir.join("framkey-signer-helper")
    };

    path.canonicalize().map_err(|error| {
        anyhow::anyhow!(
            "failed to resolve signer helper path {}: {error}",
            path.display()
        )
    })
}

fn signer_helper_command(execution: &SignerHelperExecution) -> Command {
    match execution.sandbox {
        SignerHelperSandbox::MacosProcessIdentity => Command::new(&execution.path),
        SignerHelperSandbox::MacosSandboxExecNoNetwork => {
            let mut command = Command::new("/usr/bin/sandbox-exec");
            command
                .arg("-p")
                .arg(MACOS_NO_NETWORK_SANDBOX_PROFILE)
                .arg(&execution.path);
            command
        }
        SignerHelperSandbox::DisabledByUser => Command::new(&execution.path),
        #[cfg(not(target_os = "macos"))]
        SignerHelperSandbox::UnsupportedPlatform => Command::new(&execution.path),
    }
}

fn signer_helper_sandbox(helper: &SignerHelperArgs) -> Result<SignerHelperSandbox> {
    if helper.allow_unsandboxed_signer_helper {
        return Ok(SignerHelperSandbox::DisabledByUser);
    }
    if helper.use_sandbox_exec_signer_helper {
        return macos_sandbox_exec_helper();
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

fn macos_sandbox_exec_helper() -> Result<SignerHelperSandbox> {
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

fn expected_signer_helper_blake3(helper: &SignerHelperArgs) -> Result<Option<String>> {
    let expected = helper
        .signer_helper_blake3
        .clone()
        .or_else(|| std::env::var(FRAMKEY_SIGNER_HELPER_BLAKE3_ENV).ok());
    expected.as_deref().map(normalize_blake3_hex).transpose()
}

fn normalize_blake3_hex(value: &str) -> Result<String> {
    let value = value.trim();
    let value = value.strip_prefix("0x").unwrap_or(value);
    let bytes = decode_hex_array::<32>(value)?;
    Ok(encode_hex(&bytes))
}

fn hash_file_blake3(path: &Path) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(encode_hex(hasher.finalize().as_bytes()))
}

pub(crate) fn signer_helper_report(execution: &SignerHelperExecution) -> serde_json::Value {
    json!({
        "path": execution.path.display().to_string(),
        "blake3": execution.blake3,
        "sandbox": execution.sandbox.as_str(),
    })
}
