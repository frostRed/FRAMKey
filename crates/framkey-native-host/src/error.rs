use framkey_ipc::{IpcError, IpcErrorCode};

pub(crate) fn error_to_ipc(error: anyhow::Error) -> IpcError {
    let message = error.to_string();
    let code = if message.contains("local unlock binding changed")
        || message.contains("RecoveryRequired")
    {
        IpcErrorCode::RecoveryRequired
    } else if message.contains("Touch ID") || message.contains("LocalAuthentication") {
        IpcErrorCode::TouchIdFailed
    } else if message.contains("Keychain") {
        IpcErrorCode::KeychainItemNotFound
    } else if message.contains("card") || message.contains("GBxCart") || message.contains("serial")
    {
        IpcErrorCode::CardReadFailed
    } else {
        IpcErrorCode::VaultCorrupted
    };

    IpcError { code, message }
}
