use std::io::{Read, Write};

use anyhow::Result;
use framkey_core::FramkeyError;
use framkey_ipc::{IpcErrorCode, MAX_SIGNER_HELPER_JSON_BYTES, SignerHelperResponse};

pub(crate) fn read_limited_stdin() -> Result<Vec<u8>> {
    let mut input = Vec::new();
    let limit = (MAX_SIGNER_HELPER_JSON_BYTES + 1) as u64;
    std::io::stdin()
        .lock()
        .take(limit)
        .read_to_end(&mut input)?;
    if input.len() > MAX_SIGNER_HELPER_JSON_BYTES {
        return Err(FramkeyError::invalid_data(format!(
            "signer helper request exceeds {} bytes",
            MAX_SIGNER_HELPER_JSON_BYTES
        ))
        .into());
    }
    Ok(input)
}

pub(crate) fn write_json_response(response: &SignerHelperResponse) -> Result<()> {
    let payload = serde_json::to_vec_pretty(response)?;
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(&payload)?;
    stdout.write_all(b"\n")?;
    stdout.flush()?;
    Ok(())
}

pub(crate) fn classify_error(error: &anyhow::Error) -> IpcErrorCode {
    let message = error.to_string();
    if message.contains("local unlock binding changed") {
        IpcErrorCode::RecoveryRequired
    } else if message.contains("LocalAuthentication") || message.contains("local authentication") {
        IpcErrorCode::LocalAuthenticationFailed
    } else if message.contains("Keychain") {
        IpcErrorCode::KeychainItemNotFound
    } else if message.contains("account mismatch") {
        IpcErrorCode::DangerousSignatureBlocked
    } else if message.contains("EVM") || message.contains("signer helper only supports") {
        IpcErrorCode::UnsupportedChain
    } else {
        IpcErrorCode::VaultCorrupted
    }
}
