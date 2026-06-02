use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
    process::{Child, Command, Output, Stdio},
    time::{Duration, Instant},
};

use anyhow::{Context, Result};
use framkey_crypto::encode_hex;
use framkey_ipc::{SignerHelperRequest, SignerHelperResponse};
use serde_json::{Value, json};

use crate::{
    config::{SignerHelperConfig, SignerHelperSandbox},
    constants::{MACOS_NO_NETWORK_SANDBOX_PROFILE, SIGNER_HELPER_TIMEOUT},
};

pub(crate) fn run_signer_helper(
    helper: &SignerHelperConfig,
    request: &SignerHelperRequest,
) -> Result<SignerHelperResponse> {
    verify_helper_hash(helper)?;

    let mut command = signer_helper_command(helper);
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to start signer helper {}", helper.path.display()))?;

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
            signer_helper_stderr_summary(&output.stderr)
        );
    }
    let response: SignerHelperResponse =
        serde_json::from_slice(&output.stdout).map_err(|error| {
            anyhow::anyhow!(
                "failed to parse signer helper response: {error}; stderr: {}",
                signer_helper_stderr_summary(&output.stderr)
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

    Ok(response)
}

pub(crate) fn wait_for_signer_helper_output(mut child: Child, timeout: Duration) -> Result<Output> {
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
                signer_helper_stderr_summary(&stderr)
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

pub(crate) fn signer_helper_stderr_summary(stderr: &[u8]) -> String {
    if stderr.is_empty() {
        "empty".to_owned()
    } else {
        format!("{} bytes redacted", stderr.len())
    }
}

fn signer_helper_command(helper: &SignerHelperConfig) -> Command {
    match helper.sandbox {
        SignerHelperSandbox::MacosSandboxExecNoNetwork => {
            let mut command = Command::new("/usr/bin/sandbox-exec");
            command
                .arg("-p")
                .arg(MACOS_NO_NETWORK_SANDBOX_PROFILE)
                .arg(&helper.path);
            command
        }
        SignerHelperSandbox::DisabledByConfig => Command::new(&helper.path),
        #[cfg(not(target_os = "macos"))]
        SignerHelperSandbox::UnsupportedPlatform => Command::new(&helper.path),
    }
}

fn verify_helper_hash(helper: &SignerHelperConfig) -> Result<()> {
    if let Some(expected) = &helper.expected_blake3 {
        let actual = hash_file_blake3(&helper.path)?;
        if expected != &actual {
            anyhow::bail!(
                "signer helper BLAKE3 mismatch: expected {}, got {} for {}",
                expected,
                actual,
                helper.path.display()
            );
        }
    }
    Ok(())
}

pub(crate) fn helper_report(helper: &SignerHelperConfig) -> Result<Value> {
    let hash_matches = if let Some(expected) = &helper.expected_blake3 {
        Some(hash_file_blake3(&helper.path)? == *expected)
    } else {
        None
    };
    Ok(json!({
        "ready": true,
        "hashPinned": helper.expected_blake3.is_some(),
        "hashMatches": hash_matches,
        "sandbox": helper.sandbox.as_str(),
    }))
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
