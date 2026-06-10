use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use anyhow::{Context, Result};
use framkey_ch347_helper::{
    CH347_HELPER_READ_OPERATION, CH347_HELPER_WRITE_OPERATION, Ch347HelperReadRequest,
    Ch347HelperReadResult, Ch347HelperRequest, Ch347HelperResponse, Ch347HelperResult,
    Ch347HelperWriteRequest, Ch347HelperWriteResult, read_response_bytes,
};
use serde_json::Value;

use crate::*;

pub(crate) fn run_ch347_helper_privileged(
    helper: &Ch347HelperConfig,
    request: Ch347HelperWriteRequest,
) -> Result<Ch347HelperWriteResult> {
    run_ch347_helper_with_launcher(helper, request, launch_ch347_helper_privileged)
}

pub(crate) fn run_ch347_read_helper_privileged(
    helper: &Ch347HelperConfig,
    request: Ch347HelperReadRequest,
) -> Result<Ch347HelperReadResult> {
    run_ch347_read_helper_with_launcher(helper, request, launch_ch347_helper_privileged)
}

pub(crate) fn run_ch347_helper_with_launcher(
    helper: &Ch347HelperConfig,
    request: Ch347HelperWriteRequest,
    launcher: impl FnOnce(&Path, &Path) -> Result<Ch347HelperResponse>,
) -> Result<Ch347HelperWriteResult> {
    let result = run_ch347_helper_request_with_launcher(
        helper,
        Ch347HelperRequest::Write(request),
        launcher,
    )?;
    match result {
        Ch347HelperResult::Write(result) => Ok(result),
        Ch347HelperResult::Read(_) => {
            anyhow::bail!("CH347 helper returned a read result for a write request")
        }
    }
}

pub(crate) fn run_ch347_read_helper_with_launcher(
    helper: &Ch347HelperConfig,
    request: Ch347HelperReadRequest,
    launcher: impl FnOnce(&Path, &Path) -> Result<Ch347HelperResponse>,
) -> Result<Ch347HelperReadResult> {
    let result = run_ch347_helper_request_with_launcher(
        helper,
        Ch347HelperRequest::Read(request),
        launcher,
    )?;
    match result {
        Ch347HelperResult::Read(result) => Ok(result),
        Ch347HelperResult::Write(_) => {
            anyhow::bail!("CH347 helper returned a write result for a read request")
        }
    }
}

pub(crate) fn run_ch347_helper_request_with_launcher(
    helper: &Ch347HelperConfig,
    request: Ch347HelperRequest,
    launcher: impl FnOnce(&Path, &Path) -> Result<Ch347HelperResponse>,
) -> Result<Ch347HelperResult> {
    verify_ch347_helper_hash(helper)?;
    let workspace = Ch347HelperWorkspace::new()?;
    let request_path = workspace.path("request.json");
    write_new_file(&request_path, &serde_json::to_vec_pretty(&request)?)?;

    let response = launcher(&helper.path, &request_path)?;
    helper_response_result(response)
}

pub(crate) fn ch347_helper_status_value(helper: &Ch347HelperConfig) -> Value {
    let exists = helper.path.exists();
    let blake3 = if exists {
        hash_file_blake3(&helper.path).ok()
    } else {
        None
    };
    let hash_matches = match (&helper.expected_blake3, &blake3) {
        (Some(expected), Some(actual)) => Some(expected == actual),
        (Some(_), None) => Some(false),
        (None, _) => None,
    };
    let readiness = match (exists, hash_matches) {
        (false, _) => "missing",
        (true, Some(false)) => "hash_mismatch",
        (true, _) => "ready",
    };
    serde_json::json!({
        "path": helper.path.display().to_string(),
        "exists": exists,
        "ready": readiness == "ready",
        "readiness": readiness,
        "location": signer_helper_location(&helper.path),
        "hashPinned": helper.expected_blake3.is_some(),
        "hashMatches": hash_matches,
        "blake3": blake3,
        "privilege": "macos_admin_authorization",
    })
}

pub(crate) fn verify_ch347_helper_hash(helper: &Ch347HelperConfig) -> Result<()> {
    if let Some(expected) = &helper.expected_blake3 {
        let actual = hash_file_blake3(&helper.path)?;
        if expected != &actual {
            anyhow::bail!(
                "CH347 helper BLAKE3 mismatch: expected {}, got {} for {}",
                expected,
                actual,
                helper.path.display()
            );
        }
    }
    Ok(())
}

