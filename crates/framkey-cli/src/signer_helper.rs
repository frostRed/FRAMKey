use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    time::{Duration, Instant},
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
    constants::{
        FRAMKEY_SIGNER_HELPER_BLAKE3_ENV, MACOS_NO_NETWORK_SANDBOX_PROFILE, SIGNER_HELPER_TIMEOUT,
    },
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

    match response.into_result() {
        Ok(SignerHelperResult::BuildKeychainVault(result)) => Ok((result, execution)),
        Ok(result) => {
            anyhow::bail!("unexpected signer helper result: {result:?}")
        }
        Err(error) => {
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

    match response.into_result() {
        Ok(SignerHelperResult::OpenKeychainVault(result)) => Ok((result, execution)),
        Ok(result) => {
            anyhow::bail!("unexpected signer helper result: {result:?}")
        }
        Err(error) => {
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

    match response.into_result() {
        Ok(SignerHelperResult::RecoverKeychainVault(result)) => Ok((result, execution)),
        Ok(result) => {
            anyhow::bail!("unexpected signer helper result: {result:?}")
        }
        Err(error) => {
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

    match response.into_result() {
        Ok(SignerHelperResult::PersonalSign(result)) => Ok((result, execution)),
        Ok(result) => {
            anyhow::bail!("unexpected signer helper result: {result:?}")
        }
        Err(error) => {
            anyhow::bail!("signer helper failed: {:?}: {}", error.code, error.message)
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct KeychainHelperTrustReport {
    pub(crate) helper_path: PathBuf,
    pub(crate) helper_blake3: String,
    pub(crate) helper_cdhash: String,
    pub(crate) partition_list: String,
}

pub(crate) fn trust_signer_helper_keychain_access(
    helper: &SignerHelperArgs,
    keychain: &KeychainItemArgs,
) -> Result<KeychainHelperTrustReport> {
    #[cfg(target_os = "macos")]
    {
        let item = keychain.item();
        item.validate()?;
        let execution = inspect_signer_helper(helper)?;
        let cdhash = signer_helper_cdhash(&execution.path)?;
        let partition_list = format!("cdhash:{cdhash}");
        let status = Command::new("/usr/bin/security")
            .arg("set-generic-password-partition-list")
            .arg("-s")
            .arg(&item.service)
            .arg("-a")
            .arg(&item.account)
            .arg("-S")
            .arg(&partition_list)
            .arg("login.keychain-db")
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .map_err(|error| anyhow::anyhow!("failed to run /usr/bin/security: {error}"))?;
        if !status.success() {
            anyhow::bail!("security set-generic-password-partition-list exited with {status}");
        }

        Ok(KeychainHelperTrustReport {
            helper_path: execution.path,
            helper_blake3: execution.blake3,
            helper_cdhash: cdhash,
            partition_list,
        })
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (helper, keychain);
        anyhow::bail!("macOS Keychain helper trust is only available on macOS");
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

    let output = wait_for_signer_helper_output(child, SIGNER_HELPER_TIMEOUT)?;
    if output.stdout.is_empty() {
        anyhow::bail!(
            "signer helper returned empty stdout with {}; stderr: {}",
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

fn wait_for_signer_helper_output(mut child: Child, timeout: Duration) -> Result<Output> {
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("signer helper stdout was not available"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("signer helper stderr was not available"))?;
    let stdout_reader = std::thread::spawn(move || {
        let mut stdout = stdout;
        let mut bytes = Vec::new();
        stdout.read_to_end(&mut bytes).map(|_| bytes)
    });
    let stderr_reader = std::thread::spawn(move || {
        let mut stderr = stderr;
        let mut bytes = Vec::new();
        stderr.read_to_end(&mut bytes).map(|_| bytes)
    });

    let started_at = Instant::now();
    loop {
        if let Some(status) = child.try_wait()? {
            let stdout = join_output_reader(stdout_reader, "stdout")?;
            let stderr = join_output_reader(stderr_reader, "stderr")?;
            return Ok(Output {
                status,
                stdout,
                stderr,
            });
        }
        if started_at.elapsed() >= timeout {
            let _ = child.kill();
            let _status = child.wait()?;
            let stdout = join_output_reader(stdout_reader, "stdout")?;
            let stderr = join_output_reader(stderr_reader, "stderr")?;
            anyhow::bail!(
                "signer helper timed out after {} ms; stdout: {} bytes; stderr: {}",
                timeout.as_millis(),
                stdout.len(),
                String::from_utf8_lossy(&stderr)
            );
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn join_output_reader(
    handle: std::thread::JoinHandle<std::io::Result<Vec<u8>>>,
    stream: &str,
) -> Result<Vec<u8>> {
    handle
        .join()
        .map_err(|_| anyhow::anyhow!("signer helper {stream} reader panicked"))?
        .map_err(|error| anyhow::anyhow!("failed to read signer helper {stream}: {error}"))
}

fn inspect_signer_helper(helper: &SignerHelperArgs) -> Result<SignerHelperExecution> {
    let path = signer_helper_path(helper)?;
    let blake3 = hash_file_blake3(&path)?;
    if let Some(expected) = expected_signer_helper_blake3(helper)?
        && expected != blake3
    {
        anyhow::bail!(
            "signer helper BLAKE3 mismatch: expected {}, got {} for {}",
            expected,
            blake3,
            path.display()
        );
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

fn signer_helper_cdhash(path: &Path) -> Result<String> {
    let output = Command::new("/usr/bin/codesign")
        .arg("-dv")
        .arg("--verbose=4")
        .arg(path)
        .output()
        .map_err(|error| anyhow::anyhow!("failed to run /usr/bin/codesign: {error}"))?;
    if !output.status.success() {
        anyhow::bail!(
            "codesign failed for {} with {}; stderr: {}",
            path.display(),
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let cdhash = parse_codesign_cdhash(&stderr).ok_or_else(|| {
        anyhow::anyhow!(
            "codesign output for {} did not contain CDHash",
            path.display()
        )
    })?;
    Ok(cdhash.to_owned())
}

fn parse_codesign_cdhash(output: &str) -> Option<&str> {
    output.lines().find_map(|line| {
        let cdhash = line.trim().strip_prefix("CDHash=")?.trim();
        if cdhash.len() == 40
            && cdhash
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        {
            Some(cdhash)
        } else {
            None
        }
    })
}

pub(crate) fn signer_helper_report(execution: &SignerHelperExecution) -> serde_json::Value {
    json!({
        "path": execution.path.display().to_string(),
        "blake3": execution.blake3,
        "sandbox": execution.sandbox.as_str(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_codesign_cdhash() {
        let output = r#"
Executable=/tmp/framkey-signer-helper
CDHash=2316c52c2b96f94fb72411610396b7b6ef715944
# designated => cdhash H"2316c52c2b96f94fb72411610396b7b6ef715944"
"#;

        assert_eq!(
            parse_codesign_cdhash(output),
            Some("2316c52c2b96f94fb72411610396b7b6ef715944")
        );
    }

    #[test]
    fn rejects_malformed_codesign_cdhash() {
        assert_eq!(parse_codesign_cdhash("CDHash=not-a-cdhash"), None);
        assert_eq!(
            parse_codesign_cdhash("CDHash=2316c52c2b96f94fb72411610396b7b6ef71594400"),
            None
        );
    }
}