pub(crate) fn launch_ch347_helper_privileged(
    helper_path: &Path,
    request_path: &Path,
) -> Result<Ch347HelperResponse> {
    #[cfg(target_os = "macos")]
    {
        let command = shell_command(&[
            helper_path.as_os_str(),
            std::ffi::OsStr::new("--request"),
            request_path.as_os_str(),
        ])?;
        let script = format!(
            "do shell script {} with administrator privileges",
            applescript_string_literal(&command)
        );
        let child = Command::new("/usr/bin/osascript")
            .arg("-e")
            .arg(script)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .context("failed to launch macOS administrator authorization for CH347 helper")?;
        let output = wait_for_signer_helper_output(child, CH347_HELPER_TIMEOUT)?;
        if !output.stdout.is_empty()
            && let Ok(response) = read_response_bytes(&output.stdout)
        {
            return Ok(response);
        }
        if output.status.success() {
            anyhow::bail!(
                "CH347 helper exited successfully without returning a valid JSON response"
            );
        }
        anyhow::bail!(
            "macOS administrator authorization for CH347 helper failed with {}; {}",
            output.status,
            sanitize_helper_launcher_output(&output.stderr)
        );
    }

    #[cfg(not(target_os = "macos"))]
    {
        let _ = (helper_path, request_path);
        anyhow::bail!("CH347 privileged helper is currently supported on macOS only");
    }
}

pub(crate) fn helper_response_result(response: Ch347HelperResponse) -> Result<Ch347HelperResult> {
    response
        .into_helper_result()
        .map_err(|error| anyhow::anyhow!("CH347 helper failed: {}: {}", error.code, error.message))
}

pub(crate) fn ch347_helper_request(
    input_path: PathBuf,
    flashrom_path: PathBuf,
    chip: Option<String>,
    spi_speed: Option<&str>,
    expected_size: usize,
    expected_blake3: String,
) -> Ch347HelperWriteRequest {
    Ch347HelperWriteRequest {
        operation: CH347_HELPER_WRITE_OPERATION.to_owned(),
        input_path,
        flashrom_path,
        chip,
        spispeed: spi_speed.map(str::to_owned),
        expected_size,
        expected_blake3,
    }
}

pub(crate) fn ch347_helper_read_request(
    output_dir: PathBuf,
    flashrom_path: PathBuf,
    chip: Option<String>,
    spi_speed: Option<&str>,
) -> Ch347HelperReadRequest {
    Ch347HelperReadRequest {
        operation: CH347_HELPER_READ_OPERATION.to_owned(),
        output_dir,
        flashrom_path,
        chip,
        spispeed: spi_speed.map(str::to_owned),
    }
}

pub(crate) fn shell_command(args: &[&std::ffi::OsStr]) -> Result<String> {
    args.iter()
        .map(|arg| {
            arg.to_str()
                .map(shell_quote)
                .ok_or_else(|| anyhow::anyhow!("CH347 helper path is not valid UTF-8"))
        })
        .collect::<Result<Vec<_>>>()
        .map(|args| args.join(" "))
}

pub(crate) fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        "''".to_owned()
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

pub(crate) fn applescript_string_literal(value: &str) -> String {
    let escaped = value.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{escaped}\"")
}

pub(crate) fn sanitize_helper_launcher_output(stderr: &[u8]) -> String {
    let text = String::from_utf8_lossy(stderr);
    let sanitized = text
        .chars()
        .filter(|ch| !ch.is_control())
        .collect::<String>();
    truncate_for_event(sanitized.trim(), 240)
}

struct Ch347HelperWorkspace {
    dir: PathBuf,
}

impl Ch347HelperWorkspace {
    fn new() -> Result<Self> {
        for index in 0..1000 {
            let dir = std::env::temp_dir().join(format!(
                "framkey-ch347-helper-{}-{}-{index}",
                std::process::id(),
                now_unix_ms()
            ));
            match create_private_dir(&dir) {
                Ok(()) => return Ok(Self { dir }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(anyhow::anyhow!(
                        "failed to create CH347 helper workspace {}: {error}",
                        dir.display()
                    ));
                }
            }
        }
        anyhow::bail!("failed to create a unique CH347 helper workspace")
    }

    fn path(&self, name: &str) -> PathBuf {
        self.dir.join(name)
    }
}

impl Drop for Ch347HelperWorkspace {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.dir);
    }
}

#[cfg(test)]
pub(crate) fn fake_ch347_helper_launcher(
    _helper_path: &Path,
    request_path: &Path,
) -> Result<Ch347HelperResponse> {
    let response = match framkey_ch347_helper::read_request_file(request_path)
        .and_then(framkey_ch347_helper::execute_request)
        .map(Ch347HelperResponse::ok)
    {
        Ok(response) => response,
        Err(error) => framkey_ch347_helper::error_response(&error),
    };
    Ok(response)
}
